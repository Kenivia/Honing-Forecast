# UI components

Shared pieces under `frontend/Components/Common/`.

## HoverTip

Every hover tooltip on the site. It copies the look of Wikipedia's page previews, using the site's own colours: a `--bg-muted` card with a 1px `--border-muted` ring, Wikipedia's soft drop shadow, corners rounded 2px, 14px text with 16px padding, an arrow on the card edge, and short animations (67ms): it fades in without moving, and fades out while sliding 20px away from the hovered element.

- Usage: wrap the hovered thing in the default slot (a question mark, a number, a button). The content is `text`, or the `#tip` slot for anything else (markup, images, buttons).
- `sticky` (default off): the card stays open while the cursor is on it (150ms grace to cross the gap), so it can hold buttons. Off, it has `pointer-events: none` and closes as soon as the cursor leaves. The question marks are informational and not sticky.
- `placement` (default `bottom`) is the preferred side. `@floating-ui/vue` flips to the opposite side, then below/above, when it does not fit, and shifts the card to stay on screen. The card starts 10px before the element's start edge, Wikipedia-style, and the arrow points at the element's centre.
- Opens after 50ms (Wikipedia waits about 500ms). `maxWidth` caps the card (default 400px); below that it fits its content.
- The wrapper is an inline `<span>` and is what gets measured. In a flex layout, give it a class so it sizes like the element it wraps (`TierConvertButton` passes `flex flex-col` so the button still stretches and the arrow points at it).
- `QuestionMark` is a `HoverTip` with class `question-mark`. `position="none"` means below.
- Tests click a wrapped button through `.find("button")`, since the component root is the wrapper.
- In the Playwright MCP browser, animation frames can be throttled to about one a second, so the card can look stuck mid-animation. Wait a couple of seconds before measuring or taking a screenshot.
