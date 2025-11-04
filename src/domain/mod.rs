//! Domain logic for cargo valuation lives here.

pub mod app_state;
pub mod entities;
pub mod evaluation;

#[allow(unused_imports)]
pub use app_state::{AppState, CacheResource, CacheTimestamps};
#[allow(unused_imports)]
pub use entities::{
    BestPrice, CargoEvaluation, CargoItem, Commodity, CommodityId, CrewMember, PricePoint,
    ProfitabilityParams, SellLocation,
};
#[allow(unused_imports)]
pub use evaluation::{
    evaluate_cargo_items, evaluate_item, price_summary, profitability_indicator, rank_best_prices,
    BestPriceEntry, BestPriceSuggestion, BestPriceSummary, EvaluationSummary, ProfitIndicator,
    ProfitIndicatorStatus,
};
