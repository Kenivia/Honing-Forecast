
# Work in Progress

## NOW

## Advanced honing

1. generate distribution for each element in decision space
    - what should the decision space be????
        - must be easy to follow
        - artisan breakpoint or fixed count? prolly fixed count
            - juice the first n grace, juice the first m non-graces
              - n < like 12, cut off n = inf somewhere
              - m can prolly go up like 1,2,3,4,5,10,20,30,40 etc up to like 80, also with inf or something
        - prolly one for juice one for scroll so it'll be a big square
        - force m = 0 for n < inf, so total is like (13 grace possibilities + 15 juice possibilities)^2 * 10 (starting level) that's not too bad
    - so state should be a tuple of 4 usize -> index into array, with alr_failed changing the starting level
2. figure out how to bundle this data into the binary
3. accomodate switching between states
4. accomodate scrolls in juice_info
    - probably need to rework this
5. accomodate progress & state adjustments in UI
6. voila?

## UI

- i swear something big needs to change, this UI aint it
- need to figureout how to lay everything out
  - roster bound mats toggle / selection? should only allow one char to use roster bound (boxes) mats
  - need to accomodate serca and shit
  - need to rework all the variables and everything
    - also need to accomodate forecast mode somehow? like there'll be multiple calls to optimizer ig?

### Char selection page

- roster tracking & income estimation
- copy paste from uwuowo (maybe just fetch?)
- toggle "done" upgrades
- some better way to input mats?
  - screenshot upload? screen share recording? need OCR
  - also actually deduct from the mats
  
### Misc UI

- Input arithmetic parsing(e.g. allow inputs like 25*1234 for easier boxes calculation)
- Ctrl z, delete
- Hover question mark tooltips for various systems, but still aim to be intuitive
- Adjustable week number in raw / Gold graph
- Achieved ilevel
- Something seems to be broken in drag-to-select spreadsheetgrid
- marquee for state grid also?
- rearrange where the gold cost is
  - maybe make a toggle of including tradable budget & showing actual gold spent instead?
- button to deduct costs?
- default to cumulative and add 10 percentile points maybe

## Forecast mode

- idk this will have to come after having individual character pages i think
- can definitely do chances (the cool graph) for individual mats, might have to use MC for the overall success rate?

## Serca

- how to consider 1-to-5 conversion??? prolly just 2 sets of grids and 2 sets of prices
  - convert serca mats to t4 mats, then use the lower price of the two when running out?
  
## Misc

- JUICE CHESTS AND MAYBE EVEN BOOKS CHESTS
- add assertions to a lot of prepoutput stuff
- start working on visualizing this stuff
  - put all possible states on one axis (must be small support like 5? 10? ) and sort by number of juice used, then color/ 3d height?

## Other features

- Automatic Market price integration(via some kind of API, or just updating the site at a regular interval automatically)

## Algorithm ideas

- figure out if the restarts are carrying the algorithm and maybe do SA properly
- somehow make the pity length more explicit cos right now a lot of moves don't do anything
- ~~force special state to have a non-small tail~~ actually that ~~might~~ in fact does discard optimal choices, just make sure that special neighbour moves actually has an effect

## Done / cancelled

- ~~stat test using monte carlo to make sure the results are actually correct (or at least self-consistent)~~
- ~~default to brute force when decision space < 24000 (with restriction of streaks)~~ this is prolly not worth the effort
- ~~add the test framework back, it should honestly be a lot easier now with payload~~

- ~~selectable double um balls event thing~~
- ~~no juice, full juice, no juice full scroll, full juice full scroll, FULL SCROLL etc~~ ~~i cant be bothered, probably a waste anyway, just assume juice & scrool on grace is optimal~~
~~## Algorithm ideas~~

- ~~stopping early if no improvements (an adaptive temperature / annealing schedule i guess )~~ nah trivial ones will finish fast any way

- ~~this all lead to some kind of heat map of how likely special / an upgrade has room for optimization~~
  - ~~maybe do upgrade affinity also?~~

- ~~a kind of self-crossover where the same upgrade type can copy each other?~~
- ~~just a greedy algorithm based on the prices~~ ~~um might be more complicated than i think, maybe need to add it to the decision space but that sounds incredibly uh wasteful, maybe just do it naively~~ actually can just tell user to open as they run out?

