# Honing Forecast

The [Maxroll upgrade calculator](https://maxroll.gg/lost-ark/upgrade-calculator) gives us a rough estimate on how much mats we need to pass certain upgrades, however it only gives the best, average and worst scenarios - this calculator works out everything in between.

Try it on the [website](https://kenivia.github.io/Honing-Forecast/)!

## Chance to Cost

This improves on the Maxroll calculator. How to use:

1. Tick the upgrades you want(same as Maxroll)
2. Put in **Desired chance**(of success)
3. That's about it

## Cost to Chance

This is a functionality that Maxroll currently does not have. How to use:

1. Tick the upgrades you want like on Maxroll
2. Put in how much materials you have
3. This will tell you how likely you are to succeed, and what are the bottlenecks.

## How it works

We simulate ~100000 tries at passing everything, record how much everything costed and draw some conclusions. If your budget says you have a 69% chance of success, then you suceeeded in 69000 out of 100000 universes.

For Chance to Cost, we generate some artificial budgets and see which ones match your desired chance best using the same principle as above. We balance the luck across pieces with these artificial pieces(this is to avoid the scenario where you one tap armor but pity weapon, which would yield a very high red rocks cost but a very low blue rock cost).

## WIP

### FEATURES

- Gamba simulator section
- Incorporate roster income tracking
- Book support
- Variance calculation & confidence interval

### DEV

- Write actual tests and also comment
- Get silver unlock cost somewhere(currently silver unlock cost is 0 so its labeled as WIP)(somewhere = maxroll probably)

### UI

- Make the percent sign follow the input number?(right now it just trails on the right)
- mobile dragging?
- maybe add tabs to switch between different functionalities
- ctrl z support, fix backspace & delete
