# Scanner pipeline

The scanner reads the player's material counts from a shared game screen, so they do not have to type them in. It is in active development on the `Inventory-scanner` branch. `public/WIP.md` has the author's current task list.

Code: `crates/scanner` (image processing), `frontend/Components/Character/InventoryScanner` (capture and the slot grid), plus scanner operations in the shared wasm worker. What the page shows is in `Slot grid.md`.

## Stages

1. **Capture** (TypeScript, main thread). A video track is read as a stream of frames at 30 a second, and every frame is scanned. The track comes from the browser's screen-share API, or from an uploaded image or video file drawn onto a canvas (a fake stream: an image repeats, a video is drawn frame by frame as it plays and then holds its last frame). Both go through the same frame reader, which keeps up to 8 frames while a scan runs long.
2. **Frame transfer**. For each operation the main thread reads one frame and transfers it to the worker along with the operation. The worker copies its pixels directly into a buffer inside wasm memory, closes the frame, then calls Rust. Nothing else goes in but two flags, the OCR texts that have come back and the slots the user edited by hand.
3. **Is it worth a scan** (Rust). A frame that hardly differs from the last scanned one is dropped here, before anything else. See "Skipping still frames" below.
4. **Locate the windows** (Rust). Each anchor is template-matched inside its bound, and a hit gives a root, the origin of any inventory window it locates, and a brightness estimate. On later frames anchors are re-checked at their known position. See "Anchors and layouts" below.
5. **Detect the page** (Rust). Page-tab templates at fixed offsets from each window origin say which page of that inventory is showing: the one tab that is lit. See "Page tabs" below.
6. **Read slots** (Rust), on every third scan. Each slot of the active page sits at a fixed offset. A slot whose pixels have not changed is skipped; otherwise its crop is compared against every icon template, the closest one that passes wins, and the number strip is sent off to be read. See "Slots" below.
7. **Find the tooltip** (Rust). If an item tooltip is on screen, its text lines are cut out and sent off to be read. When the texts are back, its title, stacked amount and tradability are written onto the slot that was hovered, and a chest's kind and contents go into a separate list of chests. See `Tooltips.md`.
8. **Result and loop**. Rust returns a small result, not its state (see "State between frames"). On each result the main thread first reads and posts the next frame, then passes the result's text lines to the OCR workers and the rest to the page.

Full-frame search is the expensive step and uses FFT cross-correlation. Everything after the anchor is a cheap fixed-position comparison.

## Anchors and layouts

An anchor is a piece of fixed UI that is template-matched to get a *root*: a position other things are placed from. The ordered anchor list in the scanner constants is the single place that describes them. Each entry has:

- **Variants**: alternative templates for the same anchor, so that one can be covered (cursor, tooltip) while another still holds it. Each has a bound and brightness coefficients. The root comes from the first variant that is found, because variants place it a pixel apart.
- **Bound**: where a variant is searched. One of: the whole frame; a rectangle in UI space; a rectangle in UI space that moves with the storage layout.
- **Inventories**: which inventory windows the anchor locates and each window's origin relative to its root.

A match always *locates*: the root is the matched position minus the template's stored offset, never the bound's own position. So a bound only needs to be roughly right, and its slack does not affect alignment.

The search area is put on whole pixels. Cut at a fraction of a pixel it is resampled, which moved an anchor's brightness estimate by up to two settings.

**An anchor template is cut to a whole number of screen pixels.** A template scaled naively comes out `round(w x scale)` pixels wide while the thing it matches is `w x scale` wide, so it is stretched by up to half a pixel: the match then settles about half that error off, and every window sits a fraction out at any UI height 1440 does not divide, the whole grid with it. So `whole_pixel_crop` takes the largest centred part of the template whose scaled size is whole (`floor(w x scale) / scale` UI units) and the offset moves with the cut, which makes the template pixel for pixel against the screen. Measured over the native stills this is exact where correcting the position afterwards was only first-order: the per-window offset at 1080p went from 0.17 px to 0.03, and over all heights the spread fell from 0.110 to 0.049 px, 30 more icons were recognised and the mean icon distance dropped from 8.39 to 7.48. Anchor match scores are a wash, so the gain is in placement, not detection. It is a no-op at UI heights 720, 1440 and 2160, where every template size already scales whole. Slot icons are left alone: there the template and the crop are both `round(61 x scale)` pixels covering the same 61 UI units, so they are stretched alike and nothing is misplaced. See "The game scales its UI continuously" below.

Slot grids and page tabs are defined once per inventory, relative to that inventory's window origin, so the same inventory can be located by several anchors. An inventory's origin comes from the first found anchor, in list order, that locates it.

Current anchors:

- **Storage**: the highlighted Storage button, in a pet-menu and a storage-NPC variant that never show together, each at its own fixed UI-space spot. It locates no inventory.
- **One anchor per storage window**, with two variants: the window's sort button at its top left (the char inventory anchor template, identical in all three windows) and something at its top right. For the two storages that is their group of move-to icons. For the inventory it is the search button; the icons next to it are filters and the chosen one lights up. Their roots are the window origins.
- **Char inventory**: the free-floating window, searched across the whole frame, by the same sort button and search button.

