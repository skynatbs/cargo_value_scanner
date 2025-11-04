use dioxus::prelude::*;

use crate::domain::{ProfitIndicator as IndicatorState, ProfitIndicatorStatus};

#[component]
pub fn ProfitIndicator(indicator: IndicatorState) -> Element {
    let (label, theme) = match indicator.status {
        ProfitIndicatorStatus::Green => (
            "Healthy",
            "border-emerald-500/40 bg-emerald-500/10 text-emerald-200",
        ),
        ProfitIndicatorStatus::Yellow => (
            "Watch",
            "border-amber-500/40 bg-amber-500/10 text-amber-200",
        ),
        ProfitIndicatorStatus::Red => ("Risky", "border-rose-500/40 bg-rose-500/10 text-rose-200"),
    };
    let score_display = format!("{:.0}", indicator.score);

    rsx! {
        div {
            class: "rounded-xl border px-4 py-3 {theme}",
            div {
                class: "flex items-center justify-between",
                span { class: "text-xs font-semibold uppercase tracking-wide", "Profitability" }
                span { class: "text-xs font-semibold uppercase", "{label}" }
            }
            p { class: "mt-2 text-2xl font-semibold", "{score_display}" }
            p { class: "mt-1 text-xs opacity-80", "{indicator.rationale}" }
        }
    }
}
