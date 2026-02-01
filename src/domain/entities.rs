#![allow(dead_code)]

use std::time::SystemTime;

use serde::{Deserialize, Serialize};

/// Terminal information from UEX API.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Terminal {
    pub id: i32,
    pub name: String,
    pub code: Option<String>,
    pub is_nqa: bool,
    pub system: Option<String>,
    /// Higher-level location identifiers for grouping
    pub space_station_name: Option<String>,
    pub city_name: Option<String>,
    pub outpost_name: Option<String>,
    pub planet_name: Option<String>,
    pub orbit_name: Option<String>,
}

impl Terminal {
    /// Returns the best human-readable location name for this terminal.
    /// Priority: city > space_station > outpost > orbit > planet > system
    /// Also applies known aliases (e.g., "Green Imperial Housing Exchange" → "GrimHEX")
    pub fn location_name(&self) -> String {
        let raw = self.city_name
            .clone()
            .or_else(|| self.space_station_name.clone())
            .or_else(|| self.outpost_name.clone())
            .or_else(|| self.orbit_name.clone())
            .or_else(|| self.planet_name.clone())
            .or_else(|| self.system.clone())
            .unwrap_or_else(|| "Unknown".to_string());
        
        // Apply known aliases for better searchability
        match raw.as_str() {
            "Green Imperial Housing Exchange" => "GrimHEX".to_string(),
            _ => raw,
        }
    }
    
    /// Returns true if this terminal is on a planet surface (city or outpost).
    /// Stations (space stations, orbital) are NOT planetary.
    pub fn is_planetary(&self) -> bool {
        self.city_name.is_some() || self.outpost_name.is_some()
    }
}

/// A unique location (station/city/outpost) derived from terminals.
/// Used for location selection in the planner.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Location {
    /// Display name (e.g., "Baijini Point", "Lorville")
    pub name: String,
    /// Star system
    pub system: Option<String>,
    /// One representative terminal ID at this location (for distance lookups)
    pub terminal_id: i32,
}

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
    /// Marks cargo as "hot" (stolen/illegal) — can only be sold at specific locations.
    #[serde(default)]
    pub is_hot: bool,
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
    pub scu_buy: Option<f64>,        // available stock when YOU buy
    pub scu_sell_stock: Option<f64>, // demand when YOU sell
    pub status_sell: Option<i32>,
    pub status_buy: Option<i32>,
    // Location type info
    pub city_name: Option<String>,
    pub outpost_name: Option<String>,
    pub space_station_name: Option<String>,
    pub volatility_sell: Option<f64>,
    pub buy_user_rows: Option<i32>,
    pub sell_user_rows: Option<i32>,
    pub updated_at: SystemTime,
}

impl PricePoint {
    /// Returns true if this terminal is on a planet surface (city or outpost).
    pub fn is_planetary(&self) -> bool {
        self.city_name.is_some() || self.outpost_name.is_some()
    }
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
