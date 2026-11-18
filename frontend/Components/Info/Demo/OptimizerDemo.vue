<script setup lang="ts">
import { computed, onBeforeUnmount, ref, shallowRef, watch } from "vue";
import Heatmap from "./Heatmap.vue";
import {
  demo_juices,
  demo_payload,
  demo_profile,
  FUSION_LABEL,
  PRICES,
  fusion_per_tap,
  max_taps,
  upgrade_name,
} from "./demo";
import { chance_text, cssVar, get_icon_path } from "@/Utils/Helpers";
import MaterialGraph from "@/Components/Character/MaterialDist/MaterialGraph.vue";
import ActualInstructions from "@/Components/Character/Instructions/ActualInstructions.vue";
import OptimizerRadioButton from "@/Components/Character/OptimizerControlPanel/OptimizerRadioButton.vue";
import { create_worker_bundle } from "@/WasmInterface/WorkerBundle";
import { WasmOp, type HistogramOutputs } from "@/WasmInterface/WasmTypes";
import { states_payload } from "@/WasmInterface/PayloadBuilder";
import { NormalOverride } from "@/WasmInterface/Overrides";
import { DEFAULT_CHAR_PROFILE } from "@/Stores/CharacterProfile";

// Two upgrades' joint tap distribution under a juice strategy, and how much fusion each outcome
// has to buy. The plan comes from the real optimizer (streak length 1) and histogram.
const LEVELS = [10, 9];
const profile = demo_profile(LEVELS);
const owned = ref(0);

const JUICES = demo_juices(LEVELS);
const juice_owned = ref<number[]>(JUICES.map(() => 0));
// bound stock by label
const stock = () => ({
  [FUSION_LABEL]: owned.value,
  ...Object.fromEntries(JUICES.map((j, i) => [j.label, juice_owned.value[i]])),
});
// Custom sends the ticked taps as the state, under the Optimizer override so they pass through
const CUSTOM = -1;
const strategy = ref<number>(NormalOverride.Optimizer);
const STRATEGIES = [
  { value: NormalOverride.Optimizer, label: "Optimal" },
  { value: NormalOverride.Empty, label: "No juice" },
  { value: NormalOverride.Full, label: "Full juice" },
  { value: CUSTOM, label: "Custom" },
];
// juice ids 0 and 1, in JUICES order
const JUICE_IDS = JUICES.map((_, j) => j);
// custom[upgrade][juice][tap]
const custom = ref<boolean[][][]>([]);

const optimizer = create_worker_bundle();
const histogram = create_worker_bundle();
const plan = shallowRef<any>(null);
onBeforeUnmount(() => {
  optimizer.cancel();
  histogram.cancel();
});

function custom_plan() {
  return {
    ...plan.value,
    upgrade_arr: plan.value.upgrade_arr.map((u: any) => {
      const i = LEVELS.indexOf(u.upgrade_index);
      if (i < 0) return u;
      const state = u.state.map((_: number[], t: number) =>
        JUICE_IDS.filter((j) => custom.value[i][j][t]),
      );
      return { ...u, state };
    }),
  };
}

function run_histogram() {
  if (!plan.value) return;
  const is_custom = strategy.value === CUSTOM;
  const s = is_custom ? NormalOverride.Optimizer : strategy.value;
  const override = {
    ...structuredClone(DEFAULT_CHAR_PROFILE.optimizer_override),
    normal: { juice: s, book: s },
  };
  histogram.throttled_start(WasmOp.Histogram, {
    ...states_payload(
      demo_payload(profile, stock()),
      profile,
      is_custom ? custom_plan() : plan.value,
      override,
    ),
    dist_plan: 0,
  });
}
watch(
  [owned, juice_owned],
  () =>
    optimizer.debounced_start(
      WasmOp.OptimizeAverage,
      demo_payload(profile, stock()),
      (result) => {
        if (!result) return;
        plan.value = result;
        run_histogram();
      },
    ),
  { immediate: true, deep: true },
);
watch(strategy, run_histogram);
watch(custom, () => strategy.value === CUSTOM && run_histogram(), {
  deep: true,
});

// Custom starts as a copy of the taps the shown strategy juiced before its pity
function set_strategy(v: number) {
  if (v === CUSTOM && strategy.value !== CUSTOM)
    custom.value = LEVELS.map((_, i) =>
      JUICE_IDS.map((j) =>
        Array.from(
          { length: axes[i] },
          (_, t) =>
            t < sizes.value[i] - 1 && !!shown.value[i]?.state?.[t]?.includes(j),
        ),
      ),
    );
  strategy.value = v;
}

const result = computed(() => histogram.result as HistogramOutputs | null);
const working = computed(
  () =>
    optimizer.status === "busy" ||
    histogram.status === "busy" ||
    histogram.queued,
);
// the result's upgrades in LEVELS order
const shown = computed(() =>
  LEVELS.map((l) =>
    result.value?.state_bundle.upgrade_arr.find((u) => u.upgrade_index === l),
  ),
);

const names = LEVELS.map(upgrade_name);
const costs = LEVELS.map(fusion_per_tap);
// axes stay at the no-juice pity so juicing doesn't rescale anything
const axes = LEVELS.map((l) => max_taps(l));
const max_cost = axes[0] * costs[0] + axes[1] * costs[1];
const dists = computed(() => shown.value.map((u) => u?.normal_dist ?? []));
const sizes = computed(() => dists.value.map((d) => d.length - 1));

