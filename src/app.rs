use std::time::Duration;

use dioxus::{prelude::*, signals::Signal};

use crate::{
    domain::{AppState, CacheResource, SellLocation},
    infra::uex::{CacheStatus, UexClient},
    ui::{
        components::toast::{push_toast, Toast, ToastKind, ToastMessage},
        pages::{BestPricePage, CargoPage, SettingsPage},
        shell::Shell,
    },
    util::{
        assets,
        persistence::{load_persisted_state, save_persisted_state},
    },
};

/// Shared TTL for API cache before a refresh is triggered.
pub const CACHE_TTL: Duration = Duration::from_secs(60 * 60);

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[route("/")]
    #[route("/cargo")]
    Cargo {},
    #[route("/best-price")]
    BestPrice {},
    #[route("/settings")]
    Settings {},
}

#[component]
pub fn App() -> Element {
    let state = use_signal(AppState::default);
    use_hook({
        let mut state = state.clone();
        move || {
            if let Some(saved) = load_persisted_state() {
                state.with_mut(|st| st.apply_persisted(saved));
            }
        }
    });
    use_context_provider(|| state.clone());

    let toasts = use_signal(Vec::<ToastMessage>::new);
    use_context_provider(|| toasts.clone());

    // Price fetch trigger shared across routes.
    let price_request = use_signal(|| None::<String>);
    use_context_provider(|| price_request.clone());

    let _commodities = use_resource({
        let state = state.clone();
        let toasts = toasts.clone();
        move || async move { fetch_commodities(state.clone(), toasts.clone()).await }
    });

    let _prices = use_resource({
        let state = state.clone();
        let toasts = toasts.clone();
        let price_request = price_request.clone();
        move || async move { fetch_prices(state.clone(), toasts.clone(), price_request.clone()).await }
    });

    rsx! {
        document::Link { rel: "icon", href: assets::favicon_data_uri() }
        document::Style { "{assets::main_css()}" }
        document::Style { "{assets::tailwind_css()}" }
        Router::<Route> {}
        Toast {}
    }
}

pub fn persist_user_state(state: &Signal<AppState>) {
    let snapshot = state.with(|st| st.to_persisted());
    if let Err(err) = save_persisted_state(&snapshot) {
        println!("Failed to persist user state: {err}");
    }
}

async fn fetch_commodities(
    mut state: Signal<AppState>,
    toasts: Signal<Vec<ToastMessage>>,
) -> Option<CacheStatus> {
    if let Ok(client) = UexClient::new() {
        match client.get_commodities().await {
            Ok(payload) => {
                state.with_mut(|st| {
                    st.commodities = payload.data.clone();
                    st.cache
                        .record_fetch(CacheResource::Commodities, payload.fetched_at);
                });
                if payload.status == CacheStatus::Stale {
                    push_toast(
                        toasts.clone(),
                        ToastKind::Warning,
                        "Loaded cached commodities; data might be stale.",
                    );
                }
                return Some(payload.status);
            }
            Err(err) => {
                push_toast(
                    toasts.clone(),
                    ToastKind::Error,
                    format!("Failed to load commodities: {err}"),
                );
            }
        }
    } else {
        push_toast(
            toasts.clone(),
            ToastKind::Error,
            "Failed to initialise UEX client.",
        );
    }
    None
}

async fn fetch_prices(
    mut state: Signal<AppState>,
    toasts: Signal<Vec<ToastMessage>>,
    mut price_request: Signal<Option<String>>,
) -> Option<(String, CacheStatus)> {
    let requested = price_request();
    println!("fetch_prices invoked with request: {:?}", requested);
    let Some(commodity_id) = requested else {
        println!("No commodity queued for price fetch; skipping API call.");
        return None;
    };

    let Ok(client) = UexClient::new() else {
        push_toast(
            toasts.clone(),
            ToastKind::Error,
            "Failed to initialise UEX client for prices.",
        );
        return None;
    };

    let commodity_name = state.with(|st| {
        st.commodities
            .iter()
            .find(|c| c.id == commodity_id)
            .map(|c| c.name.clone())
    });

    println!("Starting UEX price fetch for commodity {commodity_id}");

    match client
        .get_prices(&commodity_id, commodity_name.as_deref())
        .await
    {
        Ok(payload) => {
            price_request.set(None);
            println!(
                "Fetched {} price points for commodity {} (status: {:?}).",
                payload.data.len(),
                commodity_id,
                payload.status
            );
            if payload.data.is_empty() {
                println!("Warning: UEX returned zero price points for {commodity_id}");
            }
            for (idx, point) in payload.data.iter().take(5).enumerate() {
                println!(
                    "Point #{idx} -> terminal: {}, sell(min/max): {:?}/{:?}, buy(min/max): {:?}/{:?}",
                    point.terminal_name,
                    point.price_sell_min,
                    point.price_sell_max,
                    point.price_buy_min,
                    point.price_buy_max
                );
            }
            state.with_mut(|st| {
                st.price_points
                    .insert(commodity_id.clone(), payload.data.clone());
                st.cache.record_fetch(
                    CacheResource::Prices(commodity_id.clone()),
                    payload.fetched_at,
                );
                for point in &payload.data {
                    let key = point
                        .terminal_id
                        .map(|id| id.to_string())
                        .unwrap_or_else(|| point.terminal_name.clone());
                    st.sell_locations
                        .entry(key.clone())
                        .or_insert_with(|| SellLocation {
                            id: key.clone(),
                            name: point.terminal_name.clone(),
                            system: point.system.clone(),
                            kind: Some("Terminal".to_string()),
                            terminal_code: point.terminal_code.clone(),
                            armistice: false,
                        });
                }
                st.cache
                    .record_fetch(CacheResource::SellLocations, payload.fetched_at);
            });

            if payload.status != CacheStatus::Fresh {
                push_toast(
                    toasts.clone(),
                    ToastKind::Info,
                    format!("Prices for {} served from cache.", commodity_id),
                );
            }

            Some((commodity_id, payload.status))
        }
        Err(err) => {
            price_request.set(None);
            println!("UEX client failed to load prices for {commodity_id}: {err}");
            push_toast(
                toasts.clone(),
                ToastKind::Error,
                format!("Failed to load prices: {err}"),
            );
            None
        }
    }
}

#[component]
pub fn Cargo() -> Element {
    rsx! { Shell { CargoPage {} } }
}

#[component]
pub fn BestPrice() -> Element {
    rsx! { Shell { BestPricePage {} } }
}

#[component]
pub fn Settings() -> Element {
    rsx! { Shell { SettingsPage {} } }
}
