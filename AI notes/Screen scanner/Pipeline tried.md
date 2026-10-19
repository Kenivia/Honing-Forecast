# Scanner pipeline: what was tried

History behind `Pipeline.md`: measurements, earlier designs, bugs and what was dropped. Only read this when trying something new on the scanner. Recordings are named by their time (`17-28-55` and `22-55-45` are 1080p of 2026-10-07; `21-42-32` and `21-31-53` are 3440x1440 of 2026-10-08); the rest are in `ignored/recordings`.

## Anchors

- **Whole-pixel templates.** Measured over the native stills, cutting templates to whole pixels is exact where correcting the position afterwards was only first-order: the per-window offset at 1080p went from 0.17 px to 0.03, and over all heights the spread fell from 0.110 to 0.049 px, 30 more icons were recognised and the mean icon distance dropped from 8.39 to 7.48. Anchor match scores are a wash, so the gain is in placement, not detection.
- A search area cut at a fraction of a pixel was resampled, which moved an anchor's brightness estimate by up to two settings.
- **Re-checking without an estimate**: before, nothing was re-checked until there was a brightness estimate.
- **Older second variants** were "Move All Duplicate Materials" and the inventory's bottom-left button. The first is text, which the game draws anew at each resolution (it only just passed at 1080p); the second is greyed out in the storage view and needed a second template.
- **The Storage button used to be the parent of the window anchors.** Covering the button with the cursor or a tooltip then dropped the layout and started the whole-frame char inventory search (200 to 330 ms, about six times in one recording). With the layout as a special case neither recording has a single whole-frame search.
- **Why the storage windows are not placed from the Storage button**: measured against the windows' sort buttons across every native still, its position wanders by 1.7 UI units, ten times any other anchor. Its template is a large soft highlight and matches at the worst score of the set, so this is probably its own localisation rather than the game moving it.
- **The lone inventory's range** was measured 2026-10-17 on the 1440p stills in `scanner_setup/brightness/1440p raw/char inv positions`, 16:9 and forced 21:9, dragged to each corner and in its default place.
- **Its default place** costs 0.1 to 0.2 ms a scan. The roam area is a third of a 16:9 frame, and 27% of the capture at forced 21:9 on a 16:9 screen; on the stills one variant over it took 27 ms (forced 21:9) and 39 to 45 ms (16:9) natively.
- **The roam search before it was rationed**: over the whole frame it took 180 ms at 1080p and 590 ms at 3440x1440 per variant, and ran for both on every scanned frame with no window found. The one lone inventory opening on the recordings (21-31-53, frame 170, no fade) is 13% of the samples in a single frame and 15% in all at 3440x1440; before it the cursor and a small menu make 1 to 3%. With the rationing, natively the searches went from 478 to 99 on 17-28-55 (the storage layout closed for half of it) and from 186 to 22 on 21-31-53, with the same output. In the browser on 17-28-55 a search runs on 7% of scanned frames against 24%, the mean wasm call is 27 ms against 66, and 1287 of 1306 drawn frames are scanned against 924 of 1310. The near search for the second variant also fixed a variant that got covered while the other held never being found again.

## Page tabs

- A lit tab leads by 0.2 (0.98 against 0.77), also with the cursor on it (0.96 against 0.74). A tab under the cursor scores 0.82 to 0.84 as active, 0.78 to 0.93 as inactive. At the tabs' 0.8 limit that used to pass as active.
- **The older rule took the only tab that matched neither state as the active one.** Hovering tab 1 with page 2 open brings up a label that hides tab 2, so the hovered tab was that one tab: 13 page-2 slots were written onto page 1 (2026-10-07 22-55-45 recording, frame 1862), and they stayed, because a slot that later matches nothing keeps what it knew. It also read Gear Space as page 1 whenever the cursor was on tab 1.
- The stills at 720p, 1080p, 1440p and 2160p recognise the same slots on the same pages as before the change.

## Skipping still frames

