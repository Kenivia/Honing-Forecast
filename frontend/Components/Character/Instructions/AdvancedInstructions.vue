<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed } from "vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import { GridConfig } from "@/Utils/GridStyling";
import { Quaternary, Trinary } from "@/WasmInterface/PayloadBuilder";
import AdvancedRow from "./AdvancedRow.vue";
import { Upgrade } from "@/Utils/KeyedUpgrades";
import { get_any_overwritten } from "./InstructionUtils";

const { active_profile } = storeToRefs(useRosterStore());
const any_overwritten = computed(get_any_overwritten);
const relevant_result = computed(() => {
  return any_overwritten.value
    ? active_profile.value.histogram_worker_bundle.result.state_bundle
    : active_profile.value.optimizer_worker_bundle.result;
});

const grid: GridConfig = {
  grid_template_columns:
    //  66 px fits Weapon, Shoulder still doesn't fit but whatever
    "minmax(75px, 85px) minmax(200px, max-content) 80px 370px ",
};
</script>
<template>
  <section class="card-shell">
    <div class="card-header">
      <div class="card-title">
        Advanced Honing Instructions
        {{ any_overwritten ? "(Not optimized)" : "" }}
        <div
          class="question-mark"
          v-if="!any_overwritten"
          v-tooltip.right="
            'Advanced honing optimization is limited, take its advice with a grain of salt. '
          "
        />
      </div>
    </div>
    <div
      class="card-body outer-grid"
      :style="{
        '--grid-cols': grid.grid_template_columns,
      }"
    >
      <div class="mats-row">
        <span>Upgrade</span>

        <div
          class="flex w-full flex-row flex-nowrap justify-center gap-1 px-3 text-nowrap"
        >
          <span> Juice & book Instructions</span>
          <div
            class="question-mark"
            v-tooltip.left="
              '\'Juice First N Grace\' means to use juice on the first N Ancestor\'s Grace.'
            "
          />
        </div>
        <span>Succeed</span>
        <div class="flex w-full flex-row justify-center">
          Progress update
          <div
            class="question-mark ml-2"
            v-tooltip.left="
              'This is optional - updating your progress will produce instructions that save slightly more gold. '
            "
          />
        </div>
      </div>
      <div
        v-if="
          active_profile.optimizer_worker_bundle.result &&
          active_profile.histogram_worker_bundle.result
        "
        class="contents"
      >
        <div
          v-for="upgrade in relevant_result.upgrade_arr
            .filter((x: Upgrade) => !x.is_normal_honing)
            .sort((a, b) => a.upgrade_index - b.upgrade_index)"
          :key="`instructions-${upgrade.upgrade_index}-${upgrade.piece_type}-${upgrade.is_normal_honing}`"
          class="mats-row h-fit! py-1!"
        >
          <AdvancedRow :upgrade="upgrade" />
        </div>
      </div>
    </div>
  </section>
</template>
