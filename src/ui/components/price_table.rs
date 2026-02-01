use std::cmp::Ordering;

use dioxus::prelude::*;

use crate::domain::Profile;
use crate::ui::theme;

#[derive(Clone, PartialEq)]
pub struct PriceRow {
    pub location: String,
    pub sell_price_min: Option<f64>,
    pub sell_price_max: Option<f64>,
    pub buy_price_min: Option<f64>,
    pub buy_price_max: Option<f64>,
    pub stock: Option<f64>,
    pub status_sell: Option<i32>,
    pub status_buy: Option<i32>,
    pub container_sizes: Vec<f64>,
    pub updated_label: String,
    /// Terminal accepts stolen/illegal cargo (no questions asked).
    pub is_nqa: bool,
}

#[component]
pub fn PriceTable(rows: Vec<PriceRow>, profile: Profile) -> Element {
    let sort_mode = use_signal(|| SortMode::SellRange);
    let count = rows.len();
    let is_empty = rows.is_empty();
    let current_sort = sort_mode();
    let highlights = summarize_price_rows(&rows);
    let best_sell_location = highlights
        .best_sell
        .as_ref()
        .map(|(location, _)| location.clone());
    let best_buy_location = highlights
        .best_buy
        .as_ref()
        .map(|(location, _)| location.clone());
    let best_sell_value = highlights
        .best_sell
        .as_ref()
        .map(|(_, price)| format!("{price:.0} aUEC"))
        .unwrap_or_else(|| "—".to_string());
    let best_sell_caption = highlights
        .best_sell
        .as_ref()
        .map(|(location, _)| location.clone())
        .unwrap_or_else(|| "No sell data".to_string());
    let best_buy_value = highlights
        .best_buy
        .as_ref()
        .map(|(_, price)| format!("{price:.0} aUEC"))
        .unwrap_or_else(|| "—".to_string());
    let best_buy_caption = highlights
        .best_buy
        .as_ref()
        .map(|(location, _)| location.clone())
        .unwrap_or_else(|| "No buy data".to_string());
    let sell_range_summary = format_summary_range(highlights.sell_range);
    let buy_range_summary = format_summary_range(highlights.buy_range);

    let mut rendered_rows = rows
        .into_iter()
        .map(|row| {
            let is_best_sell = best_sell_location
                .as_ref()
                .map(|loc| loc == &row.location)
                .unwrap_or(false);
            let is_best_buy = best_buy_location
                .as_ref()
                .map(|loc| loc == &row.location)
                .unwrap_or(false);
            (row, is_best_sell, is_best_buy)
        })
        .collect::<Vec<_>>();

    sort_rows(&mut rendered_rows, current_sort);
    pin_highlight_rows(&mut rendered_rows);

    rsx! {
        div {
            class: "{theme::panel_border(profile)}",
            if highlights.has_data() {
                div {
                    class: "grid gap-4 {theme::table_header(profile)} px-4 py-3 text-sm sm:grid-cols-3",
                    SummaryStat {
                        title: "Best Sell (max)",
                        value: best_sell_value,
                        caption: best_sell_caption,
                        profile: profile,
                    }
                    SummaryStat {
                        title: "Best Buy (min)",
                        value: best_buy_value,
                        caption: best_buy_caption,
                        profile: profile,
                    }
                    div {
                        class: "{theme::panel_solid(profile)} p-3",
                        p { class: "text-[10px] font-semibold uppercase tracking-wide {theme::text_muted(profile)}", "Price Ranges" }
                        p { class: "text-xs {theme::text_muted(profile)}", "Sell: {sell_range_summary}" }
                        p { class: "text-xs {theme::text_muted(profile)}", "Buy: {buy_range_summary}" }
                    }
                }
            }
            header {
                class: "flex flex-wrap items-center justify-between gap-2 {theme::table_header(profile)} px-4 py-3",
                h3 { class: "text-sm font-semibold {theme::text_secondary(profile)}", "Price Points" }
                span { class: "text-xs {theme::text_muted(profile)}", "{count} sources" }
            }
            if !is_empty {
                div {
                    class: "flex flex-wrap items-center gap-2 {theme::table_header(profile)} px-4 py-2 text-xs uppercase tracking-wide",
                    span { "Sort:" }
                    button {
                        class: sort_button_class(current_sort == SortMode::SellRange, profile),
                        onclick: {
                            let mut sort_mode = sort_mode.clone();
                            move |_| sort_mode.set(SortMode::SellRange)
                        },
                        "Sell Range"
                    }
                    button {
                        class: sort_button_class(current_sort == SortMode::BuyRange, profile),
                        onclick: {
                            let mut sort_mode = sort_mode.clone();
                            move |_| sort_mode.set(SortMode::BuyRange)
                        },
                        "Buy Range"
                    }
                    button {
                        class: sort_button_class(current_sort == SortMode::Stock, profile),
                        onclick: {
                            let mut sort_mode = sort_mode.clone();
                            move |_| sort_mode.set(SortMode::Stock)
                        },
                        "Stock"
                    }
                    button {
                        class: sort_button_class(current_sort == SortMode::Demand, profile),
                        onclick: {
                            let mut sort_mode = sort_mode.clone();
                            move |_| sort_mode.set(SortMode::Demand)
                        },
                        "Demand"
                    }
                }
            }
            if is_empty {
                p { class: "px-4 py-6 text-sm {theme::text_muted(profile)}", "No price data available yet." }
            } else {
                table {
                        class: "min-w-full {theme::table_divider(profile)} text-sm",
                        thead {
                            class: "sticky top-0 z-10 {theme::table_header(profile)} text-left tracking-wide",
                            tr {
                                th { class: "px-4 py-3 font-medium", "Terminal" }
                                th { class: "px-4 py-3 font-medium text-right", "Sell Range (aUEC)" }
                                th { class: "px-4 py-3 font-medium text-right", "Buy Range (aUEC)" }
                            th { class: "px-4 py-3 font-medium text-right", "Stock (SCU)" }
                            th { class: "px-4 py-3 font-medium text-right", "Demand" }
                            th { class: "px-4 py-3 font-medium text-right min-w-[150px]", "Containers (SCU)" }
                                th { class: "px-4 py-3 font-medium", "Updated" }
                            }
                        }
                    tbody {
                        class: "{theme::table_divider(profile)}",
                        for (row, is_best_sell, is_best_buy) in rendered_rows {
                            tr {
                                class: row_hover_class(profile),
                                td { class: "px-4 py-3 font-medium {theme::text_secondary(profile)}",
                                    if row.is_nqa {
                                        span {
                                            class: "mr-1.5 inline-flex items-center rounded bg-[#3b1712] px-1.5 py-0.5 text-[10px] font-semibold {theme::text_primary(profile)}",
                                            "🏴‍☠️"
                                        }
                                    }
                                    "{row.location}"
                                }
                                td {
                                    class: "px-4 py-3 text-right {theme::text_secondary(profile)}",
                                    div { class: "flex flex-col items-end gap-1 text-xs",
                                        span { class: "text-sm font-medium", "{format_price_range(row.sell_price_min, row.sell_price_max)}" }
                                        if is_best_sell {
                                            span {
                                                class: "rounded-full border border-[#5c2a1f] px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide {theme::text_primary(profile)}",
                                                "Best Sell"
                                            }
                                        }
                                    }
                                }
                                td {
                                    class: "px-4 py-3 text-right {theme::text_secondary(profile)}",
                                    div { class: "flex flex-col items-end gap-1 text-xs",
                                        span { class: "text-sm font-medium", "{format_price_range(row.buy_price_min, row.buy_price_max)}" }
                                        if is_best_buy {
                                            span {
                                                class: "rounded-full border border-[#5c2a1f] px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide {theme::text_primary(profile)}",
                                                "Best Buy"
                                            }
                                        }
                                    }
                                }
                                td { class: "px-4 py-3 text-right {theme::text_secondary(profile)}", "{format_stock(row.stock)}" }
                                td {
                                    class: "px-4 py-3 text-right {theme::text_secondary(profile)}",
                                    DemandCell {
                                        sell: row.status_sell,
                                        buy: row.status_buy,
                                        profile: profile,
                                    }
                                }
                                td { class: "px-4 py-3 text-right {theme::text_secondary(profile)} whitespace-nowrap min-w-[150px]", "{format_containers(&row.container_sizes)}" }
                                td { class: "px-4 py-3 {theme::text_muted(profile)}", "{row.updated_label}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn row_hover_class(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "hover:bg-[#3b1712]/30",
        _ => "hover:bg-slate-800/40",
    }
}

fn format_stock(value: Option<f64>) -> String {
    match value {
        Some(v) if v.is_finite() && v > 0.0 => {
            if v >= 1000.0 {
                format!("{:.1}k", v / 1000.0)
            } else {
                format!("{:.0}", v)
            }
        }
        _ => "—".to_string(),
    }
}

fn status_label(value: Option<i32>) -> Option<&'static str> {
    match value {
        Some(3) => Some("High"),
        Some(2) => Some("Normal"),
        Some(1) => Some("Low"),
        Some(0) => Some("Unavailable"),
        _ => None,
    }
}

fn format_containers(sizes: &[f64]) -> String {
    if sizes.is_empty() {
        return "—".to_string();
    }

    let formatted: Vec<String> = sizes
        .iter()
        .filter(|size| size.is_finite() && **size > 0.0)
        .map(|size| format!("{size:.0}"))
        .collect();

    if formatted.is_empty() {
        "—".to_string()
    } else {
        formatted.join(", ")
    }
}

fn format_price_range(min: Option<f64>, max: Option<f64>) -> String {
    let min = usable(min);
    let max = usable(max);

    match (min, max) {
        (Some(min), Some(max)) if (max - min).abs() < f64::EPSILON => format!("{min:.0}"),
        (Some(min), Some(max)) => format!("{min:.0} - {max:.0}"),
        (Some(min), None) => format!("≥ {min:.0}"),
        (None, Some(max)) => format!("≤ {max:.0}"),
        _ => "—".to_string(),
    }
}

fn usable(value: Option<f64>) -> Option<f64> {
    value.filter(|v| v.is_finite() && *v > 0.0)
}

#[derive(Default)]
struct PriceHighlights {
    best_sell: Option<(String, f64)>,
    best_buy: Option<(String, f64)>,
    sell_range: Option<(f64, f64)>,
    buy_range: Option<(f64, f64)>,
}

impl PriceHighlights {
    fn has_data(&self) -> bool {
        self.best_sell.is_some()
            || self.best_buy.is_some()
            || self.sell_range.is_some()
            || self.buy_range.is_some()
    }
}

fn summarize_price_rows(rows: &[PriceRow]) -> PriceHighlights {
    let mut highlights = PriceHighlights::default();

    for row in rows {
        extend_range(&mut highlights.sell_range, row.sell_price_min);
        extend_range(&mut highlights.sell_range, row.sell_price_max);
        extend_range(&mut highlights.buy_range, row.buy_price_min);
        extend_range(&mut highlights.buy_range, row.buy_price_max);

        if let Some(value) = usable(row.sell_price_max).or_else(|| usable(row.sell_price_min)) {
            match highlights.best_sell {
                Some((_, best)) if best >= value => {}
                _ => highlights.best_sell = Some((row.location.clone(), value)),
            }
        }

        if let Some(value) = usable(row.buy_price_min).or_else(|| usable(row.buy_price_max)) {
            match highlights.best_buy {
                Some((_, best)) if best <= value => {}
                _ => highlights.best_buy = Some((row.location.clone(), value)),
            }
        }
    }

    highlights
}

fn extend_range(range: &mut Option<(f64, f64)>, candidate: Option<f64>) {
    if let Some(value) = usable(candidate) {
        match range {
            Some((min, max)) => {
                if value < *min {
                    *min = value;
                }
                if value > *max {
                    *max = value;
                }
            }
            None => {
                *range = Some((value, value));
            }
        }
    }
}

fn format_summary_range(range: Option<(f64, f64)>) -> String {
    match range {
        Some((min, max)) if (max - min).abs() < f64::EPSILON => format!("{min:.0} aUEC"),
        Some((min, max)) => format!("{min:.0} - {max:.0} aUEC"),
        None => "—".to_string(),
    }
}

#[derive(Props, Clone, PartialEq)]
struct DemandCellProps {
    sell: Option<i32>,
    buy: Option<i32>,
    profile: Profile,
}

#[component]
fn DemandCell(props: DemandCellProps) -> Element {
    let sell = status_label(props.sell).unwrap_or("—");
    let buy = status_label(props.buy).unwrap_or("—");
    rsx! {
        div { class: "flex flex-col items-end gap-0.5 text-xs",
            span { class: "{theme::text_muted(props.profile)}", "Sell: {sell}" }
            span { class: "{theme::text_muted(props.profile)}", "Buy: {buy}" }
        }
    }
}

fn pin_highlight_rows(rows: &mut Vec<(PriceRow, bool, bool)>) {
    let mut next_slot = 0;
    if let Some(idx) = rows
        .iter()
        .enumerate()
        .find(|(_, (_, is_best_sell, _))| *is_best_sell)
        .map(|(idx, _)| idx)
    {
        rows.swap(next_slot, idx);
        next_slot += 1;
    }

    if let Some(idx) = rows
        .iter()
        .enumerate()
        .skip(next_slot)
        .find(|(_, (_, _, is_best_buy))| *is_best_buy)
        .map(|(idx, _)| idx)
    {
        rows.swap(next_slot, idx);
    }
}

#[derive(Props, Clone, PartialEq)]
struct SummaryStatProps {
    title: String,
    value: String,
    caption: String,
    profile: Profile,
}

#[component]
fn SummaryStat(props: SummaryStatProps) -> Element {
    rsx! {
        div {
            class: "{theme::panel_solid(props.profile)} p-3",
            p { class: "text-[10px] font-semibold uppercase tracking-wide {theme::text_muted(props.profile)}", "{props.title}" }
            p { class: "text-lg font-semibold {theme::text_secondary(props.profile)}", "{props.value}" }
            p { class: "text-xs {theme::text_muted(props.profile)}", "{props.caption}" }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortMode {
    SellRange,
    BuyRange,
    Stock,
    Demand,
}

fn sort_button_class(active: bool, profile: Profile) -> &'static str {
    if active {
        theme::btn_small_active(profile)
    } else {
        theme::btn_small_inactive(profile)
    }
}

fn sort_rows(rows: &mut Vec<(PriceRow, bool, bool)>, mode: SortMode) {
    match mode {
        SortMode::SellRange => {
            rows.sort_by(|a, b| compare_f64_desc(best_sell_value(&a.0), best_sell_value(&b.0)))
        }
        SortMode::BuyRange => {
            rows.sort_by(|a, b| compare_f64_asc(best_buy_value(&a.0), best_buy_value(&b.0)))
        }
        SortMode::Stock => rows.sort_by(|a, b| compare_f64_desc(a.0.stock, b.0.stock)),
        SortMode::Demand => rows.sort_by(|a, b| compare_demand(&a.0, &b.0)),
    }
}

fn best_sell_value(row: &PriceRow) -> Option<f64> {
    row.sell_price_max
        .or(row.sell_price_min)
        .filter(|v| v.is_finite() && *v > 0.0)
}

fn best_buy_value(row: &PriceRow) -> Option<f64> {
    row.buy_price_min
        .or(row.buy_price_max)
        .filter(|v| v.is_finite() && *v > 0.0)
}

fn compare_f64_desc(a: Option<f64>, b: Option<f64>) -> Ordering {
    match (a, b) {
        (Some(av), Some(bv)) => bv.partial_cmp(&av).unwrap_or(Ordering::Equal),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_f64_asc(a: Option<f64>, b: Option<f64>) -> Ordering {
    match (a, b) {
        (Some(av), Some(bv)) => av.partial_cmp(&bv).unwrap_or(Ordering::Equal),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_demand(a: &PriceRow, b: &PriceRow) -> Ordering {
    let (sell_a, buy_a) = demand_score(a);
    let (sell_b, buy_b) = demand_score(b);
    match sell_b.cmp(&sell_a) {
        Ordering::Equal => buy_a.cmp(&buy_b),
        other => other,
    }
}

fn demand_score(row: &PriceRow) -> (i32, i32) {
    let sell = row.status_sell.unwrap_or(-1);
    let buy = row.status_buy.unwrap_or(5);
    (sell, buy)
}
