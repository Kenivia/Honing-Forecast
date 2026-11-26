# Scanner pipeline

The scanner reads the player's material counts off a shared game screen, so they do not have to type them in. It is a beta page; `public/WIP.md` has the author's task list.

Code: `crates/scanner` (image processing), `crates/wasm` (wrappers), `frontend/Components/Character/InventoryScanner` (capture and the slot grid), plus scanner operations in the shared wasm worker. What the page shows is in `Slot grid.md`. How to test a change is in `Developing and testing.md`. What the game draws, measured, is in `Further learnings/How the game draws.md`. What was tried and dropped is in `Further learnings/Pipeline tried.md`; read it only when trying something new here.

## Stages

1. **Capture** (TypeScript, main thread). A video track is read as a stream of frames, and every frame is scanned. The track comes from the browser's screen share, or from an uploaded image, video or debug capture drawn onto a canvas; all go through one frame reader.
2. **Frame transfer**. Per operation, the main thread transfers one frame to the worker. The worker copies its pixels into a buffer in wasm memory, closes the frame and calls Rust (`cropper_wrapper`). Nothing else goes in but the `full` flag, the OCR texts that have come back and the hand edits.
3. **Worth a scan?** A frame that hardly differs from the last scanned one is dropped ("Skipping still frames").
4. **Locate the windows.** Anchors are template-matched inside their bounds, which gives roots, window origins and a brightness estimate; later frames re-check them at their known place ("Anchors and layouts").
5. **Detect the page**: the one lit tab of each window ("Page tabs").
6. **Read slots**, on a fraction of the scans: each slot of the active page is at a fixed offset, unchanged ones are skipped, and the others are compared with the icons; the number goes off to be read ("Slots").
7. **Find the tooltip**, cut its lines out to be read, and write what it says onto the hovered slot and the chest list (`Tooltips.md`).
8. **Read the currencies** (`Currency.md`).
9. **Result and loop.** Rust returns a small result, not its state. On each result the main thread first reads and posts the next frame, then hands the result's lines to the OCR workers and the rest to the page.

Only the anchor search is expensive (FFT cross-correlation). Everything after it is a cheap fixed-position comparison.

## Anchors and layouts

An anchor is a piece of fixed UI matched to get a *root*, a position other things are placed from. `ANCHORS` in `anchor_to_slot/anchor_defs.rs` is the one place that describes them: variants (alternative templates, so that one can be covered while another holds; the root comes from the first one found), each variant's bound and brightness curve, the lone inventory's roam, and which inventory windows the anchor locates. `Bound` there documents the kinds of bound.

- **A match always locates**: the root is the matched position minus the template's stored offset, never the bound's position. So a bound only has to be roughly right.
- **Templates are cut to whole screen pixels** (`whole_pixel_crop` in `interface/setup.rs`). Scaled naively, a template is stretched by up to half a pixel, the match settles a fraction off, and every grid sits a fraction out at any height 1440 does not divide. The search area is put on whole pixels too, since resampling moves the brightness estimate. Slot icons are left alone: template and crop are rounded to the same size over the same UI units, so they are stretched alike.
- **Slot grids and page tabs are defined per inventory**, relative to its window origin, so several anchors can locate one inventory. Its origin comes from the first anchor found, in list order, that locates it.
- **Re-checking needs the brightness.** Until there is an estimate, an anchor is matched again around its place (`check_existing_anchors`).

Current anchors:

- **Storage**: the highlighted Storage button (pet-menu and NPC variants). It locates nothing, because its position wanders far more than any other anchor's.
- **One per storage window**: the sort button at its top left (the same template in all three) and its top-right icons (the storages' move-to group, the inventory's search button).
- **Char inventory**: the free-floating window, by the same two templates. It is looked for at its default place on every scan and otherwise over where it can be dragged to (below).
- **Currency**: the currency window at its default place, only while the storage layout is closed. It locates no inventory and tells no brightness, since the window is see-through (`Currency.md`).

**The storage layout is a special case, written as one** (`update_anchors`). Roster storage, character storage and the inventory always sit side by side at one of two placements (pet menu, or shifted up and left at the NPC). So its seven anchors are each looked for at their spot, and the layout counts as open while most are found. While it is open, covered anchors are looked for again on every scan, the char inventory anchor is not searched (its templates are in every storage window), and a window whose two anchors are both covered keeps its origin. So covering the Storage button does not drop the layout.

**The lone inventory** is kept by the game inside the render area, and native ultrawide behaves like forced 21:9. Its default place is searched on every scan, both variants (`lone_variant`, `WINDOW_ANCHOR_SLACK`). Where it can be dragged to (`roam`) is too slow to search every scan, so `search_lone` rations it:

