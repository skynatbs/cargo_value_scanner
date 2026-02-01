//! Sell Planner — find optimal sell locations for your cargo.

use std::collections::{HashMap, HashSet};

use dioxus::prelude::*;

use crate::domain::{AppState, CargoItem, Location, PricePoint, Terminal};
use crate::infra::uex::UexClient;
use crate::ui::theme;

/// Extract unique locations from terminals.
fn extract_locations(terminals: &[Terminal]) -> Vec<Location> {
    let mut seen: HashMap<String, Location> = HashMap::new();
    
    for t in terminals {
        let name = t.location_name();
        if !seen.contains_key(&name) {
            seen.insert(name.clone(), Location {
                name,
                system: t.system.clone(),
                terminal_id: t.id,
            });
        }
    }
    
    let mut locations: Vec<Location> = seen.into_values().collect();
    locations.sort_by(|a, b| {
        match (&a.system, &b.system) {
            (Some(sa), Some(sb)) => sa.cmp(sb).then(a.name.cmp(&b.name)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.name.cmp(&b.name),
        }
    });
    locations
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PlannerMode {
    #[default]
    OneStop,
    BestValue,
}

#[derive(Clone, Debug)]
pub struct SellPlan {
    pub stops: Vec<SellStop>,
    pub total_value: f64,
    pub total_distance: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SellStop {
    pub terminal_name: String,
    pub terminal_id: Option<i32>,
    pub system: Option<String>,
    pub items: Vec<SellItem>,
    pub stop_value: f64,
    pub is_nqa: bool,
    pub distance_from_prev: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SellItem {
    pub commodity_name: String,
    pub commodity_id: String,
    pub scu: u32,
    pub price_per_unit: f64,
    pub total_value: f64,
    pub available_stock: Option<f64>,
}

// Note: available_stock is for BUY planning (how much terminal sells), not relevant for SELL planning

#[component]
pub fn PlannerPage() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    
    let profile = state.with(|st| st.profile);
    let items = state.with(|st| st.cargo_items.clone());
    let price_map = state.with(|st| st.price_points.clone());
    let nqa_terminal_ids = state.with(|st| st.nqa_terminal_ids.clone());
    
    let mut mode = use_signal(|| PlannerMode::OneStop);
    let mut current_position = use_signal(|| None::<i32>);
    let mut position_query = use_signal(String::new);
    let mut dropdown_open = use_signal(|| false);
    
    // Load terminals
    let terminals_resource = use_resource(move || async move {
        let client = UexClient::new().ok()?;
        let cache = client.get_terminals().await.ok()?;
        Some(cache.terminals)
    });
    
    let terminals_loading = terminals_resource.read().is_none();
    let terminals = terminals_resource.read()
        .as_ref()
        .and_then(|t| t.as_ref())
        .cloned()
        .unwrap_or_default();
    
    let locations = extract_locations(&terminals);
    
    // Filter locations for autocomplete
    let query = position_query();
    let filtered_locations: Vec<Location> = if query.is_empty() {
        locations.clone()
    } else {
        let q = query.to_lowercase();
        locations.iter()
            .filter(|l| l.name.to_lowercase().contains(&q))
            .cloned()
            .collect()
    };
    
    let has_cargo = !items.is_empty();
    let has_hot_cargo = items.iter().any(|item| item.is_hot);
    let has_position = current_position().is_some();
    
    // Calculate BOTH plans for comparison
    let one_stop_plan = if has_cargo {
        Some(calculate_one_stop_plan(&items, &price_map, &nqa_terminal_ids))
    } else {
        None
    };
    
    let best_value_plan = if has_cargo {
        Some(calculate_best_value_plan(&items, &price_map, &nqa_terminal_ids))
    } else {
        None
    };
    
    let base_plan = match mode() {
        PlannerMode::OneStop => one_stop_plan.clone(),
        PlannerMode::BestValue => best_value_plan.clone(),
    };
    
    // Calculate comparison (how much more Best Value earns)
    let value_comparison = match (&one_stop_plan, &best_value_plan) {
        (Some(one), Some(best)) if one.total_value > 0.0 => {
            let diff = best.total_value - one.total_value;
            let pct = (diff / one.total_value) * 100.0;
            Some((diff, pct))
        }
        _ => None,
    };
    
    // Load distances when position is set
    let current_pos = current_position();
    let current_mode = mode();
    let plan_stops: Vec<i32> = base_plan.as_ref()
        .map(|p| p.stops.iter().filter_map(|s| s.terminal_id).collect())
        .unwrap_or_default();
    
    let distances_resource = use_resource(move || {
        let stops = plan_stops.clone();
        async move {
            let Some(origin_id) = current_pos else { return None };
            if stops.is_empty() { return None; }
            
            let client = UexClient::new().ok()?;
            let distances = client.get_terminal_distances(origin_id, &stops).await.ok()?;
            Some((origin_id, distances))
        }
    });
    
    let distances_loading = current_pos.is_some() && distances_resource.read().is_none();
    
    // Combine base plan with distances
    let final_plan: Option<SellPlan> = base_plan.map(|mut plan| {
        if let Some(Some((origin_id, ref distances))) = distances_resource.read().as_ref() {
            if current_mode == PlannerMode::BestValue {
                plan = sort_by_nearest_neighbor(plan, *origin_id, distances);
            } else {
                plan = add_distances_to_plan(plan, *origin_id, distances);
            }
        }
        plan
    });

    let selected_location_name = current_position()
        .and_then(|id| locations.iter().find(|l| l.terminal_id == id))
        .map(|l| l.name.clone());

    rsx! {
        div { class: "space-y-6",
            // Header
            section {
                class: "flex flex-wrap items-center justify-between gap-4",
                div {
                    h2 { class: "text-xl font-semibold {theme::text_secondary(profile)}", "Sell Planner" }
                    p { class: "text-sm {theme::text_muted(profile)}", "Find the best places to sell your cargo" }
                }
                if has_cargo {
                    div { class: "flex gap-2",
                        button {
                            class: mode_button_class(mode() == PlannerMode::OneStop, profile),
                            onclick: move |_| mode.set(PlannerMode::OneStop),
                            "🎯 One Stop"
                        }
                        button {
                            class: mode_button_class(mode() == PlannerMode::BestValue, profile),
                            onclick: move |_| mode.set(PlannerMode::BestValue),
                            div { class: "flex items-center gap-2",
                                span { "💎 Best Value" }
                                // Show comparison badge
                                if let Some((diff, pct)) = value_comparison {
                                    if diff > 0.0 {
                                        span {
                                            class: "rounded bg-emerald-500/20 px-1.5 py-0.5 text-[10px] font-semibold text-emerald-300",
                                            "+{pct:.0}%"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Position selector with autocomplete
            if has_cargo {
                div {
                    class: "{theme::panel_border(profile)} p-4",
                    label { class: "{theme::label_class(profile)} mb-2",
                        "📍 Current Position"
                        if terminals_loading {
                            span { class: "ml-2 {theme::text_primary(profile)} animate-pulse", "Loading stations..." }
                        }
                    }
                    div { class: "relative",
                        div { class: "flex gap-3 items-center",
                            input {
                                class: "flex-1 {theme::input_small(profile)}",
                                placeholder: "Search station (e.g. Baijini Point, Everus Harbor)...",
                                value: "{position_query}",
                                onfocus: move |_| dropdown_open.set(true),
                                oninput: move |evt| {
                                    position_query.set(evt.value());
                                    dropdown_open.set(true);
                                },
                            }
                            if has_position {
                                button {
                                    class: "{theme::btn_inactive(profile)}",
                                    onclick: move |_| {
                                        current_position.set(None);
                                        position_query.set(String::new());
                                    },
                                    "✕"
                                }
                            }
                        }
                        // Autocomplete dropdown
                        if dropdown_open() && !filtered_locations.is_empty() {
                            div {
                                class: "absolute z-50 mt-1 max-h-64 w-full overflow-y-auto {theme::panel_solid(profile)} shadow-xl",
                                for loc in filtered_locations.iter().take(20) {
                                    button {
                                        class: "w-full px-3 py-2 text-left text-sm hover:bg-[#3b1712]/50 flex items-center justify-between",
                                        onclick: {
                                            let loc = loc.clone();
                                            move |_| {
                                                position_query.set(loc.name.clone());
                                                current_position.set(Some(loc.terminal_id));
                                                dropdown_open.set(false);
                                            }
                                        },
                                        span { class: "{theme::text_secondary(profile)}", "{loc.name}" }
                                        span { class: "text-xs {theme::text_muted(profile)}", "{loc.system.as_deref().unwrap_or(\"\")}" }
                                    }
                                }
                            }
                        }
                    }
                    if let Some(name) = selected_location_name {
                        p { class: "mt-2 text-xs {theme::text_primary(profile)}", 
                            "✓ Position: {name}"
                            if distances_loading {
                                span { class: "ml-2 {theme::text_primary(profile)} animate-pulse", "Calculating route..." }
                            }
                        }
                    }
                }
            }

            // Warnings
            if has_hot_cargo {
                div {
                    class: "rounded-lg border border-orange-500/30 bg-orange-500/10 px-4 py-3 text-sm text-orange-200",
                    span { class: "mr-2", "🔥" }
                    "Hot cargo detected. Only showing no-questions-asked terminals."
                }
            }
            
            // No cargo state
            if !has_cargo {
                div {
                    class: "{theme::panel_border(profile)} px-6 py-12 text-center",
                    p { class: "{theme::text_muted(profile)}", "No cargo to plan. Add items on the Cargo page first." }
                }
            }

            // Plan results
            if let Some(plan) = final_plan {
                // Comparison card (only show if there's a meaningful difference)
                if let Some((diff, pct)) = value_comparison {
                    if diff.abs() > 100.0 {
                        div {
                            class: "{theme::panel_border(profile)} p-4",
                            div { class: "flex items-center justify-between text-sm",
                                div { class: "flex gap-6",
                                    div {
                                        span { class: "{theme::text_muted(profile)}", "🎯 One Stop: " }
                                        span { class: if mode() == PlannerMode::OneStop { "{theme::text_primary(profile)} font-semibold" } else { "{theme::text_secondary(profile)}" },
                                            "{format_auec(one_stop_plan.as_ref().map(|p| p.total_value).unwrap_or(0.0))}"
                                        }
                                    }
                                    div {
                                        span { class: "{theme::text_muted(profile)}", "💎 Best Value: " }
                                        span { class: if mode() == PlannerMode::BestValue { "{theme::text_primary(profile)} font-semibold" } else { "{theme::text_secondary(profile)}" },
                                            "{format_auec(best_value_plan.as_ref().map(|p| p.total_value).unwrap_or(0.0))}"
                                        }
                                    }
                                }
                                if diff > 0.0 {
                                    span { class: "{theme::text_primary(profile)} text-xs",
                                        "Best Value earns +{format_auec(diff)} ({pct:.1}% more)"
                                    }
                                }
                            }
                        }
                    }
                }

                // Summary card
                div {
                    class: "{theme::panel_border(profile)} p-6",
                    div { class: "flex flex-wrap items-center justify-between gap-4",
                        div {
                            p { class: "{theme::label_class(profile)}", 
                                match mode() {
                                    PlannerMode::OneStop => "Best Single Location",
                                    PlannerMode::BestValue => "Multi-Stop Route",
                                }
                            }
                            p { class: "text-3xl font-bold {theme::text_primary(profile)}", 
                                "{format_auec(plan.total_value)}" 
                            }
                        }
                        div { class: "text-right text-sm {theme::text_muted(profile)}",
                            p { "{plan.stops.len()} stop(s)" }
                            p { "{items.len()} item(s)" }
                            if let Some(dist) = plan.total_distance {
                                p { class: "{theme::text_primary(profile)}", "📏 {dist:.0} Gm total" }
                            }
                        }
                    }
                }

                // Stops list
                div { class: "space-y-4",
                    for (idx, stop) in plan.stops.iter().enumerate() {
                        StopCard {
                            key: "{idx}",
                            stop: stop.clone(),
                            stop_number: idx + 1,
                            show_number: plan.stops.len() > 1,
                            profile: profile,
                            on_mark_sold: move |sold_items: Vec<String>| {
                                state.with_mut(|st| {
                                    st.cargo_items.retain(|item| !sold_items.contains(&item.id));
                                });
                            },
                        }
                    }
                }

                // Empty plan
                if plan.stops.is_empty() {
                    div {
                        class: "{theme::panel_border(profile)} px-6 py-12 text-center",
                        p { class: "{theme::text_muted(profile)}", "No sell locations found. Try loading prices for your cargo items." }
                    }
                }

                // Multi-stop summary
                if plan.stops.len() > 1 {
                    div {
                        class: "{theme::panel_border(profile)} p-6",
                        div { class: "flex flex-wrap items-center justify-between gap-4",
                            div {
                                p { class: "{theme::label_class(profile)}", 
                                    "💰 Total Profit"
                                }
                                p { class: "text-4xl font-bold {theme::text_primary(profile)}", 
                                    "{format_auec(plan.total_value)}" 
                                }
                            }
                            div { class: "text-right text-sm {theme::text_muted(profile)}",
                                p { "{plan.stops.len()} stops" }
                                p { "{items.iter().map(|i| i.scu).sum::<u32>()} SCU total" }
                                if let Some(dist) = plan.total_distance {
                                    p { class: "{theme::text_primary(profile)}", "📏 {dist:.0} Gm route" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn StopCard(
    stop: SellStop,
    stop_number: usize,
    show_number: bool,
    profile: crate::domain::Profile,
    on_mark_sold: EventHandler<Vec<String>>,
) -> Element {
    let location_display = stop.system
        .as_ref()
        .map(|sys| format!("{} · {}", stop.terminal_name, sys))
        .unwrap_or_else(|| stop.terminal_name.clone());

    let item_ids: Vec<String> = stop.items.iter().map(|i| i.commodity_id.clone()).collect();

    rsx! {
        div {
            class: "{theme::table_container(profile)}",
            // Header
            div {
                class: "{theme::table_header(profile)} flex items-center justify-between px-4 py-3",
                div { class: "flex items-center gap-3",
                    if show_number {
                        span {
                            class: "flex h-7 w-7 items-center justify-center rounded-full bg-[#3b1712] text-sm font-bold {theme::text_primary(profile)}",
                            "{stop_number}"
                        }
                    }
                    div {
                        div { class: "flex items-center gap-2",
                            if stop.is_nqa {
                                span {
                                    class: "rounded bg-[#3b1712] px-1.5 py-0.5 text-[10px] font-semibold {theme::text_primary(profile)}",
                                    "🏴‍☠️"
                                }
                            }
                            span { class: "font-semibold {theme::text_secondary(profile)}", "{location_display}" }
                        }
                        if let Some(dist) = stop.distance_from_prev {
                            p { class: "text-xs {theme::text_muted(profile)}", "📏 {dist:.0} Gm" }
                        }
                    }
                }
                div { class: "flex items-center gap-3",
                    span { class: "text-lg font-bold {theme::text_primary(profile)}", "{format_auec(stop.stop_value)}" }
                    button {
                        class: "{theme::btn_small_inactive(profile)} transition-colors",
                        title: "Mark as sold",
                        onclick: move |_| on_mark_sold.call(item_ids.clone()),
                        "✓ Sold"
                    }
                }
            }
            // Items
            div { class: "{theme::table_divider(profile)}",
                for item in &stop.items {
                    div {
                        class: "flex items-center justify-between px-4 py-2 text-sm",
                        div {
                            span { class: "{theme::text_secondary(profile)}", "{item.commodity_name}" }
                            span { class: "ml-2 {theme::text_muted(profile)}", "× {item.scu} SCU" }
                        }
                        div { class: "text-right",
                            span { class: "{theme::text_secondary(profile)}", "{format_auec(item.total_value)}" }
                            span { class: "ml-2 text-xs {theme::text_muted(profile)}", "@ {item.price_per_unit:.0}/SCU" }
                        }
                    }
                }
            }
        }
    }
}

fn mode_button_class(active: bool, profile: crate::domain::Profile) -> &'static str {
    if active {
        theme::btn_active(profile)
    } else {
        theme::btn_inactive(profile)
    }
}

fn format_auec(value: f64) -> String {
    let rounded = value.round() as i64;
    if rounded.abs() >= 1_000_000 {
        format!("{:.1}M aUEC", value / 1_000_000.0)
    } else if rounded.abs() >= 1_000 {
        let s = format!("{}", rounded);
        let mut result = String::new();
        for (i, c) in s.chars().rev().enumerate() {
            if i > 0 && i % 3 == 0 && c != '-' {
                result.push(',');
            }
            result.push(c);
        }
        format!("{} aUEC", result.chars().rev().collect::<String>())
    } else {
        format!("{} aUEC", rounded)
    }
}

fn calculate_one_stop_plan(
    items: &[CargoItem],
    price_map: &HashMap<String, Vec<PricePoint>>,
    nqa_terminal_ids: &HashSet<i32>,
) -> SellPlan {
    let mut terminal_values: HashMap<String, (Option<i32>, Option<String>, f64, Vec<SellItem>, bool)> = 
        HashMap::new();

    for item in items {
        let Some(prices) = price_map.get(&item.commodity_id) else { continue };

        for point in prices {
            if item.is_hot {
                let is_nqa = point.terminal_id
                    .map(|id| nqa_terminal_ids.contains(&id))
                    .unwrap_or(false);
                if !is_nqa { continue; }
            }

            let Some(price) = best_sell_price(point) else { continue };
            let item_value = price * item.scu as f64;
            let is_nqa = point.terminal_id
                .map(|id| nqa_terminal_ids.contains(&id))
                .unwrap_or(false);

            let entry = terminal_values
                .entry(point.terminal_name.clone())
                .or_insert_with(|| (point.terminal_id, point.system.clone(), 0.0, Vec::new(), is_nqa));

            entry.2 += item_value;
            entry.3.push(SellItem {
                commodity_name: item.commodity_name.clone(),
                commodity_id: item.id.clone(),
                scu: item.scu,
                price_per_unit: price,
                total_value: item_value,
                available_stock: point.scu_sell_stock,
            });
        }
    }

    let best = terminal_values
        .into_iter()
        .max_by(|a, b| a.1.2.partial_cmp(&b.1.2).unwrap_or(std::cmp::Ordering::Equal));

    match best {
        Some((terminal_name, (terminal_id, system, total, sell_items, is_nqa))) => SellPlan {
            stops: vec![SellStop {
                terminal_name,
                terminal_id,
                system,
                items: sell_items,
                stop_value: total,
                is_nqa,
                distance_from_prev: None,
            }],
            total_value: total,
            total_distance: None,
        },
        None => SellPlan {
            stops: vec![],
            total_value: 0.0,
            total_distance: None,
        },
    }
}

fn calculate_best_value_plan(
    items: &[CargoItem],
    price_map: &HashMap<String, Vec<PricePoint>>,
    nqa_terminal_ids: &HashSet<i32>,
) -> SellPlan {
    let mut stops_map: HashMap<String, SellStop> = HashMap::new();
    let mut total_value = 0.0;

    for item in items {
        let Some(prices) = price_map.get(&item.commodity_id) else { continue };

        let best = prices
            .iter()
            .filter(|point| {
                if item.is_hot {
                    point.terminal_id
                        .map(|id| nqa_terminal_ids.contains(&id))
                        .unwrap_or(false)
                } else {
                    true
                }
            })
            .filter_map(|point| {
                let price = best_sell_price(point)?;
                let is_nqa = point.terminal_id
                    .map(|id| nqa_terminal_ids.contains(&id))
                    .unwrap_or(false);
                Some((point, price, is_nqa))
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((point, price, is_nqa)) = best {
            let item_value = price * item.scu as f64;
            total_value += item_value;

            let stop = stops_map
                .entry(point.terminal_name.clone())
                .or_insert_with(|| SellStop {
                    terminal_name: point.terminal_name.clone(),
                    terminal_id: point.terminal_id,
                    system: point.system.clone(),
                    items: Vec::new(),
                    stop_value: 0.0,
                    is_nqa,
                    distance_from_prev: None,
                });

            stop.stop_value += item_value;
            stop.items.push(SellItem {
                commodity_name: item.commodity_name.clone(),
                commodity_id: item.id.clone(),
                scu: item.scu,
                price_per_unit: price,
                total_value: item_value,
                available_stock: point.scu_sell_stock,
            });
        }
    }

    let mut stops: Vec<SellStop> = stops_map.into_values().collect();
    stops.sort_by(|a, b| b.stop_value.partial_cmp(&a.stop_value).unwrap_or(std::cmp::Ordering::Equal));

    SellPlan {
        stops,
        total_value,
        total_distance: None,
    }
}

fn sort_by_nearest_neighbor(
    mut plan: SellPlan,
    origin_id: i32,
    distances: &HashMap<i32, f64>,
) -> SellPlan {
    if plan.stops.is_empty() { return plan; }

    let mut sorted_stops: Vec<SellStop> = Vec::new();
    let mut remaining: Vec<SellStop> = plan.stops;
    let mut _current_pos = origin_id;
    let mut total_distance = 0.0;

    while !remaining.is_empty() {
        let (nearest_idx, nearest_dist) = remaining
            .iter()
            .enumerate()
            .filter_map(|(idx, stop)| {
                let dist = stop.terminal_id
                    .and_then(|id| distances.get(&id).copied())
                    .unwrap_or(f64::MAX);
                Some((idx, dist))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or((0, 0.0));

        let mut stop = remaining.remove(nearest_idx);
        stop.distance_from_prev = Some(nearest_dist);
        total_distance += nearest_dist;
        
        if let Some(id) = stop.terminal_id {
            _current_pos = id;
        }
        
        sorted_stops.push(stop);
    }

    plan.stops = sorted_stops;
    plan.total_distance = Some(total_distance);
    plan
}

fn add_distances_to_plan(
    mut plan: SellPlan,
    _origin_id: i32,
    distances: &HashMap<i32, f64>,
) -> SellPlan {
    let mut total_distance = 0.0;
    
    for stop in &mut plan.stops {
        if let Some(id) = stop.terminal_id {
            if let Some(&dist) = distances.get(&id) {
                stop.distance_from_prev = Some(dist);
                total_distance += dist;
            }
        }
    }
    
    plan.total_distance = Some(total_distance);
    plan
}

fn best_sell_price(point: &PricePoint) -> Option<f64> {
    point.price_sell_max
        .or(point.price_sell)
        .or(point.price_average)
        .or(point.price_sell_min)
        .filter(|p| p.is_finite() && *p > 0.0)
}
