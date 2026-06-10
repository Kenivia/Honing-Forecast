# Honing Forecast

[![discord](https://img.shields.io/discord/1477278907655786611?logo=discord)](https://discord.gg/KWDpQyvgzc)

Honing Forecast is a Lost Ark honing calculator that takes into account owned mats and generates free tap & juice usage instructions that minimizes average gold used.

Try it here! <https://honing-forecast.pages.dev/>  

## Table of Contents

- [Introduction](#introduction)
- [How to use](#how-to-use)
- [Resources](#resources)
- [Contributing](#contributing)
- [Feedback](#bug-report--feedback)

## Introduction

We want to calculate and minimize **Average gold used**. Naively (and I believe this is what other calculators do), you might think to do the following:

1. *Find the average mat cost*
2. *Subtract how much we have*
3. *Multiply by the price.*

*^ (This is incorrect) ^*

This breaks down very quickly if you think about it. For example, for a +25 weapon, it costs ~2.15m shards on average. Suppose we have 2.15m bound shards on hand. If we get lucky, we have bound shards leftover, netting us no gold; If we get unlucky, we have to buy shards, costing us gold. Clearly the average gold used should be some positive number, but the above would tell us that the average cost is 0.

### What we calculate instead

1. For all outcomes (All combinations of 1 tap, 2 tap ... pity, for every upgrade), find the material cost.
2. (For each outcome) Subtract how much we have, then turn any leftovers to 0.
3. Multiply by the price and take the weighted average

So the difference is that we do the subtraction prior to averaging and consider leftover bound mats as 0. This makes the average significantly more difficult to compute, and we use some [very clever maths](/docs/Saddlepoint%20Approximation.pdf) to do so in a reasonable amount of time (instead of what's described above).

This allows us to consider the bound mats and juices that the player owns and make a much better suggestion as to how to spend them.

## How to use

See the [guide on the website](https://honing-forecast.pages.dev/Newchar/guide)

## Resources

I explained how things work in-depth in my [Reddit post](http://reddit.com/r/lostarkgame/comments/1qwskt2/optimize_your_free_tap_juice_usage_with_the_new/), which should be a pretty good starting point. I've also written some other docs:

- [Math behind the average evaluation](/docs/Saddlepoint%20Approximation.pdf)
- [Optimizer](/docs/Optimizer.md)
- [Frontend](/docs/Frontend.md)
- [All docs](/docs/)
- [Work in progress](/public/WIP.md)

For anything else, feel free to ask me on [Discord server](https://discord.gg/KWDpQyvgzc)

## Contributing

The frontend is written in Vue3 and Typescript. The computation is in Rust which runs via wasm. You'll need the following:

1. [Rust](https://rust-lang.org/tools/install/)
2. [wasm-pack](https://wasm-bindgen.github.io/wasm-pack/)
3. [Node.js](https://nodejs.org/en/download)
4. [pnpm](https://pnpm.io/installation)

Run `pnpm install` for the dependencies, then `pnpm run dev` for the dev server.

You can take a look at [good first issues](<https://github.com/Kenivia/Honing-Forecast/labels/good first issue>) or the [WIP doc](/public/WIP.md), but any help will be greatly appreciated!

## Bug report / feedback

If you find a bug, spot an error or have a suggestion, please [submit an issue](https://github.com/Kenivia/Honing-Forecast/issues) on GitHub or send a message on [Discord](https://discord.gg/KWDpQyvgzc)

## Disclaimer

Honing Forecast not affiliated with Smilegate and Amazon Games Studio. Copyrights for game-related images and names belong to SG / AGS. The rest is open source and [licensed](/LICENSE) under AGPLv3.