- both variants when the frame has changed enough (`CHAR_INV_SEARCH_THRESH`) from the frame of the last such search (so a fade adds up), and on the scan the anchor was lost at its re-check;
- otherwise one variant every `CHAR_INV_FORCED_SEARCH_FREQ` scans, in turn;
- never waiting for a later frame, since a still screen sends none;
- the second variant only where the first one's root puts it, on every scan, so a covered variant is found again when it shows.

**UI space** is 1440p coordinates inside the 16:9 block the game lays out its UI in, assumed centred in the capture. **Only a full-screen share of a full-screen game is supported.** A windowed game puts the area off-centre, and the storage layout, the currency window and the 21:9 detection are then lost.

## Page tabs

- **A tab is lit when its active template passes and beats its inactive one by `PAGE_NUM_LEAD`**, dark the other way round, and says nothing otherwise. A hovered tab lights up a little, so the lead is what tells.
- **Slots are read only from the one lit tab.** Taking a tab that matched neither state is wrong: a hover label over a tab hides it.
- **A tooltip is tied to the page last lit** (`last_pages`) until another tab lights up or its own looks dark for `DARK_SCANS` scans while uncovered: a tall tooltip covers its own window's tabs. Slots are never read off this.
- **Character storage's Gear Space tab** has no template; while it is open no tab is lit and nothing is read.

## Skipping still frames

- Every frame is thinned to a pixel in a few each way and compared with the last scanned frame (`worth_scanning`). It is scanned when enough samples differ: a tooltip showing or moving, a page turning, but not the cursor alone. A world animating behind the windows keeps it busy.
- The next `QUIET_SCANS` frames after a change are still scanned, so the slots get their turn. A frame is also scanned while something is unfinished: a hover waiting for texts or just gone, slots left over by the budget, an unsettled currency.
- A change too small to count (a slot's number changing on its own) waits for something bigger.

## OCR runs elsewhere

A scan never waits for OCR: a line takes tens of ms in the browser and cannot be interrupted. Everything about reading text is in `crates/scanner/src/ocr` (`jobs.rs` the job and queue, `text.rs` and `number.rs` the strips, `recognize.rs` the two engines, `shapes.rs` the font reader).

- **Jobs.** Whatever needs reading is cut out as captured and queued as an `OCRLine`: the RGBA crop, its frame's brightness estimate and its `Kind`. A line identical to one seen before (pixels, kind, and for a number its icon crop and template) gets the old id and is not read again. The brightness estimate is left out of that test, since it wobbles in its last digit. A job names the read it belongs to; one asked for again more urgently is sent again.
- **The reader prepares the strip** in the OCR worker (`OCRLine::strip`, `read`). Text lines get `text_strip` (`TextParams`; sharper strips read worse). A slot's number is read by its shapes first (below) and goes to the recogniser only when they are not sure, so the OCR workers load the config too.
- **Out and back.** The main thread (`OcrRelay.ts`) hands jobs untouched to OCR workers, more instances of the same worker; texts come back with a later scan (`apply_ocr`). Crops cross by transfer. The worker count scales with hardware threads and stays small, since each loads the model twice.
- **Digits only** for slot numbers and `Digits` lines: a second engine with its alphabet cut to digits and `.+-` (ocrs fixes the alphabet per engine).
- **Order** (`pump`): lowest priority first, then a read some line of which is already taken, then the newest read, then by id. Priority is how many reads of its kind the hover already sent, plus extra for a tooltip beside no lit slot (`Tooltips.md`). A batch is one line, since a batch is what a new hover's first read may wait behind.
- Nothing is lost when OCR falls behind, only late. The OCR workers start and stop with the scanner worker, so ids never cross states.
- **Natively** there is no second worker: `run_ocr_inline` reads the queue after each scan.

## Slots

- **A shortlist** (`SHORTLIST`): icons are kept as the mean colour of a few cells of their top rows, and only the closest few are compared in full.
- **Decoys** are templates of things we do not count that look like an icon of ours (`~<icon>@<rarity>`, `Game files.md`). A slot is compared with the nearest icons and, shortlisted apart, the nearest decoys; an icon that scores under the closest decoy does not count. They are what makes the low slot limit safe.
- **Overlays** (`OVERLAYS` in `slots/constants.rs`, `overlaid` in `slots/utils.rs`): the item level and the clock. Each set of overlays an icon can have is scored with those parts left out, and the set that gains most over its `gain` is the one on the slot. The item level applies only to icons with a levelled chest (`has_level`), irrelevant ones included from `LOWEST_LEVEL`. The hover highlight and decoys are judged with the same parts left out. Icons more than `OVERLAY_REACH` under the limit are not tried. A third overlay would be a third entry.
- **A slot is several icons**: every other icon that passes within `ALTERNATIVE` of the best is kept, since chests in other trims or on two rarities score within a hundredth of each other. A chest read off a tooltip counts for the slot when drawn with any of them.
- **The slot step runs on one scan in `SLOT_UPDATE_FREQ`**, under a time budget (`SLOT_BUDGET`), so a page is read a few slots at a time. The step is bound by its budget, so making it cheaper reads slots sooner, not faster.
- **Unchanged slots are skipped**: by hash of raw pixels, then by comparison with the last crop. A slot back from lit to normal is read again when it now scores clearly better as its icon (`BETTER_LOOK`), since the two looks are too close to count as a change. Slots that match nothing are remembered too, with a stricter test (`UNMATCHED_UNCHANGED`).
- **A slot that matches nothing keeps its last icon** (the cursor or a tooltip covers it), **unless a decoy beat every icon it passed as** (`look_alike`): the earlier look was a look-alike lit by the hover glow.
- **The amount is a vote over the slot's last few reads** (`vote_amount`), newest winning a tie, forgotten when the icon changes. Until a read is back, a slot keeps its previous amount. The cost is that a number that really changes needs as many new reads as the old one had.
- **Slot numbers are read by their shapes first** (`ocr/shapes.rs`): every candidate is drawn in the game's font with its shadow over the expected background (the matched template at the screen's brightness) and scored by luma difference at quarter-pixel positions. After a first answer, the hover highlight is fitted as a gain and offset and matched again. Under `MIN_MARGIN` the number goes to the recogniser. The font is one of `public/fonts/<Language>.ttf` (cut by `fonts.py`) or a file the user picks (`check_font`), and is downloaded up front. Only English is checked against captures.
- **Numbers the shapes are not sure of are separated from the icon** (`ocr/number.rs`: the compositing model, the shadow test, the colour test, stroke joining; `Further learnings/Pipeline tried.md` says why each stays), then recognised. The number crop is used as captured, never normalised.

