//! Profile-specific theme helpers for consistent styling across pages.

use crate::domain::Profile;

// ============================================
// BUTTON STYLES
// ============================================

pub fn btn_primary(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "rounded-lg bg-[#3b1712] px-4 py-2 text-sm font-semibold text-[#ff9900] border border-[#5c2a1f] hover:bg-[#4a1f18] drake-glow",
        Profile::Trader => "rounded-lg bg-sky-500 px-4 py-2 text-sm font-semibold text-white hover:bg-sky-400",
        Profile::Miner => "rounded-lg bg-orange-500 px-4 py-2 text-sm font-semibold text-white hover:bg-orange-400",
        Profile::None => "rounded-lg bg-indigo-500 px-4 py-2 text-sm font-semibold text-white hover:bg-indigo-400",
    }
}

pub fn btn_active(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "rounded-lg px-5 py-2.5 text-sm font-semibold bg-[#3b1712] text-[#ff9900] border border-[#5c2a1f] drake-glow drake-flicker",
        Profile::Trader => "rounded-lg px-5 py-2.5 text-sm font-semibold bg-sky-500/20 text-sky-300 border border-sky-500/40 misc-glow",
        Profile::Miner => "rounded-lg px-5 py-2.5 text-sm font-semibold bg-orange-500/20 text-orange-300 border border-orange-500/40 argo-glow",
        Profile::None => "rounded-lg px-5 py-2.5 text-sm font-semibold bg-indigo-500/20 text-indigo-300 border border-indigo-500/40",
    }
}

pub fn btn_inactive(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "rounded-lg px-5 py-2.5 text-sm text-[#d4523a]/70 border border-[#3b1712] hover:border-[#5c2a1f] hover:text-[#ff9900]",
        Profile::Trader => "rounded-lg px-5 py-2.5 text-sm text-slate-400 border border-slate-700 hover:border-sky-600 hover:text-sky-300",
        Profile::Miner => "rounded-lg px-5 py-2.5 text-sm text-slate-400 border border-slate-700 hover:border-orange-600 hover:text-orange-300",
        Profile::None => "rounded-lg px-5 py-2.5 text-sm text-slate-400 border border-slate-700 hover:border-slate-600",
    }
}

pub fn btn_small_active(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "rounded px-2 py-1 text-xs font-semibold bg-[#3b1712] text-[#ff9900] border border-[#5c2a1f]",
        Profile::Trader => "rounded px-2 py-1 text-xs font-semibold bg-sky-500/20 text-sky-300 border border-sky-500/40",
        Profile::Miner => "rounded px-2 py-1 text-xs font-semibold bg-orange-500/20 text-orange-300 border border-orange-500/40",
        Profile::None => "rounded px-2 py-1 text-xs font-semibold bg-indigo-500/20 text-indigo-300 border border-indigo-500/40",
    }
}

pub fn btn_small_inactive(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "rounded px-2 py-1 text-xs text-[#d4523a]/60 border border-[#3b1712] hover:border-[#5c2a1f] hover:text-[#ff9900]",
        Profile::Trader => "rounded px-2 py-1 text-xs text-slate-500 border border-slate-700 hover:border-sky-600 hover:text-sky-300",
        Profile::Miner => "rounded px-2 py-1 text-xs text-slate-500 border border-slate-700 hover:border-orange-600 hover:text-orange-300",
        Profile::None => "rounded px-2 py-1 text-xs text-slate-500 border border-slate-700 hover:border-slate-600 hover:text-slate-300",
    }
}

// ============================================
// INPUT STYLES
// ============================================

pub fn input_class(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "rounded-lg border border-[#3b1712] bg-[#1a0a08] px-4 py-2.5 text-sm text-[#d4523a] focus:border-[#ff9900] focus:outline-none",
        Profile::Trader => "rounded-lg border border-slate-700 bg-slate-950 px-4 py-2.5 text-sm text-slate-100 focus:border-sky-500 focus:outline-none",
        Profile::Miner => "rounded-lg border border-slate-700 bg-slate-950 px-4 py-2.5 text-sm text-slate-100 focus:border-orange-500 focus:outline-none",
        Profile::None => "rounded-lg border border-slate-700 bg-slate-950 px-4 py-2.5 text-sm text-slate-100 focus:border-indigo-500 focus:outline-none",
    }
}

