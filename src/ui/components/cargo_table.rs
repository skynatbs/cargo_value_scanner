use dioxus::prelude::*;

use super::confidence_badge::ConfidenceBadge;

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
}

#[component]
pub fn CargoTable(
    rows: Vec<CargoRow>,
    selected_id: Option<String>,
    on_select: EventHandler<String>,
    on_remove: EventHandler<String>,
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
            class: "overflow-hidden rounded-xl border border-slate-800 bg-slate-900/40",
            table {
                class: "min-w-full divide-y divide-slate-800 text-sm",
                thead {
                    class: "bg-slate-900/60 text-left text-xs uppercase tracking-wide text-slate-500",
                    tr {
                        th { class: "px-4 py-3 font-medium", "Commodity" }
                        th { class: "px-4 py-3 font-medium", "SCU" }
                        th { class: "px-4 py-3 font-medium", "EV (aUEC)" }
                        th { class: "px-4 py-3 font-medium", "Best Sell" }
                        th { class: "px-4 py-3 font-medium", "Confidence" }
                        th { class: "px-4 py-3" }
                    }
                }
                tbody {
                    class: "divide-y divide-slate-800",
                    for (row, selected) in rendered_rows {
                        CargoRowView {
                            row,
                            selected,
                            on_select: on_select.clone(),
                            on_remove: on_remove.clone(),
                        }
                    }
                    if is_empty {
                        tr {
                            td {
                                class: "px-4 py-6 text-center text-sm text-slate-500",
                                colspan: "6",
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
    on_select: EventHandler<String>,
    on_remove: EventHandler<String>,
}

#[component]
fn CargoRowView(props: CargoRowViewProps) -> Element {
    let row = props.row;
    let row_class = format!(
        "cursor-pointer transition-colors {}",
        if props.selected {
            "bg-indigo-500/10"
        } else {
            "hover:bg-slate-800/40"
        }
    );
    let select_id = row.id.clone();
    let remove_id = row.id.clone();
    rsx! {
        tr {
            class: row_class,
            onclick: move |_| props.on_select.call(select_id.clone()),
            td {
                class: "px-4 py-3 font-medium text-slate-100",
                "{row.commodity_name}"
            }
            td { class: "px-4 py-3 text-slate-300", "{row.scu}" }
            td { class: "px-4 py-3 text-slate-300", {format!("{:.0}", row.expected_value)} }
            td { class: "px-4 py-3 text-slate-400", {best_location_text(&row)} }
            td {
                class: "px-4 py-3",
                ConfidenceBadge { value: row.confidence }
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
