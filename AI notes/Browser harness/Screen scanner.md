# Driving the screen scanner

Pages: `/<character>/scanner` (scan loop and slot grid) and `/<character>/setup` (icon setup, no scan loop). Both mount the same capture card.

## Feeding it frames

Screen share cannot be driven headless, so upload a file instead. The capture card has a file input named `Upload image or video`; it is hidden, so set files on it directly rather than clicking it.

```js
export default async ({ page, hf }) => {
  await hf.open(page, "/Newchar/scanner");
  await page.getByLabel("Upload image or video").setInputFiles(file);
  await page.getByText("● Live").waitFor();
  await page.locator(".grid canvas").nth(20).waitFor({ timeout: 30_000 });
};
```

- An image becomes a stream repeating that frame. A video plays once, drawn frame by frame, then repeats its last frame twice a second.
- **Headless Chromium does not draw every frame of a video playing at full speed**: under load frames get drawn twice and others skipped (a third of them in one run). To feed the scanner every frame, play the upload at half speed: before opening the page, wrap `HTMLMediaElement.prototype.play` in an init script so it sets `playbackRate = 0.5` on blob sources. Count what was drawn by wrapping `CanvasRenderingContext2D.prototype.drawImage`.
- Works headless and headed, in Chromium and Firefox, for PNG and H.264 MP4.
- Any file size works. After upload a `Game resolution` select and a `Forced 21:9` checkbox appear, preselected from the file size; if the game in the file is smaller than the file (windowed, padded), pick its real resolution or nothing is recognised.
- Ready-made inputs: `scripts/brightness/Recording 1080p.mp4` and `scripts/brightness/inputs/*.png` (lone inventory) and `scripts/brightness/inputs storage/*.png` (storage layout with a chest tooltip), both 1080p at every brightness setting; in `scripts/brightness/1080p raw`, recordings at settings 60, 0 and 100; in `scripts/brightness/1440p raw`, `inventory top left.png` and `hover tooltip.png` show a character inventory with items, the `storage ...` files show the storage layout, and `21 by 9 storage.png` is storage at forced 21:9 (tick the checkbox, keep 2560x1440).
- `scripts/brightness/1440p raw/2026-10-08 21-31-53.mp4` and `21-42-32.mp4` are a native 3440x1440 game at 60 frames a second, the pet-menu storage with many chests hovered. The upload preselects 3440x1440 with the checkbox on, which is right. Headless Chromium plays them at half speed without trouble (the shorter one took two and a half minutes).
- The three files named `2026-10-06 ...` are the near-lossless recordings (see below), all at brightness 60: one 2560x1440 in `1440p raw`, and two 3840x2160, one in `2160p raw` with a slower hover and one **misfiled in `1080p raw`** although it is 4K. The older `Recording ...` files in `1080p raw` are the lossy hardware-encoded ones.
- The grid keeps its slots after `Stop` and stays editable; starting the next capture wipes it, and the capture card says so while idle.
- The first frame is slow (full-frame anchor search). Slots get a canvas once their page has been looked at, about a second and a half for a full page; until then they show `?`. A slot's status is its border colour (`style.borderColor` holds the CSS variable, see `Slot grid.md`).
- Chests read from tooltips show in the dashboard under the grid when a slot with that chest's icon is hovered or clicked. The dashboard's inputs are labelled `Item`, `Amount` and `Tradability` (the last two only while an item is chosen; match `Tradability` exactly, the manifest has inputs ending in it), with buttons `Save edit` (disabled until amount and tradability are filled), `Retry` and `Okay`. Every frame of an uploaded recording is scanned. Texts arrive later than the frame they were cut from, so wait for the OCR queue to empty (`window.__ocr_timings` stops growing) before reading the grid.
- `Stop` ends capture. `Share screen` is the real screen-share path.
- The manifest sits under the dashboard. While slots are missing it is a warning with a `Show the manifest without them` button and a list labelled `Missing slots`. Its table is the group `Manifest materials`, with a group per material and inputs named `<label> Char bound` / `Roster bound` / `Tradable`; chest counts are `<title> count`; the card for adding a chest has `New chest count`, `New chest name`, `New chest ownership`, `Option <n> material` and `Option <n> amount`, with buttons `Add option`, `Add chest` (disabled until one option has a material and an amount) and `Remove`; the tier tabs are buttons named after `TIER_LABELS`.
- To test the manifest without a capture, set `slots` and `chests` in `ScanStore.ts` from `page.evaluate`. Import the module by the URL the app loaded it from (look it up in `performance.getEntriesByType("resource")`); a hot-reloaded file has a `?t=` query, and the bare path gives a second, unused instance.
- **A Vue warning on every slot of every grid update kills the run.** Playwright keeps logged objects alive, so the headless browser grew past 4 GB and the page stopped answering within seconds (an `active` prop given null instead of false did this). If the page hangs right after capture starts, count console warnings first.

