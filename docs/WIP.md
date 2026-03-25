
# Work in Progress

## Immediate WIP

BIG PROBLEM THE WEBSITE IS CRASHING ON CHROME???

pretty sure it's just a performance issue:

- ~~move grid & keyed_upgrade watch to actually trigger on change instead of relying on watcher~~
- move the bound / roster / tradable allocation logic to rust, let histogram worker handle everything (average, gold, tradable gold) so we're starting less workers
- ~~potentially keep the workers alive instead of starting a new one every call?~~

- ~~lock progress change UI when the optimizer is still working (because special will change the position of the upgrade and its all over from there)~~

## Roadmap

### 1st: ~~re-write the frontend~~

### 2nd: ~~Serca~~

### 3rd: ~~Advanced honing optimization~~

### 4th: ~~Tradable / bound mats distinction~~

### 4.9th : Rework header bar

There's more tabs that need to be accomodated, (OCR UI & manifest, box optimization, forecast mode etc), change everything in the header to a (collapsable) side bar with collapsable subfields

### 5th: OCR

Similar to the[Ark grid OCR](<https://airplaner.github.io/lostark-arkgrid-gem-locator-v2/>). They've used template matching and I think is probably best for me too (with exception of numbers recognition, might need some OCR model for that).

## GOALS

1. screenshare -> nothing hovered -> detect everything that can be detected, aka:
    - < 9999 mats (and boxes)
    - non-ambiguous boxes, which is pretty much just the dailies boxes
2. Things that are missing and needs hovering:
    - bound / roster bound / tradable
    - boxes
    - 9999+ mats amount

## Specifics

Just my current rough mental image of what needs to be done

### Initial scan (step 1)

This will probably take longer than 33ms which is okay, but should still be as fast as possible

- [ ] Anchoring (can also detect resolution here)
- [ ] Grids position detection
- [ ] Individual item ~~template matching~~ I mean it's not even template matching its just checking if the pictures match
- ^ There's got to be an automated way of setting this up
- [ ] Number reading (Hopefully template matching works for this otherwise we're cooked)

### Hover tooltip scan (step 2)

This step must happen FAST, like 33ms fast so that the user can skim through the mats

- [ ] Find the tooltip bounding box somehow?
- [ ] Will probably have to find the mouse? Need to template match the mouse? This way we know what's being hovered. (7 types of cursors, 5 sizes, hopefully outline doesn't matter)
- [ ] Template match for keywords inside the tooltip (tradable, etc) and probably icons also
- ^ There's got to be an automated way of setting this up

^ These 2 steps will need an intermediate debug step for sure, as in saving the image & running rust standalone so I can debug it without having to use the UI.

### UI

- [ ] Set up passing the data to WASM
- [ ] Somehow get a picture (maybe first successful initial scan gets sent back?) to js, along with grid information
- [ ] Set up some kind of system to add shading to the picture (and hover tooltip)
- [ ] Set up a manifest (list of stuff saying whether or not they've been successfully detected, need hover an whatnot)

### Decisions that I'll need to make at some point

- Support more resolutions / aspect ratios or not? This will probably depend on how "automated" my process of setting up the template matching is. I doubt it will be very tho
- How many threads? I'm thinking one thread for step1 and one thread for step 2 (which needs initial grid position info from step 1).

Needless to say this will be a LONG term endeavour

### 6th : Box opening recommendation

- Each chest type is like another state, [how_many_option_1, howmany option_2] etc, should work readily with the optimizer
  - need to convert these states into the budgets shouldn't be that hard

### 6.5th : Hard limit on juice usages

- disallow states that can use more than a certain amount of juice / scroll / books
  - this kinda falls apart with normal honing cos we can finish early, but can just make a pessimistic assumption
    - will prolly need to add an extra column in material_info for the hard limit
    - keep track of how many juices are used, sample appropriately in neighbour function

- I mean it's kinda redundant cos we can just set the price to like 9999999?

### 7th: Forecast mode

Haven't thought enough about the specifics but shouldn't be that hard once everything else is in place.

- Recommended pushing dates for main / rat alt
- seletable income choices
  - gold earning raids
  - mat earning raids, boxes etc
  - paradise (get the average from one of the reddit posts or generate my own? will need to data mine?)
  - configurable dailies
  - guild
  - unas thing
  - daily login
  - solo shop

### 8th: Optimizer Improvements

- figure out if the restarts are carrying the algorithm and maybe do SA properly
- somehow make the pity length more explicit cos right now a lot of moves don't do anything
- ~~force special state to have a non-small tail~~ actually that ~~might~~ in fact does discard optimal choices, just make sure that special neighbour moves actually has an effect
- some way to estimate how close we are to optimal because re-running this for every week is going to be a bit insane
- maybe a neural network for an (or few) initial guesses -> optimizer?

- will probably need to set up an actual elo / some kind of evaluation system, i think the test cases should just be a BIG curated list of likely scenarios - maybe collect this from users but that's kinda hard to set up
  - like there can't really be "overfitting" cos we have like 2 parameters
  - this is definitely a very much long term goal

### 9th: Price trend viewer

need to pull as much historical data as possible from loa-buddy -> store & pull from my own data base -> keep updating this

This will be the first step towards an actual hosted non-serverless website

Maybe discord integration after this / uwuowo import / cross-device storage stuff like that but that's kinda far (can maybe do some cool stats analysis with uwuowo char ilevel population data)

##

Below are some rambling / brainstorming / Misc stuff

### Misc UI

- single piece hoverable artisan (if only 1 piece selected, show % chance and artisan on graph)
- Input arithmetic parsing(e.g. allow inputs like 25*1234 for easier boxes calculation)
- drag to select
- make bundle size suffix not take up horizontal space
- Make shard bag size change actually change to the other price (and manual overwrite that one bag size)

### Misc

- add assertions to a lot of prepoutput stuff
  - also make a lot more test_payload that actually cover edge cases
- start working on visualizing this stuff
  - put all possible states on one axis (must be small support like 5? 10? ) and sort by number of juice used, then color/ 3d height?

### Optimizations

- find_min_max can probably do with some kind of caching but i can't figure it out rn
- advanced hoinging caching / pre-calculating, maybe also special

## Done / cancelled

- ~~make serca conversion add to the existing serca budget instead of overwriting~~
  - ~~need to think abt how to decouple tier conversion and mats conversion cos there'll be cases where people are in serca gear but are still earning T4 mats and want to add them that way~~
  - ~~I THINK this will just be replaced by OCR because it'll be too confusing - we need both the current 5 to 1 straight up conversion (to compare the cost of T4 vs Serca) and this adding behaviour~~
~~### Other features~~

- ~~Automatic Market price integration(via some kind of API, or just updating the site at a regular interval automatically)~~

~~add more settings validations~~

- ~~i know front end is hideous but idrk how to do better~~
- ~~Treat roster bound as~~ ~~tradable? maybe make this a toggle~~ ~~bound, because that makes no sense~~ allow user to select via a radio selector or something
  - ~~Toggle should be "treat tradable as roster bound", which should be on by default because people are more worried about how much gold tehy will actually spend~~
- ~~update the SA computation to allow for 2 breakpoints~~
  - ~~will need to have 2 different budgets and another set of prices but shouldn't be that hard~~
- ~~Re-do test payloads because i changed like every field name~~
~~I started on this but uh this is much harder than I thought and Serca is much closer so it'll have to wait on the backburner for now, but here's some steps:~~

~~NONE OF THIS DATA STUFF TURNED OUT TO BE NECESSARY because DP is op and we can just compute the adv honing distribution on the fly.~~

1. ~~Make an adapter thingy to go from (id list) -> (partially (but sufficiently) initialized cache array of array buffers), which will need to:~~
    1. ~~check if its in cache~~
    2. ~~if not, fetch (everything needed at the same time) -> put into cache~~
    3. ~~pass this array into rust (I hope this work? it honestly might not idk)~~
2. ~~Handle this on the rust side, specifically:~~
    1. ~~Make parser accept this data~~
    2. ~~ Write this data into prep_output (definitely serde skip on this one)~~
    3. ~~Make "update individual dist" and stuff update for advanced honing upgrades, which is just reading from this array~~
    4. ~~Make sure that state and everything works with this~~
    5. ~~Make neighbour function perturb this state correctly~~
3. ~~Display and allow for edit all the necessary information in js:~~
    1. ~~Current progress (xp and balls, should be text inputs and dropdown?)~~
    2. ~~suggested strategy (state)~~
4. ~~Maybe hard-code the most common setup~~
5. ~~Remember to corroborate the new dp results with old mc results~~
6. ~~Allow arbitrary juice / scroll effectiveness (currently it uses hardcoded values and ignore the inputted ones)~~
7. ~~The way the states are curerntly implemented it doesn't actually allow different types of scrolls? idk we'll cross that bridge when we get there ig~~

- ~~This kinda uses an egregious amt of memory rn, need to benchmark how much the cache actually helps and how to reduce it~~
- ~~it's also kinda just slow, will prolly benefit from hard-coding known stuff~~

1. ~~Strip out EVERYTHING that currently exists to use a single "store" (every char needs its own "store")~~
2. ~~Reorganize the UI to be:~~
    - ~~left side hotbar with a button for roster setup (which is its own "page"), hotbar will contain selectable characters later~~ ~~char & roster setup on the top probably, left sidebar for different pages including:~~
        - ~~ Main (simple status input, cost distribuiton / input, simple optimizer instructions)~~
        - ~~detailed status input~~
        - ~~Advanced optimizer insturctions (allow user inputting here also)~~

        - ~~LONG TERM chest opening optimizaiton~~
        - ~~maybe put control panel on the sidebar here also~~
        - ~~LONG TERM Importing (uwuowo, ocr)~~
        - ~~Forecast mode~~

    - ~~roster page reflects currently selected character, which will include"inputs", "cost distribution", "optimizer", "forecast" etc, need to figure out how to lay this out~~
    - ~~Similarly roster page will need:~~
        - ~~all the roster budget & prices input in a big table~~
            - ~~tradable owned, roster owned, market price, trendline maybe?~~
        - ~~character selection / a lil summary~~

    - ~~Header~~
        - ~~Logo~~
        - ~~Roster set up~~
        - ~~Characters~~
        - ~~button for making new chars~~
        - ~~LONGTERM Price trend viewer, or maybe bake this into the prices?~~

    - ~~Footer~~
~~ actually really need to figure out how to lay these out, maybe it'll be easier once I can look at everything wth my eyeballs~~

~~Specifically:~~

- ~~this opens the door for persistent special & adv caches? this would make it easy to put pre-computed table in (just write it in js on intializaiton)~~
  - ~~figure out how to serialize & deserialize ahashmap, right now its as if there's no caching~~
- ~~this also opens the door for when we know the current state is good already on optimizer calls~~

~~#### Char page - Upgrade status inputs~~

- ~~maybe import via uwuowo?~~ prolly not, even if i decide to add it it'll have to be on the roster setup page where we create a character

~~#### Char page - Material cost distribution~~

- ~~3 columns,~~ ~~6~~ ~~5 columns~~
    0.~~ enable/disable checkbox~~
    1. ~~user bound mats input, starts at 0 by default~~
    2. ~~Avg cost~~ ~~/ top x% bottom x%,~ allow custom inputs in the header~~ ~~i think just making hover more obvious is good enough, the interpolation / whatever problem is gonna be aids~~
    3. ~~Gold spent buying this mat / Gold you would've made selling this mat + Gold spent buying this mat ~~~~(maybe make this col optional)~~ selectable
    4. ~~The corresponding graphs~~

- This can get quite long so ~~need to hide useless / irrelevant graphs and a show all button or something~~ ~~3 buttons to toggle between the 3 categories~~ dropdown

~~#### Char page - Chest opening optimization~~

~~### Roster page - prices~~

- ~~Tradable, rosterbound, price inputs~~
- ~~set up api & auto importing form loa buddy~~
- ~~make bundle size suffix not take up horizontal space~~

~~### Tooltip / popup~~

- ~~Kinda need a popup in some places, maybe make this reusable but maybe not possible~~
  - ~~switching tiers warning~~
  - ~~succeeding & decuting warning / selection~~
  - ~~footer / changelog maybe~~
  - ~~hints / explanations~~
  
~~### Footer~~

- ~~add footer for discord github links and whatnot~~

~~Mixing serca and t4 is lowkey possible if we make the assumption that Serca mats -> t4 mats conversion is possible (so we just convert serca mats to x5 t4 mats and call it a day) but this assumption is kinda dumb and probably won't be very useful anyway, THEREFORE:~~

1. ~~Implement / rework something about constant.rs, maybe split up tier-invariants and things that change. Get the data from icepeng.~~
2. ~~Modify parser to know what tier is happening~~
3. ~~User can select tier, and if anything is already owned, present option to convert t4 to serca.~~ ~~Also add a button to divide by 5~~
4. ~~Add a toggle for "allow converting t4 mats to serca mats" for each char~~
    - ~~roster_config needs 2 sets of everything ig~~
    - ~~pass the converted / unconverted mats to wasm, also auto pick the better deal out of the two~~

~~MISSING DATA:~~

1. ~~Unlock costs~~
2. ~~Silver costs~~
3. ~~Cost, chance, juice information below +11~~
4. ~~Special leaps~~

- ~~Add grid lines?~~

~~### Char page - Special~~

- ~~Maybe make a row for free taps num success vs chances~~
- ~~where the special budget input should go~~

~~#### Char page - Optimizer instructions~~

- ~~text instructions instaed of boxes, no longer allow custom states input cos the UI is too cluttered~~
- ~~2 rows for juice & book, e.g. juice x10, no juice until x% artisan, juice after that~~
- ~~similar text instructions for advanced honing~~
- ~~"succeed and deduct costs" button, which prompts for either number of taps via a slider, which should show artisan & material costs / predicted material remaining~~
  - ~~just ignore <0?~~
- ~~Hover question mark tooltips for various systems, but still aim to be intuitive~~

- ~~~~Adjustable week number in raw / Gold graph~~
- ~~Achieved ilevel~~

- ~~Something seems to be broken in drag-to-select spreadsheetgrid~~
- ~~marquee for state grid also?~~
- ~~rearrange where the gold cost is~~
  - ~~maybe make a toggle of including tradable budget & showing actual gold spent instead?~~
- ~~button to deduct costs?~~
- ~~default to cumulative and ~~add 10 percentile points maybe
- "which on~~e should i pick"~~
  - ~~pre-set selections such as red vs blue, red juice vs blue juice, scroll boxes etc~~
- ~~success update based on one or a few mat types~~

~~### Char selection page~~

- ~~roster tracking & income estimation~~
- ~~copy paste from uwuowo (maybe just fetch?)~~
- ~~toggle "done" upgrades~~
- ~~also actually deduct from the mats~~
- ~~toggle using roster bound / tradable mats or not~~
- ~~maybe can do 2 breakpoints?~~
- ~~actually can draw some huge inspirations from here <https://next-gen.materialsproject.org/materials/mp-48>~~
  - ~~as in side-by-side boxes, this would work much better for mobile~~
  - ~~and a toolbar on the left to select mode / char and what not~~
- ~~will need to figure out how to have this grid system for multiple characters~~
- ~~verify advanced honing using Monte Carlo~~

- ~~JUICE CHESTS AND MAYBE EVEN BOOKS CHESTS~~
~~#### Roster page - main~~

- ~~add char, delete char, duplicate that kind of stuff~~
- ~~need to figureout how to lay everything out~~
  - ~~roster bound mats toggle / selection? should only allow one char to use roster bound (boxes) mats~~
  - ~~need to accomodate serca and shit~~
  - ~~need to rework all the variables and everything~~
    - ~~also need to accomodate forecast mode somehow? like there'll be multiple calls to optimizer ig?~~
  - ~~actually deduct from budget? will need to re-work how things work (probably deduct when success button press and skip all succeeded upgrades completely)~~

- ~~Sidebars~~
  - ~~logo -> burger on mobile~~ ~~prolly only for roster setup~~
- ~~market price import~~
- ~~resolution (is it really necessary actually)~~
  - need to re-add this to neighbor
- ~~enable/disable checkbox~~
- ~~move treatment selection to sidebar, accomodate the text somehow~~
- ~~special budget & graph~~
- ~~total gold row~~
- ~~char creation~~
- ~~timestamp & dont re-pull~~
- ~~rework the use special leaps butotn~~
~~#### Char page - Detailed status inputs~~

- ~~artisan, adv xp etc on a separate page?~~
  - ~~separate page => no more hand-holding optimization which i mean it's not designed to do anyway~~
    - ~~can still do piece-after-piece optimization~~

- ~~wire up the result of optimizer to keyed upgrades~~
  - ~~will need to set up some kind of callback on optimizer results (to write to the keyed upgrades)~~
  - ~~let histogram watch the keyed upgrades~~
- ~~make the mat distribution read from a separate worker~~
- ~~Optimizer instruction~~
- ~~add back graph hover tooltip~~
- ~~wire up the 2 annotations on material graph(need to add another annotation)~~ ~~maybe too cluttered~~ togglable

~~ #### Char page - Gold breakdown~~

- ~~This needs to show both "actual gold spent on market/raw" and "eqv gold you wouldve made from selling tradables"~~

- ~~achived ilevel & desired ilevel display~~
-~~ tier selector should be around here, show a popup / tooltip?~~

- ~~include statebundle in the "store" and only update when some thing changes, make parser its own wasm binding for that~~
  - ~~non-state changes (like price, owned) should trigger parser + eval(like the current behaviour), state changes should trigger eval only, though that's not really a huge deal cos parser call not that expensive~~
- ~~Add the two new columns in material distribution~~
  - ~~ will need to modify the backend to do this - maybe a special evaluateAverage function?~~ no needs
- ~~How to intuitively toggle between the 3 states~~  ~~ -maybe slider with 2 heads?~~
  - ~~If current is empty(everything starts here):  ~~
    - ~~if left of current is NotYet, toggle everything to the left to done~~
    - ~~othewise, toggle current to Want~~
  - ~~ If current is Want:~~
    - ~~Toggle self and everything to the left to Done~~
  - ~~If current is Done:~~
    - ~~Toggle current and everything to the right to NotYet~~
- ~~add a lil key to explain the colors~~
- ~~fix the mat distribution row such that the inputs work correctly~~
   -~~ hopefully just involves copy pasting the styles back again~~
- ~~make the roster page~~
  - ~~just the prices inputs for now is fine~~
- ~~Set up writing to setting~~
- ~~Set up special budget (just the input for now)~~
- ~~input & copy payload to re-make the test cases~~
  - ~~verify & fix whatever is wrong~~

- Set up serca
  - ~~get the json~~
    - ~~get data from icepeng~~
  - ~~add tier conversion button in control panel~~
    - ~~get the ilevel mapping~~ ~~somehow? is this on icepeng?~~ memo video
    - ~~also achieved ilevel stuff~~
    - ~~add undo button?~~
  - ~~Add Serca mats prices page~~
  - ~~add allow converting roster t4 mats to serca (should be a separate button)~~
    - ~~shouldnt be that hard?~~
      - ~~need to auto-select market price~~
      - ~~light up the one that's active~~
  - ~~get the icons and route appropriately~~

- ~~add "treatment" toggle in control panel via radio selector or something~~

- ~~some better way to input mats?~~
  - ~~screenshot upload? screen share recording? need OCR~~ no way

1. ~~generate distribution for each element in decision space~~
    - ~~what should the decision space be????~~
        - ~~must be easy to follow~~
        - ~~artisan breakpoint or fixed count?~~ fixed count
            - ~~juice the first n grace, juice the first m non-graces~~
              - ~~n < like 12, cut off n = inf somewhere~~
              - ~~m can prolly go up like 1,2,3,4,5,10,20,30,40 etc up to like 80, also with inf or something~~
        - ~~prolly one for juice one for scroll so it'll be a big square~~
        - ~~force m = 0 for n < inf, so total is like (13 grace possibilities + 15 juice possibilities)^2 * 10 (starting level) that's not too bad~~
    - ~~so state should be a tuple of ~~4~~ 2 (cos can merge n and m) usize -> index into array, with alr_failed changing the starting level~~
2. ~~figure out how to bundle this data into the binary files~~
    - ~~definitely have separate files for each non-strategy configuration~~
        - ~~so should be 2 (double balls) x 7 (cur_balls, 0 to 6, forbid chisel or maybe not) x 100 (starting xp) x 2 (10_20 or 30_40) = 2800 files~~
        - ~~each file has 31 x 31 strategies, each is 100 x 3 x 8 bytes (f64) so each file is ~2.3 MB ish which is fine i think~~
    - ~~will need to fetch the file at the parser step and store it in prep output?~~
    - ~~need to pivot the data to be in this form~~
        - ~~for each store (avg juice, avg scroll, chance)~~
    - ~~need to fetch and parse(just copy) this data in wasm~~
    - ~~there's kinda 6.3 + gigs of data so uh how to put this on github and serve on cloudflare?~~
        - ~~can definitely reduce the file size a lot, maybe binary isn't the best idea (probably postcard)~~

- ~~wire up the data to the website so i can look at the data with my eyeballs to see if it's like correct before getting more samples~~
  - ~~skip actually making sure there's enough sample size for all cases~~
    - ~~will need to change configuration to include the starting xp and cur balls again, do an inner loop in main.rs to check through each of these~~
  - ~~skip optimizer changing stuff~~
    - ~~will need state -> prob dist setup anyway tho, index -> tuple -> read dist from the big array~~
    - ~~maybe pre-emptively collapse?~~ yes
  - ~~set up the fetch stuff in js (pre-fetch at ui selection)~~
  - ~~pass the file into wasm? will need a grid of this i think? worst case 24 * 2.25 mb = 54 Mb? (this will be smaller with postcard)~~
  - ~~like pass a grid of ids and a id -> data dictionary or something~~

- ~~change serde big arrays to just be vectors~~
- ~~graph label is still off by 1~~

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

- ~~visualize these using perfplot~~ plotly instead

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
- ~~wire up the new budget / price keys~~

- ~~figure out already-spent cost and display these~~

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

- ~~cache min max values with pair~~

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