- A skipped frame costs 0.8 to 1.0 ms at 1080p and 1.7 to 2.0 at 3440x1440.
- On the four recordings, where the cursor hardly rests, 47 to 80% of frames are skipped natively and 17 to 50% in the browser (there a hover waits longer for its texts). A limit of up to 1% loses nothing on the recordings; 2% loses one chest.

## OCR

- **What moving strip-making to the OCR workers took off the scanner**: 0.4 ms per scanned frame of tooltip strips and 0.4 to 0.5 ms of slot numbers (browser, mean, the same at 1080p and 3440x1440), the second of them in bursts of up to 40 ms on a page's first look, where it was a quarter of the slot work. The mean wasm call went from 20.0 to 19.7 ms and from 13.7 to 13.1; a look at the slots takes 55 and 49 ms at the 99th percentile where it took 60 and 62.
- **What it cost**: 4 to 6% more lines are read (natively 700 against 675, 2,677 against 2,567, 2,637 against 2,516, 3,378 against 3,173 on the four recordings), with the same result. A repeat used to be told by the strip, and now by the crop, which differs where the strip does not: a number's strip is nearly black and white. The brightness estimate is re-averaged from the anchors' patches on every scan that has as many anchors as the most so far, and wobbles by 0.01 (49.02, 49.03) on a tenth of the scans; with it in the comparison, 7 to 13% more lines were read.
- **Two other ways of telling repeats were tried and dropped**: reading everything (20 to 60% more lines, same result), and skipping a line that is `close_enough` to one of the last 32 or 128 sent (half the lines, but chest rows a digit apart are over 0.99 alike, which lost a chest, and reads that only repeat the first one take away what the votes live on: the recogniser reads the same line two ways now and then, and it is the other reads that outvote it).
- Crops transferred, not copied: that does not show in the timings; the result's way back is 1.1 ms at 1080p and 5 ms at 3440x1440 whatever it carries.
- **Workers.** One per four hardware threads, at most three (3 on 16 threads, 2 on 8, 1 on 4). For a day it was half the threads less one, at most eight, after a live share queued over 400 lines that took more than 10 s to clear (2026-10-10); that was put back on 2026-10-11, to make a line cheaper and send fewer of them first.
- **Why a line is slower in the browser** (2026-10-11, 17-28-55). Natively a line is 8.3 ms because `rten` runs each model call on a thread per physical core; with `RTEN_NUM_THREADS=1` it is 24.2 ms, against 61.7 in a wasm worker (one thread, 128-bit SIMD against AVX). Building with `+relaxed-simd` gave 65.5 ms, no gain, so it is not used. A batch's time follows the lines' width with next to no fixed cost per line, so reading several lines in one call would buy little; `ocrs` pads a strip to the next 50 px.
- **What a live share asks for** (2026-10-11). A capture has no queue length. `capture_replay --ocr` prints every line sent and back, but the lines sent are the replay's own, and all five captures replay differently natively than they were recorded (the first scan that differs is 0 to 53), so only the texts back can be trusted: by their order, at least 223 lines were out at once on the fast-hovers capture and over 50 on 69% of its scans. Replayed with slow OCR the captures behave like the page did: at 600 px a scan the old code peaks at 772 lines on that capture and a hover that ends up writing something first does so 254 scans after it came up at the 90th percentile.
- **Where the work went.** By strip width, with every line read at once: titles 49%, chest rows 27%, description lines 13%, amounts 6%, slot numbers 5% (fast-hovers capture, 816k px). Half of it was for tooltips that end up writing nothing: gems, gear, chests that are in no table. Those are mostly beside slots that match no icon.
- **What was changed, and what each bought** (px read with every line read at once, the nine inputs of `ignored/recordings`, same end state as before on all of them): no title for a tooltip with a chest panel, 17 to 28% less; one clean read settling a title, a chest or a description, and a held last frame waiting for the read that is out (`Tooltips.md`), together with holding back what is beside nothing lit up, 42 to 56% less in all (816k to 477k on the fast-hovers capture, 1,098k to 483k on 21-42-32).
- **What it does to the wait.** Scans from a tooltip coming up to its first write, 90th percentile over the tooltips that write something, old code against new, fed at 15 scans a second:

  | Input | 600 px a scan | 300 px a scan |
  | --- | --- | --- |
  | fast hovers capture | 254 to 8 | 909 to 76 |
  | 18-47-43 capture | 177 to 6 | 701 to 20 |
  | 19-22-07 capture | 41 to 6 | 180 to 126 |
  | 19-56-53 capture | 192 to 7 | 628 to 37 |
  | 17-28-55 | 12 to 7 | 80 to 18 |
  | 22-55-45 | 26 to 14 | 353 to 205 |
  | 21-31-53 | 12 to 12 | 140 to 33 |
  | 21-42-32 | 28 to 11 | 386 to 48 |
  | capture of 21-31-53 | 35 to 13 | 435 to 316 |

  The end state under slow OCR is the same as with every line read at once on six of the nine inputs at 600 px: one chest of 21-31-53 and of its capture is not read (the old code lost two and three there), and one chest of 21-42-32 is put with another slot. At 300 px 22-55-45 also differs in which slot four chests are put with. No slot's status differs in any of these. What is beside nothing lit up still gets read, last, so the queue is as long as it was while the user skims and empties when they stop.
