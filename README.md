# Honing forecast(working name)

The [Maxroll upgrade calculator](https://maxroll.gg/lost-ark/upgrade-calculator) gives us a rough estimate on how much mats we need to pass certain upgrades, however it only gives the best, average and worst scenarios. This calculator aims to improve on this.

## Chance to Cost

How to use:

1. Tick the upgrades you want like on Maxroll(UI needs WORK)
2. Put in **Desired chance of success**
3. Press **Find chance of success**
4. That's about it

## Cost to Chance

This is a functionality that Maxroll does now currently have. How to use:

1. Tick the upgrades you want like on Maxroll
2. Put in how much materials you have
3. Press **Find chance of success**
4. This will tell you how likely you are to succeed, and what are the bottlenecks(the bottlenecks are not mutually exclusive, so they may add up to more than the chance of failure)

## How it works

We pull a Dr.Strange and simulate 100000(can prolly do more when i fix the code) tries at passing everything, record how much everything costed and draw some conclusions. For Chance to Cost, we generate some artifical "budgets", which balances the luck on every piece(so we don't get a case of where you pity your weapon but one tap every armor, which would yield a super high red rocks cost and low blue rocks cost) and see which one matches your Desired chance the best.

## WIP

- Cache, rework the monte carlos to do individual pieces so they can be cached for other combinations of ticks(and add them when needed)
- Actually implement special honing and juice considerations(Currently the inputs don't do anything)
- Incorporate rosted income tracking & pretty graphs(long term)
- UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI UI
- Draggable tick boxes, same as maxroll
- Interactable cells for input & output, like spreadsheet
- Labels for normal honing (+1,+2...), Labels for advanced honing, Labels for pieces
- Move advanced honing to align with normal honing, move everything else down
- Icons/color code materials
- Align buttons & input fields so the text dont go into the input boxes
- Add a % behind the cursor for desired chances of success
- Disallow non-number inputs
- prettify everything everythign is so ugly right now
