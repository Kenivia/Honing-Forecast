<script setup lang="ts">
import { RouterLink, useRoute } from "vue-router";

const route = useRoute();
</script>
<template>
  <div class="change-log-card prose card-shell card-body">
    <h1>Guide</h1>
    <h2>Step 1: Select your upgrades</h2>
    <img src="/Guide/ticks.png" />
    <h2>Step 2: Input your mats situation</h2>
    <p>
      You should untick any mats you aren't going to buy, and input owned mats.
    </p>
    <img src="/Guide/cost-dist.png" />
    <h2>
      Step 3: Follow the
      <RouterLink
        style="text-decoration: underline"
        :to="{
          name: 'instructions',
          params: { characterName: route.params.characterName as string },
        }"
        >Taps Instructions</RouterLink
      >
    </h2>
    <p>
      - In the example below, "Book x14" means that you should use book on the
      first 14 taps, then raw tap the rest.
    </p>
    <p>- If you run out of books or juice, you should buy from market.</p>
    <p>
      - All 'Juice' taps are fully juiced (like 25 out of 25 Glacier Essence).
    </p>
    <img src="/Guide/instructions.png" />
  </div>
</template>

<style scoped>
h2 {
  border-bottom: 1px solid var(--hf-text-muted);
}
</style>
