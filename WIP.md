
# Work in Progress

## Arena WIP

### Now

- add upgrade name to states when saving them, add some way to keep track of where bits are to make neighbor potentially more efficient
- limit neighbor function - 1 click 10 taps, limit amt of toggles maybe
- set up books support so state will be~~ integers ~~instead of bools ( need to add more columns to test cases, book count storage etc)
  - need to use lists of bools instead, integers doesn't quite work i think because need to keep track of costs
- modify the current too-many-juice checker to accomodate books

### Big

- implement average-optimizing evaluation(switch to this when prob = 100 also), add field to result json etc
- set up slurm on laptop
- data analysis
- how to evaluate adaptive policies???
- Also need to do special honing and even advanced honing eventually maybe

### Energy

- use newtons methods until 0 derivative or something, this bisection kinda slow
- Make a ks_012 and only call ks_01234 function when needed
- implement fft or something for medium sized complexity because lr kinda very bad(also use (and optimize) exact convolution for single piece(maybe 2))
- monte carlo at the end to verify / sanity check / just to look at it (also can compute confidence interval with variance etc)

### Algorithm ideas

- stopping early if no improvements
- start with VERY broad neighbors - all true, all false etc, then refine until limit reached (such as 10 taps), test how much the limit affects performance

## Next big step(s)

- Juice(and Book / Scroll) purchase suggestion
- Books & scroll calculations
- True juice optimization for success chance

^ this is arena

## Other features

- Automatic Market price integration(via some kind of API, or just updating the site at a regular interval automatically)
- Allow selling mats
- Raw gold graph and overall gold(including used in buying mats) lines in the
graph
- juce chest openinig optimization

## UI

- Better roster tracking & income estimation(I don't think automatic game-to-website input is possible, but something like a generic 1680 income would be nice, need information/data on this)
- Input arithmetic parsing(e.g. allow inputs like 25*1234 for easier boxes calculation)
- Ctrl z, delete
- Hover question mark tooltips for various systems, but still aim to be intuitive
- Adjustable week number in raw / Gold graph
- Achieved ilevel
- Something seems to be broken in drag-to-select spreadsheetgrid
- Artisan level editing

## DEV

- MUCH more tests, specifically:
- Test countfailure non-naive version actually works
- Test average tap and ~~truncated average tap~~
- Test Tap map generator(and improve it maybe)

- Matrix operation libraries to speed up monte carlo and what not?
- Graph is off by 0.5 all the time, and the points of interest snap the the one below(i tihnk?) which isnt quite right idk, kinda scared to touch it
- better input cleaning for spreadsheetgrids
- make get_one_tap_pity take in seeded rng
- cache using array buffer? dont know if it's worth the effort
- fix the react stuff, i think there's way too much usememo(the website feels sluggish)
- improve how the cost estimation works / verify that it actually works

## Done / cancelled

- ~~fix either the neighbor function or the prob_to_maximize function to force use owned juice/punish unused juice~~
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
- ~~maybe go back to caching results?DEFINITELY caching the monte carlo + orcale bitsets~~
- ~~Budget needed estimation -> merge chance & budget mode~~
