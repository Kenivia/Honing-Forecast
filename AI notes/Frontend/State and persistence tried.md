# State and persistence: don't try

Traps and ruled-out alternatives. The current state is in `State and persistence.md`.

## Roster and character actions

- Don't splice or swap `profiles` from a component, or keep the active character as an index across a delete or move: it silently switches character. The store actions re-point by id.
- Don't derive a new roster id from the highest existing one: a deleted roster's id is reused and the new roster inherits its chests, income, scan pages and snapshots. Integer ids also collide across users' saves (every save had a roster 0), which the analytics uploads key by.
- Don't keep a uuid beside the integer roster id (a second id to sync) or a stored `roster_order` list (a second source of truth for what the order of `profiles` already says).
- Don't append a new character to the end of `profiles`: rosters interleave. Insert after the roster's last.

## Saving and loading

- Don't replace a save that throws on load with defaults silently: the next edit overwrites it for good. Copy it to `<key>_BROKEN` first, and surface it (banner with Download / Discard).
- Don't load a `_BROKEN` copy back automatically: the user may have a new save by then.
- Don't swallow a failed `_BROKEN` copy (quota): the next write overwrites the only copy. Pause saving instead.
- Don't treat a string that decompresses to `null` as "no save".
- Don't delete only the legacy key that was read: older keys stay forever.
- Don't rely on the debounced write alone: edits in the last 500 ms before closing the tab are lost without the `pagehide` write.
- Don't save locale-formatted number strings: values corrupted when the browser locale changed.
- Don't trust a pre-V8 column's `keys` array (see the main note): values landed on the wrong material.

## Routing

- Don't check the character or change-log version in `beforeEnter`: it runs only when a route is entered, so param-only changes (`/Alice/calc` to `/Nobody/calc`) skip it. Use the global `beforeEach`.
- Don't correct an unknown character name inside `CharView` by sending it to the first character's calc: it drops the sub-page. Don't leave the URL on the old name after a rename: a reload lands on the first character.

## Tier switch and inputs

- Don't fall back to +25 for a fetched level when the highest fetched-done level is not a key of the conversion table (every character at T4's default floor): every hand-ticked Done level becomes FetchedDone and cannot be unticked with `lock_fetched_done`.
- Don't fall back to `highest_done - 1` for a wanted level with no equal: use the want.
- Don't read revert eligibility off the active profile: the character being switched may not be it.
- Don't leave numeric inputs unbounded below (`0-5` stored -5) or let a trailing operator (`5+`) throw in the change handler.

## Market page convert button

- Don't zero both source cells after converting: the leftover below one ratio is lost (12 Blue gave 2 Serca Blue and lost 2 Blue).
- Don't read a disabled label as the 999999999 sentinel; read it by value.
- Don't hand-write the tooltip's list of conversions: it drifted from `CONVERTS_FROM` (Fusion and special leaps were converted too).

## Calculator panels and deductions

- Don't hold the chance dropdown's text or colour in refs set at mount, nor store the graph lines (`enabled_annotations`): both showed the previous character's band after a switch. Derive from `chance_band`.
- Don't rebuild the histogram payload without the strategy override when the band changes.
- Don't apply a header click's mixed status to every row of a tickbox grid: a Done row goes back to Want.
- Don't show a chest state whose sum no longer matches the count (`chest_gains` ignores it); use `state_fits`.
- Don't use one global budget snapshot for auto-deduction: with auto start off, a slider on character B wrote A's budgets into B; a snapshot older than a market-page edit reverted it. Don't leave a free-tap Succeed with the snapshot set: the next slider move undoes the unlock deduction.
- Don't charge the unlock whenever `starting_num_taps` is 0: it is charged again after artisan is typed by hand. Use Rust's rule.
- Don't rebuild the manual chance without the express bonus, nor read the free-tap "remaining" once at mount.
- Don't leave slider deductions outstanding after leaving the calculator: two characters of a roster overwrite each other's roster-bound cells and market-page or Choices edits are reverted. Settle on unmount.
- Don't reuse the calculator page across characters with instruction rows keyed only by piece and level: a row keeps the previous character's local slider value. Key by profile id.
