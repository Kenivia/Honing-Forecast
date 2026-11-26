# How the game draws what the scanner reads

Measurements behind the readers, for when one has to be changed or the game changes how it draws. Sizes are 1080p pixels unless marked 1440p (the scanner's constants are 1440p and scale with the UI). Colours are at the brightness of the 1080p examples, before normalising.

## The UI scale

- The UI scales continuously with the height of the area it renders into. On native stills from 720 to 2160 lines, the fractional part of a matched icon's position is spread flat, the distance between anchors is the same in UI units to within 0.3, and the slot pitch comes out 69.745 against the assumed 69.75 (1440p units).
- The slot pitch was measured on a 1440p capture and holds at every native height. The grid is the same in every window, except that roster storage's starts a little further left. A pitch error adds up across the grid, so re-measure against a native capture rather than nudging by eye.
- The lone inventory first opens a fixed distance from the render area's right edge and top, at either aspect, and can be dragged anywhere inside that area. This was measured at 1440p only.
- Thin icons (the storage windows' move-to icons) are drawn anew at each resolution, and their mean differs by several brightness settings between resolutions.

## Brightness

- **A display gamma**: a value v (0 to 1) seen at setting 50 becomes `v ^ ((62.6 + 50) / (62.6 + setting))` at another setting. That is one number and no gain (`brightness.rs`, `GAMMA_RATIO`). Only the ratio of the line's offset to its slope can be told from captures. Black and white are fixed points.
- **It is applied to what is drawn, before blending.** Pixels that share a value at 50 do not land on one value elsewhere. White glyphs stay white and only the icon under a soft edge changes, which is why a slot number's crop is never normalised. What shows through a nearly opaque panel changes the same way: the tooltip body is brighter than its title bar at setting 0 and darker from 10 up.
- After normalising, a text colour's channel ratios are the same at every setting and on soft edges, to within 0.02. Amount yellow has green at 0.82 of red and blue at 0; description tan 0.89 and 0.54; red 0.39 and 0.37; white and grey 1. How bright a text pixel is depends on glyph coverage (110 to 255, median 150 to 200).
- A curve fitted at 1080p reads a button 2 to 3 settings high at 1440p. The sort button's flat face is level 62 at setting 70 (one level is about 1.5 settings).

## Slot numbers

Measured on the captures and stills; `ocr/shapes.rs` draws numbers by these.

- One sans-serif font at every resolution, opaque white with a soft dark shadow: **ExuberanceF-Regular**. `Binaries/Fonts/English/FontMap.xml` in `font.lpk` maps the UI's text styles to it, and French, German and Spanish do too; Russian and the Asian clients map other files. Digits advance 0.68 em and "+" 0.51 em. The size is 13 px at 1080p, scaled with the UI.
- Right-aligned at a fixed place in the slot: the right edge varies by 0.1 to 0.3 px between slots, the height by 0.1 to 0.4 px.
- Monospaced: the glyphs differ in width (a "1" is half as wide), but every digit takes the same advance, 8.80 px at 1080p and 11.75 px at 1440p.
- The shadow is a near-black (about 8 luma) halo, modelled as the digit blurred by 3 px at 1440p at four times its coverage. Close to a digit it darkens the background by about half to two thirds, and takes a larger share of a bright background than of a dark one (keeps 52% around 31 luma, 42% around 139). The digit itself reads as the font drawn with about 0.35 px of blur.
- A single item shows no number. Over 9999 shows "9999+".
- **Overlays on an icon**: an item level across the lower rows with a helmet (chests that ask for a level), and a stopwatch in the bottom-left corner (items that expire; seen on `ExpireType` 3). A slot with both has not been seen.
- **The hover highlight** is a tan glow like the ancient background. A lit look and a normal one compare at about 0.987.

## A tooltip, top to bottom

- **Title bar**: fully opaque, one flat colour (30, 34, 39), 307 px wide; 34 px tall with a one-line title, 52 px with two. The name is in the rarity colour, then a white `[X n]` that can wrap onto the second line (none for a single item). Above the text is a clean strip of bar colour, 11 rows. Ancient's pale tan is (227, 199, 168) at its fullest.
- **Body**: about 97% opaque over (14, 19, 20), so its pixels sit 0 to 8 levels above that, depending on what is behind.
  - The item icon is 64 x 64 (the native art), 7 px in from the title's left edge and 17 px below the bar, over a rarity gradient. Rarity and "Item Tier n" are to its right.
  - Optionally a right-aligned white line ("Required Item Lv. 1640").
  - Bound items have a white left-aligned "Bound to Character" / "Bound to Roster" line, then a red right-aligned "Untradable". Tradable items have a white right-aligned "Tradable" and no bind line.
  - Description lines are tan (228, 204, 130), with highlighted phrases in pale yellow (255, 255, 172). "Consumed before tradable items." is in the bright amount yellow.
  - "Amount Stacked: n" and "Total Amount Owned: n" have a white label and the number in (255, 213, 0), with a thousands separator. The stacked number starts 169 px (1440p) in, and the total 205 to 213 px in.
  - Red restriction lines, then cyan acquisition sources.
- **Chest contents**: a pure black panel inset 5 px, after the restriction lines. A header with one yellow phrase, then a row per item: a 30 px icon 18 px in from the title's left edge, the name in rarity colour, and a white "xN" under it. The headers are "Obtain [all] of the following items.", "You can [select and obtain 1] of the following items." (white text after it), "There is a [chance that you can obtain one] of the following items." (yellow to the end of the line), and "Obtain one of the following items at random" (one short yellow word, like "all"). At most 10 rows show, and a tooltip reaching the screen bottom has its last row under the "Scroll" hint.
- **Shortcut hints**: a separate, narrower box below, 80% black.
- Roman numerals in the body font are bare strokes and read as anything ("II", "?I", "!", glued to the word before or not).

## Where a tooltip goes

- It sits against the hovered slot's column. Its title starts 8 px right of the slot's right edge, or ends 12 px left of the slot's left edge when it flips left. Its top equals the slot's top unless the screen bottom would cut it, and then it is pushed up and only the column is known. Most are pushed up.
- It fades in over a few frames, during which no colour matches.
- **The cursor** is customisable and sits over the hovered slot. When the tooltip is on the right, the cursor pokes into its left edge, up to about 90 px at the largest size: the left of the title bar, most of the icon when level with the slot, and the start of the bind and amount labels. The hovered slot and its glow (a brighter inner ring about 6 px wide) are usually covered.
- Gold's tooltip sits at a fixed offset from the coin it was hovered from. Flipped left, its right edge follows the owned amount's length, and it is a line shorter when the amount fits beside "Gold Owned:" (`currency.rs`).

## The HUD bar and the currency window

- The bar is drawn at HUD Size (80 to 110%) from the render area's top left, and is hidden while the storage layout is open. The user picks which currencies it shows and in what order. At forced 21:9 on a 1080p screen (an 810-line UI) at 80%, its coins are about 10 px.
- The currency window is see-through and can be moved and its sections collapsed.
- The small coins drawn in the inventory, the window and the bar are not in the icon atlases (probably in UI packages `extract.py` does not pull), so their templates were cut from stills.