- **In the browser** (headless Chromium, 17-28-55, one run each, an hour apart, so only roughly comparable): 604 lines read where it was 809, 45 ms a line where it was 62 (fewer long title lines), and a hover's first read back in 0.11 s at the median, 0.24 at the 90th percentile and 0.35 at worst, where it was 0.23, 0.41 and 0.73. What the page ended up showing was not compared there, only natively.
- **Tried and left out.** Relaxed SIMD (above). Slot numbers behind hovers' first reads (no gain). Not reading at all what is beside nothing lit up: 2 to 15 of a recording's 30 to 130 useful tooltips are of that kind, mostly ones up for a single scan and ones whose own slot was first looked at with the cursor on it, so it would lose them. Dropping only those where a slot that matches nothing visibly changed, seen for two scans or more: still 3 to 7 useful ones on three of the recordings, since a slot first seen under the cursor matches nothing too; asking that the slot had been looked at with no tooltip up left almost nothing to drop on the live captures. Telling the hovered slot by the brightness of its border: an icon's own edge is as bright as the highlight.
- **Batch size**, measured 2026-10-11 at 1, 2, 4 and 8 (headless Chromium, three recordings, two rounds back to back, game closed): a bigger batch reads a line a little faster (4 against 2: 1 to 6% less ms a line; 8 against 1: 4 to 12%) and keeps a hover's first read waiting much longer (median ms at 1, 2, 4, 8: 109, 155, 250, 386 on 17-28-55; 167, 201, 259, 451 on 22-55-45; 183, 223, 333, 490 on 21-42-32), since the lines of a short queue then go to one worker while the others idle. The longest waits, which are backlogs, do not follow the batch. Four was the old value.
- **`tooltip_test` used to differ between runs** because of the slot step's time budget and hash map order; both are fixed.

## Slots

