# Driving the screen scanner

Pages: `/<character>/scanner` (scan loop and debug tables) and `/<character>/setup` (icon setup, no scan loop). Both mount the same capture card.

## Feeding it frames

Screen share cannot be driven headless, so upload a file instead. The capture card has a file input named `Upload image or video`; it is hidden, so set files on it directly rather than clicking it.

```js
export default async ({ page, hf }) => {
  await hf.open(page, "/Newchar/scanner");
  await page.getByLabel("Upload image or video").setInputFiles(file);
  await page.getByText("● Live").waitFor();
  await page.locator("tbody tr").nth(20).waitFor({ timeout: 30_000 });
};
```

- An image becomes a stream repeating that frame. A video plays once, then repeats its last frame.
- Works headless and headed, in Chromium and Firefox, for PNG and H.264 MP4.
- Any file size works. After upload a `Game resolution` select and a `Forced 21:9` checkbox appear, preselected from the file size; if the game in the file is smaller than the file (windowed, padded), pick its real resolution or nothing is recognised.
- Ready-made inputs: `scripts/brightness/Recording 1080p.mp4` and `scripts/brightness/inputs/*.png` at 1080p; in `scripts/brightness/1440p raw`, `inventory top left.png` and `hover tooltip.png` show a character inventory with items, the `storage ...` files show the storage layout, and `21 by 9 storage.png` is storage at forced 21:9 (tick the checkbox, keep 2560x1440).
- The results table is not cleared by `Stop` or a new upload. Reload the page between files, or stale rows will be counted.
- The first frame is slow (full-frame anchor search). Slot rows appear in the second table once the inventory is found; a full inventory page gives a little over 100 rows.
- Chests read from tooltips are listed in a third table (header `Title read`), one row per chest. An uploaded recording is scanned about 0.7 times a second headless, so only hovers that last about a second are seen.
- `Stop` ends capture. `Share screen` is the real screen-share path.

## State you can read

- The badge next to "Screen Capture" reads `● Live` or `Idle`.
- The button under the card reads `stop cropper` while the scan loop runs.
- Leaving the scanner page pauses the loop but keeps the stream; returning shows `● Live` again without a new upload.

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
- The scanner state is only large once an inventory is recognised, and operations are also much faster then, so always measure with an inventory on screen.
- Chrome warns in the console when a video frame is garbage collected without being closed. Counting that warning is a direct test for unclosed frames.
- Keep the browser window unobscured. The Firefox loop was seen stalling for tens of seconds in headed runs, most likely when its window was covered.
- A steady sawtooth a few hundred MB above idle while scanning is normal in both browsers (the state is copied across the worker boundary every frame) and is released on `Stop`.
