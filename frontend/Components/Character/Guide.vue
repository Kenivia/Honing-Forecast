<script setup lang="ts">
import { computed, ref } from "vue";
import { RouterLink } from "vue-router";

const dummy_checkbox = ref(true);

const bound_chance_text =
  "Chance to succeed all upgrades within Char-Bound material";
const roster_chance_text =
  "Chance to succeed all upgrades within Roster-Bound material";
const tradable_chance_text =
  "Chance to succeed all upgrades within Tradable material";

const dummy_select = ref(bound_chance_text);

const selected_color = computed(() => {
  return dummy_select.value === bound_chance_text
    ? "var(--bound)"
    : dummy_select.value === roster_chance_text
      ? "var(--roster)"
      : "var(--tradable)";
});
</script>
<template>
  <div class="change-log-card guide-html card-shell card-body">
    <h1>Guide</h1>
    <h2>Step 1: Select your upgrades</h2>
    <img class="mx-auto" src="/Guide/ticks.png" />
    <h2>Step 2: Input your mats situation</h2>

    <p>
      You should untick any mats you aren't going to buy, and input owned
      (char-bound) materials.
    </p>
    <img class="mx-auto" src="/Guide/cost-dist.png" />

    <p>Let me go through the meaning of each column one by one:</p>

    <h3>
      1. The checkboxes
      <input class="pl-2" type="checkbox" v-model="dummy_checkbox" /> on the
      very left
    </h3>

    <p class="pl-10!">
      If you don't plan on buying a certain mat, you should un-check the
      checkbox. The calculations will disregard this material completely.
      (Un-checking has the same effect as putting in like 99999999 owned)
    </p>

    <h3>2. <span class="text-(--bound)">Bound Mats</span></h3>

    <p class="pl-10!">
      This is where you should put your owned
      <span class="text-(--bound)">Char-bound</span> materials. Roster &
      Tradable are on a
      <RouterLink to="/market-mats"> different page</RouterLink>. The difference
      between Bound & Tradable is that any leftover Bound is treated by the
      Optimizer as 0 gold, and any leftover Tradable is treated at market price.
      (There's no difference between Roster & Char bound other than when
      deducting costs.)
    </p>

    <h3>
      3.
      <select
        class="selector"
        v-model="dummy_select"
        :style="{
          color: selected_color,
        }"
      >
        <option>{{ bound_chance_text }}</option>
        <option>{{ roster_chance_text }}</option>
        <option>{{ tradable_chance_text }}</option>
      </select>
    </h3>

    <p class="pl-10!">
      To be even more verbose, it's the chance that you succeed all upgrades
      before running out of
      <span
        :style="{
          color: selected_color,
        }"
        >{{
          selected_color == "var(--bound)"
            ? "char-bound"
            : selected_color == "var(--roster)"
              ? "char-bound AND roster-bound"
              : "char-bound, roster-bound AND tradable"
        }}</span
      >
      material of a particular material type. I couldn't come up with a name
      that could fit in the column title.
    </p>

    <h3>
      4.
      <span class="text-(--average)">Average</span>
    </h3>

    <p class="pl-10!">The average cost of each material.</p>

    <h3>
      5.
      <span class="text-(--gold)">Average Tradable Gold used</span>
    </h3>
    <p class="pl-10!">
      This is rather important: it is
      <span class="font-bold text-(--gold)">
        the average tradable gold spent on the market</span
      >
      if you buy materials when you run out (including juices).
    </p>
    <p class="mt-2 pl-10!">
      The sum of these is what the metric Optimizer will try to minimize (well
      actually it's the gold spent minus the sell value of tradables). Note that
      in most cases this does
      <span class="font-bold">NOT</span> equal to
      <span class="font-bold"> Market price </span> x
      <span class="text-(--average)">Average</span>! What it actually is is
      slightly convoluted, and it's kinda the whole point of this calculator.
      See a somewhat intuitive explanation
      <a
        href="https://www.reddit.com/r/lostarkgame/comments/1qwskt2/optimize_your_free_tap_juice_usage_with_the_new/"
        >in my reddit post</a
      >, <a href="https://github.com/Kenivia/Honing-Forecast">Github Readme</a>,
      or for the more mathematically inclined, I typed up the math in Latex
      <a
        href="https://github.com/Kenivia/Honing-Forecast/blob/main/docs/Saddlepoint%20Approximation.pdf"
      >
        here</a
      >.
    </p>

    <h3>6. The graphs</h3>

    <p class="pl-10!">
      Hover over the graphs to see what they mean. This is a generalized version
      of column 3.
    </p>
    <p class="mt-4">
      All these above assume that you follow the instructions below.
    </p>
    <h2>Step 3-1: The Instructions</h2>

    <p>
      We'll get to the right half of the instructions later, for now let's focus
      on the left side:
    </p>

    <img src="/Guide/instructions.png" class="mx-auto" />

    <p>In this example, you should:</p>
    <ol>
      <li>
        Use special leaps on +20 Shoulder, until you run out of special leaps or
        succeed
      </li>
      <li>Use special leaps on +20 Chest</li>
      <li>
        do NOT use special leaps on +19 Helmet. Use full juice & book on the
        first 9 taps, then only book on the next 14 taps
      </li>
      <li>Use special leaps on +20 Helmet</li>
    </ol>
    <p class="mt-2">
      The order we do this is important but also partly arbitrary. The only
      thing that matters is the order of the free taps, the non-free taps are
      just there to make sure you can reach them correctly. Furthermore, the
      order of identical pieces is arbitrary and can be swapped / changed around
      as long as there's no depedency issues.
    </p>
    <p class="mt-2">
      All this to say, there are many other equivalent orderings, but if in
      doubt, follow the order given.
    </p>

    <h2>Step 3-2: Updating your progress as you tap</h2>

    <p>
      For upgrades that you are supposed to free tap (and if you succeeded),
      input how much special leaps you have left, then press
      <span class="text-(--achieved)">Succeed</span> like so:
    </p>
    <img src="/Guide/freetap.png" class="mx-auto" />
    <p>
      If you failed all free taps, then press
      <span class="text-(--free-tap)">All free taps failed</span>.
    </p>
    <p class="mt-4">
      Similarly, for an non-free tap upgrade, input how many taps it took to
      succed using the slider, then press
      <span class="text-(--achieved)">Succeed</span>:
    </p>

    <img src="/Guide/nonfreetap.png" class="mx-auto" />
    <p>
      If you want, you can also update your progress before succeeding by
      pressing Confirm. For example, if you tapped 10 times and havn't succeeded
      yet, then put 10 and press Confirm.
    </p>

    <p class="mt-4">
      Using the slider will automatically deduct costs from your owned budgets
      (bottoming out at 0). If you aren't starting an upgrade from 0 artisan,
      you can input your artisan directly into the textbox instead.
    </p>

    <p class="mt-20 pl-0!">
      If you've got a quesiton, or think something isn't right, feel free to get
      in touch via
      <a href="https://discord.gg/KWDpQyvgzc"> Discord</a>! (or check out the
      <a href="https://github.com/Kenivia/Honing-Forecast"> GitHub</a>)
    </p>
  </div>
</template>

<!-- this style cannot be scoped cos it needs to go deep into the html -->
<style>
.change-log-card {
  width: min(100%, 1000px);
  padding: 18px;
  height: max-content;
}
.guide-html h1 {
  font-size: 2.5rem;
  font-weight: 600;
  margin-bottom: 1rem;
}

.guide-html h2 {
  margin-top: 1.5rem;
  margin-bottom: 0.5rem;
  font-size: x-large;
  font-weight: 500;
  border-bottom: 1px solid var(--border-main);
}

.guide-html h3 {
  margin-top: 0.5rem;
  margin-bottom: 0.125rem;
  padding-left: 1.25rem;
  font-size: large;
  font-weight: 500;
}

.guide-html a {
  color: var(--gold);
  text-decoration: underline;
}
.guide-html ol {
  list-style-type: decimal;
  padding-left: 3.25rem;
}

.guide-html ul {
  list-style-type: " - ";
  padding-left: 3.25rem;
}

.guide-html p {
  padding-left: 1.25rem;
}
</style>
