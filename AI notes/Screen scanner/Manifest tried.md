# Manifest: tried and dropped

## Conversion toggles on the export (October 2026)

Before: the manifest had a "Convert ... T4 to Serca" toggle per band (`convert`, `convert_owner`; the char-bound one on by default for a Serca character). The export wrote the converted amounts: the T4 remainder under the T4 label, `floor(T4 / ratio)` added to the Serca label, tradable coming out roster-bound.

Now the export writes T4 unconverted and the calculator counts it as Serca for a Serca character (`merge_lower_tier` in the payload). The deduction first spent T4 before Serca within a band; it now spends Serca first, since the user may not have converted in game yet and should see the value the game shows. `change_tier` used to convert the bound and free-tap T4 values into Serca ones (and back), which with the payload merge counted them twice; it converts nothing now. Converting at export baked a choice into the budgets that the user then had to undo by hand, and T4 stock of a Serca character was lost to the remainder.

## Manifest overrides (October 2026)

Before: the manifest's cells and chest counts were inputs behind an "Edit manifest" checkbox (`material_overrides`, `chest_overrides`), chests could be deleted (`deleted_chests`) and added by hand (`added_chests`), all session-only in `ScanStore.ts`.

Dropped as more trouble than they were worth once pages are saved and merged: an override per label and band has no page or window, so it cannot be told what to replace when some pages are seen again and others are not. Corrections now go on the slots before export, or on the budgets after it, and the budget snapshots record those. Chest editing moved to the calculator page (`ChestEditor.vue`), on the stored chests.

## Export overwrote the roster from one scan (October 2026)

Before: export wrote `owned[label][band]` of the scan alone into the roster's columns, and every chest into the exporting character's `chests`. Scanning a second character replaced the first one's roster-bound and tradable finds, and a roster-bound chest belonged to whichever character exported it.

Now pages are saved per owner and merged (`Manifest.md`, "Saved pages and other characters"). Merging per window, not per page, was considered and rejected: a page not opened would then count as empty. Per page accepts counting an item twice when it moved to a page not shown again.
