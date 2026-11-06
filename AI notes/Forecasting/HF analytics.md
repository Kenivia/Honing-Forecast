# HF analytics

What the site's own analytics uploads offer for forecasting income and progression (as of the 2026-11-05 pull: 2026-10-22 to 11-05, 62 rosters, 447 characters, 431 snapshots). How they are collected, pulled and loaded is in `Frontend/External services.md`, "Analytics uploads"; the scripts are `scripts/analytics/` (`pnpm analytics`).

- **Own exports are what count**: a character's bound budget is only rescanned on its own export (`by`). 101 characters have one, about 20 have them a day or more apart, 6 a week. Material time series are thin.
- **Upgrades come from the gear lookups, not the grids**: users don't keep the snapshot grids current (big spends with no grid change). The lostark.bible lookups are richer (136 Serca characters looked up a day or more apart) and give when a piece was honed, to the hour of the loadout's update.
- **Honing goes into the vambrace**: over those 136, the vambrace gained 355 levels, all armour 170, weapons 21. Vambrace levels pile at +10 and +15.
- **Spending outweighs income over days**: almost every long-tracked character honed in its window; a vambrace +10 to +15 cost roughly 10-35k Serca Red, 20-100k Serca Blue, 400-2,000 leaps, 1-2M shards. Clean (no honing) windows are rare: one so far, a week over a reset.
- **Holdings grow with gear level** (Spearman about 0.5) for Red, Blue, Leaps, Shards and both juices; Fusion and Gold do not. Special leaps get spent down to a few dozen. Bound books and scrolls are rare (under 7% of Serca characters); Serca books show up tradable only. Gold sits in the roster's tradable band, Silver is roster-wide.
- **Raid logs reach back to 2025-01**: lostark.bible lists each character's last 25 logs, so one upload gives a per-character history of weekly clears and gear score. Biased to characters that raid and log.
- **The sample is HF users**: players who plan their honing, not the population (`Market.md`, "Population").
