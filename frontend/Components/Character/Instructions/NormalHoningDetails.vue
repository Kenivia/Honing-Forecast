<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { ARTISAN_RATE, FLOAT_TOL } from "@/Utils/Constants";
import {
  clamp,
  clamp_percentage,
  clean_percentage_input,
} from "@/Utils/Helpers";
import { parse_locale_float } from "@/Utils/InputColumn";
import { to_upgrade_key, Upgrade } from "@/Utils/KeyedUpgrades";

import { storeToRefs } from "pinia";

import { computed, ref, watch } from "vue";
import { start_all_workers } from "../CharWorkerUtils";

const { active_profile } = storeToRefs(useRosterStore());

const props = defineProps<{
  upgrade: Upgrade;
}>();

const juice_info = computed(() => {
  return active_profile.value.histogram_worker_bundle.result.juice_info;
});

const starting_artisan = ref(
  parse_locale_float((props.upgrade.starting_artisan * 100).toFixed(2)) || 0,
);
watch(
  () => props.upgrade.starting_artisan,
  () => {
    starting_artisan.value = parse_locale_float(
      (props.upgrade.starting_artisan * 100).toFixed(2),
    );
  }, // This watch is here to watch for when upgrade changes (optimizer shuffled order or tick /untick), in which case props.upgrade changes
);

const current_chance_percentage = ref(
  ((Math.min(10, props.upgrade.starting_num_taps) / 10) *
    props.upgrade.base_chance +
    props.upgrade.base_chance) *
    100 || 0,
);
watch(
  () => props.upgrade.starting_num_taps,
  () => {
    current_chance_percentage.value =
      ((Math.min(10, props.upgrade.starting_num_taps) / 10) *
        props.upgrade.base_chance +
        props.upgrade.base_chance) *
      100;
  }, // This watch is here to watch for when upgrade changes (optimizer shuffled order or tick /untick), in which case props.upgrade changes
);

function write_normal_progress() {
  starting_artisan.value = clamp_percentage(starting_artisan.value);
  active_profile.value.keyed_upgrades[
    to_upgrade_key(
      props.upgrade.piece_type,
      props.upgrade.upgrade_index,
      props.upgrade.is_normal_honing,
      active_profile.value.tier,
    )
  ].starting_artisan = starting_artisan.value / 100;

  active_profile.value.keyed_upgrades[
    to_upgrade_key(
      props.upgrade.piece_type,
      props.upgrade.upgrade_index,
      props.upgrade.is_normal_honing,
      active_profile.value.tier,
    )
  ].starting_num_taps = current_chance_to_num_taps.value;

  start_all_workers();
}
const current_chance_to_num_taps = computed(() => {
  return Math.round(
    ((current_chance_percentage.value / 100 - props.upgrade.base_chance) /
      props.upgrade.base_chance) *
      10,
  );
});

// if i want to do this properly, i'd have to consider the natural base chance increase and stuff
// it's just too much effort and this is just meant to be a VERY conservative estimate QOL anyway
// it's impossible to predict exactly without knowing how many juiced taps and stuff
const maximum_individual_chance = computed(() => {
  let relevant_upgrade =
    juice_info.value.normal_uindex_to_id[props.upgrade.upgrade_index];
  if (relevant_upgrade.length === 0) {
    console.log("zeroed");
    return props.upgrade.base_chance;
  }
  const book_id = relevant_upgrade[relevant_upgrade.length - 1] || 0;
  const out =
    props.upgrade.base_chance * 2 +
    juice_info.value.all_juices[0].data.get(String(props.upgrade.upgrade_index))
      .normal_chance +
    (relevant_upgrade.length <= 1
      ? 0
      : juice_info.value.all_juices[book_id].data.get(
          String(props.upgrade.upgrade_index),
        ).normal_chance);
  console.log(
    out,
    props.upgrade.base_chance,
    juice_info.value.all_juices[0].data.get(String(props.upgrade.upgrade_index))
      .normal_chance,
    relevant_upgrade.length,
  );
  return out;
});

const artisan_multiplier = computed(
  () => props.upgrade.artisan_rate * ARTISAN_RATE * 100,
);
function parse_current_chance() {
  current_chance_percentage.value = clean_percentage_input(
    current_chance_percentage.value,
    props.upgrade.base_chance,
  );

  current_chance_percentage.value = Math.min(
    props.upgrade.base_chance * 2 * 100,
    Math.max(props.upgrade.base_chance * 100, current_chance_percentage.value),
  );

  current_chance_percentage.value = parse_locale_float(
    (
      ((current_chance_to_num_taps.value / 10) * props.upgrade.base_chance +
        props.upgrade.base_chance) *
      100
    ).toFixed(2),
  );
  // just a convenient way to round it to the nearest possible value

  const min_artisan =
    props.upgrade.base_chance *
    artisan_multiplier.value *
    current_chance_to_num_taps.value;
  const max_artisan =
    maximum_individual_chance.value *
    artisan_multiplier.value *
    current_chance_to_num_taps.value;

  starting_artisan.value = clean_percentage_input(
    clamp(min_artisan, starting_artisan.value, max_artisan),
  );
}

function parse_artisan() {
  starting_artisan.value = clean_percentage_input(starting_artisan.value, 0);

  const min_current_chance =
    props.upgrade.base_chance +
    Math.ceil(
      starting_artisan.value /
        (artisan_multiplier.value * maximum_individual_chance.value),
    ) *
      0.1 *
      props.upgrade.base_chance;

  const max_current_chance =
    props.upgrade.base_chance +
    Math.floor(
      starting_artisan.value /
        (artisan_multiplier.value * props.upgrade.base_chance),
    ) *
      0.1;
  props.upgrade.base_chance;

  current_chance_percentage.value = clean_percentage_input(
    clamp(
      100 * props.upgrade.base_chance,
      clamp(
        max_current_chance >= props.upgrade.base_chance * 2 - FLOAT_TOL
          ? 100 * min_current_chance
          : 0,
        current_chance_percentage.value,
        100 * max_current_chance,
      ),
      100 * props.upgrade.base_chance * 2,
    ),
  );
}
</script>
<template>
  <div class="flex flex-row flex-nowrap justify-center gap-2">
    <span class="w-fit">
      Starting artisan energy:
      {{
        active_profile.optimizer_worker_bundle.status === "busy"
          ? starting_artisan.toFixed(2)
          : ""
      }}
    </span>

    <div
      v-if="active_profile.optimizer_worker_bundle.status !== 'busy'"
      class="flex flex-row"
    >
      <input
        class="generic-input w-13"
        v-model="starting_artisan"
        inputmode="decimal"
        @change="parse_artisan"
      />
      <span>%</span>
    </div>
  </div>
  <div class="flex flex-row flex-nowrap justify-center gap-2">
    <span class="w-fit">
      Current base chance
      {{
        active_profile.optimizer_worker_bundle.status === "busy"
          ? current_chance_percentage.toFixed(2) + "%"
          : ""
      }}
    </span>

    <div
      v-if="active_profile.optimizer_worker_bundle.status !== 'busy'"
      class="flex flex-row"
    >
      <input
        class="generic-input w-13"
        v-model="current_chance_percentage"
        :min="upgrade.base_chance * 100"
        :max="upgrade.base_chance * 100 * 2"
        @change="parse_current_chance"
        inputmode="decimal"
      />
      <span>%</span>
    </div>
  </div>
  <div v-if="active_profile.optimizer_worker_bundle.status !== 'busy'">
    <button @click="write_normal_progress" class="generic-button self-end!">
      Confirm
    </button>
  </div>

  <div v-else>Optimizer working...</div>
</template>
