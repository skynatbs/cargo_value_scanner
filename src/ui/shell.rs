use dioxus::prelude::*;

use crate::app::Route;
use crate::domain::{AppState, Profile};
use crate::ui::pages::ProfileSelectPage;

#[component]
pub fn Shell(children: Element) -> Element {
    let state = use_context::<Signal<AppState>>();
    let profile = state.with(|s| s.profile);
    
    // Show profile selector if no profile selected
    if !profile.is_selected() {
        return rsx! {
            div { class: "min-h-screen bg-slate-950 text-slate-100 font-sans",
                ProfileSelectPage {}
            }
        };
    }
    
    let current_route = use_route::<Route>();
    let nav = use_navigator();
    
    let mut state_mut = state;
    
    // Profile tagline
    let tagline = match profile {
        Profile::Pirate => "wir laden das für dich um",
        Profile::Trader => "was soll das kosten?!",
        Profile::Miner => "rocks go brrrr",
        Profile::None => "",
    };
    
    // Profile-specific header border color
    let header_class = match profile {
        Profile::Pirate => "border-b border-[#3b1712] bg-[#1a0a08]/95 backdrop-blur px-6 py-4",
        Profile::Trader => "border-b border-sky-900/40 bg-slate-950/90 backdrop-blur px-6 py-4",
        Profile::Miner => "border-b border-orange-900/40 bg-slate-950/90 backdrop-blur px-6 py-4",
        Profile::None => "border-b border-slate-900/60 bg-slate-950/80 backdrop-blur px-6 py-4",
    };
    
    // Profile-specific title color
    let title_class = match profile {
        Profile::Pirate => "text-xl font-semibold tracking-tight text-[#ff9900]",
        Profile::Trader => "text-xl font-semibold tracking-tight text-sky-200",
        Profile::Miner => "text-xl font-semibold tracking-tight text-orange-200",
        Profile::None => "text-xl font-semibold tracking-tight",
    };
    
    rsx! {
        div { class: "min-h-screen bg-slate-950 text-slate-100 font-sans",
            header {
                class: "{header_class}",
                div { class: "mx-auto grid max-w-6xl grid-cols-[1fr_auto_1fr] items-center gap-4",
                    // Left: Profile name + tagline
                    div { class: "flex items-center gap-3",
                        span { class: "text-2xl", "{profile.emoji()}" }
                        div {
                            h1 { class: "{title_class}", "{profile.name()}" }
                            p { class: "text-xs text-slate-500 italic", "{tagline}" }
                        }
                    }
                    
                    // Center: Profile switcher
                    div { class: "flex gap-1 justify-center",
                        ProfileButton {
                            active: profile == Profile::Pirate,
                            onclick: move |_| state_mut.with_mut(|s| s.profile = Profile::Pirate),
                            label: "🏴‍☠️ Pirat",
                            theme: Profile::Pirate,
                        }
                        ProfileButton {
                            active: profile == Profile::Trader,
                            onclick: move |_| state_mut.with_mut(|s| s.profile = Profile::Trader),
                            label: "📦 Händler",
                            theme: Profile::Trader,
                        }
                        button {
                            class: "min-w-[6rem] rounded-lg px-3 py-1.5 text-sm text-slate-600 border border-slate-800 cursor-not-allowed opacity-50",
                            disabled: true,
                            title: "Coming Soon™",
                            "⛏️ Miner"
                        }
                    }
                    
                    // Right: Navigation
                    nav { class: "flex gap-2 text-sm justify-end",
                        match profile {
                            Profile::Pirate => rsx! {
                                NavButton { active: matches!(current_route, Route::Routes {}), onclick: move |_| { nav.push(Route::Routes {}); }, label: "🎯 Lauern", profile: profile }
                                NavButton { active: matches!(current_route, Route::Cargo {}), onclick: move |_| { nav.push(Route::Cargo {}); }, label: "📦 Beute", profile: profile }
                                NavButton { active: matches!(current_route, Route::Planner {}), onclick: move |_| { nav.push(Route::Planner {}); }, label: "💰 Verkauf", profile: profile }
                            },
                            Profile::Trader => rsx! {
                                NavButton { active: matches!(current_route, Route::Routes {}), onclick: move |_| { nav.push(Route::Routes {}); }, label: "🗺️ Routen", profile: profile }
                                NavButton { active: matches!(current_route, Route::Cargo {}), onclick: move |_| { nav.push(Route::Cargo {}); }, label: "📦 Cargo", profile: profile }
                                NavButton { active: matches!(current_route, Route::Planner {}), onclick: move |_| { nav.push(Route::Planner {}); }, label: "💰 Verkauf", profile: profile }
                            },
                            _ => rsx! {
                                NavButton { active: matches!(current_route, Route::Cargo {}), onclick: move |_| { nav.push(Route::Cargo {}); }, label: "📦 Cargo", profile: profile }
                                NavButton { active: matches!(current_route, Route::Planner {}), onclick: move |_| { nav.push(Route::Planner {}); }, label: "💰 Verkauf", profile: profile }
                                NavButton { active: matches!(current_route, Route::BestPrice {}), onclick: move |_| { nav.push(Route::BestPrice {}); }, label: "Best Price", profile: profile }
                            },
                        }
                        NavButton { active: matches!(current_route, Route::Settings {}), onclick: move |_| { nav.push(Route::Settings {}); }, label: "⚙️", profile: profile }
                    }
                }
            }
            main { class: "mx-auto max-w-6xl px-6 py-10",
                {children}
            }
        }
    }
}

