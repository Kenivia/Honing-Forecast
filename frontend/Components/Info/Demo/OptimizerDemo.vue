<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import Heatmap from "./Heatmap.vue";
import {
  distribution,
  fusion_per_tap,
  juices,
  max_taps,
  NUM_LEVELS,
  upgrade_name,
  type DemoUpgrade,
} from "./demo";
import { chance_text, cssVar } from "@/Utils/Helpers";
import { MATERIALS } from "@/Utils/Constants";
import MaterialGraph from "@/Components/Character/MaterialDist/MaterialGraph.vue";

// Two upgrades' joint tap distribution, and how much fusion each outcome has to buy.
const upgrades = reactive<DemoUpgrade[]>([
  { piece_type: 2, upgrade_index: 14, juice: {} },
  { piece_type: 2, upgrade_index: 15, juice: {} },
]);
const owned = ref(0);

const dists = computed(() => upgrades.map(distribution));
const sizes = computed(() => dists.value.map((d) => d.length - 1));
// axes stay at the no-juice pity so juicing doesn't rescale anything
const axes = computed(() => upgrades.map(max_taps));
const costs = computed(() => upgrades.map(fusion_per_tap));
const names = computed(() => upgrades.map(upgrade_name));
const max_cost = computed(
  () => axes.value[0] * costs.value[0] + axes.value[1] * costs.value[1],
);
const avail = computed(() =>
  upgrades.map((u) => juices(u.piece_type, u.upgrade_index)),
);
watch(max_cost, (m) => (owned.value = Math.min(owned.value, m)));

const prob = computed(() => {
  const [a, b] = dists.value;
  return (x: number, y: number) => (a[x] ?? 0) * (b[y] ?? 0);
});
// log colour range: the least and most likely outcome
const p_range = computed(() => {
  const [a, b] = dists.value.map((d) => d.slice(1).filter((p) => p > 0));
  return [Math.min(...a) * Math.min(...b), Math.max(...a) * Math.max(...b)];
});
const used = (x: number, y: number) => x * costs.value[0] + y * costs.value[1];
const shortfall = computed(() => {
  const [o, [a, b], m] = [owned.value, costs.value, max_cost.value];
  return (x: number, y: number) => Math.max(0, x * a + y * b - o) / m;
});

function mix(hex: string, t: number) {
  const c = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
  const [r, g, b] = c.map((v) => Math.round(255 + (v - 255) * t));
  return `rgb(${r},${g},${b})`;
}
const blue = cssVar("--series-blue", "#6666ff");
const orange = cssVar("--series-fusion", "#ff7824");
const prob_color = computed(() => {
  const [lo, hi] = p_range.value.map(Math.log10);
  return (v: number) =>
    mix(blue, Math.max(0, (Math.log10(v) - lo) / (hi - lo || 1)));
});
const fusion_color = (v: number) => mix(orange, v);

const taps = (x: number, y: number) => [
  `${names.value[0]}: ${x} taps`,
  `${names.value[1]}: ${y} taps`,
];
const pct = (p: number) => `${(p * 100).toPrecision(3)}%`;
const prob_tip = computed(() => {
  const p = prob.value;
  return (x: number, y: number) => [
    ...taps(x, y),
    p(x, y) ? `Chance: ${pct(p(x, y))}` : "Chance: 0 (past pity)",
  ];
});
const fusion_tip = computed(() => {
  const o = owned.value;
  return (x: number, y: number) => [
    ...taps(x, y),
    `Fusion used: ${used(x, y).toLocaleString()}`,
    `To buy: ${Math.max(0, used(x, y) - o).toLocaleString()}`,
  ];
});

// Gold spent = fusion bought at its fallback price, as a CDF on evenly spaced points
// from 0 to the both-pity cost (MaterialGraph's slope view assumes even gaps).
const PRICE = MATERIALS["Serca Fusion"].fallback_prices[0];
const BINS = 100;
const gold = computed(() => {
  const [a, b] = dists.value;
  const [ca, cb] = costs.value;
  const o = owned.value;
  const cells: [number, number][] = [];
  let mean = 0;
  a.forEach((pa, x) =>
    b.forEach((pb, y) => {
      const p = pa * pb;
      if (!p) return;
      const g = Math.max(0, x * ca + y * cb - o) * PRICE;
      cells.push([g, p]);
      mean += g * p;
    }),
  );
  cells.sort((l, r) => l[0] - r[0]);
  const top = max_cost.value * PRICE;
  const cdf: [number, number][] = [];
  let i = 0;
  let cum = 0;
  for (let k = 0; k <= BINS; k++) {
    const x = (top * k) / BINS;
    while (i < cells.length && cells[i][0] <= x + 1e-9) cum += cells[i++][1];
    cdf.push([x, Math.min(1, cum)]);
  }
  return { cdf, mean, spends: cells[cells.length - 1][0] > 0 };
});
const gold_tip = (
  x: number,
  _y: number,
  cy: number,
  _m: string,
  color: string,
) =>
  `<b style="color: var(--text-bright);">${chance_text(cy)}</b> chance to spend<br>≤ <b style="color: ${color};">${Math.round(x).toLocaleString("en-US")}</b> gold`;
