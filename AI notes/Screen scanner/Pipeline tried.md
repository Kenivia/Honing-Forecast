# Scanner pipeline: don't try these

Warnings behind `Pipeline.md`: what failed or was ruled out, with the one number that justifies it. Only read this when trying something new on the scanner. Recordings are named by their time (`17-28-55` and `22-55-45` are 1080p of 2026-10-07; `21-42-32` and `21-31-53` are 3440x1440 of 2026-10-08); the rest are in `ignored/recordings`.

## Anchors

- **Don't correct an anchor's position afterwards instead of cutting its template to whole pixels.** Over the native stills the spread of per-window offsets fell from 0.110 to 0.049 px and 30 more icons were recognised; correcting afterwards was only first-order (1080p offset 0.17 px against 0.03).
- **Don't search an area cut at a fraction of a pixel:** it is resampled, which moved an anchor's brightness estimate by up to two settings.
- **Don't wait for a brightness estimate before re-checking anchors:** nothing was re-checked until there was one.
- **Don't use text or a greyed-out button as a second variant** ("Move All Duplicate Materials", the inventory's bottom-left button). The first is drawn anew at each resolution (it only just passed at 1080p); the second is greyed out in the storage view and needed another template.
- **Don't make the Storage button the parent of the window anchors.** Covering it with the cursor or a tooltip dropped the layout and started the whole-frame char inventory search (200 to 330 ms, six times in one recording).
- **Don't place the storage windows from the Storage button:** against the windows' sort buttons its position wanders by 1.7 UI units, ten times any other anchor, across every native still. Its template is a large soft highlight and matches at the worst score of the set.
- **Don't search the lone inventory over the whole frame on every scan:** 180 ms a variant at 1080p and 590 ms at 3440x1440, run for both variants on every scanned frame with no window found. Rationed, native searches fell from 478 to 99 on 17-28-55 with identical output, and 924 of 1310 drawn frames scanned in the browser became 1287.
- **Don't drop the near search for the second variant:** a variant covered while the other held was never found again.
- **Don't set the roam search's change threshold below a few percent or above the opening of a window:** the lone inventory opening is 13 to 15% of samples in one frame at 3440x1440 (no fade), while a cursor and a small menu make 1 to 3%.

## Page tabs

- **Don't take the only tab that matched neither state as the active one.** Hovering tab 1 with page 2 open brings up a label that hides tab 2, so 13 page-2 slots were written onto page 1 (22-55-45, frame 1862) and stayed, because a slot that later matches nothing keeps what it knew. It also read Gear Space as page 1 whenever the cursor was on tab 1.
- **Don't judge a tab by its active score alone:** a tab under the cursor scores 0.82 to 0.84 as active and 0.78 to 0.93 as inactive, which passes the 0.8 limit; only the lead over the other state (0.2 lit, 0.05 hovered) tells.

## Skipping still frames

- **Don't raise the change threshold above 1% of samples:** up to 1% loses nothing on the recordings, 2% loses one chest.
- **Don't expect skipping to help a busy cursor-heavy screen:** on the four recordings only 47 to 80% of frames are skipped natively and 17 to 50% in the browser.

## OCR