## State between frames

- **Everything stays in Rust** for the life of the worker's wasm instance: `ScannerState` (a thread-local in the wasm crate), the base templates and a cache of scaled ones. The reserve operation builds it from a `ReserveInput` (settings, buffer size, config, priors). The state is never serialised; only what crosses the boundary has serde.
- **So the scanner worker is never restarted between frames.** A new capture, a settings change or a wipe terminates it, so each starts empty.
- **Starting costs a few hundred ms.** The config's pixels are msgpack bin and reach Rust as bytes (`serde_bytes`). Keep them bytes, never arrays of numbers.
- **A scan returns a `ScanResult`** (`interface/scan_result.rs`): every slot's text and status (bar the hand-edited ones), the hover, pages, currencies, detected settings, buffer info and new OCR jobs every time; slot images and chests only when written since the last result (`changed_slots`, `chests_changed`, cleared by the wrapper). A slot's image is the whole slot as last recognised.
- The first scan of a loop asks for `full` (the page may have been remounted). A reply whose `full` does not match is ignored.
- The page (`ScanStore.ts`) merges results into plain maps, reuses image objects, and updates the grid at most a few times a second.

## Who owns the stream

- The capture stream and its frame reader are one "frame source" on the main thread (`create_frame_source` in `FramePassing.ts`), held in the roster config beside the worker bundle, shared by all characters and never persisted. It carries the screen-share permission, so leaving the scanner page only pauses the loop.
- **Never `console.log` scanner results per frame.** They carry images, and Firefox keeps logged objects alive even with DevTools closed (several GB a minute on a share).
- **The scan belongs to the active character** (`ScanSwitch.ts`). Going to another character wipes it but keeps the stream. If anything is not exported (`scan_unexported`), `ScanSwitchDialog.vue` offers to move the scan, wipe it, or cancel. Moving keeps the worker and store, and the manifest works out the new character's saved pages by itself. **Reset scan** is the same wipe.
- A worker reply goes to whichever callback registered last, so the loop ignores replies it has handled or that belong to a stopped loop.

## Where the time goes

Every scan is timed: Rust stages through `dev_utils/timing.rs` (`timed`), drained by the worker after each call, plus the worker's and main thread's parts, as one record per scan in `window.__scan_timings` (`Browser harness/Screen scanner.md`). At a wide capture the frame copy and the result's way back are as large as the scan itself; the slow tail is the slot step, held to its budget. OCR is not short of workers; its long waits are bursts.

## Debug captures

A user records a session, corrects the grid by hand and sends the file in. It is then re-run on any later build: the frames are what was recorded, the reading is this build's. Code: `dev_utils/capture.rs` (format, recorder, replay), `Capture.ts`, the buttons in `Stream.vue` behind "Show debug options". Tools for working with captures are in `Developing and testing.md`.

