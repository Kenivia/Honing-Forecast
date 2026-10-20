# Choices

The character's Choices sub-page (`/:char/choices`, under Income in the sidebar): for each choice, what taking an option is worth to this character, in average gold under its plan, and a button that adds the option to the budgets. The only choices yet are the raids' bonus chests: nothing, or the chest's materials for its gold.

Code: `Components/Character/Choices/` (`Choices.vue` the page, `ChoiceJobs.ts` the runs), the choice type and budget writes in `Utils/Choices.ts`, the data in `frontend/Data/bonus_chests.json` (written by `scanner_setup/game_files/bonus_chests.py`, `Game files.md`, "Bonus chests"). `tests/choices.test.ts` covers the gold taken, the chests across tiers (T4 for Serca, special leaps included; Serca for T4), the difficulty shown and the raid order.

## A choice

`Choice` is a chest without a count: `id`, `group`, `title`, `band` and `options`, each a bag of label to amount, a cost being negative, and optionally `difficulty` and `item_level`. `others` is what else it gives, shown on hover. `PRESET_CHOICES` is the bonus chests; more presets join that list, and a choice the user makes would be the same type stored in the profile (not done yet).

**Every character sees every raid, whatever its tier**: difficulties are locked by item level, not tier, and a T4 character can be 1730 or more. The options go through `process_chests` like any chest: a Serca character's T4 materials become Serca ones at the ratio (T4 special leaps a fifth, as Serca special leaps, which Rust adds to the free-tap budget), and a T4 character's Serca materials are dropped, leaving the gold and shards, so such a chest is worth little or less than nothing.

## What an option is worth

- **The run without** (`base_job`): the budgets now (`snapshot_of`, as Income's "now") with the character's and roster's chests, optimized, then its histogram.
- **A run per option that is not empty** (`option_job`): the same with the option as a one-option chest of count 1 (`as_chest`), so it is taken; an empty option is the run without. It goes on from the optimizer state of the run without (`states_payload`), which the optimizer takes as a plan to go on from (`Calculator core/Optimizer.md`, "Chained runs"), and is queued only once that is done.
- **Worth** is the option's metric of the user's plan (`metric_per_plan[1]`) less the run without's: positive means take it. Each option is against the budgets as they are, alone, so two gates are not taken together.
- Results are kept for the session by the hash of the payload without its seed; a budget change makes new hashes and every run again. The runs go through `WasmInterface/OptimizerJobs.ts` (`Income.md`); leaving the page cancels them. About 30 s for 35 rows of a small plan; the three raids shown take a few seconds.
- What the metric sees of the gold: the chest's gold is negative in its band, taken from char bound, then roster bound, then bought (`Calculator core/Domain model.md`, "Chests").

## The page

One card: a table of gate, cost, what it gives (icons and amounts, `others` on hover), worth (green positive, red negative, "…" until run) and "Add to budget". Rows are grouped by raid, newest first: by the item level of the raid's highest difficulty (`highest_level`), whatever the character's, which is the order of release; ties keep the data's order.

**Three raids** are shown and run, the newest; "Show one more raid" adds the next each press, for the visit (back to three on another character). Hidden raids are not run.

**Difficulties**: a raid's header has a button per difficulty (`difficulties`, lowest item level first; one above the character's item level is greyed), and only the rows of the one shown are listed and run. Which is shown (`shown_difficulty`): the one picked by hand, saved per character and raid in `profile.choice_levels` (by group name), while the raid still has it; else the highest the character's achieved item level (`ilevel`, gear only) reaches; else the lowest. A click is a pick, even on the one shown already, and there is no way back to automatic but picking the one wanted. A choice without a `difficulty` always shows.

**No upgrades** (the run without has no `upgrade_info`): nothing is run, every worth is "N/A" and a warning says to select some. Every run would be 0 otherwise, which reads as "costs nothing".

**A worth of 0 with upgrades** is real: under the default plan unused char-bound and roster-bound leftovers are worthless, gold included, so a chest paid from bound gold the upgrades would not use, giving materials they do not need, changes nothing.

**Add to budget** (`take_option`) adds what the option gives to the band's column (the char-bound budget for the bonus chests; free taps to the free-tap budget), unconverted, and takes a cost from that band, then roster bound, then tradable. It is disabled when the bands together cannot pay (`affordable`). Nothing marks a choice as taken.