</script>

<template>
  <div class="flex flex-col gap-4 pl-5">
    <div class="grid gap-4 md:grid-cols-2">
      <div
        v-for="(u, i) in upgrades"
        :key="i"
        class="flex flex-col gap-2 rounded-sm border border-(--border-very-muted) p-3"
      >
        <div class="flex items-center gap-2">
          <span class="text-(--text-muted)">Upgrade {{ i + 1 }}</span>
          <span>Vambrace</span>
          <select
            v-model.number="u.upgrade_index"
            class="selector bg-(--bg-input)"
            @change="u.juice = {}"
          >
            <option v-for="l in NUM_LEVELS" :key="l" :value="l - 1">
              +{{ l }}
            </option>
          </select>
        </div>
        <span class="text-sm text-(--text-very-muted)">
          Pity {{ sizes[i] }} taps, {{ costs[i] }} fusion per tap
        </span>
        <label
          v-for="j in avail[i]"
          :key="j.id"
          class="flex items-center gap-2 text-sm"
        >
          <span class="w-40 shrink-0">
            {{ j.label }} (+{{ pct(j.chance) }})
          </span>
          <input
            :value="u.juice[j.id] ?? 0"
            @input="u.juice[j.id] = +($event.target as HTMLInputElement).value"
            type="range"
            class="w-full"
            :min="0"
            :max="max_taps(u) - 1"
          />
          <span class="w-28 shrink-0 text-right">
            first {{ u.juice[j.id] ?? 0 }} taps
          </span>
        </label>
      </div>
    </div>

    <label class="flex items-center gap-2">
      <span class="shrink-0">Bound Fusion material owned</span>
      <input
        v-model.number="owned"
        type="range"
        class="w-full"
        :min="0"
        :max="max_cost"
      />
      <span class="w-32 shrink-0 text-right">
        {{ owned.toLocaleString() }} / {{ max_cost.toLocaleString() }}
      </span>
    </label>

    <div class="grid gap-6 md:grid-cols-2">
      <div class="flex flex-col gap-2">
        <span class="font-bold">Chance of each outcome</span>
        <Heatmap
          :nx="axes[0]"
          :ny="axes[1]"
          :value="prob"
          :color="prob_color"
          :tooltip="prob_tip"
          :x_label="`${names[0]} taps`"
          :y_label="`${names[1]} taps`"
        />
        <div
          class="h-3 rounded-sm"
          :style="{ background: `linear-gradient(to right, white, ${blue})` }"
        ></div>
        <div class="flex justify-between text-sm text-(--text-muted)">
          <span>{{ pct(p_range[0]) }}</span>
          <span>log scale</span>
          <span>{{ pct(p_range[1]) }}</span>
        </div>
      </div>
      <div class="flex flex-col gap-2">
        <span class="font-bold">Fusion to buy</span>
        <Heatmap
          :nx="axes[0]"
          :ny="axes[1]"
          :value="shortfall"
          :color="fusion_color"
          :tooltip="fusion_tip"
          :x_label="`${names[0]} taps`"
          :y_label="`${names[1]} taps`"
        />
        <div
          class="h-3 rounded-sm"
          :style="{ background: `linear-gradient(to right, white, ${orange})` }"
        ></div>
        <div class="flex justify-between text-sm text-(--text-muted)">
          <span>0</span>
          <span>{{ max_cost.toLocaleString() }} (both pity, none owned)</span>
        </div>
      </div>
    </div>

    <div class="flex flex-col gap-1">
      <div class="flex items-baseline gap-2">
        <span class="text-lg text-nowrap text-(--gold)">
          Average tradable gold spent:
          {{ Math.round(gold.mean).toLocaleString("en-US") }}
        </span>
        <span class="text-sm text-(--text-very-muted)">
          (Only fusion cost considered)
        </span>
      </div>
      <MaterialGraph
        class="pl-0!"
        :data="gold.spends ? gold.cdf : null"
        material-label="Gold"
        graph-color="--gold"
        :cumulative="false"
        :annotations="[gold.mean]"
        :annotation-colors="['--average']"
        :annotation-positions="['graph']"
        :annotation-labels="['Average']"
        :tooltip-text-fn="gold_tip"
        empty_message="No tradable gold spent"
      />
    </div>
  </div>
</template>
