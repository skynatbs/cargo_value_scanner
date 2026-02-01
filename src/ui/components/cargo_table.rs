use dioxus::prelude::*;

use super::confidence_badge::ConfidenceBadge;
use crate::domain::Profile;
use crate::ui::theme;

#[derive(Clone, PartialEq)]
pub struct CargoRow {
    pub id: String,
    pub commodity_name: String,
    pub scu: u32,
    pub expected_value: f64,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub confidence: f32,
    pub best_sell_location: Option<String>,
    pub is_hot: bool,
}

#[component]
pub fn CargoTable(
    rows: Vec<CargoRow>,
    selected_id: Option<String>,
    profile: Profile,
    on_select: EventHandler<String>,
    on_remove: EventHandler<String>,
    on_toggle_hot: EventHandler<String>,
) -> Element {
    let is_empty = rows.is_empty();
    let rendered_rows = rows
        .into_iter()
        .map(|row| {
            let selected = selected_id.as_ref().map_or(false, |id| id == &row.id);
            (row, selected)
        })
        .collect::<Vec<_>>();
    rsx! {
        div {
            class: "{theme::table_container(profile)}",
            table {
                class: "min-w-full {theme::table_divider(profile)} text-sm",
                thead {
                    class: "{theme::table_header(profile)} text-left tracking-wide",
                    tr {
                        th { class: "px-4 py-3 font-medium", "Commodity" }
                        th { class: "px-4 py-3 font-medium", "SCU" }
                        th { class: "px-4 py-3 font-medium", "EV (aUEC)" }
                        th { class: "px-4 py-3 font-medium", "Best Sell" }
                        th { class: "px-4 py-3 font-medium", "Confidence" }
                        th { class: "px-4 py-3 font-medium text-center", "Hot" }
                        th { class: "px-4 py-3" }
                    }
                }
                tbody {
                    class: "{theme::table_divider(profile)}",
                    for (row, selected) in rendered_rows {
                        CargoRowView {
                            row,
                            selected,
                            profile: profile,
                            on_select: on_select.clone(),
                            on_remove: on_remove.clone(),
                            on_toggle_hot: on_toggle_hot.clone(),
                        }
                    }
                    if is_empty {
                        tr {
                            td {
                                class: "px-4 py-6 text-center text-sm {theme::text_muted(profile)}",
                                colspan: "7",
                                "Add cargo items to begin calculating expected value."
                            }
                        }
                    }
                }
            }
        }
    }
}

fn best_location_text(row: &CargoRow) -> String {
    row.best_sell_location
        .as_ref()
        .cloned()
        .unwrap_or_else(|| "n/a".to_string())
}

#[derive(Props, Clone, PartialEq)]
struct CargoRowViewProps {
    row: CargoRow,
    selected: bool,
    profile: Profile,
    on_select: EventHandler<String>,
    on_remove: EventHandler<String>,
    on_toggle_hot: EventHandler<String>,
}

#[component]
fn CargoRowView(props: CargoRowViewProps) -> Element {
    let row = props.row;
    let profile = props.profile;
    let selected_bg = match profile {
        Profile::Pirate => "bg-[#3b1712]/30",
        _ => "bg-indigo-500/10",
    };
    let hover_bg = match profile {
        Profile::Pirate => "hover:bg-[#3b1712]/20",
        _ => "hover:bg-slate-800/40",
    };
    let row_class = format!(
        "cursor-pointer transition-colors {}",
        if props.selected { selected_bg } else { hover_bg }
    );
    let select_id = row.id.clone();
    let remove_id = row.id.clone();
    let toggle_id = row.id.clone();
    let is_hot = row.is_hot;
    rsx! {
        tr {
            class: row_class,
            onclick: move |_| props.on_select.call(select_id.clone()),
            td {
                class: "px-4 py-3 font-medium {theme::text_secondary(profile)}",
                "{row.commodity_name}"
            }
            td { class: "px-4 py-3 {theme::text_secondary(profile)}", "{row.scu}" }
            td { class: "px-4 py-3 {theme::text_secondary(profile)}", {format!("{:.0}", row.expected_value)} }
            td { class: "px-4 py-3 {theme::text_muted(profile)}", {best_location_text(&row)} }
            td {
                class: "px-4 py-3",
                ConfidenceBadge { value: row.confidence }
            }
            td {
                class: "px-4 py-3 text-center",
                input {
                    r#type: "checkbox",
                    class: "h-4 w-4 cursor-pointer accent-[#ff9900]",
                    checked: is_hot,
                    onclick: move |evt| {
                        evt.stop_propagation();
                        props.on_toggle_hot.call(toggle_id.clone());
                    },
                }
            }
            td {
                class: "px-4 py-3 text-right",
                button {
                    class: "rounded-md border border-rose-500/40 px-2 py-1 text-[10px] font-semibold uppercase tracking-wide text-rose-200 hover:bg-rose-500/10",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        props.on_remove.call(remove_id.clone());
                    },
                    "Remove"
                }
            }
        }
    }
}
