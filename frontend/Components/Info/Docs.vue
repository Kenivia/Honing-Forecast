<script setup lang="ts">
import "@/info.css";
import OptimizerDemo from "./Demo/OptimizerDemo.vue";
import LeftoverDemo from "./Demo/LeftoverDemo.vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import { computed } from "vue";

// Near-mean default runs on real3 dev singles (gold spent = -metric)
const examples = [
  { upgrades: 8, result: 270139.9, best: 269902.5, cpu: 1.2 },
  { upgrades: 22, result: 17216792.2, best: 17214843.7, cpu: 0.7 },
  { upgrades: 24, result: 7371376.4, best: 7370569.9, cpu: 0.4 },
];
// Serca +15 weapon, no juice: E[max(0, fusion used - 342.5)], from the tap rule
const FUSION_SHORTFALL = 96.4667;
const roster = useRosterStore();
const fusion_example_gold = computed(() => FUSION_SHORTFALL * roster.active_effective_prices["Serca Fusion"]);
const num = (x: number, digits = 1) => x.toLocaleString(undefined, { minimumFractionDigits: digits, maximumFractionDigits: digits });
</script>

<template>
  <div class="info-html card-shell card-body prose-card">
    <h1>Overview</h1>


    <h2>1. What is being minimized? </h2>
    <p>
      We want to calculate and minimize
      <span class="font-bold text-(--gold)">Average tradable gold spent</span>.
      More specifically we model the following scenario:

      <ol>
      <li > You will buy from market if you run out of material (aka you want to hit these upgrades NOW)</li>
      <li> You don't care about any leftover bound material (unless you configure otherwise)</li>

    </ol>

    </p>

    <p>

    We chose these assumptions not necessarily because it's the most applicable to everyone,
    but because it's the only case we can actually compute in a reasonable amount of time. The hope is 
    that even if your use case doesn't completely match this, it's still good advice.



    </p>

    <p>
Here's a very strippped down demo of 2 upgrades that you can play around with:

    </p>
<p>
    <OptimizerDemo />
</p>
<p>
  This leads us to the next step:
</p>

<h2>2. How are the advices generated? </h2>
<p>
In the real game, you can change your plan after seeing what actually happens in game (e.g. Stop using juice after you run out).
However having a rock-cutting-like bot that tells you what to do every step is 1) kind of annoying to use
and 2) I don't even know how to start making such an algorithm. Therefore, we do the following:

<ol>
      <li >We must come up with a "plan" at the beginning given the information at hand. </li>
      <li >We score the plan by considering all outcomes and taking the average.  </li>
    </ol>

    Of course as players we don't have to follow the plan to the end - we can update the optimizer at any point 
   (as often as you can be bothered to) and the optimizer will give us a more informed plan.
 Empirically however the gain is quite small (median +0.3% by updating after every tap vs not updating at all), so  
 it's really not that big of a deal at all.

</p>

