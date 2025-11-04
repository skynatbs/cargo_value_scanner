#![allow(dead_code)]

use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use super::entities::{
    CargoItem, Commodity, CommodityId, PricePoint, ProfitabilityParams, SellLocation,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default)]
pub struct AppState {
    pub cargo_items: Vec<CargoItem>,
    pub commodities: Vec<Commodity>,
    pub price_points: HashMap<CommodityId, Vec<PricePoint>>,
    pub sell_locations: HashMap<String, SellLocation>,
    pub profitability: ProfitabilityParams,
    pub cache: CacheTimestamps,
}

impl AppState {
    pub fn is_stale(&self, resource: &CacheResource, ttl: Duration) -> bool {
        self.cache.is_stale(resource, ttl)
    }

    pub fn apply_persisted(&mut self, persisted: PersistedState) {
        self.cargo_items = persisted.cargo_items;
        self.profitability = persisted.profitability;
    }

    pub fn to_persisted(&self) -> PersistedState {
        PersistedState {
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
    pub cargo_items: Vec<CargoItem>,
    pub profitability: ProfitabilityParams,
}
