//! Domain logic for cargo valuation lives here.

pub mod app_state;
pub mod entities;
pub mod evaluation;
pub mod trade_route;

#[allow(unused_imports)]
pub use app_state::{AppState, CacheResource, CacheTimestamps, Profile};
#[allow(unused_imports)]
pub use entities::{
    BestPrice, CargoEvaluation, CargoItem, Commodity, CommodityId, CrewMember, Location, PricePoint,
    ProfitabilityParams, SellLocation, Terminal,
};
#[allow(unused_imports)]
pub use evaluation::{
    evaluate_cargo_items, evaluate_item, price_summary, profitability_indicator, rank_best_prices,
    BestPriceEntry, BestPriceSuggestion, BestPriceSummary, EvaluationSummary, ProfitIndicator,
    ProfitIndicatorStatus,
};
#[allow(unused_imports)]
pub use trade_route::{
    calculate_routes_for_commodity, sort_routes, TradeRoute, TradeRouteFilter, TradeRouteSort,
    TradeRouteWithQuantity,
};
