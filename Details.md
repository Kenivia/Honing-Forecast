# Details, last updated 2025/10/13

This calculator primarily relies on random samples. We generate "cost data" consisting of ~100k samples, where each sample is the cost of all selected upgrades. We will be using this to evaluate how good a given budget is. We compare the budget against a sample, if any of the mats are below the cost, we mark it as a fail. We repeat this for all ~100k data sets and obtain a "chance of success".

## Chance mode

The question is: *"How much mats do I need, to have x % chance of success?"* Turns out there's multiple answers for this - you can one tap your armor and pity your weapon, then you only need a little bit of blue but a LOT of everything else. This makes it quite difficult to give one answer to this question.

In Chance mode, we assume that every material falls in the same percentile(its actually split into 1000 buckets). For example, if 69% of peolpe used less than 100k red, 69% of people used less than 200k blue, and 69% of people ...blah blah for all 7 mats - we put these(100k, 200k, ...) together and figure out the combined chance of success of this. We try this for all percentiles. If the result falls closest to your x%, that'll be the result we display.

This is not perfect as it falsely assumes that we earn/buy mats evenly. This is a work in progress

## Budget mode

There are two sub-modes within this:

### Buying mats with gold

- DONE *"How much gold do I need for x%"* is the central question. This is fairly easy to answer(we rank the cost data by their equivalent gold cost and find the appropriate percentile according to x%). However:
- **WIP** An estimate of how much gold is spent on each mat / how much of each mats will need to be bought. This is a little bit vague, and I'm still working on how to interpret(let alone answer) this.
- DONE? Juice & Free tap usage optimized to minimize gold cost. Currently the we work out the juice/free tap usage by finding the average decrease in gold equivaleent cost, which (i think?) achieve this. This also gives an estimated gold value.
- **WIP** Juice purchasing recommendation given market price / juice box 1-to-3 opening optimization.

### Not buying mats with gold

- DONE *"What's the chance that I finish all the taps before running out of something?"* We simply comepare the given budget against each entry in cost data and record how many succeeded.
- **WIP** juice & free tap usage optimized to maximize this chance of success.

## Forecast mode

This section repeats what we've done before on a series of extrapolated budgets = what-we-have-now + week-number * income-per-week.

## Dev

This project started in Google sheets as a roster income tracker, then I wanted to compare my budget with the Maxroll average to see when I'd hit it. At some point I realized that hitting the average doesn't actually tell me my chances of seeing my ilevel go up, so I started implementing the Monte Carlo thing in Google Appscript, which is insultingly slow - when I couldn't stand it anymore I started making my own website, ported the heavy compute to Rust and wrote a front end. I had 0 experience in front end/GUI before this, so that was done with a LOT of help from various AI tools.

I'm still in the process of writing proper comments & tests, I apologize for the mess.

## Contributing

Contributions would be greatly appreciated! If you would like to add this to your own tool, there is no API to speak of (given there's no server) but you're welcome to copy any of the code(the rust compute section probably the most useful) as long as you comply with AGPL.
