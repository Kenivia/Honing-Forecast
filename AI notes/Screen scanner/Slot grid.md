# Slot grid

What the scanner page shows: every slot of the three windows, coloured by how far the scanner got with it, and a dashboard to inspect and correct one slot. The scan is treated as a snapshot: amounts are assumed not to change while it is taken.

Code: `ScanStore.ts` (state), `SlotGrid.vue`, `SlotCell.vue`, `SlotDashboard.vue`. The status is worked out in Rust (`slot_status` in `scan_result.rs`).

## Status

Each slot in a scan result carries a status, a reason and a value (the tooltip's amount, else the number on the icon). The reason is a sentence for the user, written in Rust, saying why the slot needs a hover or is an error (for a chest, how many of its column's chests were read); the dashboard shows it in its own block.

| Status | Border | When |
| --- | --- | --- |
| not in the result | white, with a `?` | the slot's page has not been looked at yet |
| Pending | white | recognised, number not read yet |
| Good | green | a material whose icon number is under 9999; or a tooltip was resolved to the slot; or a chest that is accounted for |
| NeedHover | orange | a material showing 9999 or more with no tooltip; a chest not accounted for; an icon several materials share (the books) with no tooltip saying which |
| NeedTradability | orange | would be good, but no tooltip gave its tradability. Shown as needing a hover, unless assumed (below) |
| Error | red | a tooltip was resolved to the slot but gave no amount |
| Irrelevant | greyed out | matches no icon |
| edited | dotted blue | the user set it by hand |

- **Every slot needs its tradability**, which only a tooltip gives, so every material needs a hover even when its number reads. A chest slot takes the tradability of the chest that stands for it.
- **Assumed tradability.** The sidebar on the scanner page (`ScannerControlPanel.vue`) has "Assume roster storage is tradable" and "Assume char storage and inventory are char-bound", both off by default and not persisted. With one on, `shown_slot` in `ScanStore.ts` turns those windows' `NeedTradability` slots into good ones with the assumed value, and the dashboard marks it "(assumed)". Rust is never told: it keeps reporting `NeedTradability`, and a tooltip it reads replaces the assumption.
- **A shared icon needs its label.** A book slot carries `label`, the material its tooltip named (`Tooltips.md`), and is not good without one. The dashboard shows the label in place of the icon name, and the edit list offers the materials in place of the shared icon, so a book set by hand is set to its material.
- **The icon number** has everything but digits dropped, and nothing left means 1 (a single item shows no number). `9999+` therefore reads 9999.
- **The tooltip is trusted over the icon.** A disagreement between the two is not an error; the value is simply the tooltip's.
- **A chest is a slot whose icon is a chest's template**, `<icon>@<rarity>` for a top row of `templates/chests.json` (`Game files.md`). Any other chest shows as irrelevant. The result lists the chests the slot can be (`variants`): those of every icon it may be drawn with (`Pipeline.md`), and of those the ones that ask for an item level when one is written across the slot, or the ones that do not when none is.
- **A chest that can only open one way needs no hover for its contents.** When every chest the slot can be has the same kind and contents (129 of the 295 templates are a single chest, bars of gold among them), it is good as it stands, short of its tradability.
- **Chests are matched per column**, because a pushed-up tooltip only tells the column. A chest read there counts for a slot when one of the icons it is drawn with is one the slot may be (`Tooltips.md`). The slot is then good when the chest's stacked amount is the number on the slot, when the chest was read in that very slot, when a tooltip was resolved to it, or, for a slot whose number was misread, when as many chests with its icon were read in the column as there are slots showing it.
- **Stacks with the same icon and amount in a column** all go green from one hover: their tooltips would be identical.
- **The chest's title plays no part.** What a chest holds is a row of the game's table, picked by what its tooltip lists.
- **Green for a hovered material comes when the title read is back**, since only the title says which slot the tooltip belongs to.

## The page

- The windows sit side by side as in game (roster storage, character storage, inventory) and grow with the page; slots shrink to 28 px before the windows wrap.
- Each window has page tabs. The shown page follows the game: every result lists the page each located window shows (`pages`), and a window turns to it whenever that changes. A tab clicked by hand holds until the page in game next changes.
- Hovering a slot shows it in the dashboard. Clicking one locks it: the dashboard then stays on it whatever is hovered, until it is clicked again (clicking another slot moves the lock there). The slot being shown gets a dotted white outline outside the status border.
- For a chest slot the dashboard lists the chest read in that slot, else every chest with one of its icons read in its column. A chest's icon is named after its art (`use_12_177@epic`), so the dashboard and the edit list show the titles of the chests drawn with it instead (`item_name` in `Manifest.ts`).

## Edits

- The dashboard reads left to right: the slot with its status and reason, what was read (or what was set by hand), a chest's contents, then "Correct it".
- **Save edit** sets item, amount and tradability by hand. An item needs both its amount and its tradability: while one is missing its field is outlined red and the button is disabled. "Irrelevant" needs neither, and hides those fields. **Okay** is held to the same rule: it is disabled while what was read lacks an amount or a tradability. **Okay** (errors only) keeps what was read as an edit. **Retry** forgets the slot, edit included, so it goes white and is read again.
- The page owns the edits (`edits` in `ScanStore.ts`). They also go to Rust with the next scan, in the scan options beside the OCR texts (`SlotEdit`, `apply_edits`). Rust keeps them in their own map, leaves those slots alone (no matching, no tooltip writes, not counted among a column's chests) and leaves them out of the result. A retry removes the slot from Rust's state and forces scanning to go on until it is read again.
- Addresses come out of reactive state and are copied to plain objects before they are queued; a proxy cannot be posted to the worker.
- **After Stop** the worker is gone but the store is not, so the grid stays up and edits still work; a retry then only blanks the slot. Starting a capture (or changing resolution or character with a stream open) calls `reset_scan`, and the capture card warns about that while idle.