~~## Next big step(s)~~

- ~~Juice(and Book / Scroll) purchase suggestion~~
- ~~Books & scroll calculations~~
- ~~True juice optimization for success chance~~

~~^ this is arena~~

- ~~initialize the hash properly or something idk~~
  - ~~avoid updating dist & support based on this hash~~
- ~~multiple selectable & editable express GET THIS DATA SOMEHOW???~~
  - ~~rework constant.rs to take in a json or something so it can interface with the website~~ nah
  - eventually T4.5 integration
  - ~~get the actual numbers from the korean website <https://icepeng.com/refining>~~

- ~~get all the graphs and shit back and working~~
  - ~~rework to be graphs for indivdual dimensions~~
    - ~~hoverable with a "you are here" arrow~~
- ~~slider instead of leftover value?~~

- ~~some kind of heatmap of which bits were the most impactful? but i feel like this wouldnt actually do much~~ nah
  - ~~can definitely think about which upgrades were the most impactful tho~~ done via upgrade impact, not sure how much it actually helped
  - ~~same with special~~ done via special affinity
- ~~maybe add some way to keep track of where bits are to make neighbor potentially more efficient~~
- ~~multi-thread and each worker tune one "dimension" like special, state of this one, state of other one etc?~~ just tunes less dimensions as temperature goes down,
  - maybe allow special + juice change at the same neighbour?
  - ~~do i need to allow multiple changes at high temperature or will the acceptance function take care of local one dimensional minima~~
  - ~~maybe it would never discover like in-between optimums?~~
  - ~~this definitely lends itself to soome kind of genetic algorithm~~ crossover seems to negatively affect performance whatever I do, think the landscape is globally smoother than I expected

- ~~keep a top 10 list and randomly restart to them instead of just the top 1~~
  - ^^ this also sounds like genertic algorithm

- ~~maybe do a 2 staged thing where it optimizes the special first?~~

-~~ whatever happens it must consider the fact that sometimes we dont hvae special and changing the state wont do shit~~ done via special affinity

- ~~Prob maximizing mode / main mode:~~
- ~~implement the "naive" version that assumes no mats owned~~ nah the new heuristic should capture this
- ~~store & show show breakdown of average gold costs from here~~
- ~~instead of counting unique elements in statebundle it should read from juiceinfo instead~~
- ~~update the best known solution so that only "better" states will be colored purple~~ kind confusing gonna stick to 2 colors
- ~~add "restore best known solution"?~~

- ~~v10 seems to choke on cases where there's a lot of special availiable and do very well on low special, maybe adaptive annealing will help or maybe a more informed starting position via upgrade_impact?~~

- ~~add performance aggregation in preparation for multithreaded algorithms~~

- ~~add maximize success prob optimizer button at some point~~
- ~~stream intermediate results that'd be so cool~~
  - ~~make sure all existing things work, then start work on the new page(s)~~ nope
- ~~maybe its time to get rid of Box< iterator >~~ theres no performance impact other thant memory usage apparently

- ~~cache the geom calculations  (special_prob seems fast enough so cbb rn)~~ this wont work well with multithreading

- ~~can actually save 1 evaluation at the end of special_prob by setting the zero prob... (only when special state allows all things to be skipped)~~  but like itd be trivial anyway so prolly not
- ~~multi-threading????~~
  - ~~will prep by compling using all the right tools(i hope) and then figuring it out later~~
~~## DEV~~ ALL CANCELED COS NO LONGER RELEAVNT

- ~~MUCH more tests, specifically:~~
- ~~Test countfailure non-naive version actually works~~
- ~~Test average tap and~~ ~~truncated average tap~~
- ~~Test Tap map generator(and improve it maybe)~~

