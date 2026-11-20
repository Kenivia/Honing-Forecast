# Honing Forecast

[![discord](https://img.shields.io/discord/1477278907655786611?logo=discord)](https://discord.gg/KWDpQyvgzc)

Honing Forecast is a Lost Ark honing calculator that takes into account owned mats and generates instructions that minimizes average gold used.

Try it here! <https://honing-forecast.com/>  

## Overview

This is the markdown version of the [overview page on the site](https://honing-forecast.com/info/overview), which includes interactable demo.

### 1. What is being minimized?

We want to calculate and minimize **Average tradable gold spent**. More specifically we model the following scenario:

1. You will buy from market if you run out of material (aka you want to hit these upgrades NOW)
2. You don't care about any leftover bound material (unless you configure otherwise)

We chose these assumptions not necessarily because it's the most applicable to everyone, but because it's the only case we can actually compute in a reasonable amount of time. The hope is that even if your use case doesn't completely match this, it's still good advice.

There's a demo of 2 upgrades that you can play around with [on the site](https://honing-forecast.com/info/overview).

### 2. How are the advices generated?

In the real game, you can change your plan after seeing what actually happens in game (e.g. Stop using juice after you run out). However having a rock-cutting-like bot that tells you what to do every step is 1) kind of annoying to use and 2) I don't even know how to start making such an algorithm. Therefore, we do the following:

1. We must come up with a "plan" at the beginning given the information at hand.
2. We score the plan by considering all outcomes and taking the average.

Of course as players we don't have to follow the plan to the end - we can update the optimizer at any point (as often as you can be bothered to) and the optimizer will give us a more informed plan. Empirically however the gain is quite small (median -0.3% by updating after every tap vs not updating at all), so it's really not that big of a deal at all.

#### How does the optimizer work?

The core algorithm is [Simulated annealing](https://en.wikipedia.org/wiki/Simulated_annealing). I tried a LOT of things but SA with a few post-processing steps seemed to work best. To give some typical examples (I promise these weren't cherry picked), it scores the following:

|                  | 8 upgrades | 22 upgrades  | 24 upgrades |
| ---------------- | ---------- | ------------ | ----------- |
| Optimizer result | 270,139.9  | 17,216,792.2 | 7,371,376.4 |
| Best known       | 269,902.5  | 17,214,843.7 | 7,370,569.9 |
| Gap              | 237.4 gold | 1,948.5 gold | 806.5 gold  |
| Gap / best       | 0.088%     | 0.011%       | 0.011%      |

The actual metric we use to evaluate the optimizer is actually Gap / (Avg gold cost if you had NOTHING), because as we will mention very shortly, what we optimize for is affected by the value of leftover materials and thus can be very close to 0, which inflates the metric non-uniformly.

### 3. What do the "leftover values" mean?

In part 1, I said the metric we optimize for is **Average tradable gold spent**, well I lied and it's actually:

> **Average tradable gold spent** - Value of leftover materials

By default (for "main"s) we treat char-bound and roster-bound as worthless, and tradable as the market price minus the 5% tax.
There's a graph of "how much am I losing" when spending materials of each type [on the site](https://honing-forecast.com/info/overview) that illustrates the point better than I can in words here.

I should note that raw Gold (used at the honing npc) is treated as just another material that happens to cost 1 gold each.

### 4. What's different about Honing Forecast?

As far as I know, at least for now, all other Lost Ark calculators compute their "Average gold cost" like so:

1. *Find the average cost of each material*
2. *Subtract how much we have, ignore any negatives*
3. *Multiply by the price.*

*^ (This is incorrect) ^*

This breaks down very quickly if you think about it. For example, for a +15 weapon, it costs ~342 fusion on average. Suppose we have 342 bound fusion on hand, then:

1. If we get lucky, we have bound fusion leftover, netting us no gold,
2. If we get unlucky, we have to buy fusion, costing us gold.

Clearly the average gold used should be some positive number, but the naive method would tell us that the average cost is **0 gold**. (In fact it should actually be **~96.5 fusion's worth** of gold.)

To be clear, what is calculated by the naive method is "What is the gold cost if you hit exactly the average scenario", when we should calculate "If you asked a million people to do this upgrade and average their gold costs, what would you get". As you can see above the naive method is not a good approximation for the latter. In math lingo this is a brilliant display of [Jensen's Inequality](https://en.wikipedia.org/wiki/Jensen%27s_inequality), which states that "f(the average of X) is NOT equal to The average of f(X) ". Here the correct one to compute is the latter.

#### What we calculate instead

1. For all outcomes (All combinations of 1 tap, 2 tap ... pity of every upgrade), find the material cost.
2. (For each outcome) Subtract how much we have, then turn any leftovers to 0.
3. Multiply by the price
4. Take the weighted average over all outcomes

So the difference is that we do the subtraction (aka apply "f") prior to averaging. This makes the average significantly more difficult to compute, and we use some very clever maths & algorithms (I will write this up soon™) to do so in a reasonable amount of time instead of using the brute force method described above.

## How to use

See the [guide on the site](https://honing-forecast.com/info/guide)

## Contributing

The computation is in Rust which runs via wasm, and the frontend is in Vue3 and Typescript.  You'll need the following:

1. [Rust](https://rust-lang.org/tools/install/)
2. [wasm-pack](https://wasm-bindgen.github.io/wasm-pack/)
3. [Node.js](https://nodejs.org/en/download)
4. [pnpm](https://pnpm.io/installation)

Run `pnpm install` for the dependencies, then `pnpm run dev` for the dev server.

You can take a look at the [WIP doc](/public/WIP.md), but any help will be greatly appreciated!

## Bug report / feedback

If you find a bug, spot an error or have a suggestion, please [submit an issue](https://github.com/Kenivia/Honing-Forecast/issues) on GitHub or send a message on [Discord](https://discord.gg/KWDpQyvgzc)

## Disclaimer

Honing Forecast not affiliated with Smilegate and Amazon Games Studio. Copyrights for game-related images and names belong to SG / AGS. The rest is open source and [licensed](/LICENSE) under AGPLv3.
