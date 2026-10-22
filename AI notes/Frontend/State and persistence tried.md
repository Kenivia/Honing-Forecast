# State and persistence: history

## Roster CRUD moved into the store (Oct 2026)

Before: `RosterView.vue` spliced and swapped `profiles` directly. Deleting set the active index to `index - 1` whenever the active one was at or after the deleted one, so deleting an earlier character switched to someone else; reordering left the index on the old slot. Deleting a roster left its `roster_chests`, `roster_income`, scan pages and snapshots behind, and the next "Add new roster" (highest id + 1) reused the id and inherited them.

After: store actions that re-point the active index by id and prune the roster's data. Covered by `tests/unit/stores/roster_store.test.ts` and `tests/unit/shell/roster.test.ts`.

## Load failures and legacy keys (Oct 2026)

Before: a save that threw while loading was replaced by defaults, and the first edit then overwrote it, losing it for good. Only the legacy key that was read was deleted, so a V7 key next to a V8 one stayed forever. Edits in the last 500 ms before closing the tab were lost.

After: the unreadable string is copied to `<key>_BROKEN`, every legacy key is deleted once any save is read, and `App.vue` writes on `pagehide`.

## Convert button keeps the leftover (Oct 2026)

Before: the conversion lived in `MarketView.vue`, zeroed both source cells after converting (12 Blue gave 2 Serca Blue and lost 2 Blue), read a disabled label as 999999999, and its tooltip claimed only Red, Blue and Leaps although Fusion and special leaps were converted too.

After: `convert_pooled` in `Utils/Convert.ts` keeps the leftover (roster-bound spent first), reads disabled labels by value, and the tooltip lists every conversion from `CONVERTS_FROM`. Covered by `tests/unit/market-import/convert.test.ts`.

## Tier-switch grid remap and input bounds (Oct 2026)

Before: the remap fell back to +25 for the fetched level whenever the highest fetched-done level was not a table key, which is every character still at T4's default floor (+10). So every hand-ticked Done level became FetchedDone, and with `lock_fetched_done` could not be unticked. A wanted level with no equal fell back on `highest_done - 1`, not `highest_want - 1`. Eligibility was read off the active profile. Inputs had no lower bound (`0-5` stored -5), and a trailing operator (`5+`) threw in `MaterialCell`'s change handler.

After: fetched levels convert where fetched, otherwise the new tier's floor; the want fallback uses the want; the check takes the target profile; inputs clamp to 0 and a trailing operator reads as 0. Covered by `tests/unit/utils/status_input_util.test.ts` and `input_column.test.ts`.

## Route checks in a global guard (Oct 2026)

Before: the character and change-log version checks were `beforeEnter` guards, which only run when a route is entered, so a param-only change (`/Alice/calc` to `/Nobody/calc`, version to version) skipped them. `CharView` duplicated the name lookup and corrected unknown names to the first character's calc, dropping the sub-page. A rename left the URL on the old name, so a reload landed on the first character.

After: one `router.beforeEach` checks both on every navigation and keeps the sub-page; `CharView` only maps the name to the index and replaces the route on a rename. Covered by `tests/unit/shell/router.test.ts` and `charview.test.ts`.

## Auto-deduction snapshot and unlock (Oct 2026)

Before: one global `budget_snapshot`, cleared only by an optimizer start or the slider's Succeed. With auto start off, a slider on character B wrote A's budgets minus usage into B; a free-tap Succeed left it set, so the next slider move undid the unlock deduction; a snapshot older than a market-page edit reverted it. The unlock was charged whenever `starting_num_taps` was 0, so again after artisan was typed by hand. The manual chance input rebuilt the chance without the express bonus, and the free-tap "remaining" was read once at mount.

After: per-profile snapshots, retaken when nothing is outstanding, a success folded into the snapshot (`finish_upgrade`), Rust's unlock rule, `chance_percent` / `chance_to_taps` in `HoningUtil.ts`. Covered by `tests/unit/instructions/`.

## Calculator panels follow the character (Oct 2026)

Before: the chance dropdown's text and colour were refs set when the page mounted, and the graph lines were a stored global `enabled_annotations` rewritten on mount, so after switching character (the page is not remounted) both showed the previous character's band. Changing the band rebuilt the histogram payload without the strategy override. A grid header click applied the column's mixed status to every row, so a row already Done there went back to Want. ChestState showed a state whose sum no longer matched the count, which `chest_gains` ignores.

After: the dropdown is bound to `chance_band` and reruns through `start_eval_hist`; the lines are derived from `chance_band` and `enabled_annotations` is no longer saved (an old save's field is ignored); `click_column` moves only rows at the column's status; ChestState uses `state_fits`. Covered by `tests/unit/calc/`.

## Deductions settle on leaving the calculator (Oct 2026)

Before: a character's slider deductions stayed outstanding, against its snapshot, after switching character or page. Two characters of a roster deducting in turn overwrote each other's roster-bound cells, and a market-page or Choices edit made meanwhile was reverted by the next slider move. The calculator page was also reused across characters, its instruction rows keyed only by piece and level, so a row could keep the previous character's local slider value.

After: `Calc.vue`'s unmount calls `settle_deductions` for its own character, and `CharView` keys the calculator by profile id so a switch unmounts it. Covered by `tests/unit/instructions/success_utils.test.ts` (settle), `tests/unit/shell/charview.test.ts` (remount) and `tests/unit/calc/calc_page.test.ts` (own character only).

## An unreadable save is surfaced (Oct 2026)

Before: the `<key>_BROKEN` copy was never read back or shown (only a console error), never cleaned up so it kept taking quota, and when it did not fit the failure was swallowed and the next write overwrote the only copy. A save that decompressed to `null` was taken for no save at all. After: a banner offers Download (importable backup) and Discard; with no room for the copy, saving is paused instead; a `null` decompress counts as unreadable. No automatic recovery, deliberately. Covered by `tests/unit/stores/config_storage.test.ts` and `tests/unit/shell/broken_save_banner.test.ts`.

## Roster ids: integers to uuids (V10)

Before: roster ids were small integers per browser (`add_roster` took the highest + 1), rosters were listed sorted by id, and `add_char` / `duplicate_profile` pushed to the end of `profiles`, so a roster's characters could be interleaved with another's. Every save had a roster 0, so ids collided across users and saves, and a deleted roster's id was reused (hence the cleanup on delete).

After: uuids, with the order taken from `profiles` (first appearance) and kept stable by inserting a new character after its roster's last. Considered and dropped: a uuid kept beside the integer (a second id to keep in sync), and a stored `roster_order` list (a second source of truth for what `profiles` already says). Covered by `tests/unit/stores/roster_store.test.ts` (order after deleting a roster's first characters) and the migration suites.
