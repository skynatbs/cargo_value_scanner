#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod domain;
mod infra;
mod ui;
mod util;

use dioxus::prelude::*;

#[cfg(feature = "desktop")]
use dioxus_desktop::{tao::window::WindowBuilder, Config as DesktopConfig};

use crate::util::version::APP_NAME;

fn main() {
    // Wayland explicit-sync crashes on some drivers; fall back to GL unless the caller opts in.
    if std::env::var("WAYLAND_DISPLAY").is_ok() && std::env::var("WGPU_BACKEND").is_err() {
        std::env::set_var("WGPU_BACKEND", "gl");
    }

    // WebKit's DMABUF renderer opts into explicit sync; disable it unless the user overrides.
    if std::env::var("WAYLAND_DISPLAY").is_ok()
        && std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER").is_err()
    {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    let builder = LaunchBuilder::new();

    #[cfg(feature = "desktop")]
    let builder = {
        let config = desktop! {
            DesktopConfig::new().with_window(
                WindowBuilder::new()
                    .with_title(APP_NAME)
            )
        };
        builder.with_cfg(config)
    };

    #[cfg(not(feature = "desktop"))]
    let builder = builder;

    builder.launch(app::App);
}
