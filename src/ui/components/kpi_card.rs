use dioxus::prelude::*;

#[component]
pub fn KpiCard(title: String, value: String, description: Option<String>) -> Element {
    rsx! {
        div {
            class: "rounded-xl border border-slate-800 bg-slate-900/60 p-4 shadow-sm",
            h3 { class: "text-xs font-semibold uppercase tracking-wide text-slate-500", "{title}" }
            p { class: "mt-2 text-2xl font-semibold text-slate-100", "{value}" }
            if let Some(desc) = description {
                p { class: "mt-1 text-xs text-slate-500", "{desc}" }
            }
        }
    }
}
