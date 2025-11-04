#![allow(dead_code)]

use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use super::entities::{
    CargoEvaluation, CargoItem, CommodityId, PricePoint, ProfitabilityParams, SellLocation,
};

pub struct EvaluationSummary {
    pub total_ev: f64,
    pub average_confidence: f32,
    pub items: Vec<(String, CargoEvaluation)>,
}

pub fn evaluate_cargo_items(
    items: &[CargoItem],
    prices: &HashMap<CommodityId, Vec<PricePoint>>,
) -> EvaluationSummary {
    let mut evaluations = Vec::with_capacity(items.len());
    let mut total_ev = 0.0;
    let mut confidence_sum = 0.0_f32;
    let mut counted = 0_usize;

    for item in items {
        let evaluation = evaluate_item(item, prices.get(&item.commodity_id).map(|v| v.as_slice()));
        total_ev += evaluation.ev;
        if !evaluation.ev.is_nan() {
            confidence_sum += evaluation.confidence;
            counted += 1;
        }
        evaluations.push((item.id.clone(), evaluation));
    }

    let average_confidence = if counted == 0 {
        0.0
    } else {
        confidence_sum / counted as f32
    };

    EvaluationSummary {
        total_ev,
        average_confidence,
        items: evaluations,
    }
}

pub fn evaluate_item(item: &CargoItem, price_points: Option<&[PricePoint]>) -> CargoEvaluation {
    let Some(points) = price_points else {
        return CargoEvaluation {
            ev: 0.0,
            min: None,
            max: None,
            confidence: 0.0,
        };
    };

    if points.is_empty() {
        return CargoEvaluation {
            ev: 0.0,
            min: None,
            max: None,
            confidence: 0.0,
        };
    }

    let summary = price_summary(points);
    let quantity = item.scu as f64;

    let ev = summary
        .as_ref()
        .map(|s| {
            let per_unit = s
                .max_price
                .or(Some(s.average_price))
                .unwrap_or(s.average_price);
            quantity * per_unit
        })
        .unwrap_or_default();

    CargoEvaluation {
        ev,
        min: summary
            .as_ref()
            .and_then(|s| s.min_price.map(|p| p * quantity)),
        max: summary
            .as_ref()
            .and_then(|s| s.max_price.map(|p| p * quantity)),
        confidence: summary.map(|s| s.confidence).unwrap_or_default(),
    }
}

pub struct PriceSummary {
    pub average_price: f64,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub confidence: f32,
}

pub fn price_summary(points: &[PricePoint]) -> Option<PriceSummary> {
    let mut sell_prices = Vec::new();
    let mut buy_prices = Vec::new();

    for point in points {
        if let Some(max_sell) = point
            .price_sell_max
            .filter(|price| price.is_finite() && *price > 0.0)
        {
            sell_prices.push(max_sell);
        } else if let Some(last_sell) = point
            .price_sell
            .filter(|price| price.is_finite() && *price > 0.0)
        {
            sell_prices.push(last_sell);
        } else if let Some(avg_sell) = point
            .price_average
            .filter(|price| price.is_finite() && *price > 0.0)
        {
            sell_prices.push(avg_sell);
        }

        if let Some(min_buy) = point
            .price_buy_min
            .filter(|price| price.is_finite() && *price > 0.0)
        {
            buy_prices.push(min_buy);
        } else if let Some(last_buy) = point
            .price_buy
            .filter(|price| price.is_finite() && *price > 0.0)
        {
            buy_prices.push(last_buy);
        }
    }

    if sell_prices.is_empty() {
        return None;
    }

    sell_prices.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let average_price = sell_prices.iter().sum::<f64>() / sell_prices.len() as f64;

    let min_price = buy_prices
        .iter()
        .copied()
        .min_by(|a, b| a.partial_cmp(b).unwrap());

    let max_price = sell_prices
        .iter()
        .copied()
        .max_by(|a, b| a.partial_cmp(b).unwrap());

    let confidence = compute_confidence(points);

    Some(PriceSummary {
        average_price,
        min_price,
        max_price,
        confidence,
    })
}