<h3>How does the optimizer work?</h3>
<p>
  The core algorithm is   <a href="https://en.wikipedia.org/wiki/Simulated_annealing"> Simulated annealing</a>. 
  I tried a LOT of things but SA with a few post-processing steps seemed to work best. 

  To give some typical examples (I promise these weren't cherry picked), it scores the following:
</p>
<table>
  <thead>
    <tr><th></th><th v-for="e in examples" :key="e.upgrades">{{ e.upgrades }} upgrades</th></tr>
  </thead>
  <tbody>
    <tr><th>Optimizer result</th><td v-for="e in examples" :key="e.upgrades">{{ num(e.result) }}</td></tr>
    <tr><th>Best known</th><td v-for="e in examples" :key="e.upgrades">{{ num(e.best) }}</td></tr> 
    <tr><th>Gap</th><td v-for="e in examples" :key="e.upgrades">{{ num(e.result - e.best) }} gold</td></tr>
    <tr><th>Gap / best</th><td v-for="e in examples" :key="e.upgrades">{{ num((100 * (e.result - e.best)) / e.best, 3) }}%</td></tr>
   </tbody>
</table>

<p>The actual metric we use to evaluate the optimizer
  is a actually Gap / (Avg gold cost if you had NOTHING), 
  because as we will mention 
  very shortly,
  The actual metric we optimize for considers the value of 
  leftover materials and thus can be very close to 0, which
  inflates the metric non-uniformly.

</p>

<h3>3. What do the "leftover values" mean? What does it mean to be a main or alt?</h3>
<p>
In part 1, I said the metric we optimize for is
 <span class="font-bold text-(--gold)">Average tradable gold spent</span>,
 well I lied and it's actually:
</p>

<p class="m-auto w-fit">

 <span class="font-bold text-(--gold)">Average tradable gold spent</span> - Value of leftover materials

</p>
<p>
  By default (for "main"s) we treat char-bound and roster-bound as worthless, and
  tradable as the market price minus the 5% tax. I like to think of this as 
  a graph of "how much am I losing", when spending materials of each type:
</p>

<LeftoverDemo>
  You can configure what the gradient of each segment on the Calc page as well.
</LeftoverDemo>

<p>
I'm not sure how much sense this graph makes - you can think of tapping in-game as landing
somewhere on the x axis, and the line tells you how much you lose. e.g. if you land on the left,
you can afford with bound mats, hooray.

</p>
<p>
  I should note that raw Gold (used at the honing npc) is treated as just another
   material that happens to cost 1 gold each.
</p>

<h3>
  4. What's different about Honing Forecast?
</h3>

<p>

  As far as I know, at least for now, all other Lost Ark calculators compute their "Average gold cost" like so:
</p>

 <ol>
      <li class="italic">Find the average cost of each material</li>
      <li class="italic">Subtract how much we have, ignore any negatives</li>
      <li class="italic">Multiply by the price.</li>
    </ol>
<p class="italic">^ (This is incorrect) ^</p>

<p>
  This breaks down very quickly if you think about it. For example, for a +15 weapon, it costs ~342 fusion on
  average. Suppose we have 342 bound fusion on hand, then:
  
  <ol>
    <li>  If we get lucky, we have bound fusion leftover, netting us no
  gold, </li>
  <li>If we get unlucky, we have to buy fusion, costing us gold. </li>
  </ol>
Clearly the average gold used should be some
  positive number, but the naive method would tell us that the average cost is <span  class="font-bold text-(--gold)">0 gold
  </span>. (In fact it should actually be <span class="font-bold text-(--gold)">{{ num(fusion_example_gold, 0) }} </span> gold.
</p>

<p>
  To be clear, what is calculated by the naive method is "What is the gold cost if you hit exactly the average scenario",
  when we should calculate "If you asked a million people to do this upgrade and average their gold costs, what would you get".
   As you can see above the naive method is not a good approximation for 
   the latter. In math lingo this is a brilliant display of
   <a href="https://en.wikipedia.org/wiki/Jensen%27s_inequality">Jensen's Inequality</a>, which states that 
   "f(the average of X) is NOT equal to The average of f(X) ". Here the correct one to compute is the latter.
  
</p>


<h3>What we calculate instead</h3>

 <ol>
      <li>
        For all outcomes (All combinations of 1 tap, 2 tap ... pity of every
        upgrade), find the material cost.
      </li>
      <li>
        (For each outcome) Subtract how much we have, then turn any leftovers to
        0.
      </li>
      <li>Multiply by the price</li>
      <li>Take the weighted average over all outcomes</li>
    </ol>

<p>
  So the difference is that we do the subtraction (aka apply "f") prior to averaging. This makes
  the average significantly more difficult to compute, and we use some
  very clever maths & algorithms (I will write this up soon™) to do so in a reasonable amount of time
  instead of using the brute force method described above.
</p>



<!-- <p>
  This allows us to consider the bound mats and juices that the player owns and make a much better suggestion as to
  how to spend them.
</p> -->

<!-- <p>
  We evaluate the algorithm by the metric (Difference from best known result) divided by (average gold cost if 
  you had to buy every mat). We normalize by this because the actual output (Avg tradable gold used) can be very close to 0 and 
  can inflate the metric. Our optimizer scores an average of -0.00013 on real user scenarios (This translates to saving )
</p> -->
    <!-- <ol>
      <li class="italic">Find the average cost of each material</li>
      <li class="italic">Subtract how much we have, ignore any negatives</li>
      <li class="italic">Multiply by the price.</li>
    </ol>
    <p class="italic">^ (This is incorrect) ^</p>

    <p>
      This breaks down very quickly if you think about it. For example, for a
      +25 weapon, it costs ~2.15m shards on average. Suppose we have 2.15m bound
      shards on hand. If we get lucky, we have bound shards leftover, netting us
      no gold; If we get unlucky, we have to buy shards, costing us gold.
      Clearly the average gold used should be some positive number, but the
      above would tell us that the average cost is 0.
    </p>

    <h3>What we calculate instead</h3> -->
    <!-- Naively (and I believe this is what other calculators do), you might think
      to do the following: -->

    <!--       
    <ol>
      <li>
        For all outcomes (All combinations of 1 tap, 2 tap ... pity of every
        upgrade), find the total material cost.
      </li>
      <li>
        (For each outcome) Subtract how much we have, then turn any leftovers to
        0.
      </li>
      <li>Multiply by the price</li>
      <li>Take the weighted average over all outcomes</li>
    </ol>
    <p>
      So the difference is that we do the subtraction prior to averaging and
      consider leftover bound mats as 0. This makes the average significantly
      more difficult to compute and we use some very clever maths to do so in a
      reasonable amount of time (instead of what's described above).
    </p> -->
  </div>
</template>
