<script setup lang="ts">
import { computed } from "vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import { storeToRefs } from "pinia";
import { ALL_LABELS, NUM_BASE_MATS } from "@/Utils/Constants";
import { get_icon_path } from "@/Utils/Helpers";
import { Upgrade } from "@/Utils/KeyedUpgrades";
import {
  adv_streaks,
  get_optimizer_working,
  normal_streaks,
  streak_lines,
} from "./InstructionUtils";
import { useRuntimeStore } from "@/Stores/RuntimeState";
const runtime = useRuntimeStore();

const { active_profile } = storeToRefs(useRosterStore());
const props = defineProps<{
  upgrade: Upgrade;
}>();

const juice_info = computed(() => runtime.histogram.result.juice_info);

const relevant_ids = computed((): number[] => {
  const map = props.upgrade.is_normal_honing
    ? juice_info.value.normal_uindex_to_id
    : juice_info.value.adv_uindex_to_id;
  return map[props.upgrade.piece_type_usize][props.upgrade.upgrade_index];
});

const parsed_streaks = computed(() => {
  const u = props.upgrade;
  if (u.state.length === 0) return [];
  const streaks = u.is_normal_honing
    ? normal_streaks(u, juice_info.value)
    : adv_streaks(u, relevant_ids.value);
  const lines = streak_lines(u, streaks, juice_info.value);
  return streaks.map((streak, i) => ({
    pity: streak.pity,
    line1: lines[i][0],
    line2: lines[i][1],
    icons: relevant_ids.value.map((id) => ({
      active: streak.ids.includes(id),
      path: get_icon_path(
        ALL_LABELS[active_profile.value.tier][id + NUM_BASE_MATS],
      ),
    })),
  }));
});
const optimizer_working = computed(get_optimizer_working);
</script>

<template>
  <div
    class="mr-auto flex w-fit max-w-full scrollbar-thin flex-row overflow-x-auto overflow-y-hidden pb-2"
    :style="{ opacity: !optimizer_working ? 1 : 0.5 }"
  >
    <!-- the pb-2 is for the scroll bar -->
    <div
      v-for="(parsed_streak, i) in parsed_streaks"
      :key="i"
      class="flex w-16 min-w-16 flex-col items-center justify-end"
    >
      <template v-if="parsed_streak.pity">
        <div class="opacity-50">
          <img
            :src="get_icon_path('Pity')"
            class="generic-icon ticked h-8 w-8"
          />
        </div>
      </template>
      <template v-else>
        <div
          v-for="(icon, icon_index) in parsed_streak.icons"
          :key="icon_index"
          class="can-disable-icon-wrapper"
          :class="{ disabled: !icon.active, ticked: icon.active }"
        >
          <img
            :src="icon.path"
            class="generic-icon h-8 w-8"
            :class="{ disabled: !icon.active, ticked: icon.active }"
          />
        </div>
      </template>

      <div
        class="text-(--text-main)"
        :style="{
          fontSize: upgrade.is_normal_honing
            ? 'var(--text-sm)'
            : 'var(--text-xs)',
        }"
      >
        {{ parsed_streak.line1 }}
      </div>

      <div
        class="annotation"
        :style="{
          color: upgrade.is_normal_honing
            ? 'var(--text-muted)'
            : 'var(--text-main)',
          fontSize: upgrade.is_normal_honing
            ? 'var(--text-2xs)'
            : 'var(--text-xs)',
          textWrap: upgrade.is_normal_honing ? 'wrap' : 'nowrap',
        }"
      >
        {{ parsed_streak.line2 }}
      </div>
    </div>
  </div>
</template>