#[component]
fn NavButton(
    active: bool,
    onclick: EventHandler<()>,
    label: &'static str,
    profile: Profile,
) -> Element {
    let class = match (profile, active) {
        // Drake/Pirate - CRT amber with glow
        (Profile::Pirate, true) => {
            "min-w-[5.5rem] rounded-lg border border-[#5c2a1f] bg-[#3b1712] px-4 py-2 font-semibold text-[#ff9900] drake-glow drake-flicker drake-crt"
        }
        (Profile::Pirate, false) => {
            "min-w-[5.5rem] rounded-lg border border-[#3b1712] px-4 py-2 text-[#d4523a]/70 transition hover:border-[#5c2a1f] hover:bg-[#3b1712]/40 hover:text-[#ff9900]"
        }
        // MISC/Trader - Sky blue clean
        (Profile::Trader, true) => {
            "min-w-[5.5rem] rounded-lg border border-sky-500/60 bg-sky-500/15 px-4 py-2 font-semibold text-sky-300 misc-glow"
        }
        (Profile::Trader, false) => {
            "min-w-[5.5rem] rounded-lg border border-slate-700 px-4 py-2 text-slate-400 transition hover:border-sky-700 hover:bg-sky-900/20 hover:text-sky-300"
        }
        // Argo/Miner - Orange industrial
        (Profile::Miner, true) => {
            "min-w-[5.5rem] rounded-lg border border-orange-500/60 bg-orange-500/15 px-4 py-2 font-semibold text-orange-300 argo-glow"
        }
        (Profile::Miner, false) => {
            "min-w-[5.5rem] rounded-lg border border-slate-700 px-4 py-2 text-slate-400 transition hover:border-orange-700 hover:bg-orange-900/20 hover:text-orange-300"
        }
        // Default fallback
        (_, true) => {
            "min-w-[5.5rem] rounded-lg border border-indigo-500/60 bg-indigo-500/15 px-4 py-2 font-semibold text-indigo-300"
        }
        (_, false) => {
            "min-w-[5.5rem] rounded-lg border border-transparent px-4 py-2 text-slate-400 transition hover:border-slate-700 hover:bg-slate-900/80 hover:text-slate-200"
        }
    };
    
    rsx! {
        button {
            class: "{class}",
            onclick: move |_| onclick.call(()),
            "{label}"
        }
    }
}

#[component]
fn ProfileButton(
    active: bool,
    onclick: EventHandler<()>,
    label: &'static str,
    theme: Profile,
) -> Element {
    let class = match (theme, active) {
        (Profile::Pirate, true) => {
            "min-w-[6rem] rounded-lg px-3 py-1.5 text-sm font-semibold bg-[#3b1712] text-[#ff9900] border border-[#5c2a1f] drake-glow"
        }
        (Profile::Pirate, false) => {
            "min-w-[6rem] rounded-lg px-3 py-1.5 text-sm text-[#d4523a]/60 border border-[#3b1712] hover:border-[#5c2a1f] hover:text-[#ff9900] transition"
        }
        (Profile::Trader, true) => {
            "min-w-[6rem] rounded-lg px-3 py-1.5 text-sm font-semibold bg-sky-500/20 text-sky-300 border border-sky-500/40 misc-glow"
        }
        (Profile::Trader, false) => {
            "min-w-[6rem] rounded-lg px-3 py-1.5 text-sm text-slate-500 border border-slate-800 hover:border-sky-600 hover:text-sky-400 transition"
        }
        (Profile::Miner, true) => {
            "min-w-[6rem] rounded-lg px-3 py-1.5 text-sm font-semibold bg-orange-500/20 text-orange-300 border border-orange-500/40 argo-glow"
        }
        (Profile::Miner, false) => {
            "min-w-[6rem] rounded-lg px-3 py-1.5 text-sm text-slate-500 border border-slate-800 hover:border-orange-600 hover:text-orange-400 transition"
        }
        _ => {
            "min-w-[6rem] rounded-lg px-3 py-1.5 text-sm text-slate-500 border border-slate-800 hover:border-slate-600 hover:text-slate-300 transition"
        }
    };
    
    rsx! {
        button {
            class: "{class}",
            onclick: move |_| onclick.call(()),
            "{label}"
        }
    }
}