fn compute_confidence(points: &[PricePoint]) -> f32 {
    if points.is_empty() {
        return 0.0;
    }

    let now = SystemTime::now();
    let freshness = points
        .iter()
        .map(|p| p.updated_at)
        .max()
        .map(|time| freshness_score(now, time))
        .unwrap_or(0.5);

    let max_stock = points
        .iter()
        .filter_map(|p| p.scu_sell_stock)
        .fold(0.0, f64::max);
    let stock_score = (max_stock / 5000.0).clamp(0.0, 1.0) as f32;

    let (vol_sum, vol_count) = points.iter().fold((0.0, 0), |(sum, count), point| {
        if let Some(vol) = point.volatility_sell {
            (sum + vol.abs(), count + 1)
        } else {
            (sum, count)
        }
    });
    let volatility_factor = if vol_count == 0 {
        0.7
    } else {
        let avg = (vol_sum / vol_count as f64).min(1.5);
        (1.0 - avg.min(1.0)) as f32
    };

    (0.5 * freshness + 0.25 * stock_score + 0.25 * volatility_factor).clamp(0.0, 1.0)
}

fn freshness_score(now: SystemTime, updated_at: SystemTime) -> f32 {
    let age_minutes = now
        .duration_since(updated_at)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs_f32()
        / 60.0;

    (1.0 / (1.0 + age_minutes / 60.0)).clamp(0.0, 1.0)
}

