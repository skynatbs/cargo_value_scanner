use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use dioxus::{document, prelude::*};
use tokio::time::sleep;

use crate::{
    domain::{rank_best_prices, AppState, BestPriceSummary},
    ui::components::toast::{push_toast, ToastKind, ToastMessage},
    ui::pages::cargo::request_price_fetch,
};

#[component]
pub fn BestPricePage() -> Element {
    let state = use_context::<Signal<AppState>>();
    let toasts = use_context::<Signal<Vec<ToastMessage>>>();
    let price_request = use_context::<Signal<Option<String>>>();

    let items = state.with(|st| st.cargo_items.clone());
    let price_map = state.with(|st| st.price_points.clone());
    let locations = state.with(|st| st.sell_locations.clone());

    if items.is_empty() {
        return rsx! {
            div { class: "rounded-xl border border-slate-800 bg-slate-900/40 p-6 text-sm text-slate-400",
                "Add cargo items first to generate best-price suggestions." }
        };
    }

    let missing: HashSet<_> = items
        .iter()
        .filter(|item| {
            price_map
                .get(&item.commodity_id)
                .map(|prices| prices.is_empty())
                .unwrap_or(true)
        })
        .map(|item| item.commodity_id.clone())
        .collect();

    let item_to_commodity: HashMap<_, _> = items
        .iter()
        .map(|item| (item.id.clone(), item.commodity_id.clone()))
        .collect();

    let summary = rank_best_prices(&items, &price_map, &locations);
    let suggestion_views = build_views(&summary, &item_to_commodity);
    let quick_copy = build_summary_text(&summary);
    let summary_copied = use_signal(|| false);
    let on_copy_summary = {
        let quick_copy = quick_copy.clone();
        let mut summary_copied = summary_copied.clone();
        move |_| {
            if quick_copy.trim().is_empty() {
                return;
            }
            if copy_text_to_clipboard(&quick_copy) {
                summary_copied.set(true);
                let mut summary_copied = summary_copied.clone();
                spawn(async move {
                    sleep(Duration::from_secs(2)).await;
                    summary_copied.set(false);
                });
            }
        }
    };

    let on_refresh_all = {
        let state = state.clone();
        let toasts = toasts.clone();
        let price_request = price_request.clone();
        let missing: Vec<_> = missing.iter().cloned().collect();
        move |_| {
            if let Some(id) = missing.first() {
                request_price_fetch(state.clone(), price_request.clone(), id);
                push_toast(
                    toasts.clone(),
                    ToastKind::Info,
                    format!("Refreshing price data for {} commodities...", missing.len()),
                );
            } else {
                push_toast(
                    toasts.clone(),
                    ToastKind::Info,
                    "All commodities already have price data.",
                );
            }
        }
    };

    rsx! {
        div { class: "space-y-6",
            header {
                class: "flex flex-wrap items-start justify-between gap-4",
                div {
                    h1 { class: "text-2xl font-semibold text-slate-100", "Best-Price Finder" }
                    p {
                        class: "text-sm text-slate-400",
                        "Ranks sell locations by adjusted price considering travel risk and volatility."
                    }
                }
                button {
                    class: "rounded-md border border-indigo-500/40 px-3 py-2 text-xs font-semibold uppercase tracking-wide text-indigo-200 hover:bg-indigo-500/10",
                    onclick: on_refresh_all,
                    "Refresh Missing Data"
                }
            }

            if !missing.is_empty() {
                div {
                    class: "rounded-lg border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-xs text-amber-200",
                    {format!("{} commodity price set(s) missing or stale. Refresh to improve accuracy.", missing.len())}
                }
            }

            if let Some(best) = summary.best_overall.as_ref() {
                div {
                    class: "rounded-xl border border-emerald-500/40 bg-emerald-500/10 p-4 text-emerald-100",
                    h2 { class: "text-sm font-semibold uppercase tracking-wide", "Best Overall" }
                    p { class: "mt-1 text-lg font-semibold", "{format_price_display(best.sell_price)} aUEC" }
                    p { class: "text-sm opacity-90", "{best.location_name}" }
                    if let Some(notes) = &best.notes {
                        p { class: "mt-1 text-xs opacity-80", "Notes: {notes}" }
                    }
                }
            }

            div {
                class: "space-y-4",
                for view in suggestion_views {
                    SuggestionCard {
                        view,
                        state: state.clone(),
                        price_request: price_request.clone(),
                        toasts: toasts.clone(),
                    }
                }
            }

            section {
                class: "rounded-xl border border-slate-800 bg-slate-900/40 p-4",
                div { class: "flex items-center justify-between gap-3",
                    h2 { class: "text-sm font-semibold uppercase tracking-wide text-slate-500", "Quick Summary" }
                    button {
                        class: "rounded-md border border-slate-700 px-3 py-1 text-xs font-semibold uppercase tracking-wide text-slate-200 hover:border-indigo-500 hover:text-indigo-200",
                        onclick: on_copy_summary,
                        if summary_copied() {
                            "Copied!"
                        } else {
                            "Copy"
                        }
                    }
                }
                textarea {
                    class: "mt-3 h-32 w-full rounded-lg border border-slate-800 bg-slate-950 p-3 text-sm text-slate-200",
                    value: quick_copy.clone(),
                    readonly: true,
                }
                p { class: "mt-2 text-xs text-slate-500", "Copy and share these recommendations with your crew." }
            }
        }
    }
}

