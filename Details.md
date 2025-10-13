# Details, last updated 2025/10/13

This calculator primarily relies on random samples. We generate "cost data" consisting of ~100k samples, where each sample is the cost of all selected upgrades. We will be using this to evaluate how good a given budget is. We compare the budget against a sample, if any of the mats are below the cost, we mark it as a fail. We repeat this for all ~100k data sets and obtain a "chance of success".

## Basic mode

The question is: **"How much mats do I need, to have x % chance of success?"** Turns out there's multiple answers for this - you can one tap your armor and pity your weapon, then you only need a little bit of blue but a LOT of everything else.

In Basic mode, we assume that every material falls in the same percentile(its actually split into 1000 buckets). For example, if 69% of peolpe used less than 100k red, 69% of people used less than 200k blue, and 69% of people ...blah blah for all 7 mats - we put these(100k, 200k, ...) together and figure out the combined chance of success of this. We try this for all percentiles. If the result falls closest to your x%, that'll be the result we display.

## Advanced mode

We may have an abundance of one mat and a bottleneck of another. Clearly we should produce a result that increases the bottleneck more than the abundant one, but how exactly?

We first find the optimal way to spend our gold(I'm working on allowing selling mats) to achieve the highest chance of success. Then we increase our gold budget by some increment, up until we can afford to buy all mats til pity. Then we do a similiar thing to Basic mode - we test these arbitrary budgets(that each are the highest chance given some gold) for what percent chance of success they are, and show whatever's the closest to the desired chnace slider.

### The juice, the free taps and the bane of my existence

Due to how our algorithm is implemented, evaluating the effect of a juiced tap on the overall chance of success is exceedingly difficult. This is because changing a juice usage will affect the whole cost_data and that is no beuno.

Currently we resort to simply calculating the expected decrease in mats usage and convert it into gold, then use our juice & free taps accordingly.

This is a work in progress

## Forecast mode

This section repeats what we've done before on a series of extrapolated budgets = what-we-have-now + week-number * income-per-week.

## Dev

This project started in Google sheets as a roster income tracker, then I want comparing my budget with the Maxroll average to see when I'd hit it. At some point I realized that hitting the average doesn't actually tell me my chances of seeing my ilevel go up, so I started implementing the Monte Carlo thing in Google Appscript, which is insultingly slow (although my algorithm back then was also an insult to humanity). When I couldn't stand it anymore I started making my own website, ported the heavy compute to Rust and wrote a front end. I've had 0 experience in front end/GUI before, so that was done with a LOT of help from various AI tools.

I'm still in the process of writing proper comments & tests, I apologize for the mess.

## Contributing

Contributions would be greatly appreciated! If you would like to add this to your own tool, there is no API to speak of (given there's no server) but you're welcome to copy any of code(the rust compute section probably the most useful) as long as you comply with AGPL.