- **Don't read everything to tell repeats; and don't skip a line merely `close_enough` to a recent one.** Reading everything is 20 to 60% more lines for the same result. Skipping near-repeats of the last 32 or 128 halves the lines but chest rows a digit apart are over 0.99 alike, which lost a chest, and reads that only repeat the first take away what the votes live on: the recogniser reads the same line two ways now and then, and other reads outvote it.
- **Don't compare the brightness estimate in the repeat test:** it wobbles by 0.01 on a tenth of the scans and made 7 to 13% more lines read.
- **Don't tell a repeat by the made strip:** a number's strip is nearly black and white, so different crops collide. The crop is compared instead (4 to 6% more lines, same result).
- **Don't size the OCR workers by half the threads:** at that size (up to eight) a live share queued over 400 lines that took more than 10 s to clear (2026-10-10); one per four threads, at most three, with cheaper and fewer lines, replaced it.
- **Don't expect more threads or relaxed SIMD to fix a slow line in the browser.** Natively a line is 8.3 ms because `rten` runs each call on a thread per core; with one thread it is 24.2 ms, against 61.7 in a wasm worker. `+relaxed-simd` gave 65.5 ms.
- **Don't batch several lines per call to save per-call cost:** a batch's time follows the lines' width with next to no fixed cost. Batches of 1, 2, 4, 8 read a line only 1 to 12% faster but a hover's first read waits much longer (median ms at 1, 2, 4, 8 on 17-28-55: 109, 155, 250, 386), since a short queue goes to one worker while the others idle.
- **Don't judge the queue from a live capture file:** it has no queue length, and replays differ from what was recorded. Replayed with slow OCR the old code peaked at 772 lines and a hover's first write came 254 scans after it came up at the 90th percentile (fast-hovers capture).
- **Don't read titles for tooltips with a chest panel, or every line at once.** Spent px by line kind (816k px): titles 49%, chest rows 27%, description 13%, amounts 6%, slot numbers 5%. No title for chest panels, one clean read settling a title/chest/description, and holding back what is beside nothing lit up cut 42 to 56% (816k to 477k) with the same end state, and the 90th-percentile wait from 254 to 8 scans.
- **Don't drop reads of tooltips beside nothing lit up:** 2 to 15 of a recording's 30 to 130 useful tooltips are of that kind (up for one scan, or beside a slot first seen under the cursor, which matches nothing too). Dropping only where a slot visibly changed still loses 3 to 7.
- **Don't tell the hovered slot by its border's brightness:** an icon's own edge is as bright as the highlight.
- **Don't put slot numbers behind hovers' first reads:** no gain.
- **Don't expect transferring crops instead of copying to show in the timings:** the result's way back is 1.1 ms at 1080p and 5 ms at 3440x1440 whatever it carries.

## Slots

- **Don't compare every icon in full:** 13 ms a slot against 0.7 with the shortlist, and under the time budget four slots of a briefly shown page were left unread; it recognised nothing the shortlist did not.
- **Don't drop the decoys to save matching time.** Without them over seven lossless 1080p captures the lowest true match is 0.946 and the highest look-alike 0.966; with them the highest slot that should match nothing is 0.924. They cost twice the matching time per changed slot (5.1 to 10.3 ms a scanned frame natively). Five slots read wrongly (the T3 fusion as "Fusion", four chests as look-alikes) match nothing with them.
- **Don't use a fixed "fails whole, passes without the rows" rule for overlays:** a plain chest at 0.949 whole and 0.952 without was marked levelled (`use_12_177@epic`), so was a chest with a clock (`use_4_225@ancient`), and a levelled one that passed whole on a compressed recording was taken for plain. The gain rule's margin works because leaving the level's rows out moves a slot without one by -0.011 to +0.006 and one with it by +0.043 to +0.071; the clock's corner by -0.007 to +0.005 and +0.025 to +0.048 (12 lossless captures, 2,620 slots, three resolutions).
- **Don't leave irrelevant chests' levels out of `has_level`:** the Core Essence Chest (`use_4_224@epic`, 1700) was read as the levelled `use_4_225@epic` by 0.963 against 0.966, so its tooltip never found its slot.
- **Don't count irrelevant chests' levels below the lowest level of a chest that counts (1415):** the old Congratulation Chests (`use_4_220@epic`, 650/700) are the fusion material chest's art at a lower resolution and took its slot (`shop_icon_31@epic`, 1730, by 0.001).
- **Don't raise `OVERLAY_REACH` expecting more:** 0.15 gives the same output with matching a third slower; overlays already cost 10 to 17% more per changed slot.
- **Don't take only the best icon as the slot's:** one slot scored 0.967, 0.963 and 0.957 against three different chests, and the chest left without a slot.
- **Don't expect skipping unchanged slots on a compressed recording:** 42,000 crops were compared over 1,212 scanned frames of 22-55-45 (19 µs to cut and 16 to compare: 1.2 ms a scanned frame at 1080p).
- **Don't tighten the "same look" limit to fix wrong last reads:** each wrong read stayed for 53 to 840 looks; tightening to 0.99 fixed fewer than the vote for 14% more reads.
- **Don't make the number clean-up cheaper to speed the slot step:** cutting it from 6.2 to 1.0 ms a call left the scan as long as it was; the budget-bound step just worked through more slots in its budget.
- **Don't set the slot limit back to 0.95, or leave it at 0.9.** At 0.9 unknown icons were identified on a live share; at 0.95 far fewer slots with tooltip data were recognised (4 and 11 against 19 and 27 on two recordings). 0.935 with the decoys recognises 17 more slots over the suite and loses none.
- **Don't give alternatives the slot limit:** at 0.935 they brought in the other rarity of the same art on a dozen slots.
- **Don't skip the number read of a lit look** (one scoring 0.005 under the slot's best): it put the "9999" right on 17-28-55 and three other counts wrong ("85" for 851, "6.2" for 62, "80.9" for 809).
- **Don't read a number without a vote:** before the vote only 20 of 28 slots of a recording ended with the right number although static frames read perfectly.

