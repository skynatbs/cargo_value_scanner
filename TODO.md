# Cargo Value Scanner — TODO (Dioxus 0.7 + Tauri + Tailwind)

> Target: Rust desktop app (Tauri + Dioxus 0.7), with the option to ship to Web (WASM) later. Styling via Tailwind utility classes. Data from UEX API v2. Minimize external services; keep offline-friendly with caching.


## 2. Architecture
2.1 Single binary with feature flags:
- `desktop`
2.2 Layers:
- **UI** (Dioxus): Routes, components, state stores.
- **Domain**: Cargo items, pricing, profitability, best-price ranking.
- **Infra**: UEX client, cache, config persistence (file).
2.3 Files/folders:
- `/src/app.rs` (root app + router)
- `/src/ui/*` (pages/components)
- `/src/domain/*` (entities, logic)
- `/src/infra/uex.rs` (API client)
- `/src/infra/cache.rs`, `/src/infra/config.rs`

## 3. Data Structures
3.1 Core entities:
- `CommodityId` (string), `Commodity` { id, name, category }
- `CargoItem` { id, commodity_id, commodity_name, scu: f64, refined: bool }
- `PricePoint` { location_id, location_name, price_per_scu: f64, volatility: Option<f32>, updated_at }
- `SellLocation` { id, name, system, kind, armistice: bool }
- `CargoEvaluation` { ev: f64, min: Option<f64>, max: Option[f64], confidence: f32 }
- `ProfitabilityParams` { risk_pct: f64, crew_hourly: f64, crew_size: u8, time_minutes: u16 }
- `BestPrice` { commodity_id, location_id, price_per_scu, notes }
- (future) `CrewMember` { id, name, role, weight }
3.2 AppState:
- `Vec<CargoItem>`
- `HashMap<CommodityId, Vec<PricePoint>>`
- `HashMap<String, SellLocation>`
- `ProfitabilityParams`
- Cache metadata: fetched_at per resource.

## 4. UEX API Integration
4.1 Configure base URL (UEX v2). Expose a typed client:
https://uexcorp.space/api/documentation/
- `get_commodities() -> Result<Vec<Commodity>>`
- `get_prices(commodity_id: &str) -> Result<Vec<PricePoint>>`
- `get_locations() -> Result<Vec<SellLocation>>`
4.2 Add 60 min in-memory TTL cache.
4.3 Add ETag/Last-Modified support if available; else manual TTL.
4.4 Map volatility/age to a confidence score `0.0..1.0`:
- Recent data + low volatility → high confidence
- Stale or high volatility → warn in UI
4.5 Error handling:
- Network errors → show toast + fall back to last cached data.
- Missing commodity/location → mark item as partial and continue.

## 5. Cargo Input & Value Calculation
5.1 UI form: add/edit items (commodity, SCU, refined). Autocomplete from commodity list.
5.2 Compute Expected Value (EV) per item:
- `EV = SCU * avg(price_per_scu)` using current `PricePoint`s for that commodity.
5.3 Range:
- If `min`/`max` available from UEX → show; otherwise compute from low/high quartiles if possible or omit.
5.4 Confidence:
- Derive from `volatility` and `updated_at` freshness.
5.5 Total EV:
- Sum per-item EV, show as primary KPI (with confidence badge).

## 6. Profitability Indicator
6.1 Parameters (user-configurable with sensible defaults):
- `risk_pct` (0.0–0.4), `crew_hourly`, `crew_size`, `time_minutes`
6.2 Score:
- `score = EV_total - (risk_pct * EV_total) - (crew_hourly * crew_size * time_minutes/60.0)`
6.3 Indicator:
- Green if `score >= threshold_high`
- Yellow if `threshold_low <= score < threshold_high`
- Red if `score < threshold_low`
6.4 Show thresholds in settings and display a short textual rationale (e.g., "High value / Low risk / Small crew cost").