- **Shortlist.** There are 22 material icons and 294 chest ones. Comparing all of them takes 13 ms a slot against 0.7 with the shortlist; under the time budget that left four slots of a briefly shown page unread on a recording, and on the two recordings it was tried on it recognised nothing the shortlist did not.
- **Decoys** (2026-10-11). 757 templates. Without them, over seven lossless 1080p captures the lowest true match is 0.946 and the highest look-alike 0.966; with them the highest slot that should match nothing is 0.924 (`Icon matching bench tried.md`). On the four annotated captures the five slots wrongly read as an icon (the T3 fusion as "Fusion", four chests as look-alikes) match nothing, and no other slot's icon or status changes. Over the suite 63 slot lines lose their icon and none gains or changes one: all are one of the look-alikes, 58 of them unhovered chests that needed a hover. The other five are the T3 fusion on four inputs and a legendary chest of `shop_icon_784` art read as the relic "Special Honing Materials Chest" (21-42-32, checked by its background). On two recordings a slot also loses one of its alternatives, which scored under a decoy. Cost: matching one changed slot takes twice as long (natively 5.1 to 10.3 ms a scanned frame of `slots/all_icons` on 17-28-55, with the suite's inputs running at once). In the browser on that recording the slot step is 3.2 ms a scanned frame where it was 2.4, and its 99th percentile is at the 60 ms budget. Not measured back to back.
- **Overlays**, measured on the clean looks of 12 lossless captures (2,620 slots; nine at 1080p, and one inventory at 1080p, 1440p and 2160p in `ignored/recordings/new captures`). Leaving the level's rows out moves a slot without one by -0.011 to +0.006 and one with it by +0.043 to +0.071. The clock's corner: -0.007 to +0.005 without, +0.025 to +0.048 with (two stacks at three resolutions). Every slot comes out as it should, and no slot that holds nothing of ours passes for the corner being left out. The areas hold at all three resolutions.
- **Why a gain.** It used to be "fails whole at 0.95, passes without the rows", which got it wrong three ways: a plain chest at 0.949 whole and 0.952 without was marked levelled (`use_12_177@epic` on 17-28-55, 22-55-45 and the 15-53 capture), so was a chest with a clock, the rows hiding half of it (`use_4_225@ancient` on 21-31-53), and at 0.935 a levelled one that passed whole on a compressed recording was taken for plain, which is why those icons were kept at 0.95 until 2026-10-12. They pass at 0.935 like the rest now.
- **What the overlays changed** (2026-10-12). "[Event] Artisan's Metallurgy: Level 4" with a clock is recognised at all three resolutions (0.92 to 0.93 whole, 0.97 to 0.98 without the corner). Over the suite 19 slot lines gain an icon and none loses or changes one but for the chests wrongly levelled above, which now show every chest their icon can be: levelled pouches on the brightness stills at 0.94 are the gain, and on the captures a chest of 1430 at 0.946.
- `OVERLAY_REACH` at 0.15 gives the same output, with matching a third slower than before the overlays. The overlays cost 10 to 17% more time to match a changed slot natively (2.0 to 2.3 ms on 17-28-55, with the suite's inputs running at once). Not measured in the browser.
- **Several icons**: one slot gave 0.967, 0.963 and 0.957 against three different chests.
- **On a compressed recording most slots are compared, not skipped**: 42,000 crops over the 1,212 scanned frames of 22-55-45. Natively one takes 19 µs to cut (a crop goes through the resizer, its place being fractional) and 16 to compare, which is 1.2 ms a scanned frame at 1080p and 1.6 at 3440x1440. Not looked into.
- **Small repeats taken out (2026-10-10)**: the brightness table is kept between crops, a changed slot is cut once instead of twice, and the slots beside a tooltip are worked out once a frame. Natively these were 0.34, 0.16 to 0.22 and 0.03 to 0.04 ms a scanned frame. In the browser, back to back on 17-28-55 and 21-42-32, the slot step went from 2.86 to 2.37 ms a scanned frame and from 3.39 to 3.01, pages from 0.62 to 0.48 and from 1.38 to 1.19, the tooltip step from 2.17 to 2.11 and from 4.88 to 4.73, and the mean wasm call at 3440x1440 from 13.7 to 12.7. Same output. The brightness table used to be made for every crop, 82,000 times over a recording at 5 µs each.
- **Left as they are**, both under 0.05 ms a scanned frame natively: `icon_lookup` makes a `String` key on every call (0.3 µs a call with its lock and hash, 140 a scanned frame), and a hover that ends with reads still out is written twice in `resolve_reads` when the last comes back (once a hover, 40 µs; never natively, where nothing is ever out).
- **The vote of five.** Wrong last reads are not brief: on the recordings each stayed up for 53 to 840 looks without a re-read, because a recognised slot counts as unchanged while its icon area is within 0.95 of the last crop and the number strip is not compared at all. Tightening that to 0.99 fixed fewer than the vote for 14% more reads; not done.
- **Number clean-up** is about 9 ms a call. Cutting it from 6.2 to 1.0 ms a call left the scan as long as it was: the budget-bound step just worked through more slots in its 60 ms.
- **The slot limit.** It was raised from 0.9 to 0.95 because unknown icons were being identified on a live screen share; on the two recordings that left far fewer slots recognised (4 and 11 slots with tooltip data, against 19 and 27 at 0.9). It went to 0.935 on 2026-10-11 with the decoys. On the annotated captures the Emergency Honing Materials Chest, "[Event] 1,000 Gold Bars" and the honing material selection chest (0.946 to 0.947) are read and no other slot's icon, status or value differs. Over the suite's recordings and captures 17 more slots are recognised at the end and none is lost or changed: the same chests, and on the compressed 1080p recordings Serca Blue and Serca Special Leapstone at 0.941 to 0.950, whose tooltips are now tied to them. At 0.935 the alternatives brought in the other rarity of the same art on a dozen slots of the captures, hence 0.95 for them. A lit look passes more often: the score shown is lower on some slots (0.941 for 0.971), one "9999" ends as "99" on 17-28-55, and with the level icons at 0.935 too, two chest counts there end as 180 for 80 and 128 for 28, on slots that need their tooltip anyway.
- **Tried and left out: not reading the number off a lit look** (one that scores 0.005 under the slot's best). On 17-28-55 it put the "9999" right and three other counts wrong ("85" for 851, "6.2" for 62, "80.9" for 809): on a compressed recording the best look is not the one that reads right. No end value moved anywhere else in the suite.
- On the recording 20 of 28 slots ended with the right number although static frames read perfectly, before the vote.

## Fixed-position comparison

- It used to take the best of the nine one-pixel shifts. On 2026-10-17 dropping them changed nothing read on any lossless input of `pnpm scanner-suite`; only the compressed 1440p recording (21-31-53) and its capture moved, by 0.01 to 0.02 on seven slots, with the same items. A shift won by 0.002 on average and in no consistent direction, so it was fitting noise, and slot matching (`slots/all_icons`) got about 3x faster (2.6 to 6 ms a scanned frame).
- Structural similarity was dropped: a half-pixel grid error was enough to fail it, and it scored colour variants of one icon (Blue and Serca Blue) almost alike.
- `confidence` used to return no match on a size mismatch; it asserts now, and nothing in the suite trips it.

## The UI scale

Measured on native stills at UI heights 720, 768, 810, 900, 1080, 1440, 1620 and 2160 (`icon_locate`, `anchor_locate`): the fractional part of a matched icon's position is spread flat, not clustered; the distance between any two anchors is the same in UI units at every height to within 0.3; and the slot pitch comes out 69.745 against the assumed 69.75, i.e. 0.05 UI over nine steps. The one resolution-dependent offset there was came from the template size rounding.

## Brightness

- **An estimate kept from a fade** made every later re-check fail, so nothing was ever read (the 2026-10-07 recording, which opens on the fade). With the two rules it follows the fade and is at 61.1 on the first full frame.
- **Which anchors.** The icon groups' mean comes out 9 settings apart between 1080p and 1440p. On the 1080p captures at all 21 settings the buttons' average is within 0.6 of the truth, and each button within 1.5. On the 1440p stills the buttons read 69 to 73 (the setting they were taken at is not recorded, about 70). The recordings at 60 read 59.3 and 59.5, the one at 100 reads 97.8, the one at 0 reads 0.3.
- **Picking the brightness at which a patch looks most like its template was tried instead of the curves** and is worse: 4 settings off at the dark end and up to 3.5 at the bright end, because it inherits the model's error there.
- **The gamma law** was measured on `scanner_setup/brightness/inputs storage` (the storage layout with a tooltip at all 21 settings). 1.2 to 3.2 with 2.2 at 50 would be a ratio of 60, which fits worse. On opaque icon pixels, per value 0 to 255: 0.15 levels off on average over all settings, 1.6 at worst; 0.3 at setting 0 and 0.24 at 100. A separate best exponent for each setting only gets to 0.14, so what is left is not the curve's shape. The older model (a power with ratio 66.5 and a two-constant gain) was 1.9 on average and 12 at worst, at setting 0.
- **Blending.** Pixels that share a value at 50 spread 0.7 levels at settings 40 and 60, 4 at setting 0. A slot number's soft edge pixels move 12 to 25% of what the law says while icon pixels of the same grey move 90 to 97%.
- **Effect of the law on reading** (against the older model, same build otherwise). Captures at every setting: slots recognised go from 16, 19 and 23 to 20, 23 and 24 at settings 0, 5 and 10, the same elsewhere. Recordings at 60: the same chests. The recording at 100 first read 19 chests against 24, until two tooltip colour limits that had been set by eye on the older model's values were redone (`Tooltips.md`); it now reads 26.
- **The anchor and page-tab templates were cut again for the law** (`Config and calibration.md`). They changed by 0.1 to 2 levels on average, and nothing that was read before reads differently.
- **Downscaling the capture** was tried and removed because the quantity digits need full resolution.

## Slot numbers

- The line-up: with anchor templates cut to whole pixels the grid is good to about a twentieth of a pixel, every slot across the native stills wants no more than a quarter (76% want nothing), and reads match the old two-pass search over four times the span. Dropping the line-up entirely changes one read in 369, so it stays. Lining it up on the number strip itself fails because the number hides most of it.
- **The colour test** was added for a thin "1" over the pale blue of a pouch: an edge pixel there is (181, 181, 182) over (119, 163, 244), which the first test explained as half white over the pouch, just inside its tolerance of 24, and the shadow test failed on both sides (a grey edge pixel on one, a dark neighbour of the template on the other). "100" read as "00" at brightness 61 and under and as "100" from 62. Over every slot of the stills and both brightness sweeps (2,232 reads) it changed nine: seven dropped leading "1"s now read, and two 720p slots went from one wrong read to another.
- **The stroke-joining rule**: a "6" over a pink pouch had four stroke pixels at 7 and 8 sixteenths with no shadow on the pouch's side, and read "54" for 64 on all nine reads of a recording. Limits of 0.2 to 0.4 score alike. It costs a stray "8" before "809" at three more brightness settings of the storage sweep, and at 720p it lets more of the surroundings in.
- **Without the shadow test** 690 of 1,392 recording reads are wrong.
- **Levelling the digits' brightness in the strip** was tried (whole strip, per blob, binary): no gain, and binary is much worse. Inverting made the recogniser worse on numbers.
- **Measured 2026-10-09** with all of the above in: the brightness sweeps 3 wrong of 630 and 16 of 1,089; the four recordings 7 wrong reads of 1,392 and 1 slot of 200 ending wrong, against 26 and 6 before. In the scanner itself, slots whose number agrees with their tooltip amount went from 23 of 24 to 25 of 25 and from 57 of 65 to 65 of 65 on the two 1080p recordings, and stayed 46 of 46 on 21-42-32.
- With the old bins on recognised icons the current version read 182 of 182 on the stills and 294 of 294 on the brightness sweep; the parameters were tuned on those two sets. On three static frames of the lossy recording it read 34 of 34 roster numbers.
- `slot_scores` with `DUMP` is how the ancient background and the look-alike chests were found.

## State between frames

- **What a result costs.** `result()` natively 0.25 ms at 1080p and 0.39 ms at 3440x1440 with about 350 slots known. The rest of `to_value` (0.56 and 1.21 ms in the browser) is serialising it. It was 0.30 and 0.52 ms (0.61 and 1.35 in the browser) while a chest slot's variants were collected twice, `open_alike` cloned and sorted both chests' contents on every call and the slots of a chest's column were counted before the cheaper tests. What is left is spread over the slots with nothing standing out: the icon lookups of `slot_variants`, the chests of a slot's column, and putting each slot together.
- **The stream used to be transferred whole to the worker**; it was changed for stream ownership, not because it leaked.
- **Logging round-tripped state** per frame in Firefox leaked several GB per minute on screen share.

## Where the time goes

Measured on 2026-10-10 after the scanner clean-up, all four recordings in one batch with the game closed: 17-28-55 and 22-55-45 (1080p, 30 fps files, so 15 frames a second at half speed) and 21-42-32 and 21-31-53 (3440x1440 at 60 fps, so 30 a second). Headless Chromium, 16 threads, three OCR workers, 338 slot templates. Mean ms per scanned frame:

| | 1080p (17-28-55 / 22-55-45) | 3440x1440 (21-42-32 / 21-31-53) |
| --- | --- | --- |
| Whole wasm call | 7.9 without the anchor searches / 9.0 (median 5.8 / 6.5, 99th 195 with the searches / 40) | 13.0 / 14.6 (median 10.5 / 12.0, 99th 61 / 63) |
| No tooltip / item tooltip / chest tooltip | 30 with the searches, 9.9 / 9.3, 8.7 / 7.7, 8.9 | 19.9, 23.6 / 12.0, 13.3 / 13.1, 14.0 |
| Tooltip step | 2.1 / 3.0 | 4.9 / 5.1 |
| of it: finding the title bar | 0.9 / 1.0 (median 0.1) | 1.0 / 1.3 (median 0.2) |
| of it: layout, title, icon, chest (median, each when it runs) | 1.4, 0.1, 2.6, 0.1 | 3.0, 0.2, 3.1, 0.2 |
| Slots, over every scanned frame | 2.4 / 3.1 | 3.1 / 3.3 |
| Anchors, without the searches | 2.0 / 1.1 | 1.8 / 2.4 |
| Pages | 0.5 / 0.7 | 1.2 |
| Change check | 0.3 | 0.9 / 1.0 |
| Serialising the result | 0.5 / 0.7 | 1.1 / 0.9 |
| A skipped frame's wasm call | 0.8 / 1.0 | 2.0 / 1.7 |
| Frame copy | 3.2 / 3.3 | 11.0 / 11.7 |
| Result back to the main thread | 1.0 / 1.3 | 5.6 / 6.2 |
| Whole loop, without waiting for a frame | 24 (the searches) / 14 | 30 / 33 |
| Frames taken, of those the upload drew | 99% / all | 97 / 94% |
| Of those, skipped as still | 37 / 50% | 17 / 33% |
| OCR, ms a line | 61 / 78 | 82 / 97 |
| A hover's first read back: median, 90th, worst | 0.24, 0.46, 0.71 s / 0.31, 1.28, 3.3 s | 0.33, 0.69, 1.2 s / 0.35, 1.06, 2.7 s |

- **Against 2026-10-09, before the clean-up, the wasm call is half**: 16.8 and 17.3 ms at 1080p, 30.8 and 28.5 at 3440x1440 then. Most of it is the tooltip step (8.0 and 10.1 ms then, 20.4 and 17.7 at 3440x1440): the title bar is looked for near the windows and at the hover's last place instead of over the whole frame (4 and 10 ms on every scan then), a hover's lines are only cut, not made into strips, and the icon is matched six times a hover at most. The slot step lost its number clean-up to the OCR workers (the step was 3.4 to 4.9 ms then). What each step bought is with its description here and in `Tooltips tried.md`; those figures are each a before and after measured back to back, so they do not add up to this table.
- **At 3440x1440 the frame copy is as large as the scan**, 11 ms against 13 to 15, and the result's way back another 6. The loop takes 30 to 33 ms a turn against 43 to 47, so it keeps up with 30 frames a second most of the time (97 and 94% of drawn frames taken, 70 and 84% before).
- **A frame with no tooltip is the slow kind at 3440x1440** (20 to 24 ms against 12 to 14): probably the frames where a page has just opened and its slots are matched; not checked.
- **The slow tail is the slot step, under its 60 ms budget at the 99th percentile**: 30 to 34 ms at 1080p and 45 to 46 at 3440x1440 on the looks where it runs, icon matching being nearly all of it.
- **The whole-frame search ran on 6% of 17-28-55's scans** (51 of 802, 186 ms a variant) and on 6 scans of 21-31-53 (575 ms). Measured before the search was cut to the roam area.
- **OCR is not short of workers.** 11 to 21 lines a second are asked for against the 31 to 49 three workers can read, and the queue is empty after 51 to 66% of batches. The long waits are bursts: 22-55-45 has backlogs of 110 and 154 lines that take 4 to 6 s to clear, 21-31-53 of 78 and 98.
- OCR preprocessing natively is 0.7 to 0.8 ms a line, of 9 to 11 in all.
- Firefox was last measured on 2026-10-06: 26 ms a scan (frame copy 7 ms), 96% of drawn frames scanned.

How it got here, each measured on the 1080p recordings of its day (so not comparable line by line):

- **The whole state serialised out and back on every scan**: 620 to 690 ms a scan with debug info off and 1920 on, of which moving the state was 185 to 215 and 1085 ms. It held only 1 to 4 MB of image bytes, but each byte crossed as a JS number, four times a scan. About 230 empty or unknown slots were compared with every icon on every scan (70 ms).
- **The state kept in Rust, OCR still inside the scan**: 295 to 360 ms a scan, 175 to 215 ms of it OCR (30 ms for a slot number, 60 to 100 for a tooltip line; a chest tooltip is 8 or 9 lines).
- **OCR in its own workers, unmatched slots remembered** (2026-10-06, 22 templates): 16 to 18 ms. With one OCR worker a hover's first read came back in 5 s at the median and 14 at worst; with two, 0.3 and 5 s.
- **The chest table in, 338 templates** (2026-10-09): 16.8 to 17.3 ms at 1080p, 28.5 to 30.8 at 3440x1440. The shortlist kept the slot step cheaper than it had been with 22.

## Debug captures

- **Measured (2026-10-10)** on 17-28-55 played into the page: 122 MB for 30 s in headless Chromium, 493 scans of which 75 skipped; a run with a retry and a hand edit, and one in Firefox, all replay with every digest matching, on the page and natively. Replay takes 20 s on the page and 14 natively for those 30 s. On a live 1080p share the author measured about 180 MB a minute, up to about 250 in a busy scene, and 10 to 15 scans a second while recording.
- **What recording costs a scan** (browser, 1080p): 3.5 to 4 ms for the delta and 3 to 5 for LZ4, timed as the `capture` stage. Deflate was 10% smaller at three times the time and the pure Rust zstd encoder (`ruzstd`) no smaller at twenty times.
- Native and wasm replays have not differed on the captures so far.

## Forced 21:9 and brightness settings

- **Until 2026-10-19 forced 21:9 was a checkbox only** (preselected and locked on for an ultrawide capture), and brightness was always estimated. The native bins took forced 21:9 from a still's name ("21 by 9") and `capture_replay --forced-21-9` overrode a capture's start record; both went with auto-detection, which matched the name on every still (`tooltip_test` output byte for byte the same over all still folders).
- **Detecting the game area itself** was considered, which would also handle a windowed game with a title bar, and dropped in favour of supporting only a full-screen share.
- **The game resolution was a select until 2026-10-19**, preselected from the capture size, for a windowed game inside a larger capture; the native bins guessed it with `guess_game` (height rounded to a multiple of 360). Dropped with windowed support: the capture size is the game's. The rounding had scanned the 768p and 900p stills at UI heights 720 and 1080, which found nothing; at their own size they read 33 and 34 slots. Three stills a pixel or two short (cropping mistakes) were padded at the right and bottom, which gave the same output as before byte for byte.

## Changing character

- **Until 2026-10-19 any character change cleared the scan**: `CharView` terminated the worker, and the capture component restarted the scanner if a stream was open. With no stream open the store was left alone, so a stopped scan stayed on the grid and in the manifest of the next character. Now a router guard wipes or moves it (`Pipeline.md`).
