<script setup lang="ts">
import MaterialGraph from "@/Components/Character/MaterialDist/MaterialGraph.vue";
import { storeToRefs } from "pinia";
import { computed, ref, watch } from "vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { chance_text, metric_to_text } from "@/Utils/Helpers";
import type { GoldDist } from "@/WasmInterface/WasmTypes";

// The total gold row of MaterialDist's grid: label, then the average under the gold column,
// then the distribution under the material graphs.
defineProps<{ metric: number | undefined }>();
const runtime = useRuntimeStore();
const { roster_config } = storeToRefs(useRosterStore());

// The distribution comes only with a state the optimizer is done with, so the last one stays,
// faded, while it works on the next. Rust's is net of the tradable gold owned; shown gross.
const shown = ref<{ dist: GoldDist; metric: number; owned: number } | null>(
  null,
);
const fresh = ref(false);
const no_upgrades = ref(false);
watch(
  () => runtime.histogram.result,
  (result) => {
    if (!result) return;
    no_upgrades.value = result.state_bundle.upgrade_arr.length === 0;
    fresh.value = !!result.gold_dist || no_upgrades.value;
    const owned = result.tradable_gold;
    const gross = ([x, y]: [number, number]): [number, number] => [
      x + owned,
      y,
    ];
    if (result.gold_dist)
      shown.value = {
        dist: {
          ...result.gold_dist,
          cdf: result.gold_dist.cdf.map(gross),
          luckiest: gross(result.gold_dist.luckiest),
          pity: gross(result.gold_dist.pity),
        },
        metric: result.metric_per_plan[0] - owned,
        owned,
      };
  },
  { immediate: true },
);

const dist = computed(() => shown.value?.dist);
const owned = computed(() => shown.value?.owned ?? 0);
const owned_in_range = computed(
  () =>
    !!dist.value &&
    owned.value >= dist.value.luckiest[0] &&
    owned.value <= dist.value.pity[0],
);
// The point closest to the owned gold becomes it; an end keeps its place and says it in its tooltip.
const owned_index = computed(() => {
  if (!owned_in_range.value) return -1;
  const cdf = dist.value.cdf;
  let best = 0;
  cdf.forEach(([x], i) => {
    if (Math.abs(x - owned.value) < Math.abs(cdf[best][0] - owned.value))
      best = i;
  });
  return best;
});
const graph_data = computed(() => {
  const cdf = dist.value.cdf.slice();
  const i = owned_index.value;
  if (i > 0 && i < cdf.length - 1)
    cdf[i] = [owned.value, dist.value.within_budget];
  return cdf;
});
const spread = computed(
  () =>
    !no_upgrades.value &&
    dist.value &&
    dist.value.pity[0] - dist.value.luckiest[0] >= 1,
);
const empty_message = computed(() => {
  if (no_upgrades.value) return "No upgrades selected";
  if (!dist.value) return "Waiting for the optimizer";
  const always = Math.round(dist.value.pity[0]);
  return always > 0
    ? `Always spends ${always.toLocaleString("en-US")} gold`
    : "No tradable gold spent";
});

// off the graph is not drawn
const in_range = (x: number) =>
  dist.value && x >= dist.value.luckiest[0] && x <= dist.value.pity[0]
    ? x
    : null;
const annotations = computed(() => [
  in_range(-shown.value?.metric),
  in_range(owned.value),
]);

// a chance given as log10, as a percentage that stays readable however small
function log_percent(log10_chance: number) {
  const exponent = log10_chance + 2;

  if (exponent >= 0) return `${Math.round(10 ** exponent)}%`;
  if (exponent >= -2) return `${(10 ** exponent).toFixed(2)}%`;
  const whole = Math.floor(exponent);
  return `${(10 ** (exponent - whole)).toFixed(1)}e${whole}%`;
}

const gold_text = (x: number, color: string) =>
  `<b style="color: ${color};">${Math.round(x).toLocaleString("en-US")}</b> gold`;
// only the ends are certain
const ordinary = (x: number, cy: number, color: string) =>
  `<b style="color: var(--text-bright);">${chance_text(Math.min(Math.max(cy, 1e-9), 1 - 1e-9))}</b> chance to spend<br>≤ ${gold_text(x, color)}`;

function hover_annotation(x, _y, cy, _material, color, is_last): string {
  const end = is_last || x === dist.value.luckiest[0];
  if (!end)
    return ordinary(x, cy, x === owned.value ? "var(--tradable)" : color);
  const text = is_last
    ? `<b style="color: var(--text-bright);">${log_percent(dist.value.pity[1])}</b> chance to pity EVERYTHING<br>= ${gold_text(x, color)}`
    : `<b style="color: var(--text-bright);">${log_percent(dist.value.luckiest[1])}</b> chance to hit the best-case scenario<br>= ${gold_text(x, color)}`;

  const at = is_last ? graph_data.value.length - 1 : 0;
  return owned_index.value === at
    ? `${text}<br>${ordinary(owned.value, dist.value.within_budget, "var(--tradable)")}`
    : text;
}
</script>

<template>
  <div class="mats-row h-auto! min-h-10.5 border-b-0!">
    <!-- col-span-4 with the Est. gold value column -->
    <div class="col-span-3 flex items-baseline justify-end gap-2 pr-2">
      <span class="annotation text-(--text-very-muted)">
        (raw + buying needed mats & juice)
      </span>
      <span class="text-lg text-nowrap text-(--gold)">
        Average tradable gold spent:
      </span>
    </div>
    <span class="metric-result text-right text-lg text-(--gold)">
      {{ metric_to_text(metric) }}
    </span>
    <MaterialGraph
      :class="{ 'opacity-50': !fresh }"
      :data="spread ? graph_data : null"
      material-label="Gold"
      graph-color="--gold"
      :cumulative="roster_config.cumulative_graph"
      :annotations="annotations"
      :annotation-colors="['--average', '--tradable']"
      :annotation-positions="['graph', 'bottom']"
      :annotation-labels="['Average', 'Tradable gold']"
      :tooltip-text-fn="hover_annotation"
      :empty_message="empty_message"
    />
  </div>
</template>
