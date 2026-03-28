# Honing Forecast

[![discord](https://img.shields.io/discord/1477278907655786611?logo=discord)](https://discord.gg/KWDpQyvgzc)

Honing Forecast is a Lost Ark honing calculator that takes into account owned mats and generates free tap & juice usage instructions that minimizes average gold used.

Try it here! <https://honing-forecast.pages.dev/>  

## Table of Contents

- [What's different about this calculator](#whats-different-about-this-calculator)
- [How to use](#how-to-use)
- [Resources](#resources)
- [Contributing](#contributing)
- [Feedback](#bug-report--feedback)

## What's different about this calculator

We want to calculate **Average gold used**. Naively (and I believe this is what other calculators do), you might think to do the following:

1. *Find the average mat cost*
2. *Subtract how much we have*
3. *Multiply by the price.*

- *^ (This is incorrect) ^*

This breaks down very quickly if you think about it. For example, for a +25 weapon, it costs ~2.15m shards on average. Suppose we have 2.15m bound shards on hand. If we get lucky, we have bound shards leftover, netting us no gold; If we get unlucky, we have to buy shards, costing us gold. Clearly the average gold used should be some positive number, but the above would tell us that the average cost is 0.

### What we calculate instead (**Average gold used**)

1. For all outcomes, find the material cost (e.g. 1 tap, 2 tap ... pity)
2. (For each outcome) Subtract how much we have, then turn any negatives to 0.
3. Multiply by the price and take the weighted average

So the difference is that we do the subtraction prior to averaging, and that we consider leftover bound mats as 0. This makes the average significantly more difficult to compute, and we use some [very clever maths](/docs/Saddlepoint%20Approximation.pdf) to do so in a reasonable amount of time (instead of what's described above).

This allows us to consider the bound mats and juices that the player owns and make a much better suggestion as to how to spend them.

## How to use

**1. Tick your upgrades, and input how much char-bound mats you have.** (you may optionally also input you roster bound & tradable mats on a separate page)

![Inputs](/docs/Images/Main.png)

**2. Follow the instruction for juice usage.** If you run out, you should buy from market.

![Instructions](/docs/Images/Instructions.png)

Note: Before following the instructions, make sure you have inputted your bound mats or unticked mats you don't plan on buying! For example, if you say you have 0 shards and leave it ticked, the optimizer will assume you're buying all the shards.

## Resources

- [Overview (reddit post)](http://reddit.com/r/lostarkgame/comments/1qwskt2/optimize_your_free_tap_juice_usage_with_the_new/)
- [Math behind the average evaluation (How we utilize saddlepoint approximation)](/docs/Saddlepoint%20Approximation.pdf)
- [Implementation details of the average evaluation](/docs/Average%20Evaluation.md)
- [Optimizer (simulated annealing)](/docs/Optimizer.md)
- [Work in progress](/docs/WIP.md)
- [Frontend](/docs/Frontend.md)
- [All docs](/docs/)

For anything else, you can ask me in the [Discord server](https://discord.gg/KWDpQyvgzc)!

## Contributing

You'll need the following:

1. [Rust](https://rust-lang.org/tools/install/) (this should be all you need if you just want to improve the [optimizer engine](/docs/Optimizer.md))
2. [wasm-pack](https://drager.github.io/wasm-pack/installer/)
3. [Node.js](https://nodejs.org/en/download)

Run `npm install` for the dependencies, then `npm run dev` for the dev server.

You can take a look at [good first issues](<https://github.com/Kenivia/Honing-Forecast/labels/good first issue>) or the [WIP doc](/docs/WIP.md), but any help will be greatly appreciated!

## Bug report / feedback

If you find a bug or spot an error, please submit an issue on GitHub or send a message on [Discord](https://discord.gg/KWDpQyvgzc)!

## Disclaimer

Honing Forecast not affiliated with Smilegate and Amazon Games Studio. Copyrights for game-related images and names belong to SG / AGS. The rest is open source and [licensed](/LICENSE) under AGPLv3.
