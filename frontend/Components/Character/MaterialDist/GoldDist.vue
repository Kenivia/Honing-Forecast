<script setup lang="ts">
import MaterialGraph from "@/Components/Character/MaterialDist/MaterialGraph.vue";
import { storeToRefs } from "pinia";
import { computed, ref, watch } from "vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { input_column_to_num } from "@/Utils/InputColumn";
import { metric_to_text } from "@/Utils/Helpers";
import type { GoldDist } from "@/WasmInterface/WasmTypes";

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
    return `<b style="color: white;">${log_percent(dist.value.pity[1])} </b> chance to hit hard pity <br> ≥ ${gold}`;
  if (x === dist.value.luckiest[0])
    return `<b style="color: white;">${log_percent(dist.value.luckiest[1])} </b> chance to be this lucky <br> ≤ ${gold}`;
  // only full pity is certain
  const percent = Math.round(cy * 100);
  return `<b style="color: white;">${percent === 100 ? "~100" : percent}% </b> chance to spend <br> ≤ ${gold}`;
}

// MaterialDist's columns, so the graph lines up under the material graphs
const GRID_COLS =
  "minmax(260px, 320px) minmax(70px, 90px) minmax(110px, 120px) minmax(110px, 120px) 350px";
</script>

<template>
  <div class="card-shell">
    <div class="card-header">
      <div class="card-title">Tradable gold distribution</div>
    </div>
    <div
      class="card-body outer-grid pt-0! pb-2!"
      :style="{ '--grid-cols': GRID_COLS }"
    >
      <div class="mats-row border-b-0!" role="group" aria-label="Gold spent">
        <span class="col-span-3 text-left text-(--gold)">
          Average tradable gold spent:
        </span>
        <span class="text-right text-(--gold)">
          {{ metric_to_text(no_upgrades ? null : shown?.metric) }}
        </span>
        <MaterialGraph
          :class="{ 'opacity-50': !fresh }"
          :data="spread ? dist.cdf : null"
          material-label="Gold"
          graph-color="--gold"
          :cumulative="roster_config.cumulative_graph"
          :annotations="annotations"
          :annotation-colors="['--average', '--tradable']"
          :annotation-positions="['graph', 'top']"
          :annotation-labels="['Average', 'Tradable gold']"
          :tooltip-text-fn="hover_annotation"
          :empty_message="empty_message"
        />
      </div>
    </div>
  </div>
</template>
