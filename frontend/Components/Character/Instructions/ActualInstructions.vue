<script setup lang="ts">
import { computed } from "vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import { storeToRefs } from "pinia";
import {
  ALL_LABELS,
  JOINED_ADV_JUICE,
  T4_JUICE_LABELS,
} from "@/Utils/Constants";
import { get_icon_path, toOrdinal } from "@/Utils/Helpers";
import { OneState, Upgrade } from "@/Utils/KeyedUpgrades";
import { artisan_string } from "@/Utils/HoningUtil";
import { get_optimizer_working } from "./InstructionUtils";
import VambraceWarning from "@/Components/Common/VambraceWarning.vue";

const { active_profile } = storeToRefs(useRosterStore());
const props = defineProps<{
  upgrade: Upgrade;
}>();

const juice_info = computed(() => {
  return active_profile.value.histogram_worker_bundle.result.juice_info;
});

const relevant_ids = computed(() => {
  let relevant_id_map = props.upgrade.is_normal_honing
    ? juice_info.value.normal_uindex_to_id
    : juice_info.value.adv_uindex_to_id;

  let ids: number[] =
    relevant_id_map[props.upgrade.piece_type_usize][
      props.upgrade.upgrade_index
    ];

  return ids;
});

function icon_path_for_id(id: number) {
  // console.log(id);
  return get_icon_path(ALL_LABELS[active_profile.value.tier][id + 7]);
}

interface NormalStreak {
  ids: number[];
  count: number;
  grace: boolean | null;
  pity: boolean;
}

function same_one_state(a: OneState, b: OneState) {
  const setA = new Set(a);
  const setB = new Set(b);
  if (setA.size !== setB.size) return false;
  return [...setA].every((val) => setB.has(val));
}

const streaks = computed(() => {
  if (props.upgrade.state.length === 0) return [];

  if (props.upgrade.is_normal_honing) {
    const streaks: NormalStreak[] = [];
    let current: NormalStreak | null = null;
    let index = 0;
    for (const one_state of props.upgrade.state.slice(
      0,
      props.upgrade.normal_dist.length,
    )) {
      if (
        index == props.upgrade.normal_dist.length - 2 &&
        artisan_string(props.upgrade, index, juice_info.value) === "100.00"
      ) {
        // this corresponds to not showing the pity tap
        // Rust side does not enforce that the pity tap is unjuiced (it just ignores the state after that index)
        // so we need to hide it from the user
        // however for props.upgrades that naturally has a 100% success rate (below like +5) we don't want to skip (it won't have 100% artisan)
        // just a weird edge case
        streaks.push({ pity: true, ids: [], count: 1, grace: null });
        break;
        // continue;
      }

      if (current && same_one_state(one_state, current.ids)) {
        current.count++;
      } else {
        // console.log(one_state);
        current = { ids: one_state, count: 1, pity: false, grace: null };
        streaks.push(current);
      }
      index += 1;
    }
    // console.log(streaks);
    return streaks;
  } else {
    const raw_streaks: NormalStreak[] = [];
    const juice_id = relevant_ids.value[0];
    const scroll_id = relevant_ids.value[1];
    let [juice_grace, juice_non_grace] =
      JOINED_ADV_JUICE[props.upgrade.state[0][0]];
    let [scroll_grace, scroll_non_grace] =
      JOINED_ADV_JUICE[props.upgrade.state[1][0]];
    // These 4 numbers correspond to how many taps to perform on the respective conditions
    // They range from 0 to 255, with 255 considered infinite, see rust advanced_honing/utils for what numbers they can actually take

    let both_grace = Math.min(juice_grace, scroll_grace);
    if (both_grace > 0)
      raw_streaks.push({
        ids: [juice_id, scroll_id],
        count: both_grace,
        grace: true,
        pity: false,
      });
    // console.log(streaks)
    let one_grace =
      juice_grace === scroll_grace
        ? 0
        : Math.max(juice_grace, scroll_grace) == 255
          ? 255
          : Math.max(juice_grace, scroll_grace) - both_grace;
    if (one_grace > 0)
      raw_streaks.push({
        ids: juice_grace > scroll_grace ? [juice_id] : [scroll_id],
        count: one_grace,
        grace: true,
        pity: false,
      });
    // console.log(streaks)
    let both_non_grace = Math.min(juice_non_grace, scroll_non_grace);
    if (both_non_grace > 0)
      raw_streaks.push({
        ids: [juice_id, scroll_id],
        count: both_non_grace,
        grace: false,
        pity: false,
      });
    // console.log(streaks)
    let one_non_grace =
      juice_non_grace === scroll_non_grace
        ? 0
        : Math.max(juice_non_grace, scroll_non_grace) == 255
          ? 255
          : Math.max(juice_non_grace, scroll_non_grace) - both_non_grace;
    if (one_non_grace > 0)
      raw_streaks.push({
        ids: juice_non_grace > scroll_non_grace ? [juice_id] : [scroll_id],
        count: one_non_grace,
        grace: false,
        pity: false,
      });
    // console.log(streaks)
    if (raw_streaks.length == 0) {
      raw_streaks.push({
        ids: [],
        count: 255,
        grace: true,
        pity: false,
      });
    }
    // console.log(
    //   one_grace,
    //   both_grace,
    //   juice_grace,
    //   juice_non_grace,
    //   scroll_grace,
    //   scroll_non_grace,
    //   props.upgrade.state,
    //   raw_streaks,
    // );
    return raw_streaks;
  }
});

interface ParsedIcon {
  id: number;
  active: boolean;
  path: string;
}

const parsed_streaks = computed(() => {
  // console.log("parsed recalc");
  let out = [];
  let taps = 0;
  for (let index = 0; index < streaks.value.length; index++) {
    let streak: NormalStreak = streaks.value[index];

    let is_normal = props.upgrade.is_normal_honing;

    let icons: ParsedIcon[] = streak.pity
      ? []
      : relevant_ids.value.map((id) => ({
          id,
          active: streak.ids.includes(id),
          path: icon_path_for_id(id),
        }));

    let line1: string;
    let line2: string;

    if (is_normal) {
      taps += streak.count;
      if (streak.pity) {
        line1 = `Pity`;
        line2 = `occurs on the ${toOrdinal(taps)} tap`;
      } else {
        line1 = `x${streak.count} taps`;
        line2 = `until ${artisan_string(props.upgrade, taps, juice_info.value)}% artisan`;
      }
    } else {
      let graceText = streak.grace ? "Grace" : "non-Grace";
      if (streak.ids.length === 0) {
        line1 = "Nothing";
        line2 = `on ${graceText}`;
      } else {
        line1 =
          streak.count < 255
            ? `${streak.grace ? "First" : "Any"} ${streak.count}`
            : "All";
        line2 = graceText;
      }
    }

    out.push({
      icons,
      line1,
      line2,
      pity: streak.pity,
    });
  }
  // console.log("parsed", out);
  return out;
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
    <VambraceWarning v-if="upgrade.piece_type_usize == 2" class="my-auto w" />
  </div>
</template>