## Fixed-position comparison

- **Don't take the best of nine one-pixel shifts:** it won by 0.002 on average and in no consistent direction (fitting noise), changed nothing read on any lossless input, and made slot matching about 3x slower (2.6 to 6 ms a scanned frame).
- **Don't use a structural similarity score:** a half-pixel grid error failed it, and it scored colour variants of one icon (Blue and Serca Blue) almost alike.
- **Don't return "no match" on a size mismatch:** `confidence` asserts instead; nothing in the suite trips it.

## The UI scale

- **Don't snap to whole pixels or add a per-height offset.** On native stills at UI heights 720 to 2160 the fractional part of a matched icon's position is spread flat, anchor distances are the same in UI units to within 0.3, and the slot pitch comes out 69.745 against the assumed 69.75. The only resolution-dependent offset came from rounding the template size.

## Brightness

- **Don't keep an estimate taken during a fade:** it made every later re-check fail, so nothing was ever read (the 2026-10-07 recording opens on the fade).
- **Don't average the storage windows' icon groups into the estimate:** their mean is 9 settings apart between 1080p and 1440p. The buttons' average is within 0.6 of the truth on 1080p captures at all 21 settings.
- **Don't pick the brightness at which a patch looks most like its template instead of the curves:** 4 settings off at the dark end and up to 3.5 at the bright end.
- **Don't fit the gamma with other ratios or a gain.** Ratio 60 (1.2 to 3.2 with 2.2 at 50) fits worse; a separate best exponent for each setting only gets to 0.14 levels against 0.15, so what is left is not the curve's shape. The older model (a power with ratio 66.5 and a two-constant gain) was 1.9 levels off on average and 12 at worst at setting 0.
- **Don't normalise a slot number's soft edge pixels like icon pixels:** they move 12 to 25% of what the law says, icon pixels of the same grey 90 to 97%. Pixels sharing a value at 50 spread 0.7 levels at settings 40 and 60, 4 at setting 0.
- **Don't downscale the capture:** the quantity digits need full resolution.

## Slot numbers

- **Don't line the number crop up on the number strip itself:** the number hides most of it. Dropping the line-up entirely changes one read in 369, so it stays on the icon area below.
- **Don't drop the shadow test:** 690 of 1,392 recording reads are wrong without it.
- **Don't drop the colour test:** a thin "1" over a pale blue pouch ((181, 181, 182) over (119, 163, 244)) was explained as half white and failed the shadow test on both sides; "100" read as "00" at brightness 61 and under.
- **Don't drop the stroke-joining rule:** a "6" over a pink pouch read "54" for 64 on all nine reads of a recording. Limits of 0.2 to 0.4 score alike; it costs a stray "8" before "809" at three more storage-sweep settings and lets more surroundings in at 720p.
- **Don't level the digits' brightness in the strip (whole, per blob or binary), and don't invert numbers:** no gain, binary is much worse, and inverting made the recogniser worse on numbers.
- **Don't crop the empty black left of the number by more than a few pixels:** 2 to 4 px at 1080p works, 6 to 8 px is much worse than no crop.
- The parameters were tuned on the stills and the brightness sweep (182 of 182 and 294 of 294 read); with all of the above, the sweeps read 3 wrong of 630 and 16 of 1,089.

## State between frames

