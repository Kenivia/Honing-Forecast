# Scanner pipeline

The scanner reads the player's material counts from a shared game screen, so they do not have to type them in. It is in active development on the `Inventory-scanner` branch. `public/WIP.md` has the author's current task list.

Code: `crates/scanner` (image processing), `frontend/Components/Character/InventoryScanner` (capture and debug UI), plus scanner operations in the shared wasm worker.

## Stages

1. **Capture** (TypeScript, main thread). The browser's screen-share API gives a video track, read as a stream of frames at a low frame rate. The stream is transferred to the worker.
2. **Frame transfer** (worker). For each operation the worker pulls one frame and copies its pixels directly into a buffer inside wasm memory, then calls Rust.
3. **Locate the window** (Rust). Anchor templates are searched across the whole frame. A hit fixes the inventory window's origin and gives a brightness estimate. On later frames anchors are only re-checked at their known position.
4. **Detect the page** (Rust). Page-tab templates at fixed offsets from the origin say which inventory page is showing.
5. **Read slots** (Rust). Each slot of the active page sits at a fixed offset. A slot whose pixels have not changed is skipped; otherwise its crop is compared against the icon templates, and on a match the number strip is OCR'd.
6. **Loop**. Each result immediately triggers the next frame.

Full-frame search is the expensive step and uses FFT cross-correlation. Everything after the anchor is a cheap fixed-position comparison.

## State between frames

- A `ScannerState` struct is serialised out to JS and back in on every call. JS holds it between frames, including the buffer pointer as a plain number.
- Heavier data stays in Rust statics for the life of the worker's wasm instance: the base templates, a cache of templates scaled to the current resolution, and the OCR engine. These are loaded once by the reserve operation and deliberately do not round-trip.
- Consequently the scanner worker must not be restarted between frames, unlike the optimizer worker. The buffer pointer and the statics are only valid in the instance that created them.

## Conventions

- **Reference resolution is 1440p.** Slot positions, anchor offsets and window rectangles are constants in 1440p pixels, scaled by the capture height. Positions are kept as floats relative to the window origin; nothing is normalised to 0..1.
- **Pixels are RGBA** throughout, on both sides of the boundary.
- **Fixed-position comparison needs identical dimensions.** The template path and the crop path must round scaled sizes the same way, or comparisons quietly return no match.
- **The frame buffer is sized once**, at capture start, and freed by an explicit dealloc operation. JS must rebuild its view of wasm memory every frame, since memory growth invalidates old views.
- **Brightness is normalised.** The in-game brightness setting changes pixel values, so observed crops are mapped to a fixed reference brightness before comparison. Stored templates are already normalised. See `Config and calibration.md`.
- **Capture stays at native resolution.** Downscaling was tried and removed because the quantity digits need full resolution.
- **OCR** uses the `ocrs` / `rten` crates with only the recognition model, on a preprocessed single line. Text detection is skipped because the number's position is known.

## Status

Working: capture, frame transfer, character-inventory anchor detection, brightness estimation, slot icon identification, quantity OCR, and a debug UI that shows what was recognised.

Not done yet:

- Results are not written into the calculator's material inputs. Output stops at the debug tables.
- Resolution and aspect-ratio detection is a stub with one hardcoded resolution.
- Only the character inventory has anchors. Storage and roster storage are placeholders.
- Tooltip detection and bound-versus-tradable detection are not implemented.

Demo-only: `DemoOCR.vue` and `DemoColorfilter.vue` are earlier proofs of concept, still mounted on the scanner page. `tesseract-wasm` and `eng.traineddata` are used only by the OCR demo.
