# Constants

The constants (honing and event parameters) are read from a json file, which is generated from a google sheet via a simple script.

The sheet can be found here: <https://docs.google.com/spreadsheets/d/1UWJ5TCNZ2kIZxXQwR839c-P1rlivkwdW3fOCMDMXZZo/edit?usp=sharing>

To add / update constants:

1. Make the changes to the spreadsheet
2. Export it via the [appscript](/scripts/google_sheet_export_script.gs)
3. Copy the json into [constants](/crates/core/src/constants/). You should [Minify](https://marketplace.visualstudio.com/items?itemName=josee9988.minifyall) the json because it's directly embedded into the binary, not that big of a deal tho its like 15KB.

Note:

1. Currently anything other than 2 tiers is not supported in the frontend (specififcally the tier changing logic). The Rust side should be fine with it tho, just add another `.push` in [constants.rs](/crates/core//src/constants/constants.rs).
2. Currently we do not support having more than 1 book type available. If they add one of those refund books we're cooked. There are a couple things stoppipng it from working:
    - The [optimizer engine](/crates/core/src/optimizer/v35/neighbour.rs), specifically the neighbour function, does not know how to deal with more than 1 type of books.
    - The [instructions UI](/frontend/Components/Character/Instructions/InstructionRow.vue) does not know how to display multiple types of books
    - Both of these currently assume that the index of the book is the last element in `juice_info.normal_uindex_to_id` (or scroll in `adv_uindex_to_id`).

Neither >1 book type nor refund are HUGE problems but like they're kinda annoying so we'll cross that bridge when we get there.
