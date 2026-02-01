//! Trade route calculation and ranking.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::entities::PricePoint;

/// A potential trade route: buy at A, sell at B.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TradeRoute {
    // Commodity info
    pub commodity_id: String,
    pub commodity_name: String,
    pub is_illegal: bool,
    
    // Buy terminal (where we purchase)
    pub buy_terminal_id: i32,
    pub buy_terminal_name: String,
    pub buy_system: Option<String>,
    pub buy_price: f64,
    pub buy_stock: f64,         // how much we can buy
    pub buy_user_rows: i32,     // user activity indicator
    pub buy_is_planetary: bool, // true if city or outpost
    
    // Sell terminal (where we offload)
    pub sell_terminal_id: i32,
    pub sell_terminal_name: String,
    pub sell_system: Option<String>,
    pub sell_price: f64,
    pub sell_demand: f64,       // how much terminal will accept
    pub sell_user_rows: i32,    // user activity indicator
    pub sell_is_planetary: bool,// true if city or outpost
    pub sell_is_nqa: bool,      // for pirates: no questions asked
    
    // Distance
    pub distance_gm: Option<f64>,
    
    // Calculated metrics (for 1 SCU)
    pub profit_per_scu: f64,
    pub roi_percent: f64,
}

impl TradeRoute {
    /// Calculate metrics for a given quantity.
    /// Quantity is limited by both buy_stock and sell_demand.
    pub fn for_quantity(&self, scu: u32) -> TradeRouteWithQuantity {
        // Can't buy more than available, can't sell more than demand
        let max_tradeable = (self.buy_stock as u32).min(self.sell_demand as u32);
        let quantity = scu.min(max_tradeable);
        let invest = self.buy_price * quantity as f64;
        let revenue = self.sell_price * quantity as f64;
        let profit_total = revenue - invest;
        
        TradeRouteWithQuantity {
            route: self.clone(),
            quantity,
            max_tradeable,
            invest,
            profit_total,
            profit_per_gm: self.distance_gm.map(|d| if d > 0.0 { profit_total / d } else { 0.0 }),
        }
    }
    
    /// Combined activity score (buy + sell user activity).
    pub fn activity_score(&self) -> i32 {
        self.buy_user_rows + self.sell_user_rows
    }
    
    /// Profit per Gm for 1 SCU.
    pub fn profit_per_gm(&self) -> Option<f64> {
        self.distance_gm.map(|d| if d > 0.0 { self.profit_per_scu / d } else { 0.0 })
    }
}

/// Trade route with a specific quantity calculated.
#[derive(Clone, Debug)]
pub struct TradeRouteWithQuantity {
    pub route: TradeRoute,
    pub quantity: u32,
    pub max_tradeable: u32,  // min(buy_stock, sell_demand)
    pub invest: f64,
    pub profit_total: f64,
    pub profit_per_gm: Option<f64>,
}

/// Sorting options for trade routes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TradeRouteSort {
    #[default]
    ProfitPerGm,
    RoiPercent,
    ProfitTotal,
    ActivityScore,
    Distance,
    CargoValue, // sort by buy_price (valuable goods like quant, drugs)
}

impl TradeRouteSort {
    pub fn label(&self) -> &'static str {
        match self {
            Self::ProfitPerGm => "Profit/Gm",
            Self::RoiPercent => "ROI %",
            Self::ProfitTotal => "Profit",
            Self::ActivityScore => "Traffic",
            Self::Distance => "Distanz",
            Self::CargoValue => "Warenwert",
        }
    }
}

/// Filter options for trade routes.
#[derive(Clone, Debug, Default)]
pub struct TradeRouteFilter {
    pub max_invest: Option<f64>,
    pub min_profit: Option<f64>,
    pub min_roi_percent: Option<f64>,
    pub max_distance_gm: Option<f64>,
    pub only_illegal: bool,
    pub only_nqa_sell: bool,
    pub commodity_id: Option<String>,
    pub buy_system: Option<String>,
    pub sell_system: Option<String>,
}

impl TradeRouteFilter {
    pub fn matches(&self, route: &TradeRoute, scu: u32) -> bool {
        let with_qty = route.for_quantity(scu);
        
        if let Some(max) = self.max_invest {
            if with_qty.invest > max { return false; }
        }
        if let Some(min) = self.min_profit {
            if with_qty.profit_total < min { return false; }
        }
        if let Some(min_roi) = self.min_roi_percent {
            if route.roi_percent < min_roi { return false; }
        }
        if let Some(max_dist) = self.max_distance_gm {
            if route.distance_gm.map(|d| d > max_dist).unwrap_or(false) { return false; }
        }
        if self.only_illegal && !route.is_illegal { return false; }
        if self.only_nqa_sell && !route.sell_is_nqa { return false; }
        if let Some(ref cid) = self.commodity_id {
            if &route.commodity_id != cid { return false; }
        }
        if let Some(ref sys) = self.buy_system {
            if route.buy_system.as_ref() != Some(sys) { return false; }
        }
        if let Some(ref sys) = self.sell_system {
            if route.sell_system.as_ref() != Some(sys) { return false; }
        }
        
        true
    }
}

