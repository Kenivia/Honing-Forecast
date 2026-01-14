
# Work in Progress

## Arena WIP

### Now

- streameline solve such that it takes in an initial state instead of relying on initializing in there

#### Optimizations

- ~is there really no better way to do the special probs(can i get away with an approximation some how)~
  - cache the geom calculations
  - ~~take advantage of the fact that the special costs share a common factor~~
  - ~~somehow avoid re-allocating maybe~~
  -~~ skip small mass?~~
  - ~~definitely cache this result~~

- ~~profile the things~~
- cache lattice span

- use polynomial for equal stepped stuff like for mat costs in averages
  - add a tag to the support or just do it via support index
- collapse identical supports

- ^ both of these will probabily need like a custom vecf64 wrapper kinda structs

### Misc

- figure out what's the like optimal brute threshold

- JUICE CHESTS AND MAYBE EVEN BOOKS CHESTS
  - just a greedy algorithm based on the prices

- force special state to respect upgrade order
  - just ignore invalid ones? or what
  - need to store helmet/whatever information in Upgrade,
    - get rid of non-tick input i guess?

- better average evaluation (like is there a better estimate)
  - also hunt down the edge case where sa failed - i think something wrong with the support or something

- add assertions to a lot of prepoutput stuff
  
- SURELY there's a better way to lay out juiceinfo...

- start working on visualizing this stuff

### Big

#### Scale up compute

- set up slurm on laptop
  - maybe write a dispatcher instead of naively running them in parallel
- profile the thing maybe

#### Analysis

- elo or percentage deviation? idk need to do more research
- how to visualize / interact with this data
- how to evaluate adaptive policies???

#### Advanced honing

- use monte carlo to generate the distribution, then allow for a few strategies
  - no juice, full juice, no juice full scroll, full juice full scroll, FULL SCROLL etc
  - then the state just selects one of these distributions

#### Editable artisan / progress

- each upgrade will have a starting artisan value attached, state generation and stuff should work automatically

#### Website

- multiple selectable & editable express
  - rework constant.rs to take in a json or something so it can interface with the website
  - eventually T4.5 integration

- wire up all this to the website:
  - Average mode / alt modee / efficiency mode:
    - input: market price + leftover "price"/ leftover value / care abt it or not toggle
      - avg = needed x market price - leftover x leftover price
    - output: juice advice, new avg values(also show old/naive?)

  - Prob maximizing mode / main mode:
    - input: market price, owned budget
    - output: juice advice, new prob(old prob)
  - need to figure out how the ui is gonna look like

  - (both: updatable clicks)

