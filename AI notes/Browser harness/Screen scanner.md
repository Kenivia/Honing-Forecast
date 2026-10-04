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
- **The file must be exactly 1920x1080.** Other sizes make Rust panic and the loop stops with a page error. `scripts/brightness/Recording 1080p.mp4` is a ready-made video (the original recording padded with a 1 px black border). Most PNGs in that folder are other sizes.
- The first frame is slow (full-frame anchor search). Slot rows appear in the second table once the inventory is found; a full inventory page gives a little over 100 rows.
- `Stop` ends capture. `Share screen` is the real screen-share path.

## State you can read

- The badge next to "Screen Capture" reads `● Live` or `Idle`.
- The button under the card reads `stop cropper` while the scan loop runs.
- Leaving the scanner page pauses the loop but keeps the stream; returning shows `● Live` again without a new upload.

## Real screen share, headed only

For checks that need the real capture path, launch headed and skip the picker:

- Chrome: `--auto-select-desktop-capture-source=Entire screen`, and `viewport: null`. With an emulated viewport the capture is scaled to the viewport size and Rust panics on it.
- Firefox: the user pref `media.navigator.permission.disabled: true`. The auto-accept is flaky; check for `● Live` and retry.

The game has to be on screen at 1920x1080 for anything to be recognised.

## Measuring memory

- Sample the browser's processes from outside (private bytes per process, plus system available memory). The page's own heap figure misses worker and frame memory.
- An attached automation client keeps every logged object alive, so any per-frame log of a large object shows up as a leak under Playwright even if a normal tab is fine. To measure what a user sees, start capture and then disconnect: Chrome via a remote debugging port and `connectOverCDP`, installed Firefox via `-remote-debugging-port` and a WebDriver BiDi session that is ended after the click.
- Playwright's Firefox is not the installed Firefox. A leak that reproduced only on the installed build was missed for a long time by testing the patched one.
- A full-screen static image produces no new capture frames, so the loop stops. Put something moving on screen (a blinking corner square is enough).
- The scanner state is only large once an inventory is recognised, and operations are also much faster then, so always measure with an inventory on screen.
- Chrome warns in the console when a video frame is garbage collected without being closed. Counting that warning is a direct test for unclosed frames.
- Keep the browser window unobscured. The Firefox loop was seen stalling for tens of seconds in headed runs, most likely when its window was covered.
- A steady sawtooth a few hundred MB above idle while scanning is normal in both browsers (the state is copied across the worker boundary every frame) and is released on `Stop`.
