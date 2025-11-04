#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod domain;
mod infra;
mod ui;
mod util;

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
    dioxus::launch(app::App);
}
