//! Trade Routes page — find profitable trade routes.

use dioxus::prelude::*;

use crate::domain::{
    calculate_routes_for_commodity, sort_routes, AppState, Profile, TradeRoute, TradeRouteSort,
};
use crate::infra::cache::{load_routes_cache, save_routes_cache, RoutesCache};
use crate::infra::uex::UexClient;

// ============================================
// THEME HELPERS - Manufacturer-specific styles
// ============================================

fn btn_active(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "rounded-lg px-5 py-2.5 text-sm font-semibold bg-[#3b1712] text-[#ff9900] border border-[#5c2a1f] drake-glow drake-flicker",
        Profile::Trader => "rounded-lg px-5 py-2.5 text-sm font-semibold bg-sky-500/20 text-sky-300 border border-sky-500/40 misc-glow",
        Profile::Miner => "rounded-lg px-5 py-2.5 text-sm font-semibold bg-orange-500/20 text-orange-300 border border-orange-500/40 argo-glow",
        Profile::None => "rounded-lg px-5 py-2.5 text-sm font-semibold bg-indigo-500/20 text-indigo-300 border border-indigo-500/40",
    }
}

fn btn_inactive(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "rounded-lg px-5 py-2.5 text-sm text-[#d4523a]/70 border border-[#3b1712] hover:border-[#5c2a1f] hover:text-[#ff9900]",
        Profile::Trader => "rounded-lg px-5 py-2.5 text-sm text-slate-400 border border-slate-700 hover:border-sky-600 hover:text-sky-300",
        Profile::Miner => "rounded-lg px-5 py-2.5 text-sm text-slate-400 border border-slate-700 hover:border-orange-600 hover:text-orange-300",
        Profile::None => "rounded-lg px-5 py-2.5 text-sm text-slate-400 border border-slate-700 hover:border-slate-600",
    }
}

fn btn_small_active(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "rounded px-2 py-1 text-xs font-semibold bg-[#3b1712] text-[#ff9900] border border-[#5c2a1f]",
        Profile::Trader => "rounded px-2 py-1 text-xs font-semibold bg-sky-500/20 text-sky-300 border border-sky-500/40",
        Profile::Miner => "rounded px-2 py-1 text-xs font-semibold bg-orange-500/20 text-orange-300 border border-orange-500/40",
        Profile::None => "rounded px-2 py-1 text-xs font-semibold bg-indigo-500/20 text-indigo-300 border border-indigo-500/40",
    }
}

fn btn_small_inactive(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "rounded px-2 py-1 text-xs text-[#d4523a]/60 border border-[#3b1712] hover:border-[#5c2a1f] hover:text-[#ff9900]",
        Profile::Trader => "rounded px-2 py-1 text-xs text-slate-500 border border-slate-700 hover:border-sky-600 hover:text-sky-300",
        Profile::Miner => "rounded px-2 py-1 text-xs text-slate-500 border border-slate-700 hover:border-orange-600 hover:text-orange-300",
        Profile::None => "rounded px-2 py-1 text-xs text-slate-500 border border-slate-700 hover:border-slate-600 hover:text-slate-300",
    }
}

fn input_class(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "rounded-lg border border-[#3b1712] bg-[#1a0a08] px-4 py-2.5 text-sm text-[#d4523a] focus:border-[#ff9900] focus:outline-none",
        Profile::Trader => "rounded-lg border border-slate-700 bg-slate-950 px-4 py-2.5 text-sm text-slate-100 focus:border-sky-500 focus:outline-none",
        Profile::Miner => "rounded-lg border border-slate-700 bg-slate-950 px-4 py-2.5 text-sm text-slate-100 focus:border-orange-500 focus:outline-none",
        Profile::None => "rounded-lg border border-slate-700 bg-slate-950 px-4 py-2.5 text-sm text-slate-100 focus:border-indigo-500 focus:outline-none",
    }
}

fn panel_border(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "rounded-xl border border-[#5c2a1f] bg-[#3b1712]/40",
        Profile::Trader => "rounded-xl border border-sky-800/50 bg-slate-900/40",
        Profile::Miner => "rounded-xl border border-orange-800/50 bg-slate-900/40",
        Profile::None => "rounded-xl border border-slate-800 bg-slate-900/40",
    }
}