Older second variants were "Move All Duplicate Materials" and the inventory's bottom-left button. The first is text, which the game draws anew at each resolution (it only just passed at 1080p); the second is greyed out in the storage view and needed a second template.

**The storage layout is a special case, written as one** (`update_anchors`). Roster storage, character storage and character inventory always sit side by side in the same place: one place from the pet menu, and shifted up and slightly left from the storage NPC. So its seven anchors (the button and two per window) are each looked for at their fixed spot, and the layout counts as open while three or more are found. While it is closed, both placements are tried on every scan, which is cheap. While it is open:

- the anchors that are covered are looked for again at their spot on every scan;
- the char inventory anchor is cleared and not searched, since its templates are in every storage window;
- a window whose two anchors are both covered keeps its origin.

This replaced a design where the Storage button was the parent of the window anchors. Covering the button with the cursor or a tooltip then dropped the layout and started the whole-frame char inventory search (200 to 330 ms, about six times in one recording). Now neither recording has a single whole-frame search.

Why the storage windows are not placed from the Storage button: measured against the windows' sort buttons across every native still, its position wanders by 1.7 UI units, ten times any other anchor. Its template is a large soft highlight and matches at the worst score of the set, so this is probably its own localisation rather than the game moving it, but either way it is the wrong thing to hang a grid on.

**UI space** is 1440p coordinates inside the 16:9 block the game lays its UI out in. The block is assumed to be centred in the capture, which gives the left/right padding of forced 21:9 and the letterbox of a 21:9 game on a 16:9 screen. A game windowed off-centre in a larger capture breaks UI-space bounds; whole-frame anchors are unaffected.

## Page tabs

- **A tab is lit when its active template passes and beats its inactive one by 0.1** (`TAB_LEAD`), and dark the other way round. Anything else says nothing. A lit tab leads by 0.2 (0.98 against 0.77), also with the cursor on it (0.96 against 0.74).
- **A tab under the cursor lights up a little** and scores 0.82 to 0.84 as active, 0.78 to 0.93 as inactive. At the tabs' 0.8 limit that used to pass as active.
- **The page is the one lit tab, else unknown**, and then no slot is read. A tooltip is tied to the slots of the page last lit, for as long as nothing says it changed (`Tooltips.md`). The older rule took the only tab that matched neither state as the active one. Hovering tab 1 with page 2 open brings up a label that hides tab 2, so the hovered tab was that one tab: 13 page-2 slots were written onto page 1 (2026-10-07 22-55-45 recording, frame 1862), and they stayed, because a slot that later matches nothing keeps what it knew.
- **Character storage has a fifth tab, Gear Space** (crossed swords, left of 1), with no template. While it is open no tab is lit, so nothing is read. The older rule read it as page 1 whenever the cursor was on tab 1.
- The stills at 720p, 1080p, 1440p and 2160p recognise the same slots on the same pages as before the change.

## Skipping still frames

- Every frame is thinned to one pixel in eight each way and compared with the last scanned frame. It is scanned when more than 0.5% of those differ by more than 16 in a channel: a tooltip showing up or moving on, a page turning. The cursor moving alone is under that. The whole frame is compared, so a game world animating behind the windows keeps the scanner busy.
- After the last change the next six frames are still scanned, so the slots get their turn.
- A frame is also scanned while something is unfinished: the current hover is waiting for texts (its next read is cut when they are back) or has just gone missing, or slots were left over by the time budget.
- A change too small to count is not picked up until something bigger happens. A slot's number changing on its own is one.
- A skipped frame costs the frame copy and half a millisecond. OCR texts that come back are still applied.
- On the recordings, where the cursor hardly rests, 23% and 38% of frames are skipped natively and 6% and 18% in the browser (there a hover waits longer for its texts). A still image stops being scanned after about 30 frames. Up to 1% loses nothing on the recordings; 2% loses one chest.

## OCR runs elsewhere

A scan never waits for OCR. A line of text takes 30 to 100 ms to read and cannot be interrupted, which is several frames.

- **Jobs.** Whatever needs reading (a slot's number, a tooltip's title lines, amount crops and chest rows) is prepared as the 64 px strip the recogniser takes and queued with an id (`ocr_jobs.rs`). A strip identical to one seen before gets the old id and is not read again, which makes a tooltip standing still on a lossless capture nearly free.
- **Out and back.** Each scan result lists the new jobs. The main thread (`OcrRelay.ts`) hands them, four at a time, to OCR workers: more instances of the same worker that load only the model. Their texts go back in with a later scan, where `apply_ocr` gives them to whatever was waiting. The scanner worker itself has no recogniser.
- **Workers.** One per four hardware threads, at most three. Each loads the 10 MB model again, and twice: once for text and once for slot numbers.
- **Slot numbers are read with digits only.** A job says whether it is a slot's number (`numbers`), and those are decoded with the alphabet cut to `0123456789.+-`. `ocrs` fixes the alphabet per engine and its model cannot be shared, hence the second engine. It is the same model with other characters masked out when decoding, so a "9" no longer comes back as "g" or "S". Tooltip amounts still use the full alphabet; not tried there.
- **Order.** A job's priority is how many reads its hover had already sent, so a hover's first read goes ahead of other hovers' later ones. Slot numbers count as first reads.
- **Nothing is lost when OCR falls behind**, it is only late: the queue is drained in priority order, and the hover it belongs to is kept until its texts arrive.
- **Lifetime.** The OCR workers start and stop with the scanner worker, so ids never cross from one scanner state to the next.
- **Natively** there is no second worker: `run_ocr_inline` reads the queue on the spot after each scan. `tooltip_test` can instead read only so many lines a frame (`OCR_PER_FRAME=n`), which behaves like the browser with slow OCR.