## Recording test inputs

A recording stands in for a live screen share, so it should carry the same losses and no others. The share is **not** lossless: Chromium's desktop capturer hands over ARGB and the capture client runs it through libyuv `ConvertToI420`, so the frames reaching `copyTo(dest, { format: "RGBA" })` are 4:2:0 chroma-subsampled but otherwise uncompressed. Firefox has no `MediaStreamTrackProcessor`, so there the frames come off a `<video>` instead, the same way an upload does.

OBS settings that match, used for the `2026-10-06 ...` recordings:

| Where | Setting |
| --- | --- |
| Video | Base = Output, no scaling; 30 fps (matches `CAPTURE_FPS`) |
| Advanced | Color Format NV12, Color Space 709, Color Range **Limited** |
| Output (advanced) | x264, rate control CRF, **CRF 1**, preset veryfast, profile high, mp4 |
| Source | Window or Display Capture, method Windows 10 (WGC), cursor on, transform reset |

- **CRF 1, not 0.** Lossless H.264 needs `qpprime_y_zero_transform_bypass_flag`, which exists only in the High 4:4:4 Predictive profile, so x264 stamps profile 244 on the file even when the pixels are 4:2:0. Firefox then refuses it outright (`media error 3`, "Decoder may not have the capability to handle the requested video format with YUV444 chroma subsampling") and VLC's hardware path mangles the colours. CRF 1 stays in High profile, decodes in both browsers, and its quantisation is far under the 4:2:0 loss that is being reproduced anyway. The profile is in the SPS, so a bad file has to be re-recorded, not remuxed.
- **Limited range, not full.** OBS's mp4 does tag `colr nclx` 709/limited, and the browser agrees with it; full range with a lost tag would crush blacks and skew the brightness reading.
- The older `Recording ...` files came from a hardware encoder at Main profile with no colour tags at all (the browser falls back to 709 limited, which happens to match). Their extra lossy compression is what eats the yellow out of a thin last digit in `Tooltips.md`.
- `python scripts/brightness/probe_capture.py "<file.mp4>"` prints resolution, fps, profile, chroma, range, colour tags and the x264 options read out of the SEI, and flags profile 244, non-4:2:0, full range and a frame rate away from 30. A good file reads `profile 100 (High)`, `chroma 4:2:0`, `lossless=0`, `range=limited`, `crf=1.0`.

## State you can read

- The badge next to "Screen Capture" reads `● Live` or `Idle`.
- While capturing, the element labelled `Frames scanned per second` reads the frames handed to the scanner in the last second, as `22 fps scanned`.
- The button under the card reads `stop cropper` while the scan loop runs.
- Leaving the scanner page pauses the loop but keeps the stream; returning shows `● Live` again without a new upload.

## Measuring scan time

