<script setup lang="ts">
import { computed } from "vue";

import { useRosterStore } from "@/Stores/RosterConfig";
import { storeToRefs } from "pinia";
import { JOINED_ADV_JUICE, T4_JUICE_LABELS } from "@/Utils/Constants";
import { get_icon_path, toOrdinal } from "@/Utils/Helpers";
import { Upgrade } from "@/Utils/KeyedUpgrades";
import { artisan_function } from "@/Utils/HoningUtil";

const { active_profile } = storeToRefs(useRosterStore());
const props = defineProps<{
  upgrade: Upgrade;
}>();

const juice_info = computed(() => {
  return active_profile.value.histogram_worker_bundle.result.juice_info;
});

function juice_icon_path(juice: boolean) {
  let juice_info =
    active_profile.value.histogram_worker_bundle.result.juice_info;
  let relevant_id_map = props.upgrade.is_normal_honing
    ? juice_info.normal_uindex_to_id
    : juice_info.adv_uindex_to_id;

  let relevant_upgrade = relevant_id_map[props.upgrade.upgrade_index];

  if (relevant_upgrade.length === 0) {
    return get_icon_path(T4_JUICE_LABELS[0][props.upgrade.is_weapon ? 0 : 1]);
  }

  return get_icon_path(
    T4_JUICE_LABELS[relevant_upgrade[juice ? 0 : relevant_upgrade.length - 1]][
      props.upgrade.is_weapon ? 0 : 1
    ],
  );
}

interface NormalStreak {
  juice: boolean;
  book: boolean;
  count: number;
  pity: boolean;
}
interface AdvStreak {
  juice: boolean;
  scroll: boolean;
  grace: boolean;
  count: number;
}
const streaks = computed(() => {
  if (props.upgrade.state.length === 0) return [];

  if (props.upgrade.is_normal_honing) {
    const streaks: NormalStreak[] = [];
    let current: NormalStreak | null = null;
    let index = 0;
    for (const [juice, book] of props.upgrade.state.slice(
      0,
      props.upgrade.normal_dist.length - 1,
    )) {
      if (
        index == props.upgrade.normal_dist.length - 2 &&
        artisan_function(props.upgrade, index, juice_info.value) === "100.00"
      ) {
        // this corresponds to not showing the pity tap
        // Rust side does not enforce that the pity tap is unjuiced (it just ignores the state after that index)
        // so we need to hide it from the user
        // however for props.upgrades that naturally has a 100% success rate (below like +5) we don't want to skip
        // just a weird edge case
        continue;
      }
      const hasBook = book > 0;
      if (current && current.juice === juice && current.book === hasBook) {
        current.count++;
      } else {
        current = { juice, book: hasBook, count: 1, pity: false };
        streaks.push(current);
      }
      index += 1;
    }
    streaks.push({ pity: true, juice: false, book: false, count: 1 });
    return streaks;
  } else {
    const raw_streaks: AdvStreak[] = [];
    let [juice_grace, juice_non_grace] =
      JOINED_ADV_JUICE[props.upgrade.state[0][1]];
    let [scroll_grace, scroll_non_grace] =
      JOINED_ADV_JUICE[props.upgrade.state[1][1]];
    // These 4 numbers correspond to how many taps to perform on the respective conditions
    // They range from 0 to 255, with 255 considered infinite, see rust advanced_honing/utils for what numbers they can actually take

    let both_grace = Math.min(juice_grace, scroll_grace);
    if (both_grace > 0)
      raw_streaks.push({
        juice: true,
        scroll: true,
        grace: true,
        count: both_grace,
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
        juice: juice_grace > scroll_grace,
        scroll: scroll_grace > juice_grace,
        grace: true,
        count: one_grace,
      });
    // console.log(streaks)
    let both_non_grace = Math.min(juice_non_grace, scroll_non_grace);
    if (both_non_grace > 0)
      raw_streaks.push({
        juice: true,
        scroll: true,
        grace: false,
        count: both_non_grace,
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
        juice: juice_non_grace > scroll_non_grace,
        scroll: scroll_non_grace > juice_non_grace,
        grace: false,
        count: one_non_grace,
      });
    // console.log(streaks)
    if (raw_streaks.length == 0) {
      raw_streaks.push({
        juice: false,
        scroll: false,
        grace: true,
        count: 255,
      });
    }
    // console.log(one_grace, both_grace, juice_grace, juice_non_grace, scroll_grace, scroll_non_grace, props.upgrade.state, streaks)
    return raw_streaks;
  }
});
const parsed_streaks = computed(() => {
  let out = [];
  let taps = 0;
  for (let index = 0; index < streaks.value.length; index++) {
    let streak: any = streaks.value[index];

    let isNormal = props.upgrade.is_normal_honing;
    let topIconActive = streak.juice;
    let bottomIconActive = isNormal ? streak.book : streak.scroll;
    let name_line =
      (streak.juice ? "Juice" : "") +
      ((streak.juice && streak.book) || (streak.juice && streak.scroll)
        ? " & "
        : "") +
      (streak.book ? "Book" : streak.scroll ? "Scroll" : "") +
      (!streak.juice && !streak.juice && !streak.book && !streak.scroll
        ? "Raw tap"
        : "");
    let line1: string;
    let line2: string;

    if (isNormal) {
      taps += streak.count;
      if (streak.pity) {
        line1 = `Pity`;
        line2 = `reached at ${toOrdinal(taps)} tap`;
      } else {
        line1 = `x${streak.count} taps`;
        line2 = `until ${artisan_function(props.upgrade, taps, juice_info.value)}% artisan`;
      }
    } else {
      let graceText = streak.grace ? "Grace" : "non-Grace";
      if (!streak.juice && !streak.scroll) {
        line1 = "Nothing";
        line2 = `on ${graceText}`;
      } else {
        // console.log(props.upgrade.adv_dists)
        line1 =
          streak.count < 255
            ? `First ${streak.count}`
            : streaks.value.length == 1
              ? "All"
              : "All";
        line2 = graceText;
      }
    }

    out.push({
      topIconActive,
      bottomIconActive,
      line1,
      line2,
      name_line,
      juice: streak.juice,
      book_or_scroll: streak.book || streak.scroll,
      pity: streak.pity,
    });
  }
  return out;
});
</script>

<template>
  <div class="mr-auto flex w-fit flex-row pl-3">
    <div
      v-for="(parsed_streak, i) in parsed_streaks"
      :key="i"
      class="flex w-16 flex-col items-center justify-end"
    >
      <div
        class="can-disable-icon-wrapper"
        :class="{ disabled: !parsed_streak.juice && !parsed_streak.pity }"
      >
        <img
          :src="
            parsed_streak.pity ? get_icon_path('Pity') : juice_icon_path(true)
          "
          alt="Top Mat"
          class="generic-icon h-8 w-8"
          :class="{ disabled: !parsed_streak.juice }"
        />
      </div>

      <div
        v-if="
          juice_icon_path(false) !== juice_icon_path(true) &&
          !parsed_streak.pity
        "
        class="can-disable-icon-wrapper"
        :class="{ disabled: !parsed_streak.book_or_scroll }"
      >
        <img
          :src="juice_icon_path(false)"
          alt="Bottom Mat"
          class="generic-icon h-8 w-8"
          :class="{ disabled: !parsed_streak.book_or_scroll }"
        />
      </div>

      <!-- <div v-html="streak_text.name_line"></div> -->
      <div class="text-sm text-(--text-main)">{{ parsed_streak.line1 }}</div>

      <div class="annotation">{{ parsed_streak.line2 }}</div>
    </div>
  </div>
</template>