#[component]
fn SuggestionCard(
    view: SuggestionView,
    state: Signal<AppState>,
    price_request: Signal<Option<String>>,
    toasts: Signal<Vec<ToastMessage>>,
) -> Element {
    let commodity_name = view.commodity_name.clone();
    let commodity_id = view.commodity_id.clone();
    let entries = view.entries.clone();
    rsx! {
        div {
            class: "w-full rounded-xl border border-slate-800 bg-slate-900/40 p-4",
            div { class: "flex items-center justify-between",
                h3 { class: "text-sm font-semibold text-slate-100", "{commodity_name}" }
                button {
                    class: "text-xs font-semibold uppercase tracking-wide text-indigo-300 hover:text-indigo-100",
                    onclick: move |_| {
                        if let Some(ref id) = commodity_id {
                            request_price_fetch(state.clone(), price_request.clone(), id);
                            push_toast(toasts.clone(), ToastKind::Info, "Refreshing price data...");
                        } else {
                            push_toast(toasts.clone(), ToastKind::Warning, "Commodity lookup missing for this item.");
                        }
                    },
                    "Refresh"
                }
            }
            table {
                class: "mt-3 w-full divide-y divide-slate-800 text-sm",
                thead {
                    class: "text-xs uppercase tracking-wide text-slate-500",
                    tr {
                        th { class: "py-2 text-left", "Location" }
                        th { class: "py-2 text-right", "Sell Max (aUEC)" }
                        th { class: "py-2 text-right", "Buy Min (aUEC)" }
                        th { class: "py-2 text-right", "Stock (SCU)" }
                        th { class: "py-2 text-right", "Demand" }
                        th { class: "py-2 text-right", "Containers" }
                        th { class: "py-2 text-right", "Adjusted (aUEC)" }
                    }
                }
                tbody {
                    for entry in entries {
                        tr {
                            class: "border-t border-slate-900/60 text-slate-200",
                            td { class: "py-2 text-left", "{entry.location}" }
                            td { class: "py-2 text-right", "{entry.sell_display}" }
                            td { class: "py-2 text-right", "{entry.buy_display}" }
                            td { class: "py-2 text-right", "{entry.stock_display}" }
                            td { class: "py-2 text-right", "{entry.demand_display}" }
                            td { class: "py-2 text-right", "{entry.containers_display}" }
                            td { class: "py-2 text-right", "{entry.adjusted_display}" }
                        }
                        if let Some(notes) = entry.notes {
                            tr {
                                td { class: "pb-2 text-left text-xs text-slate-500", colspan: "7", "Notes: {notes}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
struct SuggestionView {
    commodity_name: String,
    commodity_id: Option<String>,
    entries: Vec<EntryView>,
}

#[derive(Clone, PartialEq)]
struct EntryView {
    location: String,
    sell_display: String,
    buy_display: String,
    stock_display: String,
    demand_display: String,
    containers_display: String,
    adjusted_display: String,
    notes: Option<String>,
}

fn build_views(summary: &BestPriceSummary, map: &HashMap<String, String>) -> Vec<SuggestionView> {
    summary
        .suggestions
        .iter()
        .map(|suggestion| SuggestionView {
            commodity_name: suggestion.commodity_name.clone(),
            commodity_id: map.get(&suggestion.item_id).cloned(),
            entries: suggestion
                .entries
                .iter()
                .map(|entry| EntryView {
                    location: entry.location_name.clone(),
                    sell_display: format_price_display(entry.sell_price),
                    buy_display: format_price_display(entry.buy_price),
                    stock_display: format_stock_display(entry.stock),
                    demand_display: format_status_display(entry.status_sell, entry.status_buy),
                    containers_display: format_containers_display(&entry.container_sizes),
                    adjusted_display: format!("{:.0}", entry.adjusted_price.max(0.0)),
                    notes: entry.notes.clone(),
                })
                .collect(),
        })
        .collect()
}

fn build_summary_text(summary: &BestPriceSummary) -> String {
    let mut lines = Vec::new();
    for suggestion in summary.suggestions.iter() {
        if let Some(entry) = suggestion.entries.first() {
            let mut line = format!(
                "{} → {} @ {} aUEC",
                suggestion.commodity_name,
                entry.location_name,
                format_price_display(entry.sell_price)
            );

            let mut details = Vec::new();
            let stock = format_stock_display(entry.stock);
            if stock != "—" {
                details.push(format!("stock {stock}"));
            }
            let demand = format_status_display(entry.status_sell, entry.status_buy);
            if demand != "—" {
                details.push(format!("demand {demand}"));
            }
            let containers = format_containers_display(&entry.container_sizes);
            if containers != "—" {
                details.push(format!("containers {containers}"));
            }
            if !details.is_empty() {
                line.push_str(&format!(" [{}]", details.join(", ")));
            }
            if let Some(notes) = entry.notes.as_deref() {
                line.push_str(&format!(" ({notes})"));
            }
            lines.push(line);
        }
    }
    if let Some(best) = summary.best_overall.as_ref() {
        let mut line = format!(
            "Overall best: {} @ {} aUEC",
            best.location_name,
            format_price_display(best.sell_price)
        );

        let mut details = Vec::new();
        let stock = format_stock_display(best.stock);
        if stock != "—" {
            details.push(format!("stock {stock}"));
        }
        let demand = format_status_display(best.status_sell, best.status_buy);
        if demand != "—" {
            details.push(format!("demand {demand}"));
        }
        let containers = format_containers_display(&best.container_sizes);
        if containers != "—" {
            details.push(format!("containers {containers}"));
        }

        if !details.is_empty() {
            line.push_str(&format!(" [{}]", details.join(", ")));
        }
        if let Some(notes) = best.notes.as_deref() {
            line.push_str(&format!(" ({notes})"));
        }
        lines.push(line);
    }
    lines.join("\n")
}

fn format_price_display(value: Option<f64>) -> String {
    value
        .filter(|v| v.is_finite() && *v >= 0.0)
        .map(|v| format!("{:.0}", v))
        .unwrap_or_else(|| "n/a".to_string())
}

fn format_stock_display(value: Option<f64>) -> String {
    match value {
        Some(v) if v.is_finite() && v > 0.0 => format!("{:.0} SCU", v),
        _ => "—".to_string(),
    }
}

fn format_status_display(sell: Option<i32>, buy: Option<i32>) -> String {
    let sell_text = status_label(sell);
    let buy_text = status_label(buy);

    match (sell_text, buy_text) {
        (Some(sell), Some(buy)) => format!("Sell: {sell} / Buy: {buy}"),
        (Some(sell), None) => format!("Sell: {sell}"),
        (None, Some(buy)) => format!("Buy: {buy}"),
        _ => "—".to_string(),
    }
}

fn status_label(code: Option<i32>) -> Option<&'static str> {
    match code {
        Some(3) => Some("High"),
        Some(2) => Some("Normal"),
        Some(1) => Some("Low"),
        Some(0) => Some("Unavailable"),
        _ => None,
    }
}

fn format_containers_display(sizes: &[f64]) -> String {
    if sizes.is_empty() {
        return "—".to_string();
    }

    let values: Vec<String> = sizes
        .iter()
        .filter(|size| size.is_finite() && **size > 0.0)
        .map(|size| format!("{size:.0}"))
        .collect();

    if values.is_empty() {
        "—".to_string()
    } else {
        format!("{} SCU", values.join(", "))
    }
}

fn copy_text_to_clipboard(text: &str) -> bool {
    if text.trim().is_empty() {
        return false;
    }
    let payload = serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_string());
    let script = format!(
        r#"(async () => {{
            const data = {payload};
            try {{
                if (navigator.clipboard && navigator.clipboard.writeText) {{
                    await navigator.clipboard.writeText(data);
                    return true;
                }}
            }} catch (_err) {{
                // fallback
            }}
            try {{
                const textarea = document.createElement('textarea');
                textarea.value = data;
                textarea.style.position = 'fixed';
                textarea.style.opacity = '0';
                document.body.appendChild(textarea);
                textarea.focus();
                textarea.select();
                const ok = document.execCommand('copy');
                document.body.removeChild(textarea);
                return ok;
            }} catch (_err) {{
                return false;
            }}
        }})()"#
    );
    let eval = document::eval(&script);
    spawn(async move {
        let _ = eval.await;
    });
    true
}
