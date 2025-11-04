use std::time::Duration;

use dioxus::prelude::*;

use crate::util::generate_id;

const TOAST_AUTO_DISMISS: Duration = Duration::from_secs(6);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToastMessage {
    pub id: String,
    pub kind: ToastKind,
    pub text: String,
}

impl ToastMessage {
    pub fn new(kind: ToastKind, text: impl Into<String>) -> Self {
        Self {
            id: generate_id("toast"),
            kind,
            text: text.into(),
        }
    }
}

pub fn push_toast(
    mut toasts: Signal<Vec<ToastMessage>>,
    kind: ToastKind,
    message: impl Into<String>,
) {
    let text = message.into();
    toasts.with_mut(|entries| {
        if entries.len() >= 5 {
            entries.remove(0);
        }
        entries.push(ToastMessage::new(kind, text));
    });
}

#[component]
pub fn Toast() -> Element {
    let toasts = use_context::<Signal<Vec<ToastMessage>>>();
    let views = toasts()
        .into_iter()
        .map(ToastView::from)
        .collect::<Vec<_>>();

    if views.is_empty() {
        return rsx! { Fragment {} };
    }

    rsx! {
        div {
            class: "pointer-events-none fixed inset-x-0 bottom-4 flex justify-center",
            ul {
                class: "space-y-3",
                for view in views {
                    ToastCard { view, toasts: toasts.clone() }
                }
            }
        }
    }
}

#[component]
fn ToastCard(view: ToastView, toasts: Signal<Vec<ToastMessage>>) -> Element {
    let toasts_for_timer = toasts.clone();
    let toast_id = view.id.clone();
    let _auto_dismiss = use_future(move || {
        let mut toasts = toasts_for_timer.clone();
        let id = toast_id.clone();
        async move {
            tokio::time::sleep(TOAST_AUTO_DISMISS).await;
            toasts.with_mut(|items| items.retain(|toast| toast.id != id));
        }
    });

    let class = format!(
        "pointer-events-auto flex items-start gap-3 rounded-xl border px-4 py-3 shadow-lg backdrop-blur {}",
        view.theme
    );
    rsx! {
        li {
            class: class,
            span { class: "text-lg", "{view.icon}" }
            p { class: "text-sm font-medium", "{view.text}" }
            button {
                class: "ml-3 text-xs uppercase tracking-wide text-slate-300 hover:text-white",
                onclick: move |_| {
                    let target = view.id.clone();
                    toasts.with_mut(|items| items.retain(|toast| toast.id != target));
                },
                "Dismiss"
            }
        }
    }
}

#[derive(Clone, PartialEq)]
struct ToastView {
    id: String,
    text: String,
    theme: &'static str,
    icon: &'static str,
}

impl From<ToastMessage> for ToastView {
    fn from(message: ToastMessage) -> Self {
        let (theme, icon) = match message.kind {
            ToastKind::Info => ("border-sky-500/40 bg-sky-500/10 text-sky-100", "ℹ️"),
            ToastKind::Success => (
                "border-emerald-500/40 bg-emerald-500/10 text-emerald-100",
                "✅",
            ),
            ToastKind::Warning => ("border-amber-500/40 bg-amber-500/10 text-amber-100", "⚠️"),
            ToastKind::Error => ("border-rose-500/40 bg-rose-500/10 text-rose-100", "⛔"),
        };

        ToastView {
            id: message.id,
            text: message.text,
            theme,
            icon,
        }
    }
}
