# Income tried

Warnings for the Income page and its estimate (`Income.md`): what failed or was ruled out.

## The estimate

- **Don't break the fit at decreases automatically**: a different chest state, or a chest opened another way than the optimizer chose, shows as a decrease too. Runs start where the user marks them. Without breaks at all, a spend dragged the slope unless every snapshot on one side was ignored.
- **Don't allow negative income**: the fitted slope went negative across a spend without a break (-74.6 a week seen) and the author ruled income is never negative. Slope and manual rate are clamped at 0.
- **Don't place now at the clock's time in the estimate, or keep now's and the weeks' results for the session only**: the minute clock moved now's point, so the rates, so every projected week's payload hash, and leaving the page or waiting a minute re-ran all the weeks. Pin now to the time its budgets were first seen and save the results in the profile.
- **Don't store the optimizer's chest states in a snapshot**: the states of the moment are of the old upgrades and budgets and not optimized for the snapshot. Optimize each snapshot on its own, cached by payload hash.
- **Don't run snapshots cold in a priority order** (selected, now, estimate's snapshots newest first, then Before export, two at a time): replaced by one chain oldest first, each run starting from the state of the one before.

## The table

- **Don't show cells that do not exist in game** (char-bound and tradable Silver, tradable free taps, roster-bound free taps): they had inputs that meant nothing. Roster-bound free taps were kept at first since a few roster-bound chests hold special leaps, but `owned_at` counts every free tap as char bound so the cell always read 0.
- **Don't pre-fill every manual cell on each switch to manual**: it overwrote what was typed after an untick and tick. Fill only cells never typed.
- **Don't read typed values with `Number()`**: "1,000" saved as 0. Use `parse_locale_float`.

## The graphs

- **Don't draw past snapshots on the metrics graph** (the x range going back five weeks, a timeline of markers): a snapshot's metrics are under the current upgrades, which says little about the past once the upgrades change. Storing each snapshot's upgrades with their progress and running it with them was tried and dropped as too complicated; the graph starts at now.
- **Don't graph one material's amounts per snapshot with fitted lines** (hollow until run, one material at a time): redundant next to the metrics graph.
- **Don't draw the chance of running out as 1 - `chance_within` over every material of the tier**: juices included, it said nothing. Draw the chance to succeed within the chosen bands, for enabled base materials only.
