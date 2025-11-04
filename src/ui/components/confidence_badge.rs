use dioxus::prelude::*;

#[component]
pub fn ConfidenceBadge(value: f32) -> Element {
    let (label, color) = match value {
        v if v >= 0.75 => (
            "High",
            "bg-emerald-500/10 text-emerald-300 border-emerald-500/40",
        ),
        v if v >= 0.45 => (
            "Medium",
            "bg-amber-500/10 text-amber-300 border-amber-500/40",
        ),
        v if v > 0.0 => ("Low", "bg-rose-500/10 text-rose-300 border-rose-500/40"),
        _ => ("N/A", "bg-slate-700/40 text-slate-300 border-slate-600/60"),
    };

    rsx! {
        span {
            class: "inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-medium {color}",
            "{label}"
        }
    }
}