- should probably start from the user experience:
  - Simple/average/alt/efficiency mode:
    1. tick upgrades
    2. adjust price / leftover and input owned budget
    3. done (average optimizing algorithm gives a state & special state)
    4. (optional) click taps as we go and update "mats used" and stuff and update the state

  - Advanced/prob-of-success/main/play-it-safe mode:
    1. warn if applicable, tell user to untick(set price to 0) mats that they have too much of
    2. same thing as simple mode

  - Forecast mode for both: just a convenience thing, add a column of "earned per week" and how many weeks to forecast
  - (idk i'll figure out forecast later)

### Algorithm ideas

- stopping early if no improvements (an adaptive temperature / annealing schedule i guess )
- limit neighbor function - 1 click 10 taps, limit amt of toggles maybe
  - start with VERY broad neighbors - all true, all false etc, then refine until limit reached (such as 10 taps), test how much the limit affects performance
- keep a top 10 list and randomly restart to them instead of just the top 1
- some kind of heatmap of which bits were the most impactful? but i feel like this wouldnt actually do much
  - can definitely think about which upgrades were the most impactful tho
- maybe add some way to keep track of where bits are to make neighbor potentially more efficient

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
- juice chest openinig optimization

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

- ~~add a brute forced average evaluator?~~
- ~~make a more sensible suite of test cases to test the behaviour of the metric functions~~
- ~~fix the leftover evaluation like i think sometimes the trivial case is wrong~~
- ~~normalize average metric's interaction with acceptance~~
  - ~~use the "biggest seen gap" to normalize?~~
  
- ~~um make sure things are like correct like why tf did monte carlo change~~
- ~~rework how special state works (just re-oredr upgrade_arr i think)~~
  - ~~optimize the reorder with swap maybe while i'm at it~~

- ~~imporve how state bundle works~~
  - ~~store a copy of prep output in state bundle~~
  - ~~make a lot of functions that currently take in a statebundle and a prepoutput class functions of statebundle~~
  - ~~add a performacne tracker struct and put it in state bundle, track these:~~
    - ~~total states evaluated(what we have now)~~
    - ~~states evaluated with SA~~
      - ~~times that Ks was called~~
      - ~~average newton iterations~~
      - ~~times that we used edgeworth vs times that we used lugganani~~
    - ~~whatever we do with special also~~
    - ~~states evaluated with brute~~
    - ~~then also repeat all these for the "best" state~~
- ~~ optimize special?~~
- ~~bundle the streaks together see if theres any optimization there~~
- ~~also maybe get rid of the count in special state? just the order? (and maybe incorporate normal honing order?)~~
- ~~figure out why special stuff is still slightly off(or is it just how SA be)~~
- ~~figure out why MC agrees like half the time but not half the other times~~
- ~~SPECIAL iS NOT INDEPENDENT GG NEED TO FIGURE OUT HOW TO DO THIS AHHHHHHHHHHH~~
  - ~~right now its 2^n possibilities but if i change it from specifying attempt counts to "try until run out / suceess" then uh its n possibilities thats crazy~~
- ~~continuity correction? maybe not needed?~~
- ~~why is warmstart theta not working hm~~
  - ~~theta can be VERY small because budget can be VERY big -> almost all edgeworth~~
- ~~implement fft or something for medium sized complexity because lr kinda very bad~~
- ~~(also use (and optimize) exact convolution for single piece(maybe 2))~~
- ~~ add upgrade name to states when saving them,~~
- ~~ monte carlo at the end to verify / sanity check / just to look at it (also can compute confidence interval with variance etc)~~
- ~~implement brute~~
- ~~actually simulate special with monte carlo~~
- ~~debug saddleponit approximation why does it tweak out some time~~
- ~~ adjust theta bounds cos like sometimes its actually bigger than 1 i think~~
- ~~implement average-optimizing evaluation(tiebreak with this when prob = 100 also maybe?),~~
  - ~~more precisely this minimizes what the "buy mats with gold" option is doing currently, as in the average gold needed (raw + spent on buying mats)~~
    - ~~plus the value of leftovers as specified~~
  - ~~this can keep track of each dimension individually & calculate leftover precisely~~
    - ~~add field to result json etc (can think abt bound/tradable maybe)~~
  - ~~customizable sell ratio~~
- ~~allow arena to take in the various metric functions~~
- ~~separate sa / avg test cases because they take in different inputs~~
  -~~ specifically avg additionally takes in another set of prices (leftover values)~~
  - ~~need to also set up bloating of this~~
- ~~figure out how to optimize special its way too slow rn~~
  - ~~limit special state to never switch up/switch up once only? then it'll be a list of how many & which ones rather than like individual indices~~
- ~~figure out how to do neighbors for special state~~
- ~~turn a lot of helper functions to class functions of prep_output or state_bundle~~ turns out there's not that much ig
- ~~BOOKS ARE MUTUALLY EXCLUSIVE maybe just make sure that neighbor function handles it? idk~~
  - ~~change state to be (bool , i64), -1 = no books~~
- ~~procedually generate test cases~~
  - ~~figure out how to fit the juice prices into the csv~~
- ~~adjust special such that the probabilities add up to 1 ~~
- ~~ perform a final saddlepoint for P(Y1 < H1), P(Y2 < H2) ... P(Yn < Hn) warn if any is high~~
- ~~rewrite the saddlepoint stuff to actually be used in other situations~~
  - ~~initialize the stratch pads in state bundle somewhere?~~
- ~~special honing~~
  - ~~need to add a 0th slot to prob_dist or? have to strip out a lot of stuff~~
  - ~~state of this is actually integers this isnt actually so simple because we can finish early~~
  - ~~state = order of which ones to attempt first~~
    - ~~p = 1 - PRODUCT (1 - p_i *P(got special leaps left)* P(this upgrade hasn't succeeded yet))~~
    - ~~estimate this P(got special leaps left) with saddlepoint(wow this thing is tas)~~
- ~~Adjust for owned juice only for prob maximizing mode, don't ~~care abt~~ allow books because it should only be used for mains~~
  - ~~because evaluating P(Y < H) and P(X + Y < H + B) which need saddlepoint and is quite expensive~~
  - ~~probably roughly x2 for 1 type, x6? = P(Y1 < H1), P(Y2 < H2), P(X < B), P(X < B+H1), P(X < B + H2), P(X < B + H1 + H2), and this is assuming Y1 Y2 are independent which they arent...  way more for 3+ types~~
- ~~The punishment for not using juice is incorrect rn~~
  - ~~again this is much harder thank i thought fuck~~
  - ~~but if i manage to do it it can accomodate other non-sellable mats~~

- ~~rewrite Ks to allow different stuff to be in(pre calculate alpha_arr), and to allow toggling of the derivative calculations instead of having 2 funcs~~
- ~~use newtons methods until 0 derivative or something, this bisection kinda slow~~
- ~~Make a ks_012 and only call ks_01234 function when needed~~
- ~~need to add more columns to test cases, book count storage etc, add easy book adding system (specify amt of % array, which upgrades they can be on etc)~~
- ~~modify the current too-many-juice checker to accomodate books~~
- ~~ set up books support so state will be~~ integers ~~instead of bools~~
- ~~need to use lists of bools instead, integers doesn't quite work i think because need to keep track of costs~~
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
