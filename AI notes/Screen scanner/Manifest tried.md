# Manifest: tried and dropped

Warnings for the manifest and export (`Manifest.md`): what failed or was ruled out.

- **Don't convert T4 to Serca on export** (a per-band "Convert" toggle, `floor(T4 / ratio)` added to the Serca label): it baked a choice into the budgets that the user then had to undo by hand, and the T4 remainder of a Serca character was lost. The export writes T4 unconverted and the calculator counts it as Serca (`merge_lower_tier`).
- **Don't convert in `change_tier` as well**: converting the bound and free-tap T4 values there on top of the payload merge counted them twice.
- **Don't spend T4 before Serca in the deduction**: the user may not have converted in game yet and should see the value the game shows.
- **Don't add manifest overrides** (editable cells and chest counts, deleting and adding chests, session only): an override per label and band has no page or window, so it cannot be told what to replace when some pages are seen again and others are not. Correct the slots before export, or the budgets and stored chests after it.
- **Don't write one scan straight into the roster's columns and chests**: scanning a second character replaced the first one's roster-bound and tradable finds, and a roster-bound chest belonged to whoever exported it. Save pages per owner and merge.
- **Don't merge saved pages per window**: a page not opened would count as empty. Per page accepts counting an item twice when it moved to a page not shown again.
- **Don't cover roster or character storage as a whole window**: opening one never-saved page uncovered the whole window. The cover is per saved page.
