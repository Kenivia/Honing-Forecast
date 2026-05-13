<script setup lang="ts">
import {
  ALL_LABELS,
  GRAPH_COLORS,
  T4_MATS_LABELS,
  ANNOTATION_COLORS,
  ANNOTATION_POSITIONS,
  ANNOTATION_LABELS,
  PIECE_NAMES,
} from "@/Utils/Constants";
import { get_icon_path } from "@/Utils/Helpers";
import MaterialCell from "@/Components/Common/MaterialCell.vue";

import MaterialGraph from "./MaterialGraph.vue";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { computed } from "vue";

import {
  get_upgrade_map,
  to_upgrade_key,
  UpgradeStatus,
} from "@/Utils/KeyedUpgrades";
import DetailedInputRow from "./DetailedInputRow.vue";

const { active_profile } = storeToRefs(useRosterStore());

const props = defineProps<{
  grid_type: "normal" | "adv";
}>();

const relevant_grid = computed(() =>
  props.grid_type === "normal"
    ? active_profile.value.normal_grid
    : active_profile.value.adv_grid,
);

const upgrade_map = computed(() =>
  get_upgrade_map(
    active_profile.value.optimizer_worker_bundle.result?.upgrade_arr ?? null,
    active_profile.value.tier,
  ),
);
const lowest_arr = computed(() =>
  PIECE_NAMES.map(
    (_, piece_type) =>
      upgrade_map.value.get(
        to_upgrade_key(
          piece_type,
          (relevant_grid.value[piece_type] as UpgradeStatus[]).findIndex(
            (value) => value == UpgradeStatus.Want,
          ),
          true,
          active_profile.value.tier,
        ),
      ) ?? null,
  ),
);

const lowest_upgrade_index = computed(() =>
  Math.min(...lowest_arr.value.map((x) => x?.upgrade_index ?? 999)),
);
// console.log(
//   upgrade_map.value,
//   lowest_arr.value,
//   upgrade_map.value.get("0,18,true,0"),
// );
</script>

<template>
  <section class="card-shell">
    <div class="card-header">
      <div class="card-title">Detailed inputs</div>
    </div>
    <div class="card-body flex flex-col">
      <div
        v-for="(piece_name, index) in PIECE_NAMES"
        :key="piece_name"
        class="flex h-fit min-h-30 flex-row border-b border-(--border-very-muted)"
      >
        <div
          class="flex flex-col items-center justify-center"
          :style="{
            opacity:
              lowest_upgrade_index == lowest_arr[index]?.upgrade_index
                ? 1
                : 0.5,
          }"
        >
          <img
            :src="get_icon_path(piece_name)"
            :alt="piece_name"
            class="h-12 w-12 object-contain"
          />
          <span class="text-(--text-muted)">{{
            piece_name +
            (lowest_arr[index]
              ? ` +${lowest_arr[index].upgrade_index + 1}`
              : "")
          }}</span>
        </div>
        <DetailedInputRow
          :upgrade="lowest_arr[index]"
          v-if="lowest_upgrade_index == lowest_arr[index]?.upgrade_index"
        />
      </div>
    </div>
  </section>
</template>
