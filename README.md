# Honing Forecast

[![discord](https://img.shields.io/discord/1477278907655786611?logo=discord)](https://discord.gg/KWDpQyvgzc)

Honing Forecast is a Lost Ark honing calculator that computes your average gold spent based on your owned mats, and suggests a free tap & juice usage plan that minimizes this average.

Existing calculators don't (or incorrectly) take into account your bound mats, especially bound juice & books. This calculator tells you how to best use free taps and juice. It saves anywhere from 1% to 20%+ gold, depending on your use case and what you compare it to.

Try it here! <https://honing-forecast.pages.dev/>  *(It works better on desktop)*  

## Table of Contents

- [What we're calculating](#what-were-calculating)
- [How to use](#how-to-use)
- [Resources](#resources)
- [Contributing](#contributing)
- [Feedback](#bug-report--feedback)

## What we're calculating

The question we want to ask is simple:

### **How much gold will the upgrades cost, on average?**

In a vacuum, the answer is also simple - we multiply the average cost of each mat with their prices and call it a day. As far as I know this is what other calculators do. However, in Lost Ark we have **bound mats**, which has the following implications:

1. We only need to pay gold if we run out of bound mats,
2. Any bound mats leftover after the upgrade cannot be converted to gold.

This means that we have a non-linear function relating material used -> gold cost incurred.As such, **Overall Average gold cost of upgrades** =/= **Gold cost of average material used**. There's an intuitive explanation for this, namely that lucky scenarios pull down average gold cost incorrectly. Allow me to demonstrate:

### Why we can't just calculate Average cost minus bound mats

Suppose we have a standard dice, that tells us the amount of material consumed, and that we have *3* bound mat. Then we have the following:

| (Bound mat owned = 3) | Material consumed | Actually needed |
|-----------------------|-------------------|-----------------|
|                       | 1                 | 0               |
|                       | 2                 | 0               |
|                       | 3                 | 0               |
|                       | 4                 | 1               |
|                       | 5                 | 2               |
|                       | 6                 | 3               |
| Average               | 3.5               | 1               |

Notice that 3.5 -3 = 0.5 (naive calculation) =/= 1 (correct answer). The discrepency comes from the 1st and 2nd cases - we used less than our budget, but the naive calculation includes it in the average, which pulls down the average incorrectly.

As such, let us rephrase the question we set out to answer:

### **What's the average gold cost inccurred due to exceeding bound budget?**

This seemingly innoculous descrepency causes the computation to be significantly more difficult and thus the solution more sophisticated. For a more precise (and complete) definition of the problem and how we tackle it, see the [white paper](/docs/Saddlepoint%20Approximation.pdf).

## How to use

**1. Tick your upgrades, and input how much **untradable** mats you have.**

![Inputs](<https://i.redd.it/hyv473ghlphg1.png?width=1117&format=png&auto=webp&s=21d1ad94740acfbbd258f772281df2b692942e1c>)

**2. Press the big yellow button. Once it's done, it'll tell you how much things will cost.**

![Cost distribution](https://i.redd.it/mqu1pelvlphg1.png?width=1207&format=png&auto=webp&s=2252f4b4086c582dbc8f31e59ad72f0fb6821eb8)

**3. And here's how to use your free taps & juices:**

![Juice instructions](https://i.redd.it/nmgydnrylphg1.png?width=1009&format=png&auto=webp&s=f5856fe8367b2f3f9c14f8f4a0f96f7dcb9e9c5e)

**4. (Optional) As you fail/succeed taps, you can update your progress and the optimizer will consider your new situation.**

## Resources

- [Overview (reddit post)](http://reddit.com/r/lostarkgame/comments/1qwskt2/optimize_your_free_tap_juice_usage_with_the_new/)
- [Implementation details of the average evaluation](/docs/Average%20Evaluation.md)
- [Math behind the average evaluation (How we utilize saddlepoint approximation)](/docs/Saddlepoint%20Approximation.pdf)
- [Optimizer (simulated annealing)](/docs/Optimizer.md)
- [Work in progress](/docs/WIP.md)

For anything else, you can ask me in the [Discord server](https://discord.gg/KWDpQyvgzc)!

## Contributing

You'll need the following:

1. [Rust](https://rust-lang.org/tools/install/) (this should be all you need if you just want to improve the [optimizer engine](/docs/Optimizer.md))
2. [wasm-pack](https://drager.github.io/wasm-pack/installer/)
3. [Node.js](https://nodejs.org/en/download)

Run `npm install` for the dependencies, then `npm run dev` for the dev server.

You can take a look at [WIP](/docs/WIP.md), but any help will be greatly appreciated!

## Bug report / feedback

If you find a bug or spot an error, please submit an issue on GitHub or send a message on [Discord](https://discord.gg/KWDpQyvgzc)!

## Disclaimer

Honing Forecast not affiliated with Smilegate and Amazon Games Studio. Copyrights for game-related images and names belong to SG / AGS. The rest is open source and [licensed](/LICENSE) under AGPLv3.
