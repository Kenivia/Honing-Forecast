# Honing Forecast

The [Maxroll upgrade calculator](https://maxroll.gg/lost-ark/upgrade-calculator) gives us a rough estimate on how much mats we need to pass certain upgrades, however it only gives the best, average and worst scenarios - this calculator works out everything in between.

## Chance to Cost

This improves on the Maxroll calculator. How to use:

1. Tick the upgrades you want(same as Maxroll)
2. Put in **Desired chanc**(of success)
3. Press **Find estimated cost**
4. That's about it

## Cost to Chance

This is a functionality that Maxroll does not currently have. How to use:

1. Tick the upgrades you want like on Maxroll
2. Put in how much materials you have
3. Press **Find chance of success**
4. This will tell you how likely you are to succeed, and what are the bottlenecks(the bottlenecks are not mutually exclusive, so they may add up to more than the chance of failure)

## How it works

We pull a Dr.Strange and simulate ~100000 tries at passing everything, record how much everything costed and draw some conclusions. If your budget says you have a 69% chance of success, then you suceeeded in 69000 out of 100000 universes.

For Chance to Cost, we generate some artificial budgets and see which ones match your desired chance best using the same principle as above. We balance the luck across pieces with these artificial pieces(this is to avoid the scenario where you one tap armor but pity weapon, which would yield a very high red rocks cost but a very low blue rock cost).

## WIP

- Special honing and juice considerations(Currently juice inputs for CostToChance don't do anything, even if full on grace is ticked)
- Remember ticks & cost input - save presets
- Gamba simulator section
- Get silver unlock cost somewhere(currently silver unlock cost is 0 so its labeled as WIP)
- Fine tuning for advanced honing(+1 -1 when adjusting best_budget)
- Incorporate roster income tracking & pretty graphs(long term)
- UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI
- Interactable cells for input & output, like spreadsheet
- Icons/color code materials & pieces
- make the pecent sign follow the input number?(right now it just trails on the right)
- stricter input control
- prettify everything everythign is so ugly right now

~~- WRITE TESTS~~
~~- Add comments~~
~~- Align buttons & input fields so the text dont go into the input boxes~~
~~- Add a % behind the cursor for desired chances of success~~
~~- Disallow non-number inputs~~
~~- Draggable tick boxes, same as maxroll~~
~~- Labels for normal honing (+1,+2...), Labels for advanced honing, Labels for pieces~~
~~- Move advanced honing to align with normal honing, move everything else down~~
~~- Cache, rework the monte carlos to do individual pieces so they can be cached for other combinations of ticks(and add them when needed)S~~
~~- Migrate to typescript~~