fn table_container(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "rounded-xl border border-[#3b1712] bg-[#1a0a08]/60 overflow-hidden",
        Profile::Trader => "rounded-xl border border-sky-900/40 bg-slate-900/40 overflow-hidden",
        Profile::Miner => "rounded-xl border border-orange-900/40 bg-slate-900/40 overflow-hidden",
        Profile::None => "rounded-xl border border-slate-800 bg-slate-900/40 overflow-hidden",
    }
}

fn table_header(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "border-b border-[#3b1712] bg-[#3b1712]/50 text-xs uppercase text-[#d4523a]/80",
        Profile::Trader => "border-b border-sky-900/40 bg-sky-950/30 text-xs uppercase text-sky-400/70",
        Profile::Miner => "border-b border-orange-900/40 bg-orange-950/30 text-xs uppercase text-orange-400/70",
        Profile::None => "border-b border-slate-800 bg-slate-900/60 text-xs uppercase text-slate-500",
    }
}

fn table_divider(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "divide-y divide-[#3b1712]/60",
        Profile::Trader => "divide-y divide-sky-900/30",
        Profile::Miner => "divide-y divide-orange-900/30",
        Profile::None => "divide-y divide-slate-800",
    }
}

/// Trade scope: within one system or across systems.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TradeScope {
    #[default]
    Stellar,      // same system only
    Interstellar, // cross-system allowed
}

