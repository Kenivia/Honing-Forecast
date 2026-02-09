# About

**TLDR: Existing calculators don't (or incorrectly) take into account your bound mats, especially bound juice & books. This calculator tells you how to best use free taps and juice. It saves anywhere from 1% to 20%+ gold, depending on your use case and what you compare it to.**

Try it here! <https://honing-forecast.pages.dev/>  *(It works better on desktop)*  

## How to use

**1. Tick your upgrades, and input how much **untradable** mats you have.**

![Inputs](<https://i.redd.it/hyv473ghlphg1.png?width=1117&format=png&auto=webp&s=21d1ad94740acfbbd258f772281df2b692942e1c>)

**2. Press the big yellow button. Once it's done, it'll tell you how much things will cost.**

![Cost distribution](https://i.redd.it/mqu1pelvlphg1.png?width=1207&format=png&auto=webp&s=2252f4b4086c582dbc8f31e59ad72f0fb6821eb8)

**3. And here's how to use your free taps & juices:**

![Juice instructions](https://i.redd.it/nmgydnrylphg1.png?width=1009&format=png&auto=webp&s=f5856fe8367b2f3f9c14f8f4a0f96f7dcb9e9c5e)

**4. (Optional) As you fail/succeed taps, you can update your progress and the optimizer will consider your new situation.**

## If you decide to trust me, the above steps are the only things you need to know

Otherwise, I'd love to share with you how this thing works.

## Part 1) How is this different from Maxroll / Icepeng?

We must recognize that **Average gold cost =/= (Average mats used - Owned mats ) \* price**. Allow me to demonstrate:

![Maxroll](https://i.redd.it/4wfeh0jgulhg1.png?width=817&format=png&auto=webp&s=c254151dbce97280de8ebc02f4420f90f2d22c2d)

Here, I've inputted that I have exactly the average amount of Shards. However, if we get unlucky, we will need to spend gold to buy shards, while on the other hand if we get lucky we gain no gold from it.  As in, the lucky cases drags down the average shards without reducing our gold used. In fact, in this case it underestimates the average gold spent by **446,723g.**

Do note however, there are exceptions where the naive calculation is accurate, namely:

1. You can sell your leftover mats for the same amount of gold (such as tradable abidos, gold)
2. You have no mat to begin with(buying everything)
3. You have way too much mat(can afford pity)

So the above case is pretty much as bad as it gets - it fails all 3 of these criteria. So Maxroll isn't "wrong" per se, but it makes the assumption that you fall into one of these 3 categories. As it turns out, the alternative is significantly harder to calculate, as we will discuss in the part 2). Furthermore, as we will see in part 3), a single gold value does not fully capture the complexity of the problem.  

## Part 2) How do we calculate it instead?

Let us first establish what it is that we are trying to calculate. Let me try using one of these fancy quote blocks for emphasis:

>**Average equivalent gold spent buying the mats we need** (treating using tradables as if we bought them)

For each mat, we need to consider whether or not it exceeds our (bound) budget. If so, then we multiply what we need by the market price. Otherwise, we chalk it down as 0. Let's use the +25 weapon as the example again:

|(budget = 2,153,402)|Tap No.1|...everything in between...|Tap No.218|Pity|
|:-|:-|:-|:-|:-|
|Shards needed|211,500|...|4,877,000|4,898,500|
|Gold needed|0 (because needed < budget)|...|(4877000 - 2153402) \* 0.7|(4898500 - 2153402) \* 0.7|
|Probabilty to land on this|0.5%|...|0.1161%|11.4952%|

Then we take a weighted average(**multiply 2nd row and 3rd row**) to arrive at the previously mentioned 446,723g. Note that the 3rd row is the probabilty that we fail everything before it, and succeed on that tap. You might be surprised that 11 people out of 100 pity their +25, but this is the cruel world we live in.

##

## Problem: This scales extremely poorly with more upgrades

Suppose we have 2 upgrades, A and B, then we have a lot more cases to consider:

||One tap A|...|Pity-1 A|Pity A|
|:-|:-|:-|:-|:-|
|One tap B|(A=1, B=1)|...|(A=pity-1, B=1)|(A=pity, B=1)|
|...|...|...|...|...|
|Pity-1 B|(A=1, B=Pity-1)|...|(A=pity-1, B=pity-1)|(A=pity, B=pity-1)|
|Pity B|(A=1, B=Pity)|...|(A=pity-1, B=pity)|(A=pity, B=pity)|

It is important to see that we can't calculate these individually and add them - even if we are able to afford pity individually, added together we might not. The gold we need to spend depends on the outcome of every upgrade.

We can then calculate the gold needed for each spot in the grid, multiply by the probabilty that we land on that spot and add them up. If these are two +25 (or +24) upgrades, then this is a square of 219x219 \~ 48k outcomes. If we had 3, this would be a cube of \~10 million outcomes. With about 10 of these upgrades we'll be stuck calculating this until the sun dies. That is around the time that we take a step back and use our noggins.

## Solution: Saddlepoint approximation

Luckily this problem has been solved by some very smart people before us. [Saddlepoint approximation](https://en.wikipedia.org/wiki/Saddlepoint_approximation_method) is an incredibly powerful piece of mathematical machinery, which I am beyond un-qualified to explain. If you want to learn more, there's this [great book](https://books.google.co.uk/books?id=jvlWV2aIk_AC) from which I learnt like everything.

Long story short, I was able to use this method to approximate the result, and I know I've done the math right because it matches experimental results(1 million simulations, which takes about 10 seconds) with eerie accuracy (<0.1% for most cases) within 1 millisecond. It's really cool.

## Part 3) How do we minimize this cost?

From part 2), we have a way to evaluate the **Average gold spent buying mats**(note that raw gold is just another mat that costs 1g), which we want to minimize. However, it isn't necessarily clear what the optimal way to use juice and free taps is, and let me list some examples:

1. If we have enough juice for 5 taps, but if we juice one more, it pushes pity one tap earlier, which can reduce the average cost a lot. So should we buy 1 more juice? What if we need to buy 2 more?
2. If we have enough juice for 5 taps, but we have 2 pieces, is it better to split them between two or focus on one?
3. If we have 2 upgrades like +19 and +20, it might be more worth to free tap +20 in a vacuum, but then we might have free taps leftover (which are kinda useless after Serca comes out). At what point should we attempt +19 first?

Hopefully at this point it's clear that we cannot assign a single gold value to a particular juice / special leap. The difference that a juice makes depends on how many juice you've used before (or after), and there's opportunity cost because you could've used it somewhere else.

Here's a plot I made a while ago, the height / color is the cost, and the x and y axis denote how many juice to use for upgrade 1 and upgrade 2. I think they were like +23 and +25 or something.

![Example plot of cost vs strategy](https://i.redd.it/ha3atjrm9mhg1.png?width=836&format=png&auto=webp&s=d335ea8893420410fb30295da704b22255baaf4f)

We can imagine that this only gets more complex with more upgrades & free taps in the mix. Luckily, again, turns out a lot of people have ran into problems like this in the past.

## Solution: Simulated annealing

Another fancy name with SA as the initial. It perhaps sounds more scary than it is. It involves 3 steps:

1. Make a random change. This could be adding some juice, swapping a free tap order etc.
2. Evaluate how good it is via Part 2).
3. If it's better than the previous one, take it. Otherwise, roll a dice and take it anyway if it's not that bad.

So it's essentially just trying random stuff and seing what sticks. We repeat this many times (x24000) and hope that we get to a good enough solution. This is a [well studied algorithm](https://en.wikipedia.org/wiki/Simulated_annealing), and I used a heuristic (with many optimizations specific to this problem) from [this paper](https://ojs.aaai.org/index.php/SOCS/article/view/18424).

This algorithm is not perfect by any mean, but I find it difficult to come up with better solutions. If you want, you can try to find a better solution by clicking the squares or re-arranging the free tap order. (The squares will be white if it's worse than the best solution known)

## Part 4) How much gold does this save?

The TLDR is that it's marginal, unless you were doing the complete wrong thing (e.g. buying juice / books when you shouldn't, or vice versa).

Let us go through an example - my gunlancer who has been parked way too long and want to go from 1720 - 1730:

![Input example](https://i.redd.it/rrplvkjn5phg1.png?width=1117&format=png&auto=webp&s=174e82bb0018e9c568dd21b741e8b2475cd17246)

Here's the results:

![Example results](https://i.redd.it/l8jlxnhfpphg1.png?width=656&format=png&auto=webp&s=51ba70f9d087c07f59b5a2b2dd6f48f0b5dda796)

There doesn't appear to be any obvious patterns going on. I can sit here and try to explain these behaviour, but the truth is I have no clue why the algorithm chose to do this - it just tried a bunch of configurations and this happens to be the best one it's found. There might be better ones out there, but this is good enough for me.

Let's compare this with Maxroll:

|Strategy|Avg eqv gold cost|Percentage saved|
|:-|:-|:-|
|Following Maxroll\*|474,303|N/A|
|Best known strategy(found by optimizer)|419,567|11.5%|

\* Using the highest gold values suggested, no extra juice bought(because the gold values were below market price). In this case, this means free tapping all +20, juicing the first 5 taps of +20 armors(except 1), and juicing the first 12 taps of +20 weapon.

You don't understand how relieved I am to see that my months of work have not gone to waste. Let's use the same juice focus on the free taps:

|Strategy (same juice)|Avg eqv gold cost|Percentage saved|
|:-|:-|:-|
|\+20 armors -> +20 weapons(Maxroll)|447,805|N/A|
|Best (x3 +19 armors, then +20 armors, then +19 weapon, then +20 weapon)|419,567|6.31%|

So just improving our free taps saves about 6% gold.

## But wait, it get even better

The previous strategies are made with no knowledge of what actually happens in-game. It takes into account all possible outcomes (as is the nature of an average), and the strategy is somewhat of a middleground that's decent for all outcomes. However, as you press the buttons in game it reveals more information and the algorithm can make better adjustments. Allow me to demonstrate (just +19 and +20 weapon, no free taps):  

![Adaptive example](https://i.redd.it/w0mawtt1ephg1.png?width=984&format=png&auto=webp&s=7985c6e0c844d81d4862d1d047ce15c6650d4fd1)

This is the instructions at the beginning. Let's say I failed the first few taps:  

![Adaptive result](https://i.redd.it/w00v58u9ephg1.png?width=991&format=png&auto=webp&s=a1da3b9ee87a3b962b909586f3a6ace9059cd83d)

It decides that it's better to allocate the juice to the +19 (and indeed buy an extra). In this case, doing this adjustment saves about 1.7k gold for me (out of 248k), which isn't significant but isn't nothing either. Rest assured that the initial strategy considers all outcomes, and whatever happens just following that without updating won't be too bad.

## FAQ

**1. What does "Avg eqv gold cost" mean again, and why untradable mats?**

* raw gold spent tapping +
* gold spent on buying any mats you need +
* gold you would've made by selling tradables, if you had any (ignoring tax)

This is how we "punish" the optimizer. If it goes above bound budget, it's punished by how much gold the extra is worth; If it doesn't use up all the bound budget, it gets no rewards(and no punishment). As such it's incentivized to use up as much bound mats as possible.

As such if you have any tradable mats, this is NOT the actual gold you spend. You should see the breakdown of how much each mat contributed at the bottom of the instructions page for a better idea.

Unfortunately the current system does not accomodate having 2 price breakpoint (bound -> tradable -> buy from market), so we consider using tradables the same as buying from market. You should also consider the fact that you can use roster-bound / tradable mats on other characters, which may save you gold. Perhaps a more fitting name is "**Mats that you don't care about if there's some leftover**". If you don't care about your tradable mats for whatever reason, you can include that also, as long as you understand what it means.  

**2. Is this AI?**

There's no neural networks involved (everything has to run locally on your machine). AI is a spectrum from a washing machine to ChatGPT, and I'd say this is smack bang in the middle.

**3. What happened to the previous graphs and stuff?**

The previous graphs did not consider the fact that some leftover mats are un-sellable. Ironically there's no more forecast more for now because I made a lot of changes that broke how it worked.

**4. What about advanced honing?**

This is unable to optimize advanced honing(assumes that we always juice on full grace), but it does show the cost distribution in the first graph.

## Conclusion

How much gold this saves you depend entirely on your use case. Again in my alt's case going from 1720-30, this cut down the cost from \~474k (Maxroll) to \~420k (btw it's this low because I have a lot of free taps). If you were doing more upgrades or buying shards, this is likely to save you hundreds of thousands by telling you how to buy juice correctly. Whether or not this is worth your time is completely up to you. In any case I hope that the pretty graphs on the first page helps you visualize how much mats you need beyond just the average.

I hope that this will be useful for some of you, unfortunately I couldn't get this done in time for Wednesday, but perhaps this will be useful for you in the future on your way to 1730. If I have made a mistake somewhere, I do apologize but I don't have enough time

Again the website is here: [https://honing-forecast.pages.dev/](https://honing-forecast.pages.dev/) and you can check out the [GitHub](https://github.com/Kenivia/Honing-Forecast) (plz gib star)

I'd like to give my thanks to my tutor Tobias, who introduced me to simulated annealing, all my friends who have took the time to listen to me rant and gave me valuable feedback, and thank you for using my website!

[(This was a reddit post)](http://reddit.com/r/lostarkgame/comments/1qwskt2/optimize_your_free_tap_juice_usage_with_the_new/)

## Contributing

You'll need the following:

1. [Rust](https://rust-lang.org/tools/install/) (this should be all you need if you just want to improve the [optimizer engine](/crates/arena/README.md))
2. [wasm-pack](https://drager.github.io/wasm-pack/installer/)
3. [Node.js](https://nodejs.org/en/download)

Then, run `npm install` for the dependencies, then `npm run dev` for the dev server.

You can take a look at [WIP](/WIP.md) to see what you can help with, but any help will be greatly appreciated!
