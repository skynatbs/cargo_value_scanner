//! Profile selection screen — "Who are you today?"

use dioxus::prelude::*;

use crate::domain::{AppState, Profile};

#[component]
pub fn ProfileSelectPage() -> Element {
    let mut state = use_context::<Signal<AppState>>();

    rsx! {
        div { 
            class: "min-h-screen flex items-center justify-center p-8",
            div { 
                class: "max-w-4xl w-full",
                // Header
                div { class: "text-center mb-12",
                    h1 { 
                        class: "text-4xl font-bold text-slate-100 mb-3",
                        "Cargo Value Scanner"
                    }
                    p { 
                        class: "text-xl text-slate-400",
                        "Wer bist du heute?"
                    }
                }
                
                // Profile cards
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-6",
                    // Pirate
                    ProfileCard {
                        profile: Profile::Pirate,
                        title: "Pirat",
                        emoji: "🏴‍☠️",
                        description: "Beute machen, heiße Ware loswerden, lukrative Routen finden.",
                        features: vec![
                            "Lukrative Handelsrouten (zum Lauern)",
                            "Cargo-Verwaltung mit Hot-Markierung",
                            "NQA-Terminal Übersicht",
                        ],
                        enabled: true,
                        on_select: move |_| {
                            state.with_mut(|s| s.profile = Profile::Pirate);
                        },
                    }
                    
                    // Trader
                    ProfileCard {
                        profile: Profile::Trader,
                        title: "Händler",
                        emoji: "📦",
                        description: "Günstig kaufen, teuer verkaufen, Routen optimieren.",
                        features: vec![
                            "Buy & Sell Planner",
                            "Trade Route Finder",
                            "Profit-Margen Übersicht",
                        ],
                        enabled: true,
                        on_select: move |_| {
                            state.with_mut(|s| s.profile = Profile::Trader);
                        },
                    }
                    
                    // Miner (disabled)
                    ProfileCard {
                        profile: Profile::Miner,
                        title: "Miner",
                        emoji: "⛏️",
                        description: "Erze abbauen, raffinieren, bestmöglich verkaufen.",
                        features: vec![
                            "Erz-Preise",
                            "Refinery-Standorte", 
                            "Mining-Spots",
                        ],
                        enabled: false,
                        on_select: move |_| {},
                    }
                }
                
                // Footer hint
                div { class: "text-center mt-12",
                    p { class: "text-sm text-slate-600",
                        "Du kannst dein Profil jederzeit in den Einstellungen wechseln."
                    }
                }
            }
        }
    }
}

#[component]
fn ProfileCard(
    profile: Profile,
    title: &'static str,
    emoji: &'static str,
    description: &'static str,
    features: Vec<&'static str>,
    enabled: bool,
    on_select: EventHandler<()>,
) -> Element {
    let base_classes = if enabled {
        "group relative rounded-2xl border-2 p-6 cursor-pointer transition-all duration-200"
    } else {
        "group relative rounded-2xl border-2 p-6 cursor-not-allowed opacity-50"
    };
    
    let border_color = match profile {
        Profile::Pirate => "border-[#5c2a1f] hover:border-[#ff9900]/60 hover:bg-[#3b1712]/30",
        Profile::Trader => "border-emerald-500/30 hover:border-emerald-500/60 hover:bg-emerald-500/5",
        Profile::Miner => "border-amber-500/30",
        Profile::None => "border-slate-700",
    };
    
    let accent_color = match profile {
        Profile::Pirate => "text-[#ff9900]",
        Profile::Trader => "text-emerald-400",
        Profile::Miner => "text-amber-400",
        Profile::None => "text-slate-400",
    };

    rsx! {
        div {
            class: "{base_classes} {border_color} bg-slate-900/60",
            onclick: move |_| {
                if enabled {
                    on_select.call(());
                }
            },
            
            // Soon badge for disabled profiles
            if !enabled {
                div {
                    class: "absolute top-3 right-3 rounded-full bg-slate-800 px-2 py-0.5 text-[10px] font-bold text-slate-400 uppercase tracking-wider",
                    "Soon™"
                }
            }
            
            // Emoji
            div { 
                class: "text-5xl mb-4 transition-transform group-hover:scale-110",
                "{emoji}"
            }
            
            // Title
            h2 { 
                class: "text-2xl font-bold {accent_color} mb-2",
                "{title}"
            }
            
            // Description
            p { 
                class: "text-sm text-slate-400 mb-4",
                "{description}"
            }
            
            // Features
            ul { class: "space-y-1",
                for feature in features {
                    li { 
                        class: "text-xs text-slate-500 flex items-center gap-2",
                        span { class: "text-slate-600", "›" }
                        "{feature}"
                    }
                }
            }
            
            // Select hint
            if enabled {
                div {
                    class: "mt-6 text-center opacity-0 group-hover:opacity-100 transition-opacity",
                    span { 
                        class: "text-xs font-semibold {accent_color} uppercase tracking-wide",
                        "Auswählen →"
                    }
                }
            }
        }
    }
}