- ~~Matrix operation libraries to speed up monte carlo and what not?~~
- ~~Graph is off by 0.5 all the time, and the points of interest snap the the one below(i tihnk?) which isnt quite right idk, kinda scared to touch it~~
- ~~better input cleaning for spreadsheetgrids~~
- ~~make get_one_tap_pity take in seeded rng~~
- ~~cache using array buffer? dont know if it's worth the effort~~
- ~~fix the react stuff, i think there's way too much usememo(the website feels sluggish)~~
- ~~improve how the cost estimation works / verify that it actually works~~
- ~~Allow selling mats~~
- ~~Raw gold graph and overall gold(including used in buying mats) lines in thegraph~~
-~~ Artisan level editing~~
~~#### Scale up compute~~ no need because these things will be parallel already and i dont have multiple computers

- ~~set up slurm on laptop~~(maybe not)
  - ~~maybe write a dispatcher instead of naively running them in parallel~~
    -~~ wrap the current main function~~

~~#### Analysis~~

- ~~elo or percentage deviation? idk need to do more research~~
- ~~how to visualize / interact with this data~~
- ~~how to evaluate adaptive policies???~~ no need actually but uh whatever
- ~~limit neighbor function - 1 click 10 taps, limit amt of toggles maybe~~
  - ~~start with VERY broad neighbors - all true, all false etc, then refine until limit reached (such as 10 taps), test how much the limit affects performance~~

-~~ visualize these using perfplot~~ plotly instead

- ~~take average of many test cases i think (many trials?)~~
- ~~store time, states evaluated and metric~~
- ~~need to parse & aggregate this data then send to python(or just do it in python should be okay)~~
    -~~ as in keep a parsed file of data that python can read off of~~ maybe not

- ~~make solvers stream intermediate results at x1000 evaluations / second?~~

- ~~postmessage & update in js~~
  - ~~need to ignore these such that changes don't cancel ( i mean its already greyed out so maybe this will be easy)~~
- ~~rework test case csv to json and take in js objects straight up~~
  - ~~add export button to copy to clipboard~~
  - ~~maybe take in a folder of test cases instead for ease of adding more test cases~~
~~ #### Editable artisan / progress~~

- ~~each upgrade will have a starting artisan value attached, state generation and stuff should work automatically~~
- ~~rework collapseed so that p = 0 are removed, need to update everything in core also~~

- ~~store simple averages and add them up when needed~~ not worth, memory blows up
- ~~input: market price + leftover "price"/ leftover value / care abt it or not toggle~~
    -~~ ACTUALLY need to do the editable artisan thing first(which i mean shouldnt be that hard)~~

- ~~Average mode / alt modee / efficiency mode:~~

- ~~output: juice advice, new avg values(also show old/naive?)~~
- ~~need to figure out how the ui is gonna look like~~

- ~~(both: updatable clicks)~~

~~ 10 at a time toggle, need to modify neighbour function & stategrid~~

- ~~figure out a way to export state_bundle to js and be able to pass it to monte carlo appropriately~~
  - ~~literally just state and special state~~
- ~~this should also be uh editable in js by ui~~
  - ~~add uh all the owned juice and stuff~~
  -~~ all the existing functions now take in a statebundle instead (still initialize prep_output like before)~~
-~~ wire up the new budget / price keys~~

-~~ figure out already-spent cost and display these~~

- ~~add extra "unlocked" buttons and handle it accordingly in rust~~
  -~~ parse this in rust via update_individual~~
- ~~cap displayed progress / statebundle to the pity length~~
  - ~~similarly disallow juice inputs to the left of progress~~

- ~~add succeed button and handle it correctly~~
  - ~~succeededGrid, which causese it to render in js (greyed out) but~~ ~~rust ignores it(does not put it in upgrade_arr, but contributes to already-spent cost)~~ just change the prob dist
- ~~special state user input & passing into eval~~
  - ~~how to present this to user???~~
  -~~ store the "invalid index" from special probs so that js knows when to stop rendering the list~~
  - ~~also store the special probs of the active special state~~
  - ~~how to handle alr-succeeded upgrades in rust???~~

- ~~update parsing/payload building such that 00 doesn't break it what the actual fuck~~

- ~~add input box next to succeed free tap~~
- ~~adjust how the optimizer is ran (cancel when anything changes, and say so)~~

- ~~add optimizer button~~

 ~~and already-spent costs~~

