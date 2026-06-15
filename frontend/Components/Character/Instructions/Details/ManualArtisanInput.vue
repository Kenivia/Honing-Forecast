<script setup lang="ts">
import "@/Components/Character/Instructions/Details/details.css";
import { useRosterStore } from "@/Stores/RosterConfig";
import { to_upgrade_key, Upgrade } from "@/Utils/KeyedUpgrades";
import { storeToRefs } from "pinia";
import { computed } from "vue";
import QuestionMark from "@/Components/Common/QuestionMark.vue";

const starting_artisan = defineModel<string>("starting_artisan", {
  required: true,
});
const current_chance_percentage = defineModel<string>(
  "current_chance_percentage",
  { required: true },
);

const props = defineProps<{
  upgrade: Upgrade;
  optimizer_working: boolean;
  taps_since_last_input: number;
  normal_dist_length: number;
  using_slider: boolean;
  show_hints: boolean;
  perform_order: number;
  show_hide: boolean;
}>();

const emit = defineEmits<{
  manual_artisan_change: [];
  manual_chance_change: [];
  confirm: [];
  reset: [];
}>();

const { active_profile } = storeToRefs(useRosterStore());

const upgrade_key = computed(() =>
  to_upgrade_key(
    props.upgrade.piece_type,
    props.upgrade.upgrade_index,
    props.upgrade.is_normal_honing,
    active_profile.value.tier,
  ),
);
</script>

<template>
  <span class="stat-label">Current artisan energy:</span>
  <div class="flex flex-row">
    <div class="stat-input">
      <input
        class="generic-input number-border w-13"
        :style="{
          backgroundColor: using_slider ? 'transparent' : 'var(--bg-bright)',
        }"
        v-model="starting_artisan"
        inputmode="decimal"
        @change="emit('manual_artisan_change')"
      />
    </div>
    <QuestionMark
      v-if="show_hints"
      text="You can also input artisan directly here. However, costs will not be auto-deducted"
      position="left"
    />
  </div>
  <div class="button-row">
    <button
      @click="emit('confirm')"
      class="generic-button w-20!"
      :disabled="taps_since_last_input === normal_dist_length - 1"
      :style="{
        opacity: taps_since_last_input === normal_dist_length - 1 ? 0.5 : 1,
        cursor:
          taps_since_last_input === normal_dist_length - 1
            ? 'not-allowed'
            : 'pointer',
      }"
    >
      Confirm
    </button>
    <QuestionMark
      v-if="show_hints"
      text="Use the slider to update artisan & deduct costs, then press Confirm to re-run the optimizer. Careful that the order of the upgrades can change, check the far left for what upgrade it is. "
      position="left"
    />
  </div>
  <span class="stat-label">Current base chance:</span>
  <div class="stat-input">
    <input
      class="generic-input number-border w-10 pr-0!"
      v-model="current_chance_percentage"
      :min="upgrade.base_chance * 100 + upgrade.extra_chance"
      :max="upgrade.base_chance * 100 * 2 + upgrade.extra_chance"
      @change="emit('manual_chance_change')"
      inputmode="decimal"
      :style="{
        backgroundColor: using_slider ? 'transparent' : 'var(--bg-bright)',
      }"
    />
    <span>%</span>
  </div>
  <div class="button-row">
    <button
      class="reset-button"
      @click="emit('reset')"
      v-if="
        upgrade.starting_artisan > 0 ||
        upgrade.starting_num_taps > 0 ||
        !using_slider
      "
    >
      {{ !using_slider ? "Restore default" : "Reset this upgrade" }}
    </button>
    <div v-else class="contents">
      <button
        class="reset-button w-full self-center"
        v-if="perform_order != 0 && show_hide"
        @click="active_profile.keyed_upgrades[upgrade_key].expanded = false"
      >
        Hide
      </button>
    </div>
  </div>
</template>
