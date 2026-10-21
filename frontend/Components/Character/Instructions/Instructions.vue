<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, ref } from "vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import { to_upgrade_key } from "@/Utils/KeyedUpgrades";
import { GridConfig } from "@/Utils/GridStyling";
import {
  get_any_overwritten,
  get_optimizer_working,
  get_relevant_result,
  instruction_rows,
} from "./InstructionUtils";
import InstructionRow from "./InstructionRow.vue";
import QuestionMark from "@/Components/Common/QuestionMark.vue";
import { locale_to_fixed } from "@/Utils/Helpers";
import { useRuntimeStore } from "@/Stores/RuntimeState";
const runtime = useRuntimeStore();

const props = defineProps<{
  is_normal: boolean;
}>();

const { active_profile } = storeToRefs(useRosterStore());
const any_overwritten = computed(get_any_overwritten);
const relevant_upgrade_arr = computed(() =>
  instruction_rows(get_relevant_result(any_overwritten.value), props.is_normal),
);

const COLLAPSED_COUNT = 10;
const expanded = ref(false);
const shown_upgrade_arr = computed(() =>
  expanded.value
    ? relevant_upgrade_arr.value
    : relevant_upgrade_arr.value.slice(0, COLLAPSED_COUNT),
);

const grid: GridConfig = props.is_normal
  ? {
      grid_template_columns:
        //  66 px fits Weapon, Shoulder still doesn't fit but whatever
        "minmax(66px, 70px) minmax(70px,100px) minmax(80px,100px) minmax(256px, max-content) max-content",
    }
  : {
      grid_template_columns:
        "minmax(75px, 85px) minmax(256px, max-content)  max-content",
    };

const optimizer_working = computed(get_optimizer_working);
</script>
<template>
  <div class="card-shell">
    <div class="card-header">
      <div class="card-title">
        {{
          is_normal
            ? "Normal Honing Instructions"
            : "Advanced Honing Instructions"
        }}
        {{ any_overwritten ? "(Not optimized)" : "" }}

        <span v-if="optimizer_working" class="text-(--text-main)">
          ({{
            locale_to_fixed(runtime.optimizer.est_progress_percentage, 2)
          }}%)</span
        >
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
        <div v-if="is_normal" class="flex w-full flex-row justify-center">
          <div class="ml-3 w-min text-wrap">Upgrade order</div>
          <QuestionMark
            text="The purpose of this is to do the Free Taps in the specified order. There are many other equivalent orderings."
            position="right"
          />
        </div>
        <span v-if="is_normal">Special usage</span>
        <div class="title-nowrap">
          <span> Juice & {{ is_normal ? "book" : "scroll" }} Instructions</span>
          <QuestionMark
            :text="
              is_normal
                ? 'Juiced taps mean full-juice (use the maximum amount of Lava / Glacier Breath).'
                : 'Advanced honing optimization is limited, use these instructions as a rough guide.'
            "
            position="right"
          />
        </div>
        <div class="title-nowrap">
          <span>
            Progress update {{ is_normal ? "" : "(Not recommended)" }}
          </span>
          <QuestionMark
            v-if="!is_normal"
            text="Auto-cost deduction is unavailable, I suggest only doing this when you're not starting from 0. "
            position="right"
          />
        </div>
      </div>
      <div
        v-if="runtime.optimizer.result && runtime.histogram.result"
        class="contents"
      >
        <div
          v-for="[
            upgrade,
            index_in_special_state,
            perform_order,
          ] in shown_upgrade_arr"
          :key="`instructions-${upgrade.upgrade_index}-${upgrade.piece_index}-${upgrade.is_normal_honing}`"
          class="mats-row h-fit!"
        >
          <InstructionRow
            v-if="
              active_profile.keyed_upgrades[
                to_upgrade_key(
                  upgrade.piece_index,
                  upgrade.upgrade_index,
                  upgrade.is_normal_honing,
                  active_profile.tier,
                )
              ]
            "
            :upgrade="upgrade"
            :perform_order="perform_order"
            :special_invalid_index="
              runtime.histogram.result.state_bundle.special_invalid_index
            "
            :index_in_special_state="index_in_special_state"
          />
        </div>
      </div>
    </div>
    <button
      v-if="relevant_upgrade_arr.length > COLLAPSED_COUNT"
      @click="expanded = !expanded"
      class="barebone-button mx-auto w-fit text-(--text-muted)"
    >
      {{
        expanded
          ? "Collapse"
          : `Show all ${relevant_upgrade_arr.length} upgrades`
      }}
    </button>
  </div>
</template>
<style>
.title-nowrap {
  display: flex;
  width: 100%;
  flex-direction: row;
  flex-wrap: nowrap;
  justify-content: center;
  gap: 0.25rem;
  padding-left: 0.75rem;
  padding-right: 0.75rem;
  white-space: nowrap;
}
</style>
