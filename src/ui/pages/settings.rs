use dioxus::prelude::*;

use crate::{
    app::persist_user_state,
    domain::{AppState, CacheResource, ProfitabilityParams},
    ui::{
        components::toast::{push_toast, ToastKind, ToastMessage},
        pages::cargo::request_price_fetch,
    },
    util::assets,
};

#[component]
pub fn SettingsPage() -> Element {
    let state = use_context::<Signal<AppState>>();
    let toasts = use_context::<Signal<Vec<ToastMessage>>>();
    let price_request = use_context::<Signal<Option<String>>>();

    let initial_params = state.with(|st| st.profitability.clone());

    let mut risk_pct_input = use_signal(|| format!("{:.2}", initial_params.risk_pct));
    let mut crew_hourly_input = use_signal(|| format!("{:.0}", initial_params.crew_hourly));
    let mut crew_size_input = use_signal(|| initial_params.crew_size.to_string());
    let mut time_minutes_input = use_signal(|| initial_params.time_minutes.to_string());

    let cache_entries = state.with(|st| {
        st.cache
            .iter()
            .map(|(resource, time)| {
                (
                    cache_label(resource),
                    crate::ui::pages::cargo::humanize_age(*time),
                )
            })
            .collect::<Vec<_>>()
    });

    let on_apply = {
        let mut state = state.clone();
        let toasts = toasts.clone();
        move |_| {
            let parsed = parse_params(
                risk_pct_input(),
                crew_hourly_input(),
                crew_size_input(),
                time_minutes_input(),
            );

            match parsed {
                Ok(params) => {
                    state.with_mut(|st| st.profitability = params);
                    persist_user_state(&state);
                    push_toast(
                        toasts.clone(),
                        ToastKind::Success,
                        "Updated profitability parameters.",
                    );
                }
                Err(message) => {
                    push_toast(toasts.clone(), ToastKind::Error, message);
                }
            }
        }
    };

    let on_reset = {
        let mut state = state.clone();
        let toasts = toasts.clone();
        move |_| {
            let defaults = ProfitabilityParams::default();
            risk_pct_input.set(format!("{:.2}", defaults.risk_pct));
            crew_hourly_input.set(format!("{:.0}", defaults.crew_hourly));
            crew_size_input.set(defaults.crew_size.to_string());
            time_minutes_input.set(defaults.time_minutes.to_string());
            state.with_mut(|st| st.profitability = defaults);
            persist_user_state(&state);
            push_toast(
                toasts.clone(),
                ToastKind::Info,
                "Restored default profitability parameters.",
            );
        }
    };

    let on_clear_cache = {
        let mut state = state.clone();
        let toasts = toasts.clone();
        move |_| {
            state.with_mut(|st| st.cache.clear());
            push_toast(
                toasts.clone(),
                ToastKind::Info,
                "Cleared cached timestamps. Data will refresh on next fetch.",
            );
        }
    };

    let on_refresh_prices = {
        let state = state.clone();
        let toasts = toasts.clone();
        let price_request = price_request.clone();
        move |_| {
            let commodities: Vec<_> = state.with(|st| {
                st.cargo_items
                    .iter()
                    .map(|item| item.commodity_id.clone())
                    .collect()
            });
            if let Some(first) = commodities.first() {
                request_price_fetch(state.clone(), price_request.clone(), first);
                push_toast(
                    toasts.clone(),
                    ToastKind::Info,
                    "Refreshing price data for tracked commodities...",
                );
            } else {
                push_toast(toasts.clone(), ToastKind::Warning, "No cargo items yet.");
            }
        }
    };

    rsx! {
        div { class: "space-y-8",
            section {
                class: "rounded-xl border border-slate-800 bg-slate-900/40 p-6",
                h2 { class: "text-sm font-semibold uppercase tracking-wide text-slate-500", "Profitability Parameters" }
                div { class: "mt-4 grid gap-4 sm:grid-cols-2",
                    div {
                        label { class: "block text-xs font-semibold uppercase text-slate-500", "Risk % (0-0.40)" }
                        input {
                            class: "mt-1 w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-slate-100 focus:border-indigo-500 focus:outline-none",
                            value: risk_pct_input(),
                            oninput: move |evt| risk_pct_input.set(evt.value()),
                        }
                    }
                    div {
                        label { class: "block text-xs font-semibold uppercase text-slate-500", "Crew hourly cost" }
                        input {
                            class: "mt-1 w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-slate-100 focus:border-indigo-500 focus:outline-none",
                            value: crew_hourly_input(),
                            oninput: move |evt| crew_hourly_input.set(evt.value()),
                        }
                    }
                    div {
                        label { class: "block text-xs font-semibold uppercase text-slate-500", "Crew size" }
                        input {
                            class: "mt-1 w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-slate-100 focus:border-indigo-500 focus:outline-none",
                            value: crew_size_input(),
                            oninput: move |evt| crew_size_input.set(evt.value()),
                        }
                    }
                    div {
                        label { class: "block text-xs font-semibold uppercase text-slate-500", "Trip time (minutes)" }
                        input {
                            class: "mt-1 w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-slate-100 focus:border-indigo-500 focus:outline-none",
                            value: time_minutes_input(),
                            oninput: move |evt| time_minutes_input.set(evt.value()),
                        }
                    }
                }
                div { class: "mt-4 flex gap-3",
                    button { class: "rounded-lg bg-indigo-500 px-4 py-2 text-xs font-semibold uppercase tracking-wide text-white hover:bg-indigo-400", onclick: on_apply, "Apply" }
                    button { class: "rounded-lg border border-slate-600 px-4 py-2 text-xs font-semibold uppercase tracking-wide text-slate-200 hover:bg-slate-800", onclick: on_reset, "Reset Defaults" }
                }
            }

            section {
                class: "rounded-xl border border-slate-800 bg-slate-900/40 p-6",
                h2 { class: "text-sm font-semibold uppercase tracking-wide text-slate-500", "Cache Status" }
                if cache_entries.is_empty() {
                    p { class: "mt-3 text-sm text-slate-400", "No cached fetches yet." }
                } else {
                    ul {
                        class: "mt-3 space-y-2 text-sm text-slate-300",
                        for (label, age) in cache_entries {
                            li { class: "flex items-center justify-between rounded-lg border border-slate-800 bg-slate-900/60 px-3 py-2",
                                span { "{label}" }
                                span { class: "text-xs text-slate-500", "{age}" }
                            }
                        }
                    }
                }
                button { class: "mt-4 rounded-lg border border-amber-500/40 px-4 py-2 text-xs font-semibold uppercase tracking-wide text-amber-200 hover:bg-amber-500/10", onclick: on_clear_cache, "Clear Cache Timestamps" }
            }

            section {
                class: "rounded-xl border border-slate-800 bg-slate-900/40 p-6",
                h2 { class: "text-sm font-semibold uppercase tracking-wide text-slate-500", "Data Controls" }
                p { class: "mt-2 text-sm text-slate-400", "Trigger background refreshes or inspect the cache lifecycle." }
                div { class: "mt-3 flex gap-3",
                    button { class: "rounded-lg border border-indigo-500/40 px-4 py-2 text-xs font-semibold uppercase tracking-wide text-indigo-200 hover:bg-indigo-500/10", onclick: on_refresh_prices, "Refresh Price Data" }
                }
            }

            section {
                class: "flex flex-col items-center gap-3 rounded-xl border border-slate-800 bg-slate-900/40 p-6 text-center text-slate-400",
                h2 { class: "text-sm font-semibold uppercase tracking-wide text-slate-500", "Data Attribution" }
                a {
                    href: "https://uexcorp.space",
                    target: "_blank",
                    rel: "noreferrer",
                    class: "transition hover:scale-105",
                    img {
                        class: "h-12 w-auto opacity-80",
                        src: assets::uex_logo_data_uri(),
                        alt: "United Express (UEX) logo",
                    }
                }
                p {
                    class: "text-sm",
                    "Prices and commodity metadata provided courtesy of United Express (UEX)."
                }
                p {
                    class: "text-xs text-slate-500",
                    "Thank you to UEX for keeping the ‘verse informed."
                }
            }
        }
    }
}

fn parse_params(
    risk_pct: String,
    crew_hourly: String,
    crew_size: String,
    time_minutes: String,
) -> Result<ProfitabilityParams, String> {
    let risk_pct: f64 = risk_pct
        .trim()
        .parse()
        .map_err(|_| "Risk % must be a number between 0 and 0.4")?;
    if !(0.0..=0.4).contains(&risk_pct) {
        return Err("Risk % must be between 0.0 and 0.4".to_string());
    }
    let crew_hourly: f64 = crew_hourly
        .trim()
        .parse()
        .map_err(|_| "Crew hourly cost must be numeric")?;
    let crew_size: u8 = crew_size
        .trim()
        .parse()
        .map_err(|_| "Crew size must be numeric")?;
    let time_minutes: u16 = time_minutes
        .trim()
        .parse()
        .map_err(|_| "Trip time must be numeric")?;

    Ok(ProfitabilityParams {
        risk_pct,
        crew_hourly,
        crew_size,
        time_minutes,
    })
}

fn cache_label(resource: &CacheResource) -> String {
    match resource {
        CacheResource::Commodities => "Commodities".to_string(),
        CacheResource::SellLocations => "Locations".to_string(),
        CacheResource::Prices(id) => format!("Prices ({id})"),
    }
}