#[component]
pub fn RoutesPage() -> Element {
    let state = use_context::<Signal<AppState>>();
    let nqa_terminal_ids = state.with(|s| s.nqa_terminal_ids.clone());
    let profile = state.with(|s| s.profile);
    
    let is_pirate = profile == Profile::Pirate;
    
    // Different defaults for pirate vs trader
    let default_sort = if is_pirate { TradeRouteSort::CargoValue } else { TradeRouteSort::ProfitPerGm };
    
    let mut sort_by = use_signal(move || default_sort);
    let mut scu_input = use_signal(|| 100u32);
    let max_scu: u32 = 10000; // Hull-C cap is ~4600
    let mut max_invest_input = use_signal(|| String::new());
    let mut only_high_value = use_signal(|| is_pirate);
    let mut trade_scope = use_signal(|| TradeScope::Stellar);
    let mut stations_only = use_signal(|| false); // filter out planetary locations
    let mut selected_route = use_signal(|| None::<TradeRoute>); // for detail panel
    let mut system_filter = use_signal(|| String::new()); // empty = all systems
    let mut force_refresh = use_signal(|| false);
    
    // Load routes (with caching - 24h TTL)
    let routes_resource = use_resource(move || {
        let nqa = nqa_terminal_ids.clone();
        let refresh = force_refresh();
        async move {
            // Try to load from cache first (unless force refresh)
            if !refresh {
                if let Some(cache) = load_routes_cache() {
                    println!("[routes] Using cached routes ({} routes, age: {})", cache.routes.len(), cache.age_string());
                    return Some(cache.routes);
                }
            }
            
            // Cache miss or expired - fetch fresh data
            println!("[routes] Fetching fresh route data from API...");
            
            let client = UexClient::new().ok()?;
            
            // Get commodities
            let commodities_result = client.get_commodities().await.ok()?;
            let commodities = commodities_result.data;
            
            // Get ALL trade commodities (not just 25)
            let trade_commodities: Vec<_> = commodities.iter()
                .filter(|c| matches!(c.category.as_str(), 
                    "Agricultural" | "Food" | "Gas" | "Medical" | "Metal" | 
                    "Mineral" | "Scrap" | "Vice" | "Drug" | "Hallucinogen"))
                .collect();
            
            println!("[routes] Loading prices for {} commodities...", trade_commodities.len());
            
            let mut all_routes = Vec::new();
            
            for commodity in trade_commodities {
                if let Ok(prices_result) = client.get_prices(&commodity.id, Some(&commodity.name)).await {
                    let routes = calculate_routes_for_commodity(
                        &commodity.id,
                        &commodity.name,
                        false, // TODO: get is_illegal from commodity
                        &prices_result.data,
                        &nqa,
                    );
                    all_routes.extend(routes);
                }
            }
            
            println!("[routes] Calculated {} total routes", all_routes.len());
            
            // Save to cache
            let cache = RoutesCache::new(all_routes.clone());
            let _ = save_routes_cache(&cache);
            
            // Reset force refresh flag
            if refresh {
                // Note: can't set signal in async, but the flag served its purpose
            }
            
            Some(all_routes)
        }
    });
    
    let routes_loading = routes_resource.read().is_none();
    let mut routes = routes_resource.read()
        .as_ref()
        .and_then(|r| r.clone())
        .unwrap_or_default();
    
    // Apply filters
    
    // Stellar/Interstellar filter
    let scope = trade_scope();
    if scope == TradeScope::Stellar {
        routes.retain(|r| {
            // Same system: both terminals must be in the same system
            match (&r.buy_system, &r.sell_system) {
                (Some(a), Some(b)) => a == b,
                _ => false, // skip if system unknown
            }
        });
    }
    
    // Stations only filter (exclude planetary locations)
    if stations_only() {
        routes.retain(|r| !r.buy_is_planetary && !r.sell_is_planetary);
    }
    
    // System filter (when stellar mode and specific system selected)
    let sys_filter = system_filter();
    if !sys_filter.is_empty() {
        routes.retain(|r| {
            r.buy_system.as_deref() == Some(sys_filter.as_str())
        });
    }
    
    if !is_pirate {
        // Trader filters
        let max_invest: Option<f64> = max_invest_input().parse().ok();
        if let Some(max) = max_invest {
            let scu = scu_input();
            routes.retain(|r| r.buy_price * scu as f64 <= max);
        }
    }
    
    // Pirate filter: only high value cargo (>5000 aUEC/SCU)
    if only_high_value() {
        routes.retain(|r| r.buy_price >= 5000.0);
    }
    
    // Sort
    let scu = scu_input();
    sort_routes(&mut routes, sort_by(), scu, true);
    
    // Limit to top 100
    routes.truncate(100);
    
    rsx! {
        div { class: "space-y-6",
            // Header
            section {
                class: "flex flex-wrap items-center justify-between gap-4",
                div {
                    h2 { class: "text-xl font-semibold text-slate-100", 
                        if is_pirate { "Lukrative Routen" } else { "Trade Routes" }
                    }
                    p { class: "text-sm text-slate-400", 
                        if is_pirate {
                            "wo sich das lauern lohnt — wertvolle fracht, viel traffic"
                        } else {
                            "profitable routen zum handeln — buy low, sell high"
                        }
                    }
                }
                if routes_loading {
                    div { 
                        class: match profile {
                            Profile::Pirate => "flex items-center gap-2 text-[#ff9900]",
                            Profile::Trader => "flex items-center gap-2 text-sky-400",
                            Profile::Miner => "flex items-center gap-2 text-orange-400",
                            Profile::None => "flex items-center gap-2 text-indigo-400",
                        },
                        span { class: "animate-spin", "⟳" }
                        span { class: "text-sm", "lade preise..." }
                    }
                }
            }
            
            // Filters - different for pirate vs trader
            div {
                class: "{panel_border(profile)} p-5 space-y-4",
                
                // Row 1: Main filter buttons (always same elements)
                div { class: "flex flex-wrap gap-x-8 gap-y-4 items-start",
                    
                    // Stellar/Interstellar toggle (both profiles)
                    div {
                        label { class: "block text-xs font-semibold uppercase text-slate-500 mb-2", "Reichweite" }
                        div { class: "flex gap-2",
                            // Stellar with sub-menu
                            div { class: "flex flex-col items-center",
                                button {
                                    class: if trade_scope() == TradeScope::Stellar {
                                        format!("w-full {}", btn_active(profile))
                                    } else {
                                        format!("w-full {}", btn_inactive(profile))
                                    },
                                    onclick: move |_| trade_scope.set(TradeScope::Stellar),
                                    "🌍 Stellar"
                                }
                                // System sub-buttons (only when stellar)
                                if trade_scope() == TradeScope::Stellar {
                                    div { class: "flex gap-1 mt-2",
                                        button {
                                            class: if system_filter().is_empty() {
                                                btn_small_active(profile)
                                            } else {
                                                btn_small_inactive(profile)
                                            },
                                            onclick: move |_| system_filter.set(String::new()),
                                            "Alle"
                                        }
                                        button {
                                            class: if system_filter() == "Stanton" {
                                                btn_small_active(profile)
                                            } else {
                                                btn_small_inactive(profile)
                                            },
                                            onclick: move |_| system_filter.set("Stanton".to_string()),
                                            "Stanton"
                                        }
                                        button {
                                            class: if system_filter() == "Pyro" {
                                                btn_small_active(profile)
                                            } else {
                                                btn_small_inactive(profile)
                                            },
                                            onclick: move |_| system_filter.set("Pyro".to_string()),
                                            "Pyro"
                                        }
                                        button {
                                            class: if system_filter() == "Nyx" {
                                                btn_small_active(profile)
                                            } else {
                                                btn_small_inactive(profile)
                                            },
                                            onclick: move |_| system_filter.set("Nyx".to_string()),
                                            "Nyx"
                                        }
                                    }
                                }
                            }
                            button {
                                class: if trade_scope() == TradeScope::Interstellar {
                                    format!("{} self-start", btn_active(profile))
                                } else {
                                    format!("{} self-start", btn_inactive(profile))
                                },
                                onclick: move |_| trade_scope.set(TradeScope::Interstellar),
                                "🚀 Interstellar"
                            }
                        }
                    }
                    
                    // Stations only toggle (skip planetary landings)
                    if !is_pirate {
                        div {
                            label { class: "block text-xs font-semibold uppercase text-slate-500 mb-2", "Typ" }
                            button {
                                class: if stations_only() { btn_active(profile) } else { btn_inactive(profile) },
                                onclick: move |_| stations_only.set(!stations_only()),
                                "🛰️ Nur Stationen"
                            }
                        }
                    }
                    
                    // Trader-only: SCU input
                    if !is_pirate {
                        div {
                            label { class: "block text-xs font-semibold uppercase text-slate-500 mb-2", "SCU" }
                            input {
                                class: format!("w-32 {}", input_class(profile)),
                                r#type: "text",
                                inputmode: "numeric",
                                value: "{scu_input}",
                                oninput: move |e| {
                                    if let Ok(v) = e.value().parse::<u32>() {
                                        scu_input.set(v.min(max_scu));
                                    }
                                },
                            }
                        }
                    }
                    
                    // Trader-only: Max invest
                    if !is_pirate {
                        div {
                            label { class: "block text-xs font-semibold uppercase text-slate-500 mb-2", "Max Invest" }
                            input {
                                class: format!("w-40 {}", input_class(profile)),
                                placeholder: "z.B. 100000",
                                value: "{max_invest_input}",
                                oninput: move |e| max_invest_input.set(e.value()),
                            }
                        }
                    }
                    
                    // Pirate-only: High value toggle
                    if is_pirate {
                        div {
                            label { class: "block text-xs font-semibold uppercase text-slate-500 mb-2", "Filter" }
                            button {
                                class: if only_high_value() { btn_active(profile) } else { btn_inactive(profile) },
                                onclick: move |_| only_high_value.set(!only_high_value()),
                                "💎 Nur Wertvoll"
                            }
                        }
                    }
                    
                    // Refresh button
                    div {
                        label { class: "block text-xs font-semibold uppercase text-slate-500 mb-2 invisible", "Refresh" }
                        button {
                            class: btn_inactive(profile),
                            title: "Preise neu laden",
                            onclick: move |_| force_refresh.set(true),
                            "🔄"
                        }
                    }
                }
                
                // Row 2: Sort buttons (centered)
                div { class: "flex justify-center pt-2",
                    div {
                        label { class: "block text-xs font-semibold uppercase text-slate-500 mb-2 text-center", "Sortieren" }
                        div { class: "flex gap-2",
                            if is_pirate {
                                SortButton { current: sort_by(), target: TradeRouteSort::CargoValue, on_click: move |_| sort_by.set(TradeRouteSort::CargoValue), label: "Wert", profile: profile }
                                SortButton { current: sort_by(), target: TradeRouteSort::ActivityScore, on_click: move |_| sort_by.set(TradeRouteSort::ActivityScore), label: "Traffic", profile: profile }
                                SortButton { current: sort_by(), target: TradeRouteSort::ProfitPerGm, on_click: move |_| sort_by.set(TradeRouteSort::ProfitPerGm), label: "Profit/Gm", profile: profile }
                            } else {
                                SortButton { current: sort_by(), target: TradeRouteSort::ProfitPerGm, on_click: move |_| sort_by.set(TradeRouteSort::ProfitPerGm), label: "Profit/Gm", profile: profile }
                                SortButton { current: sort_by(), target: TradeRouteSort::RoiPercent, on_click: move |_| sort_by.set(TradeRouteSort::RoiPercent), label: "ROI %", profile: profile }
                                SortButton { current: sort_by(), target: TradeRouteSort::ProfitTotal, on_click: move |_| sort_by.set(TradeRouteSort::ProfitTotal), label: "Profit", profile: profile }
                                SortButton { current: sort_by(), target: TradeRouteSort::CargoValue, on_click: move |_| sort_by.set(TradeRouteSort::CargoValue), label: "Wert", profile: profile }
                            }
                        }
                    }
                }
            }
            
            // Route Detail Panel (Trader only) - ABOVE the table
            if let Some(route) = selected_route() {
                RouteDetailPanel {
                    route: route.clone(),
                    scu: scu_input(),
                    on_close: move |_| selected_route.set(None),
                }
            }
            
            // Routes table
            if !routes.is_empty() {
                div { class: table_container(profile),
                    div { class: "overflow-x-auto",
                        table { class: "w-full text-sm",
                            thead { class: table_header(profile),
                                tr {
                                    th { class: "px-4 py-3 text-left", "Ware" }
                                    th { class: "px-4 py-3 text-left", 
                                        if is_pirate { "Route (Start → Ziel)" } else { "Kaufen bei" }
                                    }
                                    if !is_pirate {
                                        th { class: "px-4 py-3 text-left", "Verkaufen bei" }
                                    }
                                    th { class: "px-4 py-3 text-right", 
                                        if is_pirate { "Wert/SCU" } else { "Invest" }
                                    }
                                    th { class: "px-4 py-3 text-right", "Profit" }
                                    if !is_pirate {
                                        th { class: "px-4 py-3 text-right", "ROI" }
                                    }
                                    th { class: "px-4 py-3 text-right", "Traffic" }
                                }
                            }
                            tbody { class: table_divider(profile),
                                for route in routes.iter() {
                                    RouteRow { 
                                        route: route.clone(), 
                                        scu: scu_input(), 
                                        is_pirate: is_pirate,
                                        on_click: if !is_pirate {
                                            Some(EventHandler::new(move |r| selected_route.set(Some(r))))
                                        } else {
                                            None
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
                p { class: "text-xs text-slate-500 text-center", 
                    "zeigt top {routes.len()} routen" 
                }
            } else if !routes_loading {
                div {
                    class: format!("{} px-6 py-12 text-center", panel_border(profile)),
                    p { class: "text-slate-400", "keine profitablen routen gefunden" }
                }
            }
        }
    }
}

#[component]
fn SortButton(
    current: TradeRouteSort,
    target: TradeRouteSort,
    on_click: EventHandler<()>,
    label: &'static str,
    profile: Profile,
) -> Element {
    let active = current == target;
    let class = match (profile, active) {
        (Profile::Pirate, true) => "rounded-lg px-4 py-2 text-sm font-semibold bg-[#3b1712] text-[#ff9900] border border-[#5c2a1f] drake-glow",
        (Profile::Pirate, false) => "rounded-lg px-4 py-2 text-sm text-[#d4523a]/70 hover:text-[#ff9900] border border-transparent hover:border-[#5c2a1f]",
        (Profile::Trader, true) => "rounded-lg px-4 py-2 text-sm font-semibold bg-sky-500/20 text-sky-300 border border-sky-500/40 misc-glow",
        (Profile::Trader, false) => "rounded-lg px-4 py-2 text-sm text-slate-400 hover:text-sky-300 border border-transparent hover:border-sky-700",
        (Profile::Miner, true) => "rounded-lg px-4 py-2 text-sm font-semibold bg-orange-500/20 text-orange-300 border border-orange-500/40 argo-glow",
        (Profile::Miner, false) => "rounded-lg px-4 py-2 text-sm text-slate-400 hover:text-orange-300 border border-transparent hover:border-orange-700",
        (_, true) => "rounded-lg px-4 py-2 text-sm font-semibold bg-indigo-500/20 text-indigo-300 border border-indigo-500/40",
        (_, false) => "rounded-lg px-4 py-2 text-sm text-slate-400 hover:text-slate-200 border border-transparent hover:border-slate-700",
    };
    
    rsx! {
        button {
            class: "{class}",
            onclick: move |_| on_click.call(()),
            "{label}"
        }
    }
}

#[component]
fn RouteRow(
    route: TradeRoute,
    scu: u32,
    is_pirate: bool,
    on_click: Option<EventHandler<TradeRoute>>,
) -> Element {
    let qty = route.for_quantity(scu);
    let activity = route.activity_score();
    
    // Traffic indicator
    let traffic_class = if activity >= 10 {
        "text-amber-400"
    } else if activity >= 5 {
        "text-yellow-400"
    } else {
        "text-slate-500"
    };
    
    let row_class = if on_click.is_some() {
        "hover:bg-slate-800/50 transition-colors cursor-pointer"
    } else {
        "hover:bg-slate-800/50 transition-colors"
    };
    
    let route_for_click = route.clone();
    
    rsx! {
        tr { 
            class: "{row_class}",
            onclick: move |_| {
                if let Some(ref handler) = on_click {
                    handler.call(route_for_click.clone());
                }
            },
            // Commodity
            td { class: "px-4 py-3",
                div { class: "flex items-center gap-2",
                    span { class: "text-slate-100 font-medium", "{route.commodity_name}" }
                    if route.is_illegal {
                        span { class: "text-[10px] text-red-400", "⚠️" }
                    }
                    if route.sell_is_nqa {
                        span { class: "text-[10px] text-amber-400", "🏴‍☠️" }
                    }
                }
            }
            
            if is_pirate {
                // Pirate: show route as single cell (Start → Ziel)
                td { class: "px-4 py-3",
                    div {
                        p { class: "text-slate-200", 
                            "{short_name(&route.buy_terminal_name)} → {short_name(&route.sell_terminal_name)}" 
                        }
                        p { class: "text-xs text-slate-500",
                            "{route.buy_system.as_deref().unwrap_or(\"?\")} → {route.sell_system.as_deref().unwrap_or(\"?\")}"
                        }
                    }
                }
            } else {
                // Trader: separate buy/sell columns
                td { class: "px-4 py-3",
                    div {
                        p { class: "text-slate-200", "{short_name(&route.buy_terminal_name)}" }
                        p { class: "text-xs text-slate-500", 
                            "{route.buy_system.as_deref().unwrap_or(\"\")} · {route.buy_price:.0} aUEC"
                        }
                    }
                }
                td { class: "px-4 py-3",
                    div {
                        p { class: "text-slate-200", "{short_name(&route.sell_terminal_name)}" }
                        p { class: "text-xs text-slate-500",
                            "{route.sell_system.as_deref().unwrap_or(\"\")} · {route.sell_price:.0} aUEC"
                        }
                    }
                }
            }
            
            // Value/Invest
            td { class: "px-4 py-3 text-right text-slate-300",
                if is_pirate {
                    "{route.buy_price:.0}"
                } else {
                    "{format_auec(qty.invest)}"
                }
            }
            
            // Profit
            td { class: "px-4 py-3 text-right font-semibold text-amber-400",
                if is_pirate {
                    "+{route.profit_per_scu:.0}"
                } else {
                    "+{format_auec(qty.profit_total)}"
                }
            }
            
            // ROI (trader only)
            if !is_pirate {
                td { class: "px-4 py-3 text-right text-indigo-300",
                    "{route.roi_percent:.1}%"
                }
            }
            
            // Traffic
            td { class: "px-4 py-3 text-right {traffic_class}",
                if activity > 0 {
                    "{activity}"
                } else {
                    "—"
                }
            }
        }
    }
}

/// Shorten terminal names for compact display.
fn short_name(name: &str) -> &str {
    // Remove common prefixes
    name.strip_prefix("Admin - ")
        .or_else(|| name.strip_prefix("TDD - Trade and Development Division - "))
        .or_else(|| name.strip_prefix("CBD - Central Business District - "))
        .unwrap_or(name)
}

fn format_auec(value: f64) -> String {
    let rounded = value.round() as i64;
    if rounded.abs() >= 1_000_000 {
        format!("{:.1}M", value / 1_000_000.0)
    } else if rounded.abs() >= 1_000 {
        format!("{:.0}k", value / 1_000.0)
    } else {
        format!("{}", rounded)
    }
}

fn format_auec_full(value: f64) -> String {
    let rounded = value.round() as i64;
    // Manual thousands formatting
    let s = format!("{}", rounded.abs());
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push('.');
        }
        result.push(c);
    }
    if rounded < 0 {
        format!("-{} aUEC", result.chars().rev().collect::<String>())
    } else {
        format!("{} aUEC", result.chars().rev().collect::<String>())
    }
}

/// Inline panel showing detailed route information.
#[component]
fn RouteDetailPanel(
    route: TradeRoute,
    scu: u32,
    on_close: EventHandler<()>,
) -> Element {
    let qty = route.for_quantity(scu);
    
    let buy_type = if route.buy_is_planetary { "🌍" } else { "🛰️" };
    let sell_type = if route.sell_is_planetary { "🌍" } else { "🛰️" };
    
    rsx! {
        div {
            class: "rounded-xl border-2 border-sky-500/50 bg-slate-900/80 p-6 mb-4 misc-glow",
            // Header
            div {
                class: "flex items-center justify-between mb-6",
                div {
                    h3 { class: "text-xl font-bold text-slate-100", "📦 {route.commodity_name}" }
                    p { class: "text-sm text-slate-400", "Route Details" }
                }
                button {
                    class: "rounded-lg px-3 py-1 text-sm text-slate-400 border border-slate-700 hover:bg-slate-800 hover:text-slate-200",
                    onclick: move |_| on_close.call(()),
                    "✕ Schließen"
                }
            }
            
            // Two-column layout: Buy | Sell
            div { class: "grid grid-cols-1 md:grid-cols-2 gap-4 mb-6",
                // Buy
                div {
                    class: "rounded-xl border border-amber-500/30 bg-amber-500/5 p-4",
                    p { class: "text-xs font-semibold uppercase text-amber-400/70 mb-2", "1️⃣ KAUFEN" }
                    p { class: "text-lg font-semibold text-slate-100", 
                        "{buy_type} {short_name(&route.buy_terminal_name)}" 
                    }
                    p { class: "text-sm text-slate-400 mb-3",
                        "{route.buy_system.as_deref().unwrap_or(\"?\")}"
                    }
                    div { class: "space-y-1 text-sm",
                        p { class: "text-slate-300", "× {qty.quantity} SCU @ {route.buy_price:.0} aUEC" }
                        p { class: "text-amber-300 font-semibold", "Invest: {format_auec_full(qty.invest)}" }
                    }
                }
                
                // Sell
                div {
                    class: "rounded-xl border border-sky-500/30 bg-sky-500/5 p-4",
                    p { class: "text-xs font-semibold uppercase text-sky-400/70 mb-2", "2️⃣ VERKAUFEN" }
                    p { class: "text-lg font-semibold text-slate-100", 
                        "{sell_type} {short_name(&route.sell_terminal_name)}" 
                    }
                    p { class: "text-sm text-slate-400 mb-3",
                        "{route.sell_system.as_deref().unwrap_or(\"?\")}"
                    }
                    div { class: "space-y-1 text-sm",
                        p { class: "text-slate-300", "× {qty.quantity} SCU @ {route.sell_price:.0} aUEC" }
                        p { class: "text-sky-300 font-semibold", "Erlös: {format_auec_full(qty.quantity as f64 * route.sell_price)}" }
                    }
                }
            }
            
            // Warning if limited
            if qty.quantity < scu {
                div {
                    class: "rounded-lg border border-amber-500/30 bg-amber-500/10 px-4 py-2 mb-4 text-sm text-amber-200",
                    "⚠️ Nur {qty.max_tradeable} SCU handelbar (Stock/Nachfrage limitiert)"
                }
            }
            
            // Summary bar
            div {
                class: "flex items-center justify-between rounded-xl bg-sky-950/40 border border-sky-900/30 px-6 py-4",
                div {
                    p { class: "text-xs text-slate-500 uppercase", "Profit" }
                    p { class: "text-2xl font-bold text-amber-400", "+{format_auec_full(qty.profit_total)}" }
                }
                div { class: "text-center",
                    p { class: "text-xs text-slate-500 uppercase", "ROI" }
                    p { class: "text-xl font-semibold text-sky-300", "{route.roi_percent:.1}%" }
                }
                div { class: "text-right",
                    p { class: "text-xs text-slate-500 uppercase", "Profit/SCU" }
                    p { class: "text-lg text-slate-200", "{route.profit_per_scu:.0} aUEC" }
                }
            }
        }
    }
}
