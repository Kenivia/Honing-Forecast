# Honing Forecast

The [Maxroll upgrade calculator](https://maxroll.gg/lost-ark/upgrade-calculator) gives us a rough estimate on how much mats we need to pass certain upgrades, however it only gives the best, average and worst scenarios - this calculator works out everything in between.

Try it on the [website](https://kenivia.github.io/Honing-Forecast/)!

## Chance to Cost

This improves on the Maxroll calculator. How to use:

1. Tick the upgrades you want(same as Maxroll)
2. Put in **Desired chance**(of success)
3. Press **Find estimated cost**
4. That's about it

## Cost to Chance

This is a functionality that Maxroll currently does not have. How to use:

1. Tick the upgrades you want like on Maxroll
2. Put in how much materials you have
3. Press **Find chance of success**
4. This will tell you how likely you are to succeed, and what are the bottlenecks(the bottlenecks are not mutually exclusive, so they may add up to more than the chance of failure)

## How it works

We simulate ~100000 tries at passing everything, record how much everything costed and draw some conclusions. If your budget says you have a 69% chance of success, then you suceeeded in 69000 out of 100000 universes.

For Chance to Cost, we generate some artificial budgets and see which ones match your desired chance best using the same principle as above. We balance the luck across pieces with these artificial pieces(this is to avoid the scenario where you one tap armor but pity weapon, which would yield a very high red rocks cost but a very low blue rock cost).

## WIP

### FEATURES

- Special honing and juice considerations(WIP)(need to add: input UI like maxroll for mat prices, toggle between maximizing special leap value and maximizing chance to succeed,)
- Remember ticks & cost input - save presets
- Gamba simulator section
- Visualize distribution of cost(7 lines/bars showing distribution of each, maybe scatter graph?)
- Incorporate roster income tracking & pretty graphs

### DEV

- Get silver unlock cost somewhere(currently silver unlock cost is 0 so its labeled as WIP)(somewhere = maxroll probably)
- Fine tuning (+1 -1 when adjusting best_budget)(need to implement in rust)
- rework App.tsx, split into funcitons or something there has to be a better way
- Optimize monte carlos - can pre-generate special leaps taps like tap map, move the current tap map into the loop

### UI

- UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI
- Interactable cells for input & output, like spreadsheet
- Icons/color code materials & pieces
- Make the pecent sign follow the input number?(right now it just trails on the right)
- Stricter input control
- show the bottlenecks in a pie chart?
- mobile dragging
- dark mode(the input goes white rn, and when it loads theres a white flash)
- maybe use some kind of existing library to make everything less ugly