## Slots

- **338 icons, a shortlist of 16.** There are 22 material icons and 294 chest ones (`Game files.md`). Each is kept as the mean colour of 12 cells of its top rows, and only the 16 closest to the slot that way are compared in full. Comparing all of them takes 13 ms a slot against 0.7; under the time budget that left four slots of a briefly shown page unread on a recording, and on the two recordings it was tried on it recognised nothing the shortlist did not.
- **Item level across the slot.** A chest that asks for an item level has it written over the lower part of its icon, a helmet and "1640". Rows 18 to 33 of the 39 compared (1440p) are under it. A chest icon that fails as a whole is compared again with those rows left out, and passes on the 23 that remain: the strip under the number and the last few rows. This is only done for icons with a chest that has a level (`level` in `chests.json`, the game's `ReUseBalanceLevel`), and the slot is then marked `levelled`, which says which of the icon's chests it can be. Such slots score 0.90 to 0.91 whole and 0.95 to 0.98 without the rows.
- **A slot is several icons.** Plain chests in other trims, and one chest on two rarities' backgrounds, score within a hundredth of each other: one slot gave 0.967, 0.963 and 0.957 against three different chests. So the best icon is the slot's, and every other that passes within 0.02 of it is kept as an alternative; a chest read off a tooltip counts for the slot when it is drawn with any of them. Taking only the best left chests without a slot.
- **Every third scan.** Slots change slowly and a tooltip can be gone in a few frames, so the slot step runs on one scan in three (10 times a second at 30 fps).
- **Unchanged.** A slot whose raw pixels hash the same as last time is skipped outright. Otherwise it is compared with the crop last seen, and only a slot that differs is matched against the icons again.
- **Slots that match nothing are remembered too** (empty, unknown, or covered), so they are not matched again on every scan. Their unchanged test is strict (0.995), or an icon the cursor is slowly leaving would stay unrecognised.
- **A time budget** (60 ms a look) stops matching and number clean-up for the scan; the slots left over are done on the next looks. Opening a page therefore takes about a second and a half to read fully, instead of holding up one scan for 0.7 s.
- **The number** is cleaned up on the spot (about 9 ms, the expensive part) and its strip sent to OCR. Until the text is back the slot keeps its previous amount if it holds the same icon.
- **The amount is a vote over the slot's last five reads** (`vote_amount`), compared by their digits, the newest winning a tie, forgotten when the icon changes. A wrong read is usually the last one a slot gets before its page goes away (cursor or highlight on it), and it used to stand. They are not brief: on the recordings each stayed up for 53 to 840 looks without a re-read, because a recognised slot counts as unchanged while its icon area is within 0.95 of the last crop and the number strip is not compared at all. Tightening that to 0.99 fixed fewer than the vote for 14% more reads; not done. The cost of the vote: a number that really changes needs as many new reads as the old one had, and a still screen gives one.

## State between frames

- Everything stays in Rust for the life of the worker's wasm instance: the `ScannerState` (a thread-local in the wasm crate), the base templates and a cache of templates scaled to the current resolution. The reserve operation builds all of it from the config and resolution.
- Consequently the scanner worker must not be restarted between frames, unlike the optimizer worker. A new capture, resolution or character terminates it, so each starts with empty scan state.
- A scan returns a `ScanResult` (`scan_result.rs`): the text and status of every slot looked at so far (matched or not, but not the ones edited by hand), the hover summary, the buffer info and the new OCR jobs, every time; slot images, debug entries and the chest list only when they were written since the last result. A slot's image is the whole slot with its number, as last recognised, or as last seen while it never was. Images cross as `Uint8Array`. The state marks what was written (`changed_slots`, `changed_debug`, `chests_changed`) and the wrapper clears the marks after serialising.
- The first scan of a loop asks for `full`, which sends everything, because the page may have been remounted or a reply dropped. A reply whose `full` does not match the request is ignored.
- Every scanner result carries the buffer info, which is how the capture card knows a state exists. The worker keeps its own copy of the pointer from the reserve result.
- The page (`ScanStore.ts`) merges every result into plain maps and reuses the image objects, so slots that did not change do not redraw their canvases. The grid itself is updated at most four times a second.
- Setup sends the config and the new icon positions with a frame and gets the config back, in the config file's own format (pixels as number arrays).

## Who owns the stream

- The capture stream and its frame reader live in one "frame source" object on the main thread, stored in the roster config next to the scanner worker bundle. It is shared by all characters and never persisted. Because it carries the user's screen-share permission, it outlives the capture component: leaving the scanner page only pauses the loop, and coming back resumes it without asking again.
- Each frame is transferred to the worker with its operation, so exactly one side owns it and the worker closes it. The older design transferred the whole stream to the worker instead; it was changed for stream ownership, not because it leaked.
- **Never `console.log` scanner results per frame.** They carry images. Firefox keeps recently logged objects alive even with DevTools closed, so logging the old round-tripped state leaked several GB per minute on screen share. Chrome only does this with DevTools or an automation client attached. The worker and worker bundle log only the operation name for scanner operations.
- Stopping capture terminates the worker, which frees the frame buffer and the Rust statics together. The next capture reserves again.
- Changing character clears the scanner state and terminates the worker but keeps the stream; the capture component then reserves a fresh state.
- A worker reply is delivered to whichever callback was registered last, so the scan loop ignores replies it has already handled or that belong to a stopped loop.

## Where the time goes

Every scan is timed. Rust stages accumulate in `timing.rs` (`timed("name", ...)`) and are drained by the worker after each call; the worker and main thread add their own parts, and one record per scan is pushed to `window.__scan_timings`. `Browser harness/Screen scanner.md` says how to read it.

Measured again on 2026-10-09 with the chest table in (338 slot templates), on the recordings the chest tests use: `1080p raw/2026-10-07 17-28-55` and `22-55-45` (30 fps files, so 15 frames a second at half speed) and `1440p raw/2026-10-08 21-42-32` and `21-31-53` (3440x1440 at 60 fps, so 30 a second). Headless Chromium, 16 threads, three OCR workers. Mean ms per scanned frame:

| | 1080p (17-28-55 / 22-55-45) | 3440x1440 (21-42-32 / 21-31-53) |
| --- | --- | --- |
| Whole wasm call | 16.8 without the anchor searches / 17.3 (median 14.5 / 15.2, 99th 74 / 60) | 30.8 / 28.5 (median 27.6 / 24.1, 99th 90 / 87) |
| No tooltip / item tooltip / chest tooltip | 12 to 13 / 17.5 / 20 to 21 | 23 to 26 / 26 / 32 to 35 |
| Tooltip step | 8.0 / 10.1 | 20.4 / 17.7 |
| of it: finding the title bar | 4.0 | 10.7 / 9.6 |
| of it: layout, title, icon, chest (each when it runs) | 1.4, 1.4, 2.7, 2.9 | 3.0, 1.8, 3.1, 3.3 |
| Slots, over every scanned frame | 3.4 / 4.1 | 4.9 / 4.4 |
| Anchors and pages, without the searches | 2.4 / 1.9 | 3.2 / 3.5 |
| Change check | 0.3 | 0.9 |
| Serialising the result | 0.6 / 0.8 | 1.2 / 0.8 |
| Frame copy | 3.2 | 10.1 / 9.3 |
| Result back to the main thread | 1.5 / 1.2 | 6.4 / 5.3 |
| Whole loop, without waiting for a frame | 65 / 22 | 47 / 43 |
| Frames taken, of those the upload drew | 85% (the searches) / all | 70 / 84% |
| OCR, ms a line | 58 / 73 | 82 / 79 |
| A hover's first read back: median, 90th, worst | 0.23, 0.44, 0.76 s / 0.26, 1.26, 3.4 s | 0.31, 0.65, 1.4 s / 0.24, 0.69, 1.8 s |

- **Against 2026-10-06 at 1080p nothing got slower but the tooltip step**, 7 to 8-10 ms (the chest rows are matched against the whole table, about 3 ms a frame with a chest tooltip). The slot step is cheaper than it was, 3.4 to 4.1 ms against 5.7 to 8.7, with six times the templates: the shortlist does that.
- **At 3440x1440 the loop is the limit, not the frames.** It never waits for a frame (`read` 0.1 ms median) and takes 43 to 47 ms a turn, so 21 to 27 frames a second of the 30 offered. Finding the title bar is the largest single part, 10 ms on every frame; it scans the whole frame, 2.4 times the pixels of 1080p. Frame copy and the result's way back grow the same way.
- **The slow tail is still the slot step at its 60 ms budget**: the scans over 60 ms have 51 to 62 ms of slots in them (icons 25 to 37, numbers 10 to 15). about 1% of scans at 1080p, 4 to 5% at 1440p.
- **17-28-55 loses its anchors for eight stretches** (about half its length in all) and runs the whole-frame search 183 times at 183 ms; with those its mean wasm call is 60 ms. Why they are lost was not looked at. 21-31-53 searches 8 times in its first 4 s, at 590 ms each.
- **OCR is not short of workers.** 11 to 19 lines a second are asked for, three workers are busy 21 to 51% of the time, and the queue is empty after 54 to 78% of batches. The long waits are bursts: 22-55-45 has backlogs of 115 and 126 lines that take 3 to 6 s to clear, and 73 of its 577 first reads waited over a second. A line is 300 to 380 px of strip, about 200 ms per 1000 px at 1080p and 240 at 1440p (the scan worker is busier there).

Measured with `pnpm scanner-profile` (2026-10-06, the two 1080p recordings in `scripts/brightness/1080p raw`, played at half speed so the scanner sees every frame, which feeds it 15 a second; storage layout open, headless Chromium, 16 threads so three OCR workers). Mean per scanned frame, the two recordings being the busier one with the large cursor and the quieter one:

| | Debug info off | Debug info on (busier only) |
| --- | --- | --- |
| Whole wasm call | 16 / 18 ms (median 11, 99th percentile 72 to 78) | 17 ms (median 11, 99th 77) |
| No tooltip / item tooltip / chest tooltip | 13 to 17 / 15 / 19 to 22 ms | 14 / 16 / 21 ms |
| Tooltip step (finding the title bar 4 ms of it) | 6.8 / 7.2 ms | 7.8 ms |
| Slots, averaged over the two scans in three that skip them | 5.7 / 8.7 ms | 6.9 ms (debug crops 0.5) |
| Anchors and page detection | 1.5 / 2.0 ms | 1.8 ms |
| Serialising the result (`to_value`) | 0.2 ms | 0.4 ms |
| Frame copy | 4 ms | 4 ms |
| Result back to the main thread, then merging it | 0.6 / 0.5, then 0.1 ms | 1.1, then 0.1 ms |
| Table update, at most 4 a second | 1.5 / 1.0 ms each | 6 ms each (99th 16) |
| Frames scanned, of those the upload drew | 91 / 71% | 92% |

- The wasm call is about a third of a 15 fps frame, so the loop spends most of its wall clock waiting for the next frame (`read`, 36 ms mean).
- **The slow tail is all the slot step.** Of the 62 scans over 60 ms in the busier recording, the slot step is 59.5 ms of the mean: icon matching 32 ms and number clean-up 14 ms, stopped by the 60 ms budget. Everything else stays within a millisecond or two of its usual cost.
- **The slot step is budget-bound, so making it cheaper buys slots, not milliseconds.** Cutting the alignment search from 74 samples to 9 took number clean-up from 16.3 to 3.1 ms a frame it ran (median; 99th 49.7 to 16.0) and from 6.20 to 0.99 ms a call in the native bench, but the whole wasm call stayed at 16 ms mean and 70 ms at the 99th. The step just works through more slots in its 60 ms: 2.1 to 2.8 number clean-ups a look. It shows up as slots being recognised sooner after a page opens, not as a shorter scan.
- Matching only runs on the 30% of scanned frames where a slot changed; `slots` costs nothing on the rest.
- A skipped frame costs the frame copy plus 0.5 ms in wasm.
- OCR: 62 to 66 ms a line, 2590 lines over the busier recording's 118 s of playback. A hover's first read came back in 0.27 s at the median and 1.0 s at the 90th percentile (worst 2.2 s); the quieter recording 0.24 s and 0.46 s.
- With one OCR worker on that recording the median was 5 s and the worst 14 s; with two, 0.3 s and 5 s (measured at 6 fps capture).
- A whole-frame anchor search takes 200 to 330 ms. It used to run about six times in the busier recording, when the Storage button got covered; it no longer runs while the storage layout is open, and the longest scan there went from 357 to 139 ms.
- Firefox: 26 ms a scan (frame copy 7 ms), 96% of drawn frames scanned.
- Reading results against the native harness run over every frame: `Tooltips.md`.

Before that, with the state staying in Rust but OCR still inside the scan (same recordings, 6 fps capture), mean per scan:

| | Debug info on | Debug info off |
| --- | --- | --- |
| Whole scan | 330 to 360 ms (2.7 to 3.0 scans/s) | 295 to 315 ms (3.1 to 3.4 scans/s) |
| No tooltip / item tooltip / chest tooltip | 165 to 205 / 290 to 400 / 625 to 730 ms | 155 to 165 / 245 to 360 / 585 to 660 ms |
| OCR, all calls | 185 to 215 ms | 175 to 185 ms |
| Re-matching unrecognised slots | 70 ms | 70 ms |
| Waiting for a frame | 30 to 35 ms | 25 to 40 ms |
| Returning the result (serialise and clone) | 7 to 8 ms | under 1 ms |
| Frame copy | 4 to 5 ms | 4 ms |
| Table update, at most 4 a second | 14 ms each | 1.5 ms each |

OCR and the unrecognised slots were what was left: a scan without a tooltip was about 160 ms, of which 70 ms was the re-matching and 15 to 50 ms slot OCR. About half of the 6 fps frames were scanned.

Before that, with the whole state serialised out and back on every scan (9% of the 6 fps frames scanned with debug info on):

| | Debug info on (default) | Debug info off |
| --- | --- | --- |
| Whole scan | 1920 ms (0.5 scans/s) | 620 to 690 ms (1.5 scans/s) |
| Moving the state (clone to worker, deserialise, serialise, clone back) | 1085 ms | 185 to 215 ms |
| Vue re-rendering the tables | 400 ms | 70 to 80 ms |
| OCR, all calls | 300 ms | 265 to 280 ms |
| Re-matching unrecognised slots against every icon | 70 ms | 70 ms |
| Everything else (frame copy 3 ms, tooltip detection 4 ms, anchors, pages, unchanged-slot checks) | under 30 ms | under 30 ms |

- The state holds only 1 to 4 MB of image bytes, but each byte crossed as a JS number in an array, four times per scan.
- OCR costs about 30 ms for a slot number and 60 to 100 ms for a tooltip line. A scan with no tooltip has 1 to 3 calls (slots under the cursor being re-read), a chest tooltip 8 to 9, about 600 ms.
- About 230 slots (empty or unknown) are never remembered, so each is compared with every icon on every scan.
- The first scan takes 0.8 to 1.3 s, mostly matching and reading every slot once.

## Conventions

- **Reference resolution is 1440p.** Slot positions, anchor offsets and window rectangles are constants in 1440p pixels, scaled by the game's UI height. Positions are kept as floats relative to the window origin; nothing is normalised to 0..1.
- **The game resolution is chosen by the user, never detected.** The capture can be a whole screen with the game windowed inside it, so its size only seeds a guess: once a capture starts, the capture card shows a resolution list with the closest entry preselected, and a "Forced 21:9" checkbox. An ultrawide capture gets the 21:9 list and the checkbox locked on. The choice is shared module state, not persisted, and is re-guessed on every new capture.
- **UI height** is the height of the 16:9 (or 21:9) area the game renders into: the smaller of the game height and width x 9/16 (or 27/64; the game's 21:9 is really 64:27, measured as 2560x1080 on a 2560-wide capture). The scale factor is that height, rounded, over 1440. Rust derives both on every scan call from the three values the frontend sends.
- **Changing the resolution restarts the scanner** like a character change does (worker terminated, state cleared, reserve again), because cached templates and remembered slots are scale-specific. The results table keeps its old rows until the first new result.
- **Pixels are RGBA** throughout, on both sides of the boundary.
- **Fixed-position comparison is a colour distance, not a structural score.** It is the mean absolute RGB difference, taken as the best of the nine one-pixel shifts, reported as a confidence (1 - distance / 255). The pass limit is 0.9 unless the template carries its own required confidence, which only the page tabs do (0.8). Anchor search by template matching scores differently but uses the same rule: 0.9 unless the template carries a required confidence, which no anchor does yet. Structural similarity was dropped: a half-pixel grid error was enough to fail it, and it scored colour variants of one icon (Blue and Serca Blue) almost alike. Because variants can both pass, slots take the best icon, never the first.
- **The game scales its UI continuously.** Nothing is snapped to whole pixels and no step is rounded. Measured on native stills at UI heights 720, 768, 810, 900, 1080, 1440, 1620 and 2160 (`icon_locate`, `anchor_locate`): the fractional part of a matched icon's position is spread flat, not clustered, so positions are not integers; the distance between any two anchors is the same in UI units at every height to within 0.3; and the slot pitch comes out 69.745 against the assumed 69.75, i.e. 0.05 UI over nine steps. So a resolution-dependent offset is ours, not the game's. The one that was there came from the template size rounding, above.
- **Slot pitch was measured on a 1440p capture** (69.75 px), and holds to 0.006 UI per step across all native heights. The grid is the same in every window except that roster storage's starts about 0.9 px further left. A pitch error accumulates across the grid, so re-measure against a native capture rather than nudging it by eye.
- **Fixed-position comparison needs identical dimensions.** The template path and the crop path must round scaled sizes the same way, or comparisons quietly return no match.
- **The frame buffer is sized once**, at capture start, and freed when the worker is terminated. JS must rebuild its view of wasm memory every frame, since memory growth invalidates old views.
- **Brightness is normalised.** The in-game brightness setting changes pixel values, so observed crops are mapped to a fixed reference brightness before comparison. Stored templates are already normalised. See `Config and calibration.md`.
- **The brightness estimate is an average that only gets better.** An anchor variant gives its own estimate when it is found: the mean of the matched patch through a curve fitted for that variant. One that matched the wrong thing gives nonsense. The setting does not change during a session, so the estimate used is the average over the most anchors ever found together (or as many again), and it stays when some of them are lost. An anchor only counts once it has passed its re-check on a later scan, and no longer counts when it fails one. Until any has, the estimate is the first anchor's own. `tooltip_test` prints each change and each anchor's own estimate to stderr.
- **A window fading in must not fix the estimate.** The storage layout fades in over about 20 frames and its anchors match from the first dim one (the search score ignores brightness), reading 0 and then rising. An estimate kept from then made every later re-check fail, so nothing was ever read (the 2026-10-07 recording, which opens on the fade). Two rules: an anchor's estimate is taken again from the frame at every re-check it passes, and when no anchor passes, the count goes back to none so the next anchor found sets the estimate. On that recording it follows the fade and is at 61.1 on the first full frame. The cost: when every anchor is lost (storage closed) the next match, right or wrong, sets the estimate, as at the start of a session.
- **Which anchors tell the brightness.** The buttons do (sort, search, Storage): 5 in the storage layout, 2 on the lone inventory. The storage windows' icon groups do not: thin icons are drawn anew at each resolution and their mean comes out 9 settings apart between 1080p and 1440p. On the 1080p captures at all 21 settings the average is within 0.6 of the truth, and each button within 1.5. On the 1440p stills the buttons read 69 to 73 (the setting they were taken at is not recorded, about 70). The recordings at 60 read 59.3 and 59.5, the one at 100 reads 97.8, the one at 0 reads 0.3.
- **Picking the brightness at which a patch looks most like its template was tried instead of the curves** and is worse: 4 settings off at the dark end and up to 3.5 at the bright end, because it inherits the model's error there.
- **What the setting is: a display gamma.** Measured on `scripts/brightness/inputs storage` (the storage layout with a tooltip at all 21 settings), a value seen at setting 50 becomes `value ^ ((62.6 + 50) / (62.6 + setting))` at another setting, with values as 0 to 1. Normalising to 50 is the inverse, and that is the whole model (`brightness.rs`): one number, no gain. It reads as a gamma that rises in a straight line with the setting; only the ratio 62.6 of that line's offset to its slope can be told from captures (1.2 to 3.2 with 2.2 at 50 would be 60, which fits worse). Black and white are fixed points of a power law.
- **How well it fits.** On opaque icon pixels, per value 0 to 255: 0.15 levels off on average over all settings, 1.6 at worst; 0.3 at setting 0 and 0.24 at 100. A separate best exponent for each setting only gets to 0.14, so what is left is not the curve's shape. The older model (a power with ratio 66.5 and a two-constant gain) was 1.9 on average and 12 at worst, at setting 0.
- **The gamma is applied to what is drawn, before it is blended.** Pixels that share a value at 50 do not land on one value elsewhere (spread 0.7 levels at settings 40 and 60, 4 at setting 0). White glyphs are the clear case: a slot number's soft edge pixels move 12 to 25% of what the law says while icon pixels of the same grey move 90 to 97%, because white stays white and only the icon under the edge changes. So the number crop is still not normalised like the rest (see the number bullet below), and tooltip text edges come out a few percent too dark or too bright when normalised per pixel. What shows through a nearly opaque panel is the same thing: the tooltip body is brighter than its title bar at setting 0 and darker from 10 up.
- **Effect of the law on reading** (against the older model, same build otherwise). Captures at every setting: slots recognised go from 16, 19 and 23 to 20, 23 and 24 at settings 0, 5 and 10, the same elsewhere. Recordings at 60: the same chests. The recording at 100 first read 19 chests against 24, until two tooltip colour limits that had been set by eye on the older model's values were redone (`Tooltips.md`); it now reads 26.
- **The anchor and page-tab templates were cut again for the law** (see `Config and calibration.md`). They changed by 0.1 to 2 levels on average, and nothing that was read before reads differently.
- **Capture stays at native resolution.** Downscaling was tried and removed because the quantity digits need full resolution.
- **Slot numbers are separated from the icon with a compositing model** (`image_utils/number.rs`). The game's brightness setting changes the icon but not the number drawn over it (measured: digit pixels are identical across all 21 settings). So the number crop is used as captured, never normalised, and the expected background is the matched icon's template moved to the on-screen brightness. The template is lined up to a quarter pixel on the icon area below the number, which nothing covers; lining it up on the number strip itself fails because the number hides most of it. That line-up is a single quarter-pixel step either way, nine samples: with anchor templates cut to whole pixels the grid is good to about a twentieth of a pixel, every slot across the native stills wants no more than a quarter (76% want nothing), and reads match the old two-pass search over four times the span. Dropping it entirely changes one read in 369, so it stays. Each pixel is then explained as white digit over shadowed background, which gives how much white the template cannot account for and how much the template had to be darkened. A white pixel is a digit if the template cannot explain it, or if it has the number's dark shadow within reach on both sides (digits over a white part of the icon). Nothing else survives, so icon highlights are gone. A third test goes by colour (`alpha_by_colour`): shading keeps a background's hue and white washes it out, so how grey a pixel is over a coloured part of the icon says how much of it is digit, however bright that part is. It was added for a thin "1" over the pale blue of a pouch: an edge pixel there is (181, 181, 182) over (119, 163, 244), which the first test explained as half white over the pouch, just inside its tolerance of 24, and the shadow test failed on both sides (a grey edge pixel on one, a dark neighbour of the template on the other). "100" read as "00" at brightness 61 and under and as "100" from 62. The colour test only speaks where all nine template neighbours are coloured (channel spread of 40 or more). Over every slot of the stills and both brightness sweeps (2,232 reads) it changed nine: seven dropped leading "1"s now read, and two 720p slots went from one wrong read to another. A fourth rule joins strokes: a pixel short of the limit (`low_alpha`, 0.3 against 0.55) still counts where it touches a digit pixel. A "6" over a pink pouch had four stroke pixels at 7 and 8 sixteenths with no shadow on the pouch's side, and read "54" for 64 on all nine reads of a recording. Limits of 0.2 to 0.4 score alike. It costs a stray "8" before "809" at three more brightness settings of the storage sweep, and at 720p it lets more of the surroundings in. The shadow test has to stay: without it 690 of 1,392 recording reads are wrong, since nothing else vouches for a digit over a white part of an icon. Levelling the digits' brightness in the strip was tried (whole strip, per blob, binary): no gain, and binary is much worse. After that: blobs of a few pixels are removed, the empty black left of the number is cut off with a small margin (the recogniser is touchy about it: 2 to 4 px at 1080p works, 6 to 8 px is much worse than no crop), and the strip is resized to 64 px, white on black. Inverting made the recogniser worse on numbers, although tooltip text is inverted and reads better that way.
- `cargo run --release --bin slot_scores -- <still>...` prints, for every slot that looks like something, the four icons it is closest to, each with its score, its score without the item level's rows and its place in the shortlist's order, whatever the limits. `DUMP=<folder>` also writes each slot as seen next to its closest icon. This is how the ancient background and the look-alike chests were found.
- **Working on the number clean-up.** One function makes the strip (`number_strip` in `ocr.rs`, with a `NumberParams`), and two bins sit on it:
  - `number_bench` re-reads every slot number the scanner asks for, under each entry of its `variants()` list and with both alphabets, from stills or a piped recording; `scripts/tooltips/number_bench.py` scores the output. Stills of one scene at many brightness settings are scored against the read most agree on, recordings against the tooltip amounts, per read and per slot (last read, and the vote of five). Add a variant to the list to try something.
  - `number_probe` writes every stage of one slot's clean-up as images each time that slot is read (captured crop, expected background, white by fit, white by colour, shadow, mask, strip), and `scripts/tooltips/number_stages.py` stacks them into one picture per read.
  - Measured 2026-10-09 with all of the above in: the brightness sweeps 3 wrong of 630 and 16 of 1,089; the four recordings 7 wrong reads of 1,392 and 1 slot of 200 ending wrong, against 26 and 6 before. In the scanner itself, slots whose number agrees with their tooltip amount went from 23 of 24 to 25 of 25 and from 57 of 65 to 65 of 65 on the two 1080p recordings, and stayed 46 of 46 on 21-42-32.
- Older: `cargo run --release --bin number_dump` and `number_test` dump number and icon crops from stills and score pre-processing with the real recogniser against hand labels (`scripts/tooltips/number_labels`). Those labels are from before the chest table and no longer line up with a fresh dump. On recognised icons the current version reads 182 of 182 on the stills and 294 of 294 on the brightness sweep; the parameters were tuned on those two sets. On three static frames of the lossy recording it reads 34 of 34 roster numbers.
- **OCR** uses the `ocrs` / `rten` crates with only the recognition model, on a preprocessed single line, 64 px tall. Text detection is skipped because the number's position is known.

## Status

Working: capture, frame transfer, anchor detection for the character inventory and the storage layout (pet and NPC, 16:9, forced 21:9 and a native 3440x1440 game), page detection, brightness estimation, slot icon identification, quantity OCR, tooltip reading including chests matched against the game's own table, and a slot grid that shows every slot with its status and lets the user correct it (`Slot grid.md`).

Not done yet:

- Results are not written into the calculator's material inputs. Output stops at the manifest under the slot grid (`Manifest.md`).
- Icon placement is now checked on native stills at 720p, 768p, 900p, 1080p, 1440p and 2160p and forced 21:9 at 1080p, 1440p and 2160p (UI heights 810, 1080 and 1620), all in `scripts/brightness/<height> raw`. The quantity OCR still degrades as the scale drops and has only been judged at 1080p and 1440p.
- A slot number read while the cursor, the hover highlight or a fading tooltip is over the slot can be wrong, and the last read is what is kept. On the recording 20 of 28 slots end with the right number although static frames read perfectly.
- The icon matcher lets other items through as "Fusion" (gems and accessories on the same dark blue background).
- A crop that falls outside the frame (inventory window partly off the capture) still panics.
- A session that only ever finds one brightness-telling anchor keeps its error (up to 1.5 settings at 1080p).
- A brightness setting changed in game during a session is not picked up until the scanner restarts.
- **`public/ScannerConfig.msgpack` is 7.8 MB**, from 0.5: 294 more icons at 64x64, each pixel channel written as a msgpack number. It is fetched when the scanner page opens and crosses to the worker as arrays of numbers. Writing the pixels as bytes would take it to 5.5 MB, as PNG to about 2; not done.
- At 1080p from a compressed recording, slots that are plainly a known icon score just under the 0.95 limit and are not read (Serca Blue 0.945 with the next icon at 0.83; a chest at 0.946). The same slots lead every other art by 0.05 to 0.13, where an empty or unknown slot leads by under 0.02, so passing on the lead as well as on the score would read them. Measured with `slot_scores`, not tried in the scanner.
- The two book icons pass at 0.93 instead of 0.95 (`Config and calibration.md`); nothing else has its own slot limit.
- The roster page tabs for "page 1 inactive" and "page 2 active" were built from character-storage tab pixels, which look the same.
- Tooltip reading works in the native harness and in the browser on an uploaded recording, where every frame is scanned. Its limits are listed in `Tooltips.md`.
- While no anchor is found the whole-frame search runs on every scanned frame, 200 to 330 ms each. A still screen is not scanned, but with no inventory open over a moving game world the scanner uses a full core.
- The storage layout's fixed spots assume the game is centred in the capture, like all UI-space bounds. The window recordings are not: they are 1918 wide and 1078 or 1070 tall with the rows missing at the bottom, so the UI is 1 or 5 px off the assumed place. 1 px is inside the slack; at 5 px the storage layout is not found (the native runs pad that recording to 1080 rows).
- A page takes about a second and a half to be read after it opens (slots are done a few at a time).
- The slot pass limit was raised from 0.9 to 0.95 because unknown icons were being identified on a live screen share. On the two recordings that leaves far fewer slots recognised, and so fewer hovers resolved to a slot (4 and 11 slots with tooltip data, against 19 and 27 at 0.9). Chests do not depend on it.
