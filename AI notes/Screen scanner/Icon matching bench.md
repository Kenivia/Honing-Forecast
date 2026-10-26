# Icon matching bench

An offline test of how a slot is told to be a template's icon ("close enough", `Pipeline.md`, "Slots"), on the debug captures. What it found on 2026-10-11, which is where the decoys and the slot limit of 0.935 came from, is in `Icon matching bench tried.md`; read it only when trying a new way of matching.

## The tools

1. `cargo run --release --bin slot_dump -- <file.hfcap> target/slot-bench/<name>` replays a capture and writes every distinct look of every slot: the normalised icon crop the scanner compares (`_i`), the same with 3 px around it (`_m`), the whole slot as captured (`_r`), and `index.jsonl` with the scans each look was seen on and how many of those had a tooltip over or beside the slot. It also writes the end record and the templates at that UI height. Captures were dumped as `a-<time>` (the four annotated ones), `b-<time>` (the older ones) and `c-1080`, `c-1440`, `c-2160` (one inventory at three resolutions, `ignored/recordings/new captures`).
2. `scanner_setup/scanner/icon_data.py` labels the slots from the end record and picks each slot's clean look.
3. `scanner_setup/scanner/icon_bench.py [--brief] [--lossy] [method ...]` scores comparison methods, each without and with the decoys (the `~` templates `slot_dump` wrote). Its docstring says what it prints. Scores are cached per method in `target/slot-bench/scores`. `target/` is not watched by vite; the dumps are hundreds of thousands of files.

- **A slot's hash has to be of the icon area alone.** The frame around a slot lets the game world through, so with a few pixels of margin every frame is a new look.
- **The clean look** is the one the other tooltip-free looks are closest to, weighted by scans. The look seen longest is often one under a tooltip that was not detected for a few scans.
- **The two 1440p captures (`b-19-22`, `b-22-31`) come from a compressed recording** and are left out unless `--lossy` is given: a screen share is lossless.
- **Labels were checked against the game's art** (`find_icon.py`, rarity by the bare background), and the corrections are in `LABELS` in `icon_data.py`. The `c-*` labels are the scanner's own, not checked by hand, so they say whether something changes, not whether it was right.
- `find_icon.py` cannot be trusted on a slot with an item level across it (scores of 40 and more).
- The bench scores templates made in Python, which score true matches 0.003 higher than the scanner's own.

## Not done

- The bench scores the 60 nearest templates of all, not the scanner's two shortlists of 16, and only clean looks: nothing under the hover highlight. Under it a rare chest passes as its own art on ancient, which the scanner undoes on the next clean look (`Pipeline.md`, "Slots").
- An item level across a decoy chest: the decoy is compared with the same parts left out as the icon (`Pipeline.md`, "Overlays"). No capture has such a slot next to an icon of ours that it looks like, so this is untested.
