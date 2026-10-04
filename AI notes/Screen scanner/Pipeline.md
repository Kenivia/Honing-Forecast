# Scanner pipeline

The scanner reads the player's material counts from a shared game screen, so they do not have to type them in. It is in active development on the `Inventory-scanner` branch. `public/WIP.md` has the author's current task list.

Code: `crates/scanner` (image processing), `frontend/Components/Character/InventoryScanner` (capture and debug UI), plus scanner operations in the shared wasm worker.

## Stages

1. **Capture** (TypeScript, main thread). A video track is read as a stream of frames at a low frame rate. The track comes from the browser's screen-share API, or from an uploaded image or video file drawn onto a canvas (a fake stream: an image repeats, a video plays once and then holds its last frame). Both go through the same frame reader.
2. **Frame transfer**. For each operation the main thread reads one frame and transfers it to the worker along with the operation. The worker copies its pixels directly into a buffer inside wasm memory, closes the frame, then calls Rust.
3. **Locate the window** (Rust). Anchor templates are searched across the whole frame, wherever the game sits in it. A hit fixes the inventory window's origin and gives a brightness estimate. On later frames anchors are only re-checked at their known position.
4. **Detect the page** (Rust). Page-tab templates at fixed offsets from the origin say which inventory page is showing.
5. **Read slots** (Rust). Each slot of the active page sits at a fixed offset. A slot whose pixels have not changed is skipped; otherwise its crop is compared against every icon template, the closest one that passes wins, and the number strip is OCR'd.
6. **Loop**. Each result immediately triggers the next frame.

Full-frame search is the expensive step and uses FFT cross-correlation. Everything after the anchor is a cheap fixed-position comparison.

## State between frames

- A `ScannerState` struct is serialised out to JS and back in on every call. JS holds it between frames, including the buffer pointer as a plain number.
- Heavier data stays in Rust statics for the life of the worker's wasm instance: the base templates, a cache of templates scaled to the current resolution, and the OCR engine. These are loaded once by the reserve operation and deliberately do not round-trip.
- Consequently the scanner worker must not be restarted between frames, unlike the optimizer worker. The buffer pointer and the statics are only valid in the instance that created them.

## Who owns the stream

- The capture stream and its frame reader live in one "frame source" object on the main thread, stored in the roster config next to the scanner worker bundle. It is shared by all characters and never persisted. Because it carries the user's screen-share permission, it outlives the capture component: leaving the scanner page only pauses the loop, and coming back resumes it without asking again.
- Each frame is transferred to the worker with its operation, so exactly one side owns it and the worker closes it. The older design transferred the whole stream to the worker instead; it was changed for stream ownership, not because it leaked.
- **Never `console.log` the scanner state, or anything holding it, per frame.** Once an inventory is recognised the state is tens of MB (every slot carries raw icon images). Firefox keeps recently logged objects alive even with DevTools closed, so logging the state leaked several GB per minute on screen share. Chrome only does this with DevTools or an automation client attached. The worker and worker bundle log only the operation name for scanner operations.
- Stopping capture terminates the worker, which frees the frame buffer and the Rust statics together. The next capture reserves again.
- Changing character clears the scanner state and terminates the worker but keeps the stream; the capture component then reserves a fresh state.
- A worker reply is delivered to whichever callback was registered last, so the scan loop ignores replies it has already handled or that belong to a stopped loop.

## Conventions

- **Reference resolution is 1440p.** Slot positions, anchor offsets and window rectangles are constants in 1440p pixels, scaled by the game's UI height. Positions are kept as floats relative to the window origin; nothing is normalised to 0..1.
- **The game resolution is chosen by the user, never detected.** The capture can be a whole screen with the game windowed inside it, so its size only seeds a guess: once a capture starts, the capture card shows a resolution list with the closest entry preselected, and a "Forced 21:9" checkbox. An ultrawide capture gets the 21:9 list and the checkbox locked on. The choice is shared module state, not persisted, and is re-guessed on every new capture.
- **UI height** is the height of the 16:9 (or 21:9) area the game renders into: the smaller of the game height and width x 9/16 (or 9/21). The scale factor is that height, rounded, over 1440. Rust derives both on every scan call from the three values the frontend sends.
- **Changing the resolution restarts the scanner** like a character change does (worker terminated, state cleared, reserve again), because cached templates and remembered slots are scale-specific. The results table keeps its old rows until the first new result.
- **Pixels are RGBA** throughout, on both sides of the boundary.
- **Fixed-position comparison is a colour distance, not a structural score.** It is the mean absolute RGB difference, taken as the best of the nine one-pixel shifts, with one pass limit shared by anchors, page tabs and slots. Structural similarity was dropped: a half-pixel grid error was enough to fail it, and it scored colour variants of one icon (Blue and Serca Blue) almost alike. Because variants can both pass, slots take the best icon, never the first.
- **Slot pitch was measured on a 1440p capture** (69.75 px). A pitch error accumulates across the grid, so re-measure against a native capture rather than nudging it by eye.
- **Fixed-position comparison needs identical dimensions.** The template path and the crop path must round scaled sizes the same way, or comparisons quietly return no match.
- **The frame buffer is sized once**, at capture start, and freed when the worker is terminated. The explicit dealloc operation still exists but the frontend no longer calls it. JS must rebuild its view of wasm memory every frame, since memory growth invalidates old views.
- **Brightness is normalised.** The in-game brightness setting changes pixel values, so observed crops are mapped to a fixed reference brightness before comparison. Stored templates are already normalised. See `Config and calibration.md`.
- **Capture stays at native resolution.** Downscaling was tried and removed because the quantity digits need full resolution.
- **OCR** uses the `ocrs` / `rten` crates with only the recognition model, on a preprocessed single line. Text detection is skipped because the number's position is known.

## Status

Working: capture, frame transfer, character-inventory anchor detection, brightness estimation, slot icon identification, quantity OCR, and a debug UI that shows what was recognised.

Not done yet:

- Results are not written into the calculator's material inputs. Output stops at the debug tables.
- Only 1080p and 1440p have been tested on native captures. Other scales were tested on resampled 1440p images: icons are recognised, but the quantity OCR degrades as the scale drops.
- Quantity OCR still misreads or returns nothing for some slots at every scale.
- A crop that falls outside the frame (inventory window partly off the capture) still panics.
- Only the character inventory has anchors. Storage and roster storage are placeholders.
- Tooltip detection and bound-versus-tradable detection are not implemented.

Demo-only: `DemoOCR.vue` and `DemoColorfilter.vue` are earlier proofs of concept, still mounted on the scanner page. `tesseract-wasm` and `eng.traineddata` are used only by the OCR demo.
