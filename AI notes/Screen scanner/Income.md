# Income

The character's Income sub-page (`/:char/income`, linked under Scanner in the sidebar): the roster's budget snapshots (`Manifest.md`, "Snapshots") as one character has them over time, a graph, and an estimate of income per week. It is also the snapshot explorer; there is no other snapshot page.

Code: `Components/Character/Income/` (`Income.vue` the page, `IncomeGraph.vue` the Chart.js graph, `IncomeJobs.ts` the optimizer runs), the pure parts in `Utils/Income.ts`, and `payload_for` / `states_payload` in `PayloadBuilder.ts`. `tests/income.test.ts` covers the resets, the fit, a Serca snapshot's values and a reload.

## What a snapshot holds for a character

`owned_at`: per label of the character's **current** tier, its bound band and the roster's two bands, with `merge_lower_tier` (T4 into Serca, tradable T4 into roster bound), plus what the chests give (`chest_gains` of `process_chests` over its own and the roster's chests). Free taps are the special leap row, all char bound (`[free taps, 0, 0]`): the free-tap budget (T4 special leaps merged) plus every band of the special leap gains. Otherwise they go through the same code as any material; their roster-bound and tradable values and income are always 0.

**The chests are opened as an optimizer run decides.** A snapshot keeps no chest states. Each snapshot is run through the optimizer, then the histogram with the optimizer's states, under the character's current upgrades and settings and the snapshot's own materials, chests and prices (`payload_for`; prices are the region's per bundle key and its bundle choice, stored in the snapshot). The result, the chest gains and the two metrics the calculator shows (average tradable gold spent, value of the leftovers, as `metric_per_plan`), is kept in the snapshot under the character, by a hash of the payload (`results`, the latest four per character). Upgrades changing means new hashes and new runs. Until its run is back a point uses every chest on its first option, as Rust does without a state, and is drawn hollow. "Now" is run the same way and kept for the session only.

`IncomeJobs.ts` runs two at a time on two worker bundles of its own: the selected snapshot first, then now (if included), then those the estimate uses, newest first, then the "Before export" ones. Ignored snapshots run only when selected. Leaving the page cancels them. The histogram's payload is built with the optimizer result's `prep_output.juice_info`, so it does not need the calculator's histogram to have run.

## The estimate

- Uses "After export" and "Manual" snapshots that are not ignored, plus now when "Include now" is ticked (offered only while now differs from the latest snapshot, materials and chests compared, prices not). There are two of that checkbox, one by the graph and one above the table, on one value, not saved.
- One least squares slope per material through runs of points, each run with its own intercept (`fit`). A run starts at a snapshot the user marked "Start a new line here" (`breaks`, for a spend before it); there is no automatic break at decreases, since a different chest state or a chest opened another way would look like one. Ignoring (`ignored`) drops a point altogether. Both are saved on the snapshot, for every character, and both apply to every material. The steps and days above the table are summed within runs.
- Week boundaries (Thursday 10:00 UTC, both regions) are only drawn.

## The page

- **Band group**: Char bound, + Roster bound, + Tradable. The graph shows the bands up to it summed; the **table shows that one band alone**, both its "Owned now" and "Income / week". The fit is linear in the values, so the table's per-band slopes add up to the graph's.
- **Materials** shown on the graph: icon toggles, Gold and the tier's Fusion by default, not saved. Each material has its own y axis (gold and fusion differ by orders of magnitude), alternating left and right.
- **The x range** is centred on now, the larger of five weeks and the age of the oldest snapshot either side.
- **Timeline** under the graph, lined up with its plot area (the graph emits it): a marker per snapshot, "Before export" in a lower lane, ignored ones faded. Hover shows one in the dashboard, a click locks it, as on the slot grid. The selected one is a white line on the graph.
- **Dashboard**: time and kind, "Ignore in the estimate" (not for "Before export"), Delete, the two metrics under the current upgrades, and what changed since the snapshot before (`snapshot_diff` for this character, chest stacks included). Comparing two arbitrary snapshots is gone.
- **Table**: every material of the tier and Free taps; nothing known is 0. "Type the income by hand" switches to manual, pre-filling every band of every row with the auto value (rounded); the auto value stays beside each input. In manual the graph's line starts at now's value with the summed manual rates; in auto it is each run's line over its own points, the last run on to the end of the range. A break shows as a bar before the snapshot on the timeline.
- **Manual income is saved** per band: char bound and free taps (under the special leap label) in the profile (`bound_income`, `income_manual`), roster bound and tradable per roster (`roster_income[roster_id]`, `[roster bound, tradable]`), so every character of the roster shares them. Nothing reads them outside this page yet.
