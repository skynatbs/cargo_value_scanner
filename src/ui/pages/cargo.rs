use std::collections::HashMap;

use dioxus::prelude::*;

use crate::{
    app::{persist_user_state, CACHE_TTL},
    domain::{
        evaluate_cargo_items, profitability_indicator, AppState, CacheResource, CargoItem,
        Commodity, PricePoint,
    },
    ui::components::{
        cargo_table::{CargoRow, CargoTable},
        kpi_card::KpiCard,
        price_table::{PriceRow, PriceTable},
        profit_indicator::ProfitIndicator,
        toast::{push_toast, ToastKind, ToastMessage},
    },
    util::generate_id,
};

#[component]
pub fn CargoPage() -> Element {
    let state = use_context::<Signal<AppState>>();
    let toasts = use_context::<Signal<Vec<ToastMessage>>>();
    let price_request = use_context::<Signal<Option<String>>>();

    let mut commodity_query = use_signal(String::new);
    let mut scu_input = use_signal(String::new);
    let selected_item = use_signal(|| None::<String>);

    let commodities = state.with(|st| st.commodities.clone());
    let items = state.with(|st| st.cargo_items.clone());
    let price_map = state.with(|st| st.price_points.clone());
    let profitability = state.with(|st| st.profitability.clone());

    let summary = evaluate_cargo_items(&items, &price_map);
    let indicator = profitability_indicator(summary.total_ev, &profitability);

    let evaluation_lookup: HashMap<_, _> = summary
        .items
        .iter()
        .map(|(id, evaluation)| (id.clone(), evaluation.clone()))
        .collect();

    let rows: Vec<CargoRow> = items
        .iter()
        .map(|item| {
            let evaluation = evaluation_lookup.get(&item.id);
            let best_sell = price_map
                .get(&item.commodity_id)
                .and_then(|points| best_sell_info(points));
            let (best_sell_location, best_sell_price) = match best_sell {
                Some(info) => (
                    Some(format!("{} ({:.0} aUEC)", info.location, info.price)),
                    Some(info.price),
                ),
                None => (None, None),
            };
            let expected_value = best_sell_price
                .map(|price| price * item.scu as f64)
                .or_else(|| evaluation.map(|eval| eval.ev))
                .unwrap_or_default();
            CargoRow {
                id: item.id.clone(),
                commodity_name: item.commodity_name.clone(),
                scu: item.scu,
                expected_value,
                min_value: evaluation.and_then(|eval| eval.min),
                max_value: evaluation.and_then(|eval| eval.max),
                confidence: evaluation.map(|eval| eval.confidence).unwrap_or_default(),
                best_sell_location,
            }
        })
        .collect();

    let total_ev_display = format!("{:.0}", summary.total_ev);
    let average_confidence = summary.average_confidence;

    let selected_id = selected_item();
    let price_breakdown = selected_id.as_ref().and_then(|id| {
        let commodity_id = items
            .iter()
            .find(|item| &item.id == id)
            .map(|item| item.commodity_id.clone())?;
        let data = price_map.get(&commodity_id)?.clone();
        let rows = data
            .into_iter()
            .map(|point| PriceRow {
                location: point.terminal_name,
                sell_price_min: point
                    .price_sell_min
                    .or(point.price_sell)
                    .or(point.price_average),
                sell_price_max: point
                    .price_sell_max
                    .or(point.price_sell)
                    .or(point.price_average),
                buy_price_min: point.price_buy_min.or(point.price_buy),
                buy_price_max: point.price_buy_max.or(point.price_buy),
                stock: point.scu_sell_stock,
                status_sell: point.status_sell,
                status_buy: point.status_buy,
                container_sizes: point.container_sizes.clone(),
                updated_label: humanize_age(point.updated_at),
            })
            .collect::<Vec<_>>();
        Some((rows, commodity_id))
    });

    let on_submit = {
        let state = state.clone();
        let toasts = toasts.clone();
        let price_request = price_request.clone();
        let mut commodity_query = commodity_query.clone();
        let mut scu_input = scu_input.clone();
        let mut selected_item = selected_item.clone();
        move |evt: FormEvent| {
            evt.prevent_default();
            let query = commodity_query().trim().to_string();
            if query.is_empty() {
                push_toast(
                    toasts.clone(),
                    ToastKind::Warning,
                    "Pick a commodity first.",
                );
                return;
            }

            let commodity = state.with(|st| {
                st.commodities
                    .iter()
                    .find(|c| {
                        c.name.eq_ignore_ascii_case(&query) || c.id.eq_ignore_ascii_case(&query)
                    })
                    .cloned()
            });

            let Some(commodity) = commodity else {
                push_toast(
                    toasts.clone(),
                    ToastKind::Error,
                    "Commodity not found. Use the autocomplete list.",
                );
                return;
            };

            let delta = match scu_input().trim().parse::<i32>() {
                Ok(value) if value != 0 => value,
                _ => {
                    push_toast(
                        toasts.clone(),
                        ToastKind::Error,
                        "Enter a non-zero SCU adjustment (positive to add, negative to subtract).",
                    );
                    return;
                }
            };

            match adjust_cargo_item(
                state.clone(),
                &commodity,
                delta,
                selected_item.clone(),
                toasts.clone(),
            ) {
                CargoAdjustResult::Added(new_id, commodity_id) => {
                    commodity_query.set(String::new());
                    scu_input.set(String::new());
                    selected_item.set(Some(new_id.clone()));
                    request_price_fetch(state.clone(), price_request.clone(), &commodity_id);
                }
                CargoAdjustResult::Updated(item_id) => {
                    commodity_query.set(String::new());
                    scu_input.set(String::new());
                    selected_item.set(Some(item_id));
                }
                CargoAdjustResult::Removed => {
                    commodity_query.set(String::new());
                    scu_input.set(String::new());
                }
                CargoAdjustResult::Error => {}
            }
        }
    };

    let on_remove = {
        let mut state = state.clone();
        let toasts = toasts.clone();
        let mut selected_item = selected_item.clone();
        move |id: String| {
            state.with_mut(|st| st.cargo_items.retain(|item| item.id != id));
            if selected_item().as_ref() == Some(&id) {
                selected_item.set(None);
            }
            persist_user_state(&state);
            push_toast(toasts.clone(), ToastKind::Info, "Cargo item removed.");
        }
    };

    let on_select = {
        let mut selected_item = selected_item.clone();
        move |id: String| {
            selected_item.set(Some(id));
        }
    };

    let on_refresh_prices = {
        let state = state.clone();
        let price_request = price_request.clone();
        let toasts = toasts.clone();
        let selected_item = selected_item.clone();
        move |_| {
            if let Some(selected) = selected_item() {
                if let Some(commodity_id) = state.with(|st| {
                    st.cargo_items
                        .iter()
                        .find(|item| item.id == selected)
                        .map(|item| item.commodity_id.clone())
                }) {
                    request_price_fetch(state.clone(), price_request.clone(), &commodity_id);
                    push_toast(toasts.clone(), ToastKind::Info, "Refreshing price data...");
                }
            } else {
                push_toast(
                    toasts.clone(),
                    ToastKind::Warning,
                    "Select a cargo row first.",
                );
            }
        }
    };

    let (price_rows, selected_commodity_id) = match price_breakdown {
        Some((rows, id)) => (rows, Some(id)),
        None => (Vec::new(), None),
    };

    rsx! {
        div { class: "space-y-8",
            section {
                class: "grid gap-4 sm:grid-cols-3",
                KpiCard {
                    title: "Total Expected Value".to_string(),
                    value: total_ev_display,
                    description: Some("Sum of all cargo EV (aUEC)".to_string()),
                }
                KpiCard {
                    title: "Average Confidence".to_string(),
                    value: format!("{:.0}%", average_confidence * 100.0),
                    description: Some("Weighted by evaluated items".to_string()),
                }
                ProfitIndicator { indicator: indicator }
            }

            section {
                class: "grid gap-6 lg:grid-cols-[2fr,1fr]",
                div {
                    class: "space-y-4",
                    form {
                        class: "flex flex-wrap items-end gap-4 rounded-xl border border-slate-800 bg-slate-900/40 px-4 py-4",
                        onsubmit: on_submit,
                        div { class: "flex-1 min-w-[200px]",
                            label { class: "block text-xs font-semibold uppercase text-slate-500", "Commodity" }
                            input {
                                class: "mt-1 w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-slate-100 focus:border-indigo-500 focus:outline-none",
                                value: commodity_query(),
                                oninput: move |evt| commodity_query.set(evt.value().to_string()),
                                list: "commodity-list",
                                placeholder: "e.g. Agricultural Supplies",
                            }
                            datalist {
                                id: "commodity-list",
                                for commodity in commodities.iter() {
                                    option { value: commodity.name.clone() }
                                }
                            }
                        }
                        div { class: "w-32",
                            label { class: "block text-xs font-semibold uppercase text-slate-500", "SCU" }
                            input {
                                class: "mt-1 w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-slate-100 focus:border-indigo-500 focus:outline-none",
                                inputmode: "decimal",
                                value: scu_input(),
                                oninput: move |evt| scu_input.set(evt.value().to_string()),
                                placeholder: "32",
                            }
                        }
                        button {
                            class: "rounded-lg bg-indigo-500 px-4 py-2 text-sm font-semibold text-white hover:bg-indigo-400",
                            r#type: "submit",
                            "Add Cargo"
                        }
                    }

                    CargoTable {
                        rows,
                        selected_id: selected_id.clone(),
                        on_select,
                        on_remove,
                    }
                }

                div {
                    class: "space-y-4",
                    div { class: "flex items-center justify-between",
                        h2 { class: "text-sm font-semibold text-slate-200", "Price Breakdown" }
                        button {
                            class: "text-xs font-semibold uppercase tracking-wide text-indigo-300 hover:text-indigo-100",
                            onclick: on_refresh_prices,
                            "Refresh"
                        }
                    }
                    if let Some(ref commodity_id) = selected_commodity_id {
                        p { class: "text-xs text-slate-500", "Commodity ID: {commodity_id}" }
                    }
                    PriceTable { rows: price_rows }
                }
            }
        }
    }
}

