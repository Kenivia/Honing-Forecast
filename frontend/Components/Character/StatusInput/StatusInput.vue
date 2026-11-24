<script setup lang="ts">
import TickboxGrid from "./TickboxGrid.vue";
import { storeToRefs } from "pinia";
import { check_revert_ilevel_ok } from "@/Utils/Helpers";
import TierConvertButton from "@/Components/Common/TierConvertButton.vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import Uwuowo from "@/Components/Common/Uwuowo/Uwuowo.vue";
import SignInHint from "@/Components/Common/Uwuowo/SignInHint.vue";
import QuestionMark from "@/Components/Common/QuestionMark.vue";
import RoleToggle from "@/Components/Character/RoleToggle.vue";
import { bands_editing } from "@/Components/Character/MaterialDist/LeftoverBands.vue";
import { ilevel } from "@/Utils/HoningUtil.js";
import { apply_results } from "@/Components/Common/Uwuowo/ApplyResults.js";
import {
  change_tier,
  tier_label,
  tier_tooltip,
  tier_warning,
} from "./StatusInputUtil.js";

const roster_store = useRosterStore();
const { active_profile, active_region } = storeToRefs(roster_store);

function scroll_to_bands() {
  bands_editing.value = true;
  const el = document.getElementById("leftover-bands")!;
  el.scrollIntoView({ behavior: "smooth", block: "center" });
  // flash twice once the scroll has mostly settled
  el.animate(
    [
      { backgroundColor: "transparent" },
      { backgroundColor: "var(--bg-very-bright)" },
      { backgroundColor: "transparent" },
    ],
    { duration: 500, delay: 300, iterations: 2, easing: "ease-in-out" },
  );
}
</script>

<template>
  <div
    class="flex w-fit max-w-[calc(min(100%,1032px))] flex-row flex-wrap items-start justify-center gap-2.5"
  >
    <div class="card-shell">
      <div class="card-header gap-2">
        <span class="card-title"
          >Normal Honing ({{
            active_profile.tier === 1 ? "Serca" : "T4"
          }})</span
        >
        <TierConvertButton
          compact
          :labelText="tier_label(active_profile)"
          :tooltipText="tier_tooltip(active_profile)"
          :checkEligibility="
            () => check_revert_ilevel_ok(active_profile) === true
          "
          @change-tier="() => change_tier(active_profile)"
          :show-tooltip-only-on-disabled="false"
          :warning="tier_warning(active_profile)"
        />
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
        :profile="active_profile"
        :region_change="roster_store.active_region_change"
        :region="active_region"
        :apply="
          (result, force_t4, _name, class_name) =>
            apply_results(active_profile, result, force_t4, false, class_name)
        "
      />
      <SignInHint />

      <div class="flex flex-col">
        <label class="text-no-wrap text-(--achieved)"
          >Achieved ilevel: {{ ilevel(active_profile, "achieved") }}</label
        >
        <label class="text-no-wrap text-(--pending)"
          >Pending ilevel: {{ ilevel(active_profile, "pending") }}</label
        >
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
      <div class="my-1 flex flex-row items-center gap-2">
        <span>Treat as: </span>
        <RoleToggle role="group" aria-label="Treat as" />
        <QuestionMark>
          This controls how we value leftover materials, find out more
          <button
            class="underline hover:text-(--text-bright)"
            @click="scroll_to_bands"
          >
            here</button
          >. You can have multiple 'main's
        </QuestionMark>
      </div>
    </div>
  </div>
</template>
