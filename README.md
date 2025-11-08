# Cargo Value Scanner

Cargo Value Scanner helps Star Citizen traders decide where to move their cargo. Launch the packaged app for your platform (Windows, macOS, or Linux) and follow the guide below to explore every feature.

> **Tip:** An internet connection is required so the app can fetch live prices from the UEX API. When data fails to load you’ll see a toast notification explaining why.

---

## 1. Main Layout & Navigation

- **Header tabs:** `Cargo`, `Best Price`, and `Settings`. The active tab is highlighted.  
- **Toasts:** Status messages appear at the bottom of the screen and auto-dismiss after a few seconds.  
- **Sorting & filtering:** Most tables include sort buttons or controls—look near the top of each table.

---

## 2. Cargo Tab

1. **Add cargo / adjust quantities**
   - Use the **Commodity** autocomplete to pick an item by name.
   - Enter a positive number of **SCU** to add to that commodity, or a negative number to subtract from what you already have.
   - The app keeps only **one row per commodity**: adjustments are accumulated, and entering a negative value that brings the total to zero (or below) removes the row entirely.
   - You cannot subtract cargo that hasn’t been added yet; a toast will tell you if that happens.
2. **Review the cargo table**
   - Columns show EV (expected value), SCU, best sell location, and a confidence badge.
   - Click a row to reveal the **Price Breakdown** on the right.
3. **Price Breakdown**
   - Displays every known terminal for the selected commodity.
   - Columns include `Sell Range (aUEC)`, `Buy Range (aUEC)`, `Stock (SCU)`, `Demand`, `Containers (SCU)`, and `Updated`.
   - Use the **Sort** buttons (Sell Range, Buy Range, Stock, Demand) to reorder the table.
   - “Best Sell” / “Best Buy” badges highlight optimal terminals and are pinned to the top.
4. **Refresh buttons**
   - `Refresh` above the table fetches fresh data for the selected cargo item.
   - Toasts indicate whether data came from cache or a new API call.

---

## 3. Best Price Tab

This view ranks each cargo item’s top sell destinations.

1. **Best Overall card**
   - Shows the single best location across all cargo, with notes about risk (cross-system, armistice, etc.).
2. **Commodity cards**
   - Each card stretches the full width for readability.
   - Columns mirror the Price Breakdown (sell max, buy min, stock, demand, containers, adjusted price).
   - Use the per-card **Refresh** button to update prices for that commodity only.
3. **Quick Summary**
   - Read-only text area summarizing recommendations.
   - Use the **Copy** button to send the text to your clipboard (button briefly shows “Copied!” once successful).

---

## 4. Settings Tab

Use this tab to adjust cache TTLs (how long commodity/price data stays “fresh”) and view debugging info.

- **Refresh commodities** or **Clear cache** when the dataset feels stale.  
- The UI reflects when a data set is missing or out-of-date (warnings in yellow banners).

---

## 5. Practical Tips

- **Confidence meter:** Combines age, volatility, and stock levels. Low confidence means you should refresh data or expect more price variance.
- **Demand column wording:**  
  - `Sell: High/Normal/Low/Unavailable` → how eager the terminal is to buy from you.  
  - `Buy: High/Normal/Low/Unavailable` → whether the terminal sells that commodity back to you.  
  - “Unavailable” means that trade side is offline.
- **Keyboard & mouse shortcuts:**  
  - Click any cargo row to focus it; use the refresh button directly above the Price Breakdown to update that row.

---

## 6. Updates

- Open the **Settings** tab to see the installed version (derived from the current Git tag when available) and trigger a GitHub release check.
- Tap **Check for updates** to compare your build with the latest tag, then use **Update** to jump straight to the repository.

---

Enjoy planning profitable runs! If you encounter issues (missing data, API failures, etc.) the toast log will provide guidance on what went wrong. Upcoming builds may include additional analytics—keep an eye on release notes for new features.  