const prob = computed(() => {
  const [a, b] = dists.value;
  return (x: number, y: number) => (a[x] ?? 0) * (b[y] ?? 0);
});
// log colour range: the least and most likely outcome
const p_range = computed(() => {
  const [a, b] = dists.value.map((d) => d.slice(1).filter((p) => p > 0));
  if (!a.length || !b.length) return [1, 1];
  return [Math.min(...a) * Math.min(...b), Math.max(...a) * Math.max(...b)];
});
const used = (x: number, y: number) => x * costs[0] + y * costs[1];
const shortfall = computed(() => {
  const o = owned.value;
  return (x: number, y: number) => Math.max(0, used(x, y) - o) / max_cost;
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
  `${names[0]}: ${x} taps`,
  `${names[1]}: ${y} taps`,
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

// the histogram's gold distribution of the plan, like the calculator's
const gold = computed(() => {
  const r = result.value;
  if (!r?.gold_dist) return null;
  const { cdf, luckiest, pity } = r.gold_dist;
  return {
    cdf: pity[0] - luckiest[0] >= 1 ? cdf : null,
    mean: -r.metric_per_plan[0],
  };
});
const gold_tip = (
  x: number,
  _y: number,
  cy: number,
  _m: string,
  color: string,
) =>
  `<b style="color: var(--text-bright);">${chance_text(Math.min(Math.max(cy, 1e-9), 1 - 1e-9))}</b> chance to spend<br>≤ <b style="color: ${color};">${Math.round(x).toLocaleString("en-US")}</b> gold`;
</script>

<template>
  <div class="flex flex-col gap-4 pl-5">
    <div class="flex flex-col gap-1">
      <label class="flex items-center gap-2">
        <span class="w-56 shrink-0">Bound Fusion material owned</span>
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
      <label
        v-for="(j, i) in JUICES"
        :key="j.label"
        class="flex items-center gap-2"
      >
        <span class="w-56 shrink-0">Bound {{ j.label }} owned</span>
        <input
          v-model.number="juice_owned[i]"
          type="range"
          class="w-full"
          :min="0"
          :max="j.max"
        />
        <span class="w-32 shrink-0 text-right">
          {{ juice_owned[i].toLocaleString() }} / {{ j.max.toLocaleString() }}
        </span>
      </label>
    </div>

    <div class="grid gap-4 md:grid-cols-[auto_1fr_1fr]">
      <div
        class="flex flex-col gap-1 rounded-sm border border-(--border-very-muted) p-3"
      >
        <span class="text-(--text-muted)">Juice & book usage</span>
        <OptimizerRadioButton
          :options="STRATEGIES"
          :model-value="strategy"
          :on-change="set_strategy"
        />
      </div>
      <div
        v-for="(name, i) in names"
        :key="name"
        class="flex min-w-0 flex-col gap-2 rounded-sm border border-(--border-very-muted) p-3"
      >
        <div class="flex items-baseline justify-between gap-2">
          <span class="font-bold">{{ name }}</span>
          <span class="text-sm text-(--text-very-muted)">
            Pity {{ sizes[i] > 0 ? sizes[i] : axes[i] }} taps,
            {{ costs[i] }} fusion per tap
          </span>
        </div>
        <ActualInstructions
          v-if="shown[i] && result"
          :upgrade="shown[i]"
          :juice_info="result.juice_info"
          :tier="profile.tier"
          :working="working"
        />
        <span v-else class="text-sm text-(--text-very-muted)">
          Waiting for the optimizer
        </span>
      </div>
    </div>

    <!-- one row per upgrade and juice, a box per tap before the current pity -->
    <div
      v-if="strategy === CUSTOM && custom.length"
      class="flex w-full flex-col gap-1 rounded-sm border border-(--border-very-muted) p-3"
    >
      <template v-for="(name, i) in names" :key="name">
        <div
          v-for="(j, k) in JUICE_IDS"
          :key="j"
          class="flex items-center gap-2"
        >
          <span class="w-56 shrink-0">{{ name }} {{ JUICES[k].label }}</span>
          <div class="flex flex-wrap gap-0.5">
            <button
              v-for="t in Math.max(0, (sizes[i] > 0 ? sizes[i] : axes[i]) - 1)"
              :key="t"
              class="can-disable-icon-wrapper cursor-pointer"
              :class="custom[i][j][t - 1] ? 'ticked' : 'disabled'"
              :title="`Tap ${t}`"
              @click="custom[i][j][t - 1] = !custom[i][j][t - 1]"
            >
              <img
                :src="get_icon_path(JUICES[k].label)"
                class="generic-icon h-6 w-6"
                :class="{ disabled: !custom[i][j][t - 1] }"
              />
            </button>
          </div>
        </div>
      </template>
    </div>

    <div class="flex flex-col gap-1" :class="{ 'opacity-50': working }">
      <div class="flex items-baseline gap-2">
        <span class="text-lg text-nowrap text-(--gold)">
          Average tradable gold spent:
          {{ gold ? Math.round(gold.mean).toLocaleString("en-US") : "…" }}
        </span>
        <span class="text-sm text-(--text-very-muted)">
          (Only fusion and juice costs considered, their prices are made-up to
          demonstrate the point)
        </span>
      </div>
      <MaterialGraph
        class="pl-0!"
        :data="gold?.cdf ?? null"
        material-label="Gold"
        graph-color="--gold"
        :cumulative="false"
        :annotations="[gold?.mean ?? null]"
        :annotation-colors="['--average']"
        :annotation-positions="['graph']"
        :annotation-labels="['Average']"
        :tooltip-text-fn="gold_tip"
        :empty_message="
          gold ? 'No tradable gold spent' : 'Waiting for the optimizer'
        "
      />
    </div>

    <div class="grid gap-6 md:grid-cols-2" :class="{ 'opacity-50': working }">
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
  </div>
</template>
