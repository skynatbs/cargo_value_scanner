#![allow(dead_code)]

//! Thin asynchronous client for the UEX API v2.
//!
//! - Provides typed accessors for commodities and price lookups.
//! - Maintains a simple 60-minute in-memory cache with stale fallbacks.

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};

use reqwest::{Client, Url};
use serde::{de::DeserializeOwned, Deserialize};
use thiserror::Error;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tokio::sync::Mutex;

use crate::domain::{Commodity, CommodityId, PricePoint};

const DEFAULT_BASE_URL: &str = "https://api.uexcorp.uk/2.0/";
const DEFAULT_TTL: Duration = Duration::from_secs(60 * 60);
const USER_AGENT: &str = "cargo-value-scanner/0.1.0";

#[derive(Debug, Error)]
pub enum UexClientError {
    #[error("invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("http request error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("api error: {0}")]
    Api(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CacheStatus {
    Fresh,
    Cached,
    Stale,
}

#[derive(Clone, Debug)]
pub struct CachedPayload<T> {
    pub data: T,
    pub fetched_at: SystemTime,
    pub status: CacheStatus,
}

impl<T> CachedPayload<T> {
    fn new(data: T, fetched_at: SystemTime, status: CacheStatus) -> Self {
        Self {
            data,
            fetched_at,
            status,
        }
    }
}

#[derive(Default)]
struct UexCache {
    commodities: Option<Cached<Vec<Commodity>>>,
    prices: HashMap<CommodityId, Cached<Vec<PricePoint>>>,
}

#[derive(Debug, Deserialize)]
struct ApiEnvelope<T> {
    status: String,
    #[serde(default)]
    http_code: Option<u16>,
    data: Option<T>,
    #[serde(default)]
    message: Option<String>,
}

impl UexCache {
    fn clear(&mut self) {
        self.commodities = None;
        self.prices.clear();
    }
}

#[derive(Clone)]
pub struct UexClient {
    http: Client,
    base_url: Url,
    cache: Arc<Mutex<UexCache>>,
    ttl: Duration,
}

impl UexClient {
    pub fn new() -> Result<Self, UexClientError> {
        Self::with_base_url(DEFAULT_BASE_URL)
    }

    pub fn with_base_url(base: &str) -> Result<Self, UexClientError> {
        let base_url = Url::parse(base)?;
        let http = Client::builder().user_agent(USER_AGENT).build()?;
        Ok(Self {
            http,
            base_url,
            cache: Arc::new(Mutex::new(UexCache::default())),
            ttl: DEFAULT_TTL,
        })
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    pub async fn get_commodities(&self) -> Result<CachedPayload<Vec<Commodity>>, UexClientError> {
        if let Some(payload) = self.cached_commodities().await {
            return Ok(payload);
        }

        let url = self.url("commodities")?;
        match self
            .fetch_data::<Vec<CommodityDto>>(self.http.get(url))
            .await
        {
            Ok(response) => {
                let data = response
                    .into_iter()
                    .map(Commodity::from)
                    .collect::<Vec<_>>();
                Ok(self.store_commodities(data, CacheStatus::Fresh).await)
            }
            Err(error) => {
                if let Some(stale) = self.cached_commodities_stale().await {
                    return Ok(stale);
                }
                Err(error)
            }
        }
    }

    pub async fn get_prices(
        &self,
        commodity_id: &str,
        commodity_name: Option<&str>,
    ) -> Result<CachedPayload<Vec<PricePoint>>, UexClientError> {
        if let Some(payload) = self.cached_prices(commodity_id).await {
            return Ok(payload);
        }

        let mut attempts: Vec<(String, String)> =
            vec![("id_commodity".to_string(), commodity_id.to_string())];
        if let Some(name) = commodity_name {
            attempts.push(("commodity_name".to_string(), name.to_string()));
        }

        let mut last_error: Option<UexClientError> = None;
        for (key, value) in attempts {
            let mut url = self.url("commodities_prices")?;
            url.query_pairs_mut().append_pair(&key, &value);

            println!("Requesting UEX prices from {url}");

            match self
                .fetch_data::<serde_json::Value>(self.http.get(url.clone()))
                .await
            {
                Ok(raw) => {
                    println!(
                        "UEX price payload ({key}={value}): {}",
                        serde_json::to_string_pretty(&raw).unwrap_or_else(|_| raw.to_string())
                    );
                    let data = parse_price_points(raw);
                    println!(
                        "UEX parsed {} entries for commodity {commodity_id} (key: {key}). Sample: {:?}",
                        data.len(),
                        data.first()
                    );
                    let status = if data.is_empty() {
                        CacheStatus::Cached
                    } else {
                        CacheStatus::Fresh
                    };
                    return Ok(self.store_prices(commodity_id, data, status).await);
                }
                Err(error) => {
                    println!(
                        "UEX price request failed for {url}: {error}; retrying with next identifier if available."
                    );
                    last_error = Some(error);
                }
            }
        }

        if let Some(stale) = self.cached_prices_stale(commodity_id).await {
            return Ok(stale);
        }

        Err(last_error
            .unwrap_or_else(|| UexClientError::Api("Unable to load commodity prices".to_string())))
    }

    pub async fn clear_cache(&self) {
        self.cache.lock().await.clear();
    }

    async fn cached_commodities(&self) -> Option<CachedPayload<Vec<Commodity>>> {
        let cache = self.cache.lock().await;
        cache
            .commodities
            .as_ref()
            .and_then(|entry| entry.if_fresh(self.ttl))
    }

    async fn cached_commodities_stale(&self) -> Option<CachedPayload<Vec<Commodity>>> {
        let cache = self.cache.lock().await;
        cache.commodities.as_ref().map(Cached::stale)
    }

    async fn cached_prices(&self, commodity_id: &str) -> Option<CachedPayload<Vec<PricePoint>>> {
        let cache = self.cache.lock().await;
        let result = cache
            .prices
            .get(commodity_id)
            .and_then(|entry| entry.if_fresh(self.ttl));
        if result.is_some() {
            println!("Serving cached UEX prices for commodity {commodity_id}");
        }
        result
    }

    async fn cached_prices_stale(
        &self,
        commodity_id: &str,
    ) -> Option<CachedPayload<Vec<PricePoint>>> {
        let cache = self.cache.lock().await;
        cache.prices.get(commodity_id).map(Cached::stale)
    }

    async fn store_commodities(
        &self,
        data: Vec<Commodity>,
        status: CacheStatus,
    ) -> CachedPayload<Vec<Commodity>> {
        let fetched_at = SystemTime::now();
        let payload = CachedPayload::new(data.clone(), fetched_at, status);
        let mut cache = self.cache.lock().await;
        cache.commodities = Some(Cached::new(data, fetched_at));
        payload
    }

    async fn store_prices(
        &self,
        commodity_id: &str,
        data: Vec<PricePoint>,
        status: CacheStatus,
    ) -> CachedPayload<Vec<PricePoint>> {
        let fetched_at = SystemTime::now();
        let payload = CachedPayload::new(data.clone(), fetched_at, status);
        let mut cache = self.cache.lock().await;
        cache
            .prices
            .insert(commodity_id.to_string(), Cached::new(data, fetched_at));
        payload
    }

    async fn fetch_data<T>(&self, builder: reqwest::RequestBuilder) -> Result<T, UexClientError>
    where
        T: DeserializeOwned,
    {
        let response = builder.send().await?.error_for_status()?;
        let envelope: ApiEnvelope<T> = response.json().await?;
        let ApiEnvelope {
            status,
            data,
            message,
            ..
        } = envelope;

        if status.eq_ignore_ascii_case("ok") {
            data.ok_or_else(|| UexClientError::Api("response missing data".into()))
        } else {
            Err(UexClientError::Api(message.unwrap_or(status)))
        }
    }

    fn url(&self, path: &str) -> Result<Url, url::ParseError> {
        self.base_url.join(path)
    }
}

struct Cached<T> {
    value: T,
    fetched_at: SystemTime,
}

impl<T: Clone> Cached<T> {
    fn new(value: T, fetched_at: SystemTime) -> Self {
        Self { value, fetched_at }
    }

    fn if_fresh(&self, ttl: Duration) -> Option<CachedPayload<T>> {
        if self
            .fetched_at
            .elapsed()
            .map(|elapsed| elapsed <= ttl)
            .unwrap_or(false)
        {
            Some(CachedPayload::new(
                self.value.clone(),
                self.fetched_at,
                CacheStatus::Cached,
            ))
        } else {
            None
        }
    }

    fn stale(&self) -> CachedPayload<T> {
        CachedPayload::new(self.value.clone(), self.fetched_at, CacheStatus::Stale)
    }
}

#[derive(Debug, Deserialize)]
struct CommodityDto {
    #[serde(deserialize_with = "string_from_json")]
    id: String,
    name: String,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    weight_scu: Option<f64>,
    #[serde(alias = "date_modified", alias = "dateModified", default)]
    date_modified: Option<i64>,
}

impl From<CommodityDto> for Commodity {
    fn from(value: CommodityDto) -> Self {
        Self {
            id: value.id,
            name: value.name,
            category: value.kind.unwrap_or_else(|| "Unknown".to_string()),
            code: value.code,
            weight_scu: value.weight_scu,
        }
    }
}

#[derive(Debug, Deserialize)]
struct CommodityPriceDto {
    #[serde(default)]
    id_terminal: Option<i32>,
    #[serde(default)]
    terminal_name: Option<String>,
    #[serde(default)]
    terminal_code: Option<String>,
    #[serde(default)]
    star_system_name: Option<String>,
    #[serde(default)]
    price_sell: Option<f64>,
    #[serde(default)]
    price_sell_min: Option<f64>,
    #[serde(default)]
    price_sell_max: Option<f64>,
    #[serde(default)]
    price_buy: Option<f64>,
    #[serde(default)]
    price_buy_min: Option<f64>,
    #[serde(default)]
    price_buy_max: Option<f64>,
    #[serde(default, alias = "price_sell_avg")]
    price_sell_average: Option<f64>,
    #[serde(default)]
    container_sizes: Option<String>,
    #[serde(default)]
    scu_sell_stock: Option<f64>,
    #[serde(default)]
    status_sell: Option<i32>,
    #[serde(default)]
    status_buy: Option<i32>,
    #[serde(default)]
    volatility_price_sell: Option<f64>,
    #[serde(default, alias = "date_modified", alias = "dateModified")]
    date_modified: Option<i64>,
    #[serde(default, alias = "updated_at", alias = "updatedAt")]
    updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CommodityPriceWrapper {
    data: Vec<CommodityPriceDto>,
}

impl From<CommodityPriceDto> for PricePoint {
    fn from(dto: CommodityPriceDto) -> Self {
        Self {
            terminal_id: dto.id_terminal,
            terminal_name: dto
                .terminal_name
                .unwrap_or_else(|| "Unknown terminal".to_string()),
            system: dto.star_system_name,
            terminal_code: dto.terminal_code,
            price_sell_min: dto.price_sell_min,
            price_sell: dto.price_sell,
            price_sell_max: dto.price_sell_max,
            price_buy_max: dto.price_buy_max,
            price_buy: dto.price_buy,
            price_buy_min: dto.price_buy_min,
            price_average: dto.price_sell_average,
            container_sizes: parse_container_sizes(dto.container_sizes.as_deref()),
            scu_sell_stock: dto.scu_sell_stock,
            status_sell: dto.status_sell,
            status_buy: dto.status_buy,
            volatility_sell: dto.volatility_price_sell,
            updated_at: parse_timestamp_fields(dto.date_modified, dto.updated_at),
        }
    }
}

fn parse_price_points(value: serde_json::Value) -> Vec<PricePoint> {
    if let Ok(entries) = serde_json::from_value::<Vec<CommodityPriceDto>>(value.clone()) {
        return entries.into_iter().map(PricePoint::from).collect();
    }

    if let Ok(wrapper) = serde_json::from_value::<CommodityPriceWrapper>(value) {
        return wrapper.data.into_iter().map(PricePoint::from).collect();
    }

    Vec::new()
}

#[allow(dead_code)]
fn parse_price_entry(entry: serde_json::Value) -> Option<PricePoint> {
    serde_json::from_value::<CommodityPriceDto>(entry)
        .map(PricePoint::from)
        .ok()
}

#[allow(dead_code)]
fn parse_timestamp_value(value: Option<&serde_json::Value>) -> SystemTime {
    match value {
        Some(serde_json::Value::Number(number)) => number
            .as_i64()
            .and_then(|secs| {
                if secs >= 0 {
                    Some(SystemTime::UNIX_EPOCH + Duration::from_secs(secs as u64))
                } else {
                    None
                }
            })
            .unwrap_or_else(SystemTime::now),
        Some(serde_json::Value::String(string)) => parse_timestamp_str(Some(string.as_str())),
        _ => SystemTime::now(),
    }
}

fn parse_timestamp_str(raw: Option<&str>) -> SystemTime {
    raw.and_then(|value| {
        OffsetDateTime::parse(value, &Rfc3339).ok().and_then(|dt| {
            if dt.unix_timestamp() >= 0 {
                let secs = dt.unix_timestamp() as u64;
                let nanos = dt.nanosecond() as u64;
                SystemTime::UNIX_EPOCH
                    .checked_add(Duration::from_secs(secs))
                    .and_then(|time| time.checked_add(Duration::from_nanos(nanos)))
            } else {
                None
            }
        })
    })
    .unwrap_or_else(SystemTime::now)
}

fn parse_timestamp_fields(epoch: Option<i64>, iso: Option<String>) -> SystemTime {
    if let Some(secs) = epoch {
        if secs >= 0 {
            return SystemTime::UNIX_EPOCH + Duration::from_secs(secs as u64);
        }
    }

    if let Some(string) = iso.as_deref() {
        return parse_timestamp_str(Some(string));
    }

    SystemTime::now()
}

fn parse_container_sizes(raw: Option<&str>) -> Vec<f64> {
    raw.map(|value| {
        value
            .split(|c| c == '|' || c == ',' || c == ';')
            .filter_map(|part| part.trim().parse::<f64>().ok())
            .collect()
    })
    .unwrap_or_default()
}

fn string_from_json<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct StringOrNumber;

    impl<'de> serde::de::Visitor<'de> for StringOrNumber {
        type Value = String;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or number")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(value)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(value.to_string())
        }
    }

    deserializer.deserialize_any(StringOrNumber)
}