- **Don't serialise the state out and back every scan:** 620 to 690 ms a scan (1920 with debug info on) at 1080p, because every image byte crossed as a JS number four times a scan; about 230 empty or unknown slots were also compared with every icon each scan (70 ms).
- **Don't run OCR inside the scan:** 295 to 360 ms a scan with 175 to 215 ms of it OCR; with OCR in its own workers and unmatched slots remembered, 16 to 18 ms. With one OCR worker a hover's first read came back in 5 s at the median and 14 at worst; with two, 0.3 and 5 s.
- **Don't transfer the stream whole to the worker:** it was changed for stream ownership, not because it leaked.
- **Don't log round-tripped state per frame:** Firefox leaked several GB per minute on screen share.
- **Don't expect `result()` to be where the time is:** 0.25 ms at 1080p and 0.39 ms at 3440x1440 with about 350 slots; serialising it is 0.56 and 1.21 ms in the browser.
- **Don't make the config's pixels msgpack numbers:** 16.7 million JS numbers posted to the scanner and each OCR worker took 4.7 s (Chromium; 4.1 s Firefox) from upload to first scan reply; as bytes 0.65 s and 0.77 s.

## Where the time goes

- **Don't treat the wasm call as the whole loop at wide captures:** at 3440x1440 the frame copy (11 ms) is as large as the scan (13 to 15 ms) and the result's way back another 6; the loop takes 30 to 33 ms a turn against 33 ms a frame at 30 fps.
- **Don't expect OCR to need more workers:** 11 to 21 lines a second are asked for against the 31 to 49 three workers can read; long waits are bursts (backlogs of 110 and 154 lines took 4 to 6 s to clear on 22-55-45).
- **Don't compare browser timing runs made an hour apart:** the same build gave 13.1 and 16.4 ms wasm calls.
- **Don't search the whole frame for the title bar every scan:** 4 and 10 ms on every scan at 1080p and 3440x1440, which was most of the tooltip step (`Tooltips tried.md`).

## Debug captures

- **Don't take the recorder's frame from the browser a second time:** two copies of every frame out of the browser fight each other. With the scanner working on the same video the browser copy went from 6 to 18 ms, copy and record 33 ms at the median and 55 at p95, and the video dropped to 27 fps. Forwarding the scanner's own copy: 15 ms and 24 at p95, 3.7 ms a frame to the scanner.
- **Don't use a plain RGB delta or deflate/zstd for frames:** RGB delta costs 22 ms against 18 with RGBA and "nothing for an unchanged frame" (24% bigger files); recording costs 3.5 to 4 ms for the delta and 3 to 5 for LZ4, deflate is 10% smaller at three times the time and the pure Rust zstd (`ruzstd`) no smaller at twenty times.
- **Don't call `init()` on every message in the recorder worker:** frames arriving during the first load started a second instance and the file lost the second frame, so every delta after it was off.
- **Don't replay at the recorded pace:** whether a short hover got all its lines read depended on the machine. It left a 20,000 Gold Chest's description unread in the e2e merge test, hiding the manifest; the click on `Export to calculator` waited out its 15 minutes.
- **Don't keep a per-scan trace or digest in the file** (`HFCAP004`): it only held on the recording build, natively not even there for certain, and the suite, e2e tests and `--check` all re-ran anyway. Inputs' timestamps and edits were never read.
- **Don't keep the slot time budget on a native re-run:** a suite run's captures changed with the machine's load (on `16-29` it cut some scans).
- **Don't simulate the browser's OCR throughput natively** (`Sim` with `OCR_PX`): removed with exact replay; a browser worker read about 5,500 px a second and live captures came out at 350 to 600 px a scan for all workers together.
- **Don't record a frame skipped as still with no pixels, or texts keyed by line id** (`HFCAP003`): a later build that would have scanned that frame could not, and ids shift once the scanner asks for other lines, so only the recording build could replay.

## Forced 21:9, resolution and native ultrawide

- **Don't offer forced 21:9 as a checkbox or the game resolution as a select, or guess the resolution from a still's name or by rounding the height:** the rounding (multiple of 360) scanned the 768p and 900p stills at UI heights 720 and 1080, which found nothing; at their own size they read 33 and 34 slots.
- **Don't detect the game area itself:** it would also handle a windowed game with a title bar, but only a full-screen share is supported.
- **Don't size the buffer from the track's `getSettings()`:** Chrome reported 3440x1920 on a 3440x1440 screen, the frames left the bottom 480 rows at zero, the scale and centring were off, nothing was found on a native 3440x1440 screen, and the share reserved twice.
- **Don't take the render area at 21:9 as 64:27 centred:** at 3440x1440 that puts the right edge 13 px in, and the lone inventory dragged to the right edge lay 15 px past its roam bound (slack 6).

## Changing character

- **Don't clear the scan on a character change only while a stream is open:** with none open the store was left alone, so a stopped scan stayed on the grid and in the manifest of the next character.