/// Sort routes by the given criteria.
pub fn sort_routes(routes: &mut [TradeRoute], sort: TradeRouteSort, scu: u32, descending: bool) {
    routes.sort_by(|a, b| {
        let ord = match sort {
            TradeRouteSort::ProfitPerGm => {
                let a_ppg = a.profit_per_gm().unwrap_or(0.0);
                let b_ppg = b.profit_per_gm().unwrap_or(0.0);
                a_ppg.partial_cmp(&b_ppg).unwrap_or(std::cmp::Ordering::Equal)
            }
            TradeRouteSort::RoiPercent => {
                a.roi_percent.partial_cmp(&b.roi_percent).unwrap_or(std::cmp::Ordering::Equal)
            }
            TradeRouteSort::ProfitTotal => {
                let a_qty = a.for_quantity(scu);
                let b_qty = b.for_quantity(scu);
                a_qty.profit_total.partial_cmp(&b_qty.profit_total).unwrap_or(std::cmp::Ordering::Equal)
            }
            TradeRouteSort::ActivityScore => {
                a.activity_score().cmp(&b.activity_score())
            }
            TradeRouteSort::Distance => {
                let a_d = a.distance_gm.unwrap_or(f64::MAX);
                let b_d = b.distance_gm.unwrap_or(f64::MAX);
                a_d.partial_cmp(&b_d).unwrap_or(std::cmp::Ordering::Equal)
            }
            TradeRouteSort::CargoValue => {
                a.buy_price.partial_cmp(&b.buy_price).unwrap_or(std::cmp::Ordering::Equal)
            }
        };
        if descending { ord.reverse() } else { ord }
    });
}

/// Calculate all profitable trade routes for a commodity.
/// 
/// API terminology (from PLAYER perspective):
/// - `price_buy` = price at which YOU BUY (spend money)
/// - `price_sell` = price at which YOU SELL (receive money)
/// - `scu_buy` = available stock to buy
/// - `scu_sell_stock` = demand (how much terminal will accept)
pub fn calculate_routes_for_commodity(
    commodity_id: &str,
    commodity_name: &str,
    is_illegal: bool,
    prices: &[PricePoint],
    nqa_terminal_ids: &HashSet<i32>,
) -> Vec<TradeRoute> {
    let mut routes = Vec::new();
    
    // Find all buy terminals (where price_buy > 0 AND scu_buy > 0)
    let buy_terminals: Vec<_> = prices.iter()
        .filter(|p| p.price_buy.map(|v| v > 0.0).unwrap_or(false))
        .filter(|p| p.scu_buy.map(|s| s > 0.0).unwrap_or(false)) // must have stock
        .collect();
    
    // Find all sell terminals (where price_sell > 0 AND scu_sell_stock > 0)
    let sell_terminals: Vec<_> = prices.iter()
        .filter(|p| p.price_sell.map(|v| v > 0.0).unwrap_or(false))
        .filter(|p| p.scu_sell_stock.map(|s| s > 0.0).unwrap_or(false)) // must have demand
        .collect();
    
    // Create all profitable combinations
    for buy in &buy_terminals {
        let buy_price = buy.price_buy.unwrap_or(0.0);
        let buy_stock = buy.scu_buy.unwrap_or(0.0);
        if buy_price <= 0.0 || buy_stock <= 0.0 { continue; }
        
        for sell in &sell_terminals {
            let sell_price = sell.price_sell.unwrap_or(0.0);
            let sell_demand = sell.scu_sell_stock.unwrap_or(0.0);
            if sell_price <= 0.0 || sell_demand <= 0.0 { continue; }
            
            // Skip if same terminal
            if buy.terminal_id == sell.terminal_id { continue; }
            
            // Skip if not profitable (sell price must be higher than buy price)
            let profit_per_scu = sell_price - buy_price;
            if profit_per_scu <= 0.0 { continue; }
            
            let roi_percent = (profit_per_scu / buy_price) * 100.0;
            
            // Sanity check: ROI should be reasonable
            if !roi_percent.is_finite() || roi_percent <= 0.0 { continue; }
            
            let sell_is_nqa = sell.terminal_id
                .map(|id| nqa_terminal_ids.contains(&id))
                .unwrap_or(false);
            
            routes.push(TradeRoute {
                commodity_id: commodity_id.to_string(),
                commodity_name: commodity_name.to_string(),
                is_illegal,
                buy_terminal_id: buy.terminal_id.unwrap_or(0),
                buy_terminal_name: buy.terminal_name.clone(),
                buy_system: buy.system.clone(),
                buy_price,
                buy_stock,
                buy_user_rows: buy.buy_user_rows.unwrap_or(0),
                buy_is_planetary: buy.is_planetary(),
                sell_terminal_id: sell.terminal_id.unwrap_or(0),
                sell_terminal_name: sell.terminal_name.clone(),
                sell_system: sell.system.clone(),
                sell_price,
                sell_demand,
                sell_user_rows: sell.sell_user_rows.unwrap_or(0),
                sell_is_planetary: sell.is_planetary(),
                sell_is_nqa,
                distance_gm: None, // will be filled later
                profit_per_scu,
                roi_percent,
            });
        }
    }
    
    routes
}