- ~~force special state to respect upgrade order~~
  - ~~just ignore invalid ones? or what~~
  - ~~need to store helmet/whatever information in Upgrade,~~
    - ~~get rid of non-tick input i guess?~~
- ~~crawling back to performance~~
- ~~pre-allocate scratch space maybe?~~ dont need u_arr at all

-~~ cache min max values with pair~~

- ~~aka step size & max size~~
  -~~ also min_delta of both top side and bottom side~~
  -~~ can use this to calculate lattice span also at least for linear cases~~

- ~~how to better guess theta~~
- ~~SURELY there's a better way to lay out juiceinfo...~~ yes there was
- ~~hunt down the discrepancy with juiced states~~
  - ~~rewrite how juiceinfo is laid out and use id in the state immediately ( only same "ids" arr in juice info to convert)~~
  - ~~clean up how support stuff get updated (update_individual)~~
  - ~~just hope that the problem will reveal itself when i tidy up the mess ig~~ it did
- ~~figure out what's the like optimal brute threshold~~ support size estimate turns out to be kinda useless
- ~~test how good newton is compared to what im doing now~~ a decent amount, keeping householder yippy
- ~~test how much linear helps~~ a good amt in some cases
- ~~use higher derivatives than newton~~
  -~~ will need to um strip out how the current error dection works~~

- ~~figure out what exactly needs to be cloned in Upgrade~~
- ~~use polynomial for equal stepped stuff like for mat costs in averages~~
  - ~~need to work out the math first~~
- ~~cache lattice span~~ or just compute it more cleverly

- ~~figure out how to update best_state_so_far without cloning all the time~~
- ~~figure out how not to allocate on every ks call? i thought i was avoiding that~~
- ~~figure out why sa is failing sometimes~~
  - ~~force it to never underflow~~
  - ~~also logp is lowkey kinda uselesscos scaling alpha doesn't actually help~~

- ~~better average evaluation (like is there a better estimate)~~
  - ~~also hunt down the edge case where sa failed - i think something wrong with the support or something~~
- ~~profile the thing maybe~~
- ~~I should separate upgrade_arr from prep_output cos everything else in prep_output doesn't change with the state, realisitically it should pass a~~
- ~~is there really no better way to do the special probs(can i get away with an approximation some how)~~
  - ~~take advantage of the fact that the special costs share a common factor~~
  - ~~somehow avoid re-allocating maybe~~
  -~~ skip small mass?~~
  - ~~definitely cache this result~~
- ~~profile the things~~
- ~~^ both of these will probabily need like a custom vecf64 wrapper kinda structs~~
- ~~add a tag to the support or just do it via support index~~
- ~~collapse identical supports~~
- ~~refactor stuff so that everytihng is a state class function~~
- ~~streameline solve such that it takes in an initial state instead of relying on initializing in there~~
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
- ~~optimize special?~~
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
- ~~add upgrade name to states when saving them,~~
- ~~monte carlo at the end to verify / sanity check / just to look at it (also can compute confidence interval with variance etc)~~
- ~~implement brute~~
- ~~actually simulate special with monte carlo~~
- ~~debug saddleponit approximation why does it tweak out some time~~
- ~~adjust theta bounds cos like sometimes its actually bigger than 1 i think~~
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
- ~~perform a final saddlepoint for P(Y1 < H1), P(Y2 < H2) ... P(Yn < Hn) warn if any is high~~
- ~~rewrite the saddlepoint stuff to actually be used in other situations~~
  - ~~initialize the stratch pads in state bundle somewhere?~~
- ~~special honing~~
  - ~~need to add a 0th slot to prob_dist or? have to strip out a lot of stuff~~
  - ~~state of this is actually integers this isnt actually so simple because we can finish early~~
  - ~~state = order of which ones to attempt first~~
    - ~~p = 1 - PRODUCT (1 - p_i *P(got special leaps left)* P(this upgrade hasn't succeeded yet))~~
    - ~~estimate this P(got special leaps left) with saddlepoint(wow this thing is tas)~~
- ~~Adjust for owned juice only for prob maximizing mode, don't care abt allow books because it should only be used for mains~~
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
- ~~set up books support so state will be~~ integers ~~instead of bools~~
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
