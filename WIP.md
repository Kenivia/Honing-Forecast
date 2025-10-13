
# Work in Progress

## Next big step(s)

- ~~True juice usage optimization(maximize chance of success)~~ might not be possible, at least with beam search because evaluating the effect is way too hard but WE WILL TRY
- ~~option for Bottleneck-aware juice/free tap gold value estimation~~ but the gold value will depend on how we've used it... idk if this is possible
- Allow beamsearch to go below min indices(allow selling materials)

## Other features

- Books & scroll calculations
- Automatic Market price integration(via some kind of API, or just updating the site at a regular interval automatically)
- Juice box opening optimization(do i open for red or blue)
- Confidence intervals(might have to actually learn some math first)

## UI

- Better roster tracking & income estimation(I don't think automatic game-to-website input is possible, but something like a generic 1680 income would be nice, need information/data on this)
- Input arithmetic parsing(e.g. allow inputs like 25*1234 for easier boxes calculation)
- Ctrl z, delete
- Hover question mark tooltips for various systems, but still aim to be intuitive
- Adjustable week number in raw / Gold graph

## DEV

- MUCH more tests, need to actually test the components(like count_failure, tap_map and such), also need implement more integration tests
- ~~Pre-compute prob_dist of groups of upgrades(such as +n all 5 armours etc)~~ not possible with juice being a thing i think
- Matrix operation libraries to speed up monte carlo and what not?
- ~~maybe go back to caching results?~~ DEFINITELY caching the monte carlo + orcale bitsets
- Graph is off by 0.5 all the time, and the points of interest snap the the one below(i tihnk?) which isnt quite right idk, kinda scared to touch it
- Improve beam search algorithm.
- Sort out input parsing (clamping) for desired_chance, prolly make a new variable
- better input cleaning for spreadsheetgrids