pub fn request_price_fetch(
    state: Signal<AppState>,
    mut price_request: Signal<Option<String>>,
    commodity_id: &str,
) {
    let resource = CacheResource::Prices(commodity_id.to_string());
    let needs_fetch = state.with(|st| {
        let stale = st.is_stale(&resource, CACHE_TTL);
        let missing = st
            .price_points
            .get(commodity_id)
            .map(|points| points.is_empty())
            .unwrap_or(true);
        stale || missing
    });

    if needs_fetch {
        println!("Queueing price fetch for {commodity_id} (stale: {needs_fetch})");
        price_request.set(Some(commodity_id.to_string()));
    } else {
        println!("Skipping price fetch for {commodity_id}; cache still fresh.");
    }
}

pub fn humanize_age(updated_at: std::time::SystemTime) -> String {
    use std::time::SystemTime;

    let now = SystemTime::now();
    let age = now.duration_since(updated_at).unwrap_or_default().as_secs();
    if age < 60 {
        format!("{age}s ago")
    } else if age < 3_600 {
        format!("{}m ago", age / 60)
    } else if age < 86_400 {
        format!("{}h ago", age / 3_600)
    } else {
        format!("{}d ago", age / 86_400)
    }
}

#[derive(Clone)]
struct BestSellInfo {
    location: String,
    price: f64,
}

