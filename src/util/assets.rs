use std::{borrow::Cow, sync::OnceLock};

use rust_embed::RustEmbed;

/// Embed the entire `assets/` directory into the binary.
#[derive(RustEmbed)]
#[folder = "assets"]
struct EmbeddedAssets;

static MAIN_CSS: OnceLock<String> = OnceLock::new();
static TAILWIND_CSS: OnceLock<String> = OnceLock::new();
static FAVICON_DATA_URI: OnceLock<String> = OnceLock::new();
static UEX_LOGO_DATA_URI: OnceLock<String> = OnceLock::new();

/// Returns the contents of `assets/main.css` as a static string.
pub fn main_css() -> &'static str {
    MAIN_CSS
        .get_or_init(|| load_text("/assets/main.css"))
        .as_str()
}

/// Returns the contents of `assets/tailwind.css` as a static string.
pub fn tailwind_css() -> &'static str {
    TAILWIND_CSS
        .get_or_init(|| load_text("/assets/tailwind.css"))
        .as_str()
}

/// Returns a data URI for the favicon.
pub fn favicon_data_uri() -> &'static str {
    FAVICON_DATA_URI
        .get_or_init(|| load_data_uri("/assets/favicon.ico"))
        .as_str()
}

/// Returns a data URI for the UEX logo.
pub fn uex_logo_data_uri() -> &'static str {
    UEX_LOGO_DATA_URI
        .get_or_init(|| load_data_uri("/assets/uex_logo.svg"))
        .as_str()
}

fn load_text(path: &str) -> String {
    let asset = load_asset(path);
    String::from_utf8(asset.into_owned())
        .unwrap_or_else(|_| panic!("Embedded asset {path} is not valid UTF-8"))
}

fn load_data_uri(path: &str) -> String {
    let asset = load_asset(path);
    let mime = guess_mime(path);
    let encoded = encode_base64(asset.as_ref());
    format!("data:{mime};base64,{encoded}")
}

fn load_asset(path: &str) -> Cow<'static, [u8]> {
    let canonical = canonical_asset_path(path);
    EmbeddedAssets::get(&canonical)
        .map(|file| file.data)
        .unwrap_or_else(|| panic!("Failed to locate embedded asset: {path}"))
}

fn canonical_asset_path(path: &str) -> String {
    let trimmed = path.trim_start_matches('/');
    if let Some(rest) = trimmed.strip_prefix("assets/") {
        rest.to_string()
    } else {
        trimmed.to_string()
    }
}

fn guess_mime(path: &str) -> &'static str {
    if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else {
        "application/octet-stream"
    }
}

fn encode_base64(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity((input.len() + 2) / 3 * 4);

    for chunk in input.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);

        output.push(TABLE[(b0 >> 2) as usize] as char);
        output.push(TABLE[(((b0 & 0b11) << 4) | (b1 >> 4)) as usize] as char);

        if chunk.len() > 1 {
            output.push(TABLE[(((b1 & 0b1111) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            output.push('=');
        }

        if chunk.len() > 2 {
            output.push(TABLE[(b2 & 0b0011_1111) as usize] as char);
        } else {
            output.push('=');
        }
    }

    output
}
