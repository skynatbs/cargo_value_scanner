use dioxus::prelude::*;

use crate::app::Route;

#[component]
pub fn Shell(children: Element) -> Element {
    let current_route = use_route::<Route>();
    rsx! {
        div { class: "min-h-screen bg-slate-950 text-slate-100 font-sans",
            header {
                class: "border-b border-slate-900/60 bg-slate-950/80 backdrop-blur px-6 py-4",
                div { class: "mx-auto flex max-w-6xl flex-wrap items-center justify-between gap-4",
                    div {
                        h1 { class: "text-2xl font-semibold tracking-tight", "Cargo Value Scanner" }
                        p { class: "text-xs uppercase tracking-[0.2em] text-slate-500", "Trade Planning Toolkit" }
                    }
                    nav { class: "flex gap-2 text-sm",
                        Link {
                            class: nav_link_class(matches!(current_route, Route::Cargo {})),
                            to: Route::Cargo {},
                            "Cargo"
                        }
                        Link {
                            class: nav_link_class(matches!(current_route, Route::BestPrice {})),
                            to: Route::BestPrice {},
                            "Best Price"
                        }
                        Link {
                            class: nav_link_class(matches!(current_route, Route::Settings {})),
                            to: Route::Settings {},
                            "Settings"
                        }
                    }
                }
            }
            main { class: "mx-auto max-w-6xl px-6 py-10",
                {children}
            }
        }
    }
}

fn nav_link_class(active: bool) -> String {
    if active {
        "rounded-lg border border-indigo-500/60 bg-indigo-500/15 px-4 py-2 font-semibold text-indigo-100 shadow-inner shadow-indigo-500/20"
            .to_string()
    } else {
        "rounded-lg border border-transparent px-4 py-2 text-slate-300 transition hover:border-slate-700 hover:bg-slate-900/80"
            .to_string()
    }
}
