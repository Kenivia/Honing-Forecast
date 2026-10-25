# Market

Groundwork for forecasting material prices: the data on file and a viewer, no model yet (preliminary, 2026-10-25). The data lives in `ignored/data_fetcher/` (untracked); `scripts/market/app.py` (`pnpm market`) plots it, prices or population, with release markers. Sources that were ruled out are in `Market tried.md`.

## Data

- **Prices**: `fetch_data.py` appends the latest prices of every listed item for NAE and EUC (`market_data.jsonl`, not plotted) and tops up the daily min/max/average per item (`history.jsonl`, from 2025-07-09) from the third-party market API the site's Worker proxies (`Frontend/External services.md`). Only the missing days are fetched, up to yesterday (UTC); the last stored day is fetched again and the later fetch wins. The Serca materials have history from Serca's launch (2026-04-15); the Tremor books are not listed by the API yet.
- **Population**: `uwuowo.py` appends lostark.bible's `/stats/generic` page: characters and rosters per 10-ilvl bracket, NA and CE together, and per-class counts at or above each 20-ilvl step up to 1800. Counts are characters the site has seen (someone looked them up), so its growing coverage inflates them: compare shares, not counts. Pages exist for 2026-03-31 (Wayback), 05-19 to 06-29 and from 10-25.
- **The gap** (06-29 to 10-20) is estimated in `parse_estimate` from lostark.bible's per-class combat power samples, which character pages carry per ilvl bracket: one bard page a day (`uwuowo2.jsonl`) and archived pages of other classes (`wayback_cpd.jsonl`, `wayback_cpd.py`). The bard share at or above X among 1700+ is scaled to the stats page's bards at the last page before; bards to all classes and the 1700+ share are interpolated between the pages around the gap. Held out, the interpolation is within 0.4 points of every May–June page, and the estimate within 0.2 points of the archived all-class check on 10-09 and 10-10.
- **Events**: `scripts/market/events.csv`: raids, raid events, modes and honing changes since 2025-09, the date of each update's maintenance, with a source link per row.

## Observations so far

- Destruction stones (EUC) went from about 150 to 350 between early February and mid March 2026, before Serca: in line with Kurzan Front and Chaos Dungeon stones becoming character-bound on 2026-02-04.
- Destruction stones jumped from about 25 to 220 in mid July 2025; not explained.
- Characters at 1760+ went from about 2% (end of June) to 5% (late October) of those the site has seen.

## Open

- The page after the gap (10-25) comes after Vambraces (10-21); if they raised item levels, the end of the estimate is off.
- No release notes were reachable for June to October 2025, so events and honing changes there may be missing.
