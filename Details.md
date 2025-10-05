# Details, last updated 2025/10/5

The main functionalities of the calculator are based on random samples. We generate "cost data" consisting of ~100k samples, where each sample is the cost of all selected upgrades. Note that we don't stop for failures in this data generation - we just keep going til everything succeeds and record how much things costed.

We use random samples because I'm pretty sure it's not practical to calculate all the possibilities, though I'd love to be proven wrong.

## Chance mode

To implement chance mode we actually need to start thinking about "budgets". In the end, we want a budget that has roughly x% chance of passing - as in all 7 numbers are bigger than their counterpart in x% of samples. We also want a budget that is a good representative. For example, a budget that assumes you will one tap all your weapons and pity all your armors is not a good representative, even if the odds of that happening matches x%.  

To achieve this, we balance the budgets by percentile - all components of a budget are forced to be in the same percentile (in their own dimension) and as such, all components have the same chance of failing. A side effect of this is that the end result is not a "real" cost that is actually achieved by putting together costs of taps (oh well).

The alternative is to produce a budget that minimizes the equivalent gold cost while also having roughly/at least x% chance of passing. I'm still trying to figure out how to do that...

## Budget mode

Now budget mode is much easier - we just count how many samples are below our budget. The tough part is figuring out how to best use your juice & free taps. Currently we calculate the expected gold value of the juice/free tap and pick the highest valued taps. This is the best strategy if your goal is to preserve your other mats, but it's not the true way to maximize your chance of success (although it also does that pretty well).

Currently if "Custom Gold value" is unticked, it just uses a hard-coded list of gold values. I originally wanted to make that a "Optimize juice & freetap usage" button, as in it optimizes the chance of success rather than the average gold values. This removes the need for an arbitrary gold value list and would be useful if your goal is just to pass the upgrades ASAP(such as on your main). However this is much harder a problem than I thought(if you know how to do it please help).

From a user standpoint, perhaps the hardest part is actually tallying up all the mats in the inventory, especially if you have a messy one. Not sure if anything can be done about that, but perhaps the wonders of ✨Ai✨ will save us. On the other hand market price integration might be possible with something like [loa buddy](https://www.reddit.com/r/lostarkgame/comments/1ly5qjv/loa_buddy_is_now_out_marketdata_calculator_tools/).

## Forecast mode

This section just applies the previous two to the projected budget. The income input is currently completely up to the user. There are better income tracking tools out there, and lot of the time a personal spreadsheet is easier. Maybe in the future we can integrate a tool tho.

## Dev

This project started in Google sheets as a roster income tracker, then I want comparing my budget with the Maxroll average to see when I'd hit it. At some point I realized that hitting the average doesn't actually tell me my chances of seeing my ilevel go up, so I started implementing the Monte Carlo thing in Google Appscript, which is insultingly slow (although my algorithm back then was also an insult to humanity). When I couldn't stand it anymore I started making my own website, ported the heavy compute to Rust and wrote a front end. I've had 0 experience in front end/GUI before, so that was done with a LOT of help from various AI tools.

I'm still in the process of writing proper comments & tests, I apologize for the mess.

## Work in Progress

### Features

- Confidence intervals(might have to actually learn some math first)
- Success chance optimization in juice/free tap usage(right now it just calculates a gold value, want to replace non-custom gold value with true highest success chance).
- Gold eqv cost minimized chance mode
- Automatic Market price integration
- Juice box opening optimization(do i open for red or blue)
- Bottleneck-aware juice/free tap gold value estimation

### UI

- Better roster tracking & income estimation(I don't think automatic game-to-website input is possible)
- Input arithmetic parsing(e.g. allow inputs like 25*1234 for easier boxes calculation)
- Books & scroll calculations(Although for the most part just use them same way as juices)
- Ctrl z, delete
- Hover question mark tooltips
- Adjustable week number in raw / Gold graph

### DEV

- MUCH more tests, need to actually test the components(like count_failure, tap_map and such), also need implement more integration tests
- Pre-compute prob_dist of groups of upgrades(such as +n all 5 armours etc)
- Matrix operation libraries to speed up monte carlo and what not? maybe go back to caching results?

## Contributing

Contributions would be greatly appreciated! If you would like to add this to your own tool, there is no API to speak of (given there's no server) but you're welcome to copy any of code(the rust compute section probably the most useful) as long as you comply with AGPL.
