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
import HoverTip from "@/Components/Common/HoverTip.vue";
import ItemList from "./ItemList.vue";
const runtime = useRuntimeStore();

const { active_profile } = storeToRefs(useRosterStore());
// juice_info, tier and working default to the calculator's
const props = defineProps<{
  upgrade: Upgrade;
  juice_info?: any;
  tier?: number;
  working?: boolean;
}>();

const juice_info = computed(
  () => props.juice_info ?? runtime.histogram.result.juice_info,
);

const relevant_ids = computed((): number[] => {
  const map = props.upgrade.is_normal_honing
    ? juice_info.value.normal_uindex_to_id
    : juice_info.value.adv_uindex_to_id;
  return map[props.upgrade.piece_type_usize][props.upgrade.upgrade_index];
});

const label_of = (id: number) =>
  ALL_LABELS[props.tier ?? active_profile.value.tier][id + NUM_BASE_MATS];
// how many of a juice or book one tap of this upgrade takes
const per_tap = (id: number) => {
  const u = props.upgrade;
  const one = juice_info.value.all_juices[id][u.piece_type_usize].get(
    u.upgrade_index,
  );
  return u.is_normal_honing ? one.normal_amt_used : one.adv_amt_used;
};

const parsed_streaks = computed(() => {
  const u = props.upgrade;
  if (u.state.length === 0) return [];
  const streaks = u.is_normal_honing
    ? normal_streaks(u)
    : adv_streaks(u, relevant_ids.value);
  const lines = streak_lines(u, streaks);
  let taps = 0;
  return streaks.map((streak, i) => {
    const first = taps + 1;
    taps += streak.count;
    return {
      ...streak,
      first,
      line1: lines[i][0],
      line2: lines[i][1],
      items: streak.ids.map((id) => ({ id, label: label_of(id) })),
      icons: relevant_ids.value.map((id) => ({
        active: streak.ids.includes(id),
        path: get_icon_path(label_of(id)),
      })),
    };
  });
});

const tap_range = (first: number, count: number) =>
  count === 1
    ? `on tap ${first}`
    : `from tap ${first} to tap ${first + count - 1}`;
const has_breath = (items: { label: string }[]) =>
  items.some((item) => item.label.includes("Breath"));
// the streak's items with their amount per tap, times `taps`; read only when its card shows
const amounts = (items: { id: number; label: string }[], taps: number) =>
  items.map((item) => ({ label: item.label, amount: per_tap(item.id) * taps }));

const optimizer_working = computed(
  () => props.working ?? get_optimizer_working(),
);
</script>

<template>
  <div
    class="mr-auto flex w-fit max-w-full scrollbar-thin flex-row overflow-x-auto overflow-y-hidden pb-2"
    :style="{ opacity: !optimizer_working ? 1 : 0.5 }"
  >
    <!-- the pb-2 is for the scroll bar -->
    <HoverTip
      v-for="(s, i) in parsed_streaks"
      :key="i"
      :sticky="false"
      placement="top"
      class="flex w-16 min-w-16 flex-col items-center justify-end text-center"
    >
      <template v-if="s.pity">
        <div class="opacity-50">
          <img
            :src="get_icon_path('Pity')"
            class="generic-icon ticked h-8 w-8"
          />
        </div>
      </template>
      <template v-else>
        <div
          v-for="(icon, icon_index) in s.icons"
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
        {{ s.line1 }}
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
        {{ s.line2 }}
      </div>

      <template v-if="!s.pity" #tip>
        <template v-if="upgrade.is_normal_honing">
          <template v-if="s.items.length === 0">
            Use nothing {{ tap_range(s.first, s.count) }}.
          </template>
          <template v-else>
            Use <b v-if="has_breath(s.items)">full&nbsp;</b
            ><ItemList :items="s.items" />
            {{ tap_range(s.first, s.count) }}.<br />That's
            <ItemList :items="amounts(s.items, 1)" />
            per tap,<br />for a total of
            <ItemList :items="amounts(s.items, s.count)" />.<br />You should buy
            them if you run out.
          </template>
        </template>
        <template v-else-if="s.items.length === 0">
          Use no juice or scrolls on this upgrade.
        </template>
        <template v-else>
          Use <ItemList :items="s.items" /> for
          <template v-if="s.grace">
            {{ s.count === 255 ? "all" : `the first ${s.count}` }} Ancestor's
            Grace procs.
          </template>
          <template v-else>
            {{ s.count === 255 ? "all" : `any ${s.count} of the` }} ordinary
            taps.
          </template>
        </template>
      </template>
    </HoverTip>
  </div>
</template>
