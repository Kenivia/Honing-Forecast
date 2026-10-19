# Manifest: tried and dropped

## Manifest overrides (October 2026)

Before: the manifest's cells and chest counts were inputs behind an "Edit manifest" checkbox (`material_overrides`, `chest_overrides`), chests could be deleted (`deleted_chests`) and added by hand (`added_chests`), all session-only in `ScanStore.ts`.

Dropped as more trouble than they were worth once pages are saved and merged: an override per label and band has no page or window, so it cannot be told what to replace when some pages are seen again and others are not. Corrections now go on the slots before export, or on the budgets after it, and the budget snapshots record those. Chest editing moved to the calculator page (`ChestEditor.vue`), on the stored chests.

## Export overwrote the roster from one scan (October 2026)

Before: export wrote `owned[label][band]` of the scan alone into the roster's columns, and every chest into the exporting character's `chests`. Scanning a second character replaced the first one's roster-bound and tradable finds, and a roster-bound chest belonged to whichever character exported it.

Now pages are saved per owner and merged (`Manifest.md`, "Saved pages and other characters"). Merging per window, not per page, was considered and rejected: a page not opened would then count as empty. Per page accepts counting an item twice when it moved to a page not shown again.