## 7. Best-Price Finder
7.1 For each cargo item, collect top N sell locations by `price_per_scu`.
7.2 Compute a simple travel penalty heuristic:
- Cross-system? +X
- Armistice? +Y
- Known hotspot? +Z (manual list for now)
7.3 Rank by `adjusted = price_per_scu - penalty`.
7.4 Show per-item top 3 and a combined "best overall" suggestion.
7.5 Provide quick-copy summary for sharing.

## 8. UI (Dioxus + Tailwind)
8.1 Routes:
- `/cargo` (default): table editor with EV & confidence
- `/best-price`: recommendations
- `/settings`: risk/cost params, cache controls
- (future) `/crew`
8.2 Components:
- `KpiCard`, `CargoTable`, `PriceTable`, `ConfidenceBadge`, `ProfitIndicator`, `Toast`
8.3 Tailwind:
- Use utility classes directly in Dioxus elements (`class="grid gap-4 ..."`).
- Ship a compiled `tailwind.css`; inject into desktop/web host.
8.4 Export:
- CSV export of current evaluation to user-chosen path (Tauri dialog).
8.5 Accessibility: keyboard nav for table, readable color contrast.

## 9. Persistence, Caching & Config
9.1 Config file (JSON) stored in app data dir (Tauri path):
- defaults for risk/cost/time, cache TTLs, favorite locations.
9.2 Cache:
- Memory map + optional on-disk snapshot for offline mode.
9.3 Clear cache button + indicator of data age in UI.
9.4 Offline fallback: use snapshot; mark confidence as low.

## 10. Error Handling & Telemetry
10.1 Use `thiserror/anyhow` and `tracing` with `INFO` default.
10.2 Map common failures to user-facing messages (timeout, 429, schema change).
10.3 Implement naive exponential backoff (jitter) for UEX API.
10.4 Add a minimal health panel showing last fetch status per resource.

## 11. Packaging & Targets
11.1 Desktop (primary): Tauri bundle for Windows/Linux/macOS.
11.2 Web (optional): WASM build with Dioxus web renderer; host static files.
11.3 Include fallback commodity list JSON in assets for offline boot.
11.4 Embed version/build info; generate `CHANGELOG.md` entries per tag.

## 12. Future: P2P Crew & Auto-Split
12.1 P2P session (no central server):
- Host generates short join code (UUID v4 short).
- Transport: WebRTC DataChannels or QUIC (evaluate `webrtc` or `quinn`).
- Sync: host authoritative; incremental diffs; optional CRDT later.
- Security: X25519 key exchange, AEAD for messages.
12.2 Auto-Split:
- Rules: Equal, Weighted (role/risk), Captain’s Cut (fixed 10% off the top).
- Base amount: realized sell value or EV if hypothetical.
- Export payout CSV and human-readable summary.
12.3 Discord share (optional): webhook post with summary.

## 13. Testing
13.1 Unit tests for EV calculation, profitability score, ranking heuristic.
13.2 Mock UEX client (feature `mock_uex`) with canned JSON files.
13.3 Integration tests: cache TTL, offline fallback, error surfaces.
13.4 Golden tests for CSV export and summary generation.

## 14. Documentation
14.1 `README.md` with screenshots and usage steps.
14.2 `CONFIG.md` documenting all tunables and defaults.
14.3 `UEX_MAPPING.md` documenting field mapping & assumptions (volatility → confidence).
14.4 Comment endpoints and expected JSON schema; provide examples.

## 15. MVP Completion Criteria (v0.1.0)
15.1 Manual cargo input.  
15.2 Live UEX fetch (commodities, prices, locations).  
15.3 EV + profit indicator visible and reactive.  
15.4 Cache with offline fallback.  
15.5 CSV export.  

---

### Notes for the AI Agent
- Prefer pure Rust paths; avoid relying on external binaries at runtime.
- Keep network layer isolated (`infra::uex`) to swap or stub easily.
- Keep UI pure & deterministic; use a central state store to avoid prop-drilling.
- Never crash the UI on network/schema errors; degrade gracefully and surface confidence.
- Ensure Tailwind build runs in CI and `tailwind.css` is versioned with the app.
