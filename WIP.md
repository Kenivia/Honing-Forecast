
# Work in Progress

## Next big step(s)

- ~~True juice usage optimization(maximize chance of success)~~ might not be possible, at least with beam search because evaluating the effect is way too hard but WE WILL TRY
- Books & scroll calculations

## Other features

- Automatic Market price integration(via some kind of API, or just updating the site at a regular interval automatically)
- Confidence intervals(might have to actually learn some math first)

## UI

- Better roster tracking & income estimation(I don't think automatic game-to-website input is possible, but something like a generic 1680 income would be nice, need information/data on this)
- Input arithmetic parsing(e.g. allow inputs like 25*1234 for easier boxes calculation)
- Ctrl z, delete
- Hover question mark tooltips for various systems, but still aim to be intuitive
- Adjustable week number in raw / Gold graph

## DEV

- MUCH more tests, specifically:
- Test countfailure non-naive version actually works
- Test average tap and ~~truncated average tap~~
- Test Tap map generator(and improve it maybe)
- Matrix operation libraries to speed up monte carlo and what not?
- ~~maybe go back to caching results?~~ DEFINITELY caching ~~the monte carlo~~ + orcale bitsets
- Graph is off by 0.5 all the time, and the points of interest snap the the one below(i tihnk?) which isnt quite right idk, kinda scared to touch it
- better input cleaning for spreadsheetgrids

## Done / cancelled

- ~~Pre-compute prob_dist of groups of upgrades(such as +n all 5 armours etc)~~ not possible with juice being a thing i think
- ~~make longtermresult debounce use the universal function~~
- ~~Allow beamsearch to go below min indices(allow selling materials)~~
- ~~Improve beam search algorithm.~~
- ~~Sort out input parsing (clamping) for desired_chance, prolly make a new variable~~
- ~~Test beamsearch actually finding the best index(or close enough)~~
- ~~Better juice usage recommendation and value estimation to answer - Should I buy juice? How should I use juice~~
- ~~Test bitset success count evaluations~~
- ~~Optimize how chance_to_chance works~~
- ~~Juice box opening optimization(do i open for red or blue)~~
