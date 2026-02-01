#![allow(dead_code)]

use std::{
    collections::{HashMap, HashSet},
    time::{Duration, SystemTime},
};

use super::entities::{
    CargoItem, Commodity, CommodityId, PricePoint, ProfitabilityParams, SellLocation,
};
use serde::{Deserialize, Serialize};

/// Player profile / playstyle for the current session.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Profile {
    #[default]
    None,
    Pirate,
    Trader,
    Miner,
}

impl Profile {
    pub fn name(&self) -> &'static str {
        match self {
            Profile::None => "None",
            Profile::Pirate => "Pirat",
            Profile::Trader => "Händler", 
            Profile::Miner => "Miner",
        }
    }
    
    pub fn emoji(&self) -> &'static str {
        match self {
            Profile::None => "❓",
            Profile::Pirate => "🏴‍☠️",
            Profile::Trader => "📦",
            Profile::Miner => "⛏️",
        }
    }
    
    pub fn is_selected(&self) -> bool {
        !matches!(self, Profile::None)
    }
}

#[derive(Clone, Debug, Default)]
pub struct AppState {
    /// Currently selected player profile.
    pub profile: Profile,
    pub cargo_items: Vec<CargoItem>,
    pub commodities: Vec<Commodity>,
    pub price_points: HashMap<CommodityId, Vec<PricePoint>>,
    pub sell_locations: HashMap<String, SellLocation>,
    pub profitability: ProfitabilityParams,
    pub cache: CacheTimestamps,
    /// Terminal IDs that are "no questions asked" (accept hot cargo).
    /// Loaded from API and cached locally with game version tracking.
    pub nqa_terminal_ids: HashSet<i32>,
}

impl AppState {
    pub fn is_stale(&self, resource: &CacheResource, ttl: Duration) -> bool {
        self.cache.is_stale(resource, ttl)
    }

    pub fn apply_persisted(&mut self, persisted: PersistedState) {
        self.profile = persisted.profile;
        self.cargo_items = persisted.cargo_items;
        self.profitability = persisted.profitability;
    }

    pub fn to_persisted(&self) -> PersistedState {
        PersistedState {
            profile: self.profile,
            cargo_items: self.cargo_items.clone(),
            profitability: self.profitability.clone(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CacheTimestamps {
    entries: HashMap<CacheResource, SystemTime>,
}

impl CacheTimestamps {
    pub fn record_fetch(&mut self, resource: CacheResource, fetched_at: SystemTime) {
        self.entries.insert(resource, fetched_at);
    }

    pub fn fetched_at(&self, resource: &CacheResource) -> Option<SystemTime> {
        self.entries.get(resource).copied()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = (&CacheResource, &SystemTime)> {
        self.entries.iter()
    }

    pub fn is_stale(&self, resource: &CacheResource, ttl: Duration) -> bool {
        self.fetched_at(resource)
            .map(|time| time.elapsed().map(|elapsed| elapsed > ttl).unwrap_or(true))
            .unwrap_or(true)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CacheResource {
    Commodities,
    Prices(CommodityId),
    SellLocations,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PersistedState {
    #[serde(default)]
    pub profile: Profile,
    pub cargo_items: Vec<CargoItem>,
    pub profitability: ProfitabilityParams,
}
