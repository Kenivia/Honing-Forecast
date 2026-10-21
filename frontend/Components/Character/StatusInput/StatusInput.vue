<script setup lang="ts">
import TickboxGrid from "./TickboxGrid.vue";
import { storeToRefs } from "pinia";
import { check_revert_ilevel_ok } from "@/Utils/Helpers";
import TierConvertButton from "@/Components/Common/TierConvertButton.vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import Uwuowo from "@/Components/Common/Uwuowo/Uwuowo.vue";
import { ilevel } from "@/Utils/HoningUtil.js";
import { apply_results } from "@/Components/Common/Uwuowo/ApplyResults.js";
import {
  change_tier,
  tier_label,
  tier_tooltip,
  tier_warning,
} from "./StatusInputUtil.js";

const roster_store = useRosterStore();
const { active_profile, all_profiles, active_region } =
  storeToRefs(roster_store);
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
          all_profiles.findIndex((profile) => profile.id === active_profile.id)
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
          :labelText="tier_label(active_profile)"
          :tooltipText="tier_tooltip(active_profile)"
          :checkEligibility="
            () => check_revert_ilevel_ok(active_profile) === true
          "
          @change-tier="() => change_tier(active_profile)"
          :show-tooltip-only-on-disabled="false"
          :warning="tier_warning(active_profile)"
        />
        <label class="control-panel-checkbox-row border-0!">
          <input disabled type="checkbox" class="cursor-not-allowed" />
          <span class="text-(--text-muted)"
            >September expresses have no honing discount</span
          >
        </label>
        <label
          v-if="active_profile.tier == 0"
          class="control-panel-checkbox-row border-0!"
        >
          <input disabled checked type="checkbox" class="cursor-not-allowed" />
          <span class="text-(--text-muted)"
            >All adv honing now have x2 grace</span
          >
        </label>
      </div>
    </div>
  </div>
</template>
