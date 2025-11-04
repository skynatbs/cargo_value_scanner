use std::sync::atomic::{AtomicUsize, Ordering};

pub mod assets;
pub mod persistence;
pub mod version;

static ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

pub fn generate_id(prefix: &str) -> String {
    let value = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}-{value}")
}
