//! Persistent on-disk caching for terminal data with TTL + version tracking.

use std::{
    collections::HashSet,
    fs,
    path::PathBuf,
    sync::OnceLock,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::domain::{Terminal, TradeRoute};

const CACHE_FILENAME: &str = "terminal_cache.json";
const ROUTES_CACHE_FILENAME: &str = "routes_cache.json";

/// Cache TTL: 7 days. Terminals don't change often (only with major patches).
pub const TERMINAL_CACHE_TTL: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/// Cached terminal data with TTL + version tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalCache {
    /// Game version when this cache was created.
    pub game_version: String,
    /// Unix timestamp (seconds) when this cache was created.
    pub cached_at: u64,
    /// All terminals from the API.
    pub terminals: Vec<Terminal>,
}

impl TerminalCache {
    /// Create a new cache with current timestamp.
    pub fn new(game_version: String, terminals: Vec<Terminal>) -> Self {
        let cached_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            game_version,
            cached_at,
            terminals,
        }
    }

    /// Get all NQA terminal IDs as a HashSet for fast lookup.
    pub fn nqa_terminal_ids(&self) -> HashSet<i32> {
        self.terminals
            .iter()
            .filter(|t| t.is_nqa)
            .map(|t| t.id)
            .collect()
    }

    /// Check if a terminal ID is NQA.
    pub fn is_nqa(&self, terminal_id: i32) -> bool {
        self.terminals
            .iter()
            .any(|t| t.id == terminal_id && t.is_nqa)
    }

    /// Check if cache has expired (older than TTL).
    pub fn is_expired(&self) -> bool {
        self.age() > TERMINAL_CACHE_TTL
    }

    /// Get cache age as Duration.
    pub fn age(&self) -> Duration {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Duration::from_secs(now.saturating_sub(self.cached_at))
    }

    /// Human-readable age string.
    pub fn age_string(&self) -> String {
        let secs = self.age().as_secs();
        if secs < 60 {
            format!("{secs}s")
        } else if secs < 3600 {
            format!("{}m", secs / 60)
        } else if secs < 86400 {
            format!("{}h", secs / 3600)
        } else {
            format!("{}d", secs / 86400)
        }
    }
}

/// Get the cache file path (in app data directory).
fn cache_path() -> PathBuf {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let base = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("cargo-value-scanner");
        
        // Ensure directory exists
        let _ = fs::create_dir_all(&base);
        
        base.join(CACHE_FILENAME)
    })
    .clone()
}

/// Load terminal cache from disk, if it exists.
pub fn load_terminal_cache() -> Option<TerminalCache> {
    let path = cache_path();
    
    if !path.exists() {
        println!("[cache] No terminal cache found at {}", path.display());
        return None;
    }

    match fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(cache) => {
                println!("[cache] Loaded terminal cache from {}", path.display());
                Some(cache)
            }
            Err(e) => {
                println!("[cache] Failed to parse terminal cache: {e}");
                None
            }
        },
        Err(e) => {
            println!("[cache] Failed to read terminal cache: {e}");
            None
        }
    }
}

/// Save terminal cache to disk.
pub fn save_terminal_cache(cache: &TerminalCache) -> Result<(), std::io::Error> {
    let path = cache_path();
    let content = serde_json::to_string_pretty(cache)?;
    fs::write(&path, content)?;
    println!(
        "[cache] Saved terminal cache ({} terminals, version {}) to {}",
        cache.terminals.len(),
        cache.game_version,
        path.display()
    );
    Ok(())
}

/// Get the stored game version from cache, if available.
pub fn cached_game_version() -> Option<String> {
    load_terminal_cache().map(|c| c.game_version)
}

// ============================================================================
// Trade Routes Cache (24h TTL)
// ============================================================================

/// Cache TTL for routes: 24 hours.
pub const ROUTES_CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);

/// Cached trade routes with TTL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutesCache {
    /// Unix timestamp (seconds) when this cache was created.
    pub cached_at: u64,
    /// All calculated trade routes.
    pub routes: Vec<TradeRoute>,
}

impl RoutesCache {
    /// Create a new cache with current timestamp.
    pub fn new(routes: Vec<TradeRoute>) -> Self {
        let cached_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self { cached_at, routes }
    }

    /// Check if cache has expired (older than 24h).
    pub fn is_expired(&self) -> bool {
        self.age() > ROUTES_CACHE_TTL
    }

    /// Get cache age as Duration.
    pub fn age(&self) -> Duration {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Duration::from_secs(now.saturating_sub(self.cached_at))
    }

    /// Human-readable age string.
    pub fn age_string(&self) -> String {
        let secs = self.age().as_secs();
        if secs < 60 {
            format!("{secs}s")
        } else if secs < 3600 {
            format!("{}m", secs / 60)
        } else if secs < 86400 {
            format!("{}h", secs / 3600)
        } else {
            format!("{}d", secs / 86400)
        }
    }
}

/// Get the routes cache file path.
fn routes_cache_path() -> PathBuf {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let base = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("cargo-value-scanner");
        let _ = fs::create_dir_all(&base);
        base.join(ROUTES_CACHE_FILENAME)
    })
    .clone()
}

/// Load routes cache from disk, if it exists and is not expired.
pub fn load_routes_cache() -> Option<RoutesCache> {
    let path = routes_cache_path();
    
    if !path.exists() {
        println!("[routes-cache] No cache found");
        return None;
    }

    match fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<RoutesCache>(&content) {
            Ok(cache) => {
                if cache.is_expired() {
                    println!("[routes-cache] Cache expired (age: {})", cache.age_string());
                    return None;
                }
                println!(
                    "[routes-cache] Loaded {} routes (age: {})",
                    cache.routes.len(),
                    cache.age_string()
                );
                Some(cache)
            }
            Err(e) => {
                println!("[routes-cache] Failed to parse: {e}");
                None
            }
        },
        Err(e) => {
            println!("[routes-cache] Failed to read: {e}");
            None
        }
    }
}

/// Save routes cache to disk.
pub fn save_routes_cache(cache: &RoutesCache) -> Result<(), std::io::Error> {
    let path = routes_cache_path();
    let content = serde_json::to_string(cache)?; // compact, not pretty (can be large)
    fs::write(&path, content)?;
    println!(
        "[routes-cache] Saved {} routes to {}",
        cache.routes.len(),
        path.display()
    );
    Ok(())
}
