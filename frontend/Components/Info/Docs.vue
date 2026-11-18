<script setup lang="ts">
import "@/info.css";
import OptimizerDemo from "./Demo/OptimizerDemo.vue";
</script>

<template>
  <div class="info-html card-shell card-body prose-card">
    <h1>Introduction</h1>
    <p></p>

    <h2>Part 1: What are we minimizing?</h2>
    <p>
      We want to calculate and minimize
      <span class="font-bold text-(--gold)">Average tradable gold spent</span>.
      More specifically we model the following use case:

      <ol>
      <li > You will buy from market if you run out of material (aka you want to hit these upgrades NOW)</li>
      <li> You don't care about any leftover bound material (you can configure this)</li>

    </ol>

    </p>

    <p>

      In an ideal world I would've loved to model more 

    We chose these assumptions not necessarily because it's the most applicable to everyone,
    but because it's the only case we can actually compute. In fact
    in the optimization step we will make even more assumptions to keep things within 
    the realm of reasonable compute time. 
    This means that we give up the case of "honing until we run out of x"


      <!-- <span class="font-bold">
        We assume any leftover bound material is NOT worth any gold (unless you
        configure otherwise)</span
      >. -->
    </p>

    <OptimizerDemo />

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