pub fn input_small(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "rounded-lg border border-[#3b1712] bg-[#1a0a08] px-3 py-2 text-sm text-[#d4523a] focus:border-[#ff9900] focus:outline-none",
        Profile::Trader => "rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-slate-100 focus:border-sky-500 focus:outline-none",
        Profile::Miner => "rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-slate-100 focus:border-orange-500 focus:outline-none",
        Profile::None => "rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-slate-100 focus:border-indigo-500 focus:outline-none",
    }
}

// ============================================
// PANEL / CONTAINER STYLES
// ============================================

pub fn panel_border(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "rounded-xl border border-[#5c2a1f] bg-[#3b1712]/40",
        Profile::Trader => "rounded-xl border border-sky-800/50 bg-slate-900/40",
        Profile::Miner => "rounded-xl border border-orange-800/50 bg-slate-900/40",
        Profile::None => "rounded-xl border border-slate-800 bg-slate-900/40",
    }
}

pub fn panel_solid(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "rounded-xl border border-[#3b1712] bg-[#1a0a08]/60",
        Profile::Trader => "rounded-xl border border-slate-800 bg-slate-900/40",
        Profile::Miner => "rounded-xl border border-slate-800 bg-slate-900/40",
        Profile::None => "rounded-xl border border-slate-800 bg-slate-900/40",
    }
}

// ============================================
// TABLE STYLES
// ============================================

pub fn table_container(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "rounded-xl border border-[#3b1712] bg-[#1a0a08]/60 overflow-hidden",
        Profile::Trader => "rounded-xl border border-sky-900/40 bg-slate-900/40 overflow-hidden",
        Profile::Miner => "rounded-xl border border-orange-900/40 bg-slate-900/40 overflow-hidden",
        Profile::None => "rounded-xl border border-slate-800 bg-slate-900/40 overflow-hidden",
    }
}

pub fn table_header(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "border-b border-[#3b1712] bg-[#3b1712]/50 text-xs uppercase text-[#d4523a]/80",
        Profile::Trader => "border-b border-sky-900/40 bg-sky-950/30 text-xs uppercase text-sky-400/70",
        Profile::Miner => "border-b border-orange-900/40 bg-orange-950/30 text-xs uppercase text-orange-400/70",
        Profile::None => "border-b border-slate-800 bg-slate-900/60 text-xs uppercase text-slate-500",
    }
}

pub fn table_divider(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "divide-y divide-[#3b1712]/60",
        Profile::Trader => "divide-y divide-sky-900/30",
        Profile::Miner => "divide-y divide-orange-900/30",
        Profile::None => "divide-y divide-slate-800",
    }
}

// ============================================
// TEXT STYLES
// ============================================

pub fn text_primary(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "text-[#ff9900]",
        Profile::Trader => "text-sky-300",
        Profile::Miner => "text-orange-300",
        Profile::None => "text-indigo-300",
    }
}

pub fn text_secondary(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "text-[#d4523a]",
        Profile::Trader => "text-slate-300",
        Profile::Miner => "text-slate-300",
        Profile::None => "text-slate-300",
    }
}

pub fn text_muted(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "text-[#d4523a]/60",
        Profile::Trader => "text-slate-500",
        Profile::Miner => "text-slate-500",
        Profile::None => "text-slate-500",
    }
}

pub fn label_class(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "block text-xs font-semibold uppercase text-[#d4523a]/70",
        Profile::Trader => "block text-xs font-semibold uppercase text-slate-500",
        Profile::Miner => "block text-xs font-semibold uppercase text-slate-500",
        Profile::None => "block text-xs font-semibold uppercase text-slate-500",
    }
}

// ============================================
// ACCENT / HIGHLIGHT STYLES
// ============================================

pub fn accent_text(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "text-[#ff9900]",
        Profile::Trader => "text-emerald-400",
        Profile::Miner => "text-amber-400",
        Profile::None => "text-indigo-400",
    }
}

pub fn link_class(profile: Profile) -> &'static str {
    match profile {
        Profile::Pirate => "text-xs font-semibold uppercase tracking-wide text-[#ff9900] hover:text-[#ff9900]/80",
        Profile::Trader => "text-xs font-semibold uppercase tracking-wide text-indigo-300 hover:text-indigo-100",
        Profile::Miner => "text-xs font-semibold uppercase tracking-wide text-orange-300 hover:text-orange-100",
        Profile::None => "text-xs font-semibold uppercase tracking-wide text-indigo-300 hover:text-indigo-100",
    }
}