pub fn profitability_indicator(total_ev: f64, params: &ProfitabilityParams) -> ProfitIndicator {
    let crew_cost =
        params.crew_hourly * params.crew_size as f64 * (params.time_minutes as f64 / 60.0);
    let risk_penalty = params.risk_pct * total_ev;
    let score = total_ev - risk_penalty - crew_cost;

    if total_ev <= 0.0 {
        return ProfitIndicator {
            status: ProfitIndicatorStatus::Red,
            score,
            rationale: "No estimated value yet".to_string(),
        };
    }

    let threshold_high = total_ev * 0.6;
    let threshold_low = total_ev * 0.25;

    let status = if score >= threshold_high {
        ProfitIndicatorStatus::Green
    } else if score >= threshold_low {
        ProfitIndicatorStatus::Yellow
    } else {
        ProfitIndicatorStatus::Red
    };

    let rationale = format!(
        "Net = {:.0} - risk {:.0} - crew {:.0}",
        total_ev, risk_penalty, crew_cost
    );

    ProfitIndicator {
        status,
        score,
        rationale,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProfitIndicator {
    pub status: ProfitIndicatorStatus,
    pub score: f64,
    pub rationale: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfitIndicatorStatus {
    Green,
    Yellow,
    Red,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BestPriceSummary {
    pub suggestions: Vec<BestPriceSuggestion>,
    pub best_overall: Option<BestPriceEntry>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BestPriceSuggestion {
    pub item_id: String,
    pub commodity_name: String,
    pub entries: Vec<BestPriceEntry>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BestPriceEntry {
    pub location_id: Option<String>,
    pub location_name: String,
    pub sell_price: Option<f64>,
    pub buy_price: Option<f64>,
    pub adjusted_price: f64,
    pub stock: Option<f64>,
    pub status_sell: Option<i32>,
    pub status_buy: Option<i32>,
    pub container_sizes: Vec<f64>,
    pub notes: Option<String>,
}

pub fn rank_best_prices(
    items: &[CargoItem],
    prices: &HashMap<CommodityId, Vec<PricePoint>>,
    locations: &HashMap<String, SellLocation>,
) -> BestPriceSummary {
    const CROSS_SYSTEM_PENALTY: f64 = 75.0;
    const ARMISTICE_PENALTY: f64 = 25.0;
    const HOTSPOT_PENALTY: f64 = 40.0;
    const HOME_SYSTEM: &str = "Stanton";

    let hotspots = ["Grim Hex", "Spider", "Jumptown"];

    let mut suggestions = Vec::new();
    let mut best_overall: Option<BestPriceEntry> = None;

    for item in items {
        let Some(points) = prices.get(&item.commodity_id) else {
            continue;
        };

        let mut entries: Vec<BestPriceEntry> = points
            .iter()
            .filter_map(|point| {
                let sell_price = point
                    .price_sell_max
                    .or(point.price_sell)
                    .or(point.price_average)
                    .filter(|value| value.is_finite());

                let sell_price = sell_price?;

                let buy_price = point
                    .price_buy_min
                    .or(point.price_buy)
                    .or(point.price_average)
                    .filter(|value| value.is_finite());

                let location_key = point
                    .terminal_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| point.terminal_name.clone());

                let location = locations
                    .get(&location_key)
                    .or_else(|| locations.get(&point.terminal_name));
                let mut penalty = 0.0;

                let armistice = location.map(|loc| loc.armistice).unwrap_or(false);
                let system_name = location
                    .and_then(|loc| loc.system.clone())
                    .or_else(|| point.system.clone())
                    .unwrap_or_else(|| "Unknown".to_string());

                let cross_system = system_name != HOME_SYSTEM;
                let hotspot = hotspots
                    .iter()
                    .any(|spot| point.terminal_name.contains(spot));

                if cross_system {
                    penalty += CROSS_SYSTEM_PENALTY;
                }
                if armistice {
                    penalty += ARMISTICE_PENALTY;
                }
                if hotspot {
                    penalty += HOTSPOT_PENALTY;
                }

                let adjusted_price = sell_price - penalty;
                Some(BestPriceEntry {
                    location_id: point
                        .terminal_id
                        .map(|id| id.to_string())
                        .or_else(|| location.map(|loc| loc.id.clone())),
                    location_name: if location.is_some() {
                        format!("{} ({})", point.terminal_name, system_name)
                    } else {
                        point.terminal_name.clone()
                    },
                    sell_price: Some(sell_price),
                    buy_price,
                    adjusted_price,
                    stock: point.scu_sell_stock,
                    status_sell: point.status_sell,
                    status_buy: point.status_buy,
                    container_sizes: point.container_sizes.clone(),
                    notes: build_notes(
                        cross_system,
                        armistice,
                        hotspot,
                        point.scu_sell_stock,
                        point.status_sell,
                        point.status_buy,
                    ),
                })
            })
            .collect();

        entries.sort_by(|a, b| b.adjusted_price.partial_cmp(&a.adjusted_price).unwrap());
        entries.truncate(3);

        if let Some(top) = entries.first() {
            if let Some(current) = best_overall.as_ref() {
                if current.adjusted_price < top.adjusted_price {
                    best_overall = Some(top.clone());
                }
            } else {
                best_overall = Some(top.clone());
            }
        }

        if !entries.is_empty() {
            suggestions.push(BestPriceSuggestion {
                item_id: item.id.clone(),
                commodity_name: item.commodity_name.clone(),
                entries,
            });
        }
    }

    BestPriceSummary {
        suggestions,
        best_overall,
    }
}

fn build_notes(
    cross_system: bool,
    armistice: bool,
    hotspot: bool,
    stock: Option<f64>,
    status_sell: Option<i32>,
    status_buy: Option<i32>,
) -> Option<String> {
    let mut notes = Vec::new();
    if cross_system {
        notes.push("Cross-system".to_string());
    }
    if armistice {
        notes.push("Armistice".to_string());
    }
    if hotspot {
        notes.push("Hotspot".to_string());
    }
    if let Some(level) = status_label(status_sell) {
        notes.push(format!("Sell {level}"));
    }
    if let Some(level) = status_label(status_buy) {
        notes.push(format!("Buy {level}"));
    }
    if let Some(stock_value) = stock {
        if stock_value.is_finite() {
            if stock_value < 500.0 {
                notes.push("Low stock".to_string());
            } else if stock_value > 5000.0 {
                notes.push("High stock".to_string());
            }
        }
    }

    if notes.is_empty() {
        None
    } else {
        Some(notes.join(", "))
    }
}

fn status_label(value: Option<i32>) -> Option<&'static str> {
    match value {
        Some(3) => Some("High"),
        Some(2) => Some("Normal"),
        Some(1) => Some("Low"),
        Some(0) => Some("Offline"),
        _ => None,
    }
}
