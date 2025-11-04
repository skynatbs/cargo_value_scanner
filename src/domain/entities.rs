#![allow(dead_code)]

use std::time::SystemTime;

use serde::{Deserialize, Serialize};

/// Identifier for commodities returned by the UEX API.
pub type CommodityId = String;

#[derive(Clone, Debug, PartialEq)]
pub struct Commodity {
    pub id: CommodityId,
    pub name: String,
    pub category: String,
    pub code: Option<String>,
    pub weight_scu: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CargoItem {
    pub id: String,
    pub commodity_id: CommodityId,
    pub commodity_name: String,
    pub scu: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PricePoint {
    pub terminal_id: Option<i32>,
    pub terminal_name: String,
    pub system: Option<String>,
    pub terminal_code: Option<String>,
    pub price_sell_min: Option<f64>,
    pub price_sell: Option<f64>,
    pub price_sell_max: Option<f64>,
    pub price_buy_max: Option<f64>,
    pub price_buy: Option<f64>,
    pub price_buy_min: Option<f64>,
    pub price_average: Option<f64>,
    pub container_sizes: Vec<f64>,
    pub scu_sell_stock: Option<f64>,
    pub status_sell: Option<i32>,
    pub status_buy: Option<i32>,
    pub volatility_sell: Option<f64>,
    pub updated_at: SystemTime,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SellLocation {
    pub id: String,
    pub name: String,
    pub system: Option<String>,
    pub kind: Option<String>,
    pub terminal_code: Option<String>,
    pub armistice: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CargoEvaluation {
    pub ev: f64,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub confidence: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProfitabilityParams {
    pub risk_pct: f64,
    pub crew_hourly: f64,
    pub crew_size: u8,
    pub time_minutes: u16,
}

impl Default for ProfitabilityParams {
    fn default() -> Self {
        Self {
            risk_pct: 0.2,
            crew_hourly: 150.0,
            crew_size: 1,
            time_minutes: 60,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BestPrice {
    pub commodity_id: CommodityId,
    pub location_id: Option<String>,
    pub sell_price: Option<f64>,
    pub buy_price: Option<f64>,
    pub adjusted_price: Option<f64>,
    pub stock: Option<f64>,
    pub status_sell: Option<i32>,
    pub status_buy: Option<i32>,
    pub container_sizes: Vec<f64>,
    pub notes: Option<String>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub struct CrewMember {
    pub id: String,
    pub name: String,
    pub role: String,
    pub weight: f32,
}
