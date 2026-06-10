<script setup lang="ts">
import TickboxGrid from "./TickboxGrid.vue";
import { storeToRefs } from "pinia";
import {
  check_adv_all_done,
  check_all_plus_20,
  check_revert_ilevel_ok,
} from "@/Utils/Helpers";
import { computed } from "vue";
import {
  ADV_COLS,
  ALL_LABELS,
  NORMAL_COLS,
  NUM_PIECES,
  PLUS_TIER_CONVERSION,
} from "@/Utils/Constants";
import TierConvertButton from "@/Components/Common/TierConvertButton.vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import Uwuowo from "@/Components/Common/Uwuowo/Uwuowo.vue";
import { ilevel } from "@/Utils/HoningUtil.js";
import { apply_results } from "@/Components/Common/Uwuowo/ApplyResults.js";
import { change_tier } from "./StatusInputUtil.js";

const roster_store = useRosterStore();
const { active_profile, all_profiles, active_region } =
  storeToRefs(roster_store);

const tooltip_text = computed(() => {
  return active_profile.value.tier == 0
    ? check_all_plus_20() && check_adv_all_done()
      ? "Eligible for conversion to T4.5 Serca gear"
      : "Warning! " +
        [
          !check_adv_all_done() ? "All Adv honing will be set to +40" : null,
          !check_all_plus_20() ? "All Gear will be set to +11 (Serca)" : null,
        ]
          .filter((x) => x !== null)
          .join(", \n")
    : check_revert_ilevel_ok() === true
      ? "Can go back to T4"
      : "Cannot convert back to T4 because +" +
        String(check_revert_ilevel_ok()) +
        " cannot be mapped directly to a T4 upgrade";
});
const tier_label_text = computed(() => {
  return active_profile.value.tier == 0
    ? "Convert to T4.5 Serca"
    : "Revert back to T4";
});
</script>

<template>
  <div
    class="flex w-fit max-w-[calc(min(100%,1055px))] flex-row flex-wrap items-start justify-center gap-2.5"
  >
    <div class="card-shell">
      <div class="card-header">
        <span class="card-title"
          >Normal Honing ({{
            active_profile.tier === 1 ? "Serca" : "T4"
          }})</span
        >
      </div>

      <TickboxGrid grid_type="normal" />
    </div>

    <div v-if="active_profile.tier == 0" class="card-shell w-fit max-w-full">
      <div class="card-header">
        <span class="card-title">Advanced Honing</span>
      </div>

      <TickboxGrid grid_type="adv" />
    </div>
    <div class="card-shell w-fit max-w-full p-2">
      <Uwuowo
        :name="active_profile.char_name"
        :profile_index="
          all_profiles.findIndex(
            (profile) => profile.char_name === active_profile.char_name, // a bit cursed but shoul be fine
          )
        "
        :name_change="(new_name) => (active_profile.char_name = new_name)"
        :region_change="roster_store.active_region_change"
        :region="active_region"
        :apply="
          (result, force_t4) =>
            apply_results(active_profile, result, force_t4, false)
        "
      />
      <div class="flex flex-col">
        <label class="text-no-wrap text-(--achieved)"
          >Achieved ilevel: {{ ilevel(active_profile, "achieved") }}</label
        >
        <label class="text-no-wrap text-(--pending)"
          >Pending ilevel: {{ ilevel(active_profile, "pending") }}</label
        >
        <TierConvertButton
          :labelText="tier_label_text"
          :tooltipText="tooltip_text"
          :checkEligibility="() => check_revert_ilevel_ok() === true"
          @change-tier="() => change_tier(active_profile)"
          :show-tooltip-only-on-disabled="false"
          :warning="
            active_profile.tier == 0 &&
            !(check_all_plus_20() && check_adv_all_done())
          "
        />
        <label class="control-panel-checkbox-row border-0!">
          <input v-model="active_profile.express_event" type="checkbox" />
          <span>Express event (June)</span>
        </label>
      </div>
    </div>
  </div>
</template>
