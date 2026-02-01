use dioxus::prelude::*;

use crate::domain::Profile;
use crate::ui::theme;

#[component]
pub fn KpiCard(title: String, value: String, description: Option<String>, profile: Profile) -> Element {
    rsx! {
        div {
            class: "{theme::panel_border(profile)} p-4 shadow-sm",
            h3 { class: "{theme::label_class(profile)}", "{title}" }
            p { class: "mt-2 text-2xl font-semibold {theme::text_secondary(profile)}", "{value}" }
            if let Some(desc) = description {
                p { class: "mt-1 text-xs {theme::text_muted(profile)}", "{desc}" }
            }
        }
    }
}