`pnpm scanner-profile [--firefox] [--keep] [recording...]` does the whole thing: it builds the wasm and starts vite if nothing answers on :5173, plays each recording through the scanner page, writes a dump per recording to `target/scan-profiles/`, and prints the stage breakdown. With no recording it does the two `2026-10-07` ones in `scripts/brightness/1080p raw`; the 3440x1440 ones have to be named. `--keep` leaves the dev server running. `scripts/browser/scan_summary.mjs <dump>...` re-prints a dump, so a run can be re-read without replaying anything.

- The browse script empties the page's three arrays every two seconds and keeps the records itself, since the page only holds the last 5000 and the longer 1440p recording makes 7,600.
- One recording takes twice its length and a little more: it plays at half speed (headless Chromium skips frames at full speed) and then waits for the OCR queue to drain.
- **Do not edit any project file while a run is in flight.** Vite reloads the page, which empties the timing arrays; a dump with 0 scans is that. A `cargo build` alongside does the same another way: the upload is still loading when the script starts counting frames, and it sees none.
- The scripts are `profile_scanner.mjs` (the wrapper), `profile_scanner_browse.mjs` (the browse script) and `scan_summary.mjs`.

The dumps are the three arrays the page keeps, all in ms:

`window.__scan_timings` holds one record per scan (last 5000): `read` (waiting for a frame), `to_worker`, `copy`, `wasm_call`, `to_main`, `process_result` (merging the result, not rendering), `total`, and `rust`, a list of `[name, total, calls]` for the stages inside the wasm call. `frame_time` is the frame's timestamp in microseconds. A scan saw a tooltip if `rust` has `tooltip/layout`, a chest if it has `tooltip/chest`. A frame that was not scanned because too little changed has no `anchors` (it still has `changed`, `apply_ocr` and `to_value`). Skip the first record, which is the whole-frame anchor search, and the records after the video ended, which are its last frame repeating twice a second and show up as a `frame_time` gap of half a second. `window.__scan_renders` has the duration of each grid update. `window.__ocr_timings` has one record per OCR batch: `jobs`, `width` (px of strip), `took`, `waited` (how long its first reads took from being queued) and `left` (lines still queued); a batch with no first read in it records `waited: 0`, so those have to be dropped before taking a median. The scanner's own `rust` list no longer has any OCR in it.

## Real screen share, headed only

For checks that need the real capture path, launch headed and skip the picker:

- Chrome: `--auto-select-desktop-capture-source=Entire screen`, and `viewport: null`. With an emulated viewport the capture is scaled to the viewport size, so the game no longer matches any selectable resolution.
- Firefox: the user pref `media.navigator.permission.disabled: true`. The auto-accept is flaky; check for `● Live` and retry.

The game has to be on screen, with its resolution selected, for anything to be recognised.

## Measuring memory

- Sample the browser's processes from outside (private bytes per process, plus system available memory). The page's own heap figure misses worker and frame memory.
- An attached automation client keeps every logged object alive, so any per-frame log of a large object shows up as a leak under Playwright even if a normal tab is fine. To measure what a user sees, start capture and then disconnect: Chrome via a remote debugging port and `connectOverCDP`, installed Firefox via `-remote-debugging-port` and a WebDriver BiDi session that is ended after the click.
- Playwright's Firefox is not the installed Firefox. A leak that reproduced only on the installed build was missed for a long time by testing the patched one.
- A full-screen static image produces no new capture frames, so the loop stops. Put something moving on screen (a blinking corner square is enough).
- Results only carry images once an inventory is recognised, and operations are also much faster then, so always measure with an inventory on screen.
- Chrome warns in the console when a video frame is garbage collected without being closed. Counting that warning is a direct test for unclosed frames.
- Keep the browser window unobscured. The Firefox loop was seen stalling for tens of seconds in headed runs, most likely when its window was covered.
- The few-hundred-MB sawtooth seen while scanning came from copying the state across the worker boundary every frame. That no longer happens; memory has not been re-measured since.