fn best_sell_info(points: &[PricePoint]) -> Option<BestSellInfo> {
    points
        .iter()
        .filter_map(|point| {
            let price = point
                .price_sell_max
                .or(point.price_sell)
                .or(point.price_average)
                .or(point.price_sell_min)?;
            if !price.is_finite() || price <= 0.0 {
                return None;
            }
            let location = point
                .system
                .as_ref()
                .map(|system| format!("{} · {}", point.terminal_name, system))
                .unwrap_or_else(|| point.terminal_name.clone());
            Some(BestSellInfo { location, price })
        })
        .max_by(|a, b| a.price.partial_cmp(&b.price).unwrap())
}

enum CargoAdjustResult {
    Added(String, String),
    Updated(String),
    Removed,
    Error,
}

fn adjust_cargo_item(
    mut state: Signal<AppState>,
    commodity: &Commodity,
    delta: i32,
    mut selected_item: Signal<Option<String>>,
    toasts: Signal<Vec<ToastMessage>>,
) -> CargoAdjustResult {
    let mut result = CargoAdjustResult::Error;
    let mut toast: Option<(ToastKind, String)> = None;

    state.with_mut(|st| {
        if let Some(index) = st
            .cargo_items
            .iter()
            .position(|item| item.commodity_id == commodity.id)
        {
            let current = st.cargo_items[index].scu as i32;
            let new_total = current + delta;
            if new_total <= 0 {
                let removed = st.cargo_items.remove(index);
                if selected_item().as_ref() == Some(&removed.id) {
                    selected_item.set(None);
                }
                result = CargoAdjustResult::Removed;
                toast = Some((
                    ToastKind::Info,
                    format!(
                        "Removed {} after adjustment (new total would be {new_total}).",
                        commodity.name
                    ),
                ));
            } else {
                st.cargo_items[index].scu = new_total as u32;
                let id = st.cargo_items[index].id.clone();
                result = CargoAdjustResult::Updated(id.clone());
                toast = Some((
                    ToastKind::Success,
                    format!("Updated {} to {new_total} SCU.", commodity.name),
                ));
            }
        } else if delta < 0 {
            result = CargoAdjustResult::Error;
            toast = Some((
                ToastKind::Error,
                format!(
                    "Cannot subtract {delta} SCU because {name} is not in your cargo list.",
                    name = commodity.name
                ),
            ));
        } else {
            let item_id = generate_id("cargo");
            let new_item = CargoItem {
                id: item_id.clone(),
                commodity_id: commodity.id.clone(),
                commodity_name: commodity.name.clone(),
                scu: delta as u32,
            };
            st.cargo_items.push(new_item);
            result = CargoAdjustResult::Added(item_id.clone(), commodity.id.clone());
            toast = Some((
                ToastKind::Success,
                format!("Tracked {} with {delta} SCU.", commodity.name),
            ));
        }
    });

    if !matches!(result, CargoAdjustResult::Error) {
        persist_user_state(&state);
    }

    if let Some((kind, message)) = toast {
        push_toast(toasts, kind, message);
    }

    result
}