- **The file** (`.hfcap`, `HFCAP005`) is records of a tag byte, a little-endian u32 length and the payload: `S` the start (size, forced 21:9 and brightness as given, the commit, the priors), `I` per frame (XOR with the frame before, RGBA, LZ4; empty when unchanged), `E` the end (JSON). Older versions are not read.
- **Only the end record's hand edits are right answers.** The rest of it is what the page showed at download, wrong readings included.
- **The recorder worker** gets each frame from the scanner worker (the cropper op's `forward` flag), not the browser, and keeps two buffers in turn (`InputRecorder`). Frames waiting are capped (`RECORDER_BACKLOG`), and recording stops at a size limit (`LIMIT`). One wasm instance per worker: the worker keeps its first `init()` promise.
- **A capture starts from an empty state**, so `Record debug capture` restarts the scanner, and anything else that restarts it throws the recording away.
- **A re-run repeats exactly**: every slot is read at once (no budget), and each frame is scanned only once the texts of the one before are back (natively right after each scan, on the page once no line is out). On the page, a `.hfcap` uploaded into the file input replays with the start record's settings, then shows `Replay finished, re-run on this build.`, with the end record on `window.__capture_reference`.

## Conventions

- **Reference resolution is 1440p.** Constants are in 1440p pixels relative to window origins, scaled by the UI height, kept as floats; nothing is normalised to 0..1.
- **The capture size is the game resolution**, taken from the frames, not the track's settings, which can disagree. The frame source holds back the first frame and reserves at its size (`measure` in `FramePassing.ts`). A frame of another size restarts the scanner.
- **UI height** is the height of the area the game renders into: the smaller of the capture height and width x 9/16 (or 27/64, the real ratio of the game's 21:9). The game draws over the whole capture width (`render_width`). The scale factor is that height, rounded, over 1440. Both are worked out on every scan.
- **Forced 21:9 and brightness are each detected or given** (`Detected<T>` in `scanner_state.rs`; `scan_settings` in `ScanSettings.ts`). The capture card's "Troubleshooting" section has an Aspect and a Brightness select; edits are a draft that `Apply` saves (localStorage, with the number font) and that restarts the scanner. While any setting is off its default, the section stays open.
- **Forced 21:9 is detected from letterbox bars** (`detect_21_9`) on the first lit frame and never looked at again; a capture wider than 2.1 is 21:9 without a look.
- **Manual brightness** is used from scan 0.
- **`Rect` knows its space** (`image_utils/common.rs`): `Space::Ui` or `Space::Screen(ui_height)`, asserted by `scaled`, `get_offset` and `crop_buffer`; `pixel_size()` is the only rounding.
- **Fixed-position comparison is a colour distance**: the mean absolute RGB difference over the template without its outermost pixels, unshifted, as a confidence. `confidence` asserts equal sizes and spaces. Templates may carry their own required confidence, else the default (`DEFAULT_CLOSE_ENOUGH_CONFIDENCE` in `close_enough.rs`) applies. Slots use the lower `SLOT_CONFIDENCE`, which the decoys make safe, and take the best icon, never the first. Alternatives keep the default, or the other rarity of the same art comes in.
- **Pixels are RGBA** across the boundary, OCR jobs included. Only the strip is grey.
- **The frame buffer is sized once**, at capture start. JS rebuilds its view of wasm memory every frame, since memory growth invalidates views.
- **Brightness is normalised** to a reference setting before comparing (`normalize_brightness`, once, by whoever made the crop; the LUT is kept until the estimate moves). Left as captured on purpose: a slot's number crop, its display image, an anchor's search area and its brightness patch.
- **The estimate only gets better**: each anchor variant gives one through its fitted curve when found. The estimate used is the average over the most anchors ever found together, counted only once they pass a re-check on a later scan. Because a window fading in reads low, an anchor's estimate is retaken at every re-check it passes. When no anchor passes, the count resets, so the next anchor found sets it. Only the buttons tell brightness, not the thin icon groups.
- **Capture stays at native resolution**: the digits need it.

## Not done

- Results reach the calculator only through the manifest's export (`Manifest.md`).
- Quantity OCR has only been judged at 1080p and 1440p; it degrades at smaller scales. Placement is checked on native stills from 720p to 2160p and forced 21:9 at 1080p to 2160p.
- A slot number read under the cursor, the highlight or a fading tooltip can be wrong; the vote and the tooltip's amount cover it.
- A crop outside the frame (an inventory partly off the capture) panics, live and on replay.
- A session that only ever finds one brightness-telling anchor keeps its error. A brightness or 21:9 change in game is not picked up until a restart.
- `public/ScannerConfig.msgpack` is about 17 MB of raw RGBA; as PNG it would be a few MB.
- The roster tabs "page 1 inactive" and "page 2 active" are character-storage tab pixels, which look the same.
- With no inventory open over a moving world, the roam search runs on most scans. A lone inventory fading in, and one at 16:9, have not been recorded; its default place and range are measured at 1440p only.
- A recording a few rows short of the game's height puts the UI off its assumed place; within the slack it still works.
