# HF analytics

What the site's own analytics uploads offer for forecasting income and progression (preliminary, 2026-10-25: only the sanity report has been run). How they are collected, pulled and loaded is in `Frontend/External services.md`, "Analytics uploads"; the scripts are `scripts/analytics/` (`pnpm analytics`).

- **Too short for income yet**: 32 rosters and 164 snapshots, all from 2026-10-22 to 10-25, most rosters in one session. Income needs differences across sessions days apart (`External services.md`, "Reading the data").
- **Raid logs reach back to 2025-01**: lostark.bible lists each character's last 25 logs, so one upload gives a per-character history of weekly clears and gear score. Biased to characters that raid and log.
- **Gear** (item level per loadout) only from 2026-10-24, 55 characters; each exporter's follow-up a day later gives a before and after.
- **The sample is HF users**: players who plan their honing, not the population (`Market.md`, "Population").
