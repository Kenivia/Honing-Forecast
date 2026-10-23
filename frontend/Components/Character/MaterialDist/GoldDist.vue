<script setup lang="ts">
import MaterialGraph from "@/Components/Character/MaterialDist/MaterialGraph.vue";
import { storeToRefs } from "pinia";
import { computed, ref, watch } from "vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { input_column_to_num } from "@/Utils/InputColumn";
import { metric_to_text } from "@/Utils/Helpers";
import type { GoldDist } from "@/WasmInterface/WasmTypes";

// The total gold row of MaterialDist's grid: label, then the average under the gold column,
// then the distribution under the material graphs.
defineProps<{ metric: number | undefined }>();
const runtime = useRuntimeStore();
const { roster_config, active_tradable_mats_owned } =
  storeToRefs(useRosterStore());

// The distribution comes only with a state the optimizer is done with, so the last one stays,
// faded, while it works on the next.
const shown = ref<{ dist: GoldDist; metric: number } | null>(null);
const fresh = ref(false);
const no_upgrades = ref(false);
watch(
  () => runtime.histogram.result,
  (result) => {
    if (!result) return;
    no_upgrades.value = result.state_bundle.upgrade_arr.length === 0;
    fresh.value = !!result.gold_dist || no_upgrades.value;
    if (result.gold_dist)
      shown.value = {
        dist: result.gold_dist,
        metric: result.metric_per_plan[0],
      };
  },
  { immediate: true },
);

const dist = computed(() => shown.value?.dist);
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

const tradable_gold = computed(
  () => input_column_to_num(active_tradable_mats_owned.value)["Gold"] ?? 0,
);
// off the graph is not drawn
const in_range = (x: number) =>
  dist.value && x >= dist.value.luckiest[0] && x <= dist.value.pity[0]
    ? x
    : null;
const annotations = computed(() => [
  in_range(-shown.value?.metric),
  in_range(tradable_gold.value),
]);

// a chance given as log10, as a percentage that stays readable however small
function log_percent(log10_chance: number) {
  const exponent = log10_chance + 2;
  if (exponent >= 0) return `${Math.round(10 ** exponent)}%`;
  const whole = Math.floor(exponent);
  return `${(10 ** (exponent - whole)).toFixed(1)}e${whole}%`;
}

function hover_annotation(x, _y, cy, _material, color, is_last): string {
  const gold = `<b style="color: ${color};"> ${Math.round(x).toLocaleString("en-US")} </b> gold`;
  if (is_last)
    return `<b style="color: white;">${log_percent(dist.value.pity[1])} </b> chance to pity EVERYTHING <br> ≥ ${gold}`;
  if (x === dist.value.luckiest[0])
    return `<b style="color: white;">${log_percent(dist.value.luckiest[1])} </b> chance to use ≤ ${gold}`;
  // only full pity is certain
  const percent = Math.round(cy * 100);
  return `<b style="color: white;">${percent === 100 ? "~100" : percent}% </b> chance to spend <br> ≤ ${gold}`;
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
      :data="spread ? dist.cdf : null"
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
