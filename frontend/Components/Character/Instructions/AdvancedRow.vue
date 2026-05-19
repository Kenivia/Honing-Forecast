<script setup lang="ts">
import { get_piece_name, get_icon_path } from "@/Utils/Helpers";
import { computed, ref } from "vue";
import { Upgrade } from "@/Utils/KeyedUpgrades";
import ActualInstructions from "./ActualInstructions.vue";
import SuccessPopup from "./SuccessPopup.vue";
import AdvancedDetails from "./AdvancedDetails.vue";
import { get_any_overwritten, get_optimizer_working } from "./InstructionUtils";

const props = defineProps<{
  upgrade: Upgrade;
}>();

const any_overwritten = computed(get_any_overwritten);

const show_success_modal = ref(false);
function onSucceedClick() {
  show_success_modal.value = true;
}
const optimizer_working = computed(get_optimizer_working);
</script>

<template>
  <div class="flex flex-col items-center">
    <span>{{
      (upgrade.is_normal_honing ? "" : "Advanced ") +
      get_piece_name(upgrade) +
      " +" +
      String((upgrade.upgrade_index + 1) * (upgrade.is_normal_honing ? 1 : 10))
    }}</span>
    <img
      :src="get_icon_path(get_piece_name(upgrade))"
      :alt="get_piece_name(upgrade)"
      class="generic-icon h-8 w-8"
    />
  </div>

  <ActualInstructions :upgrade="props.upgrade" />

  <div class="contents" v-if="!any_overwritten">
    <div class="w-full">
      <button
        @click="onSucceedClick"
        class="generic-button w-full! text-wrap! text-(--achieved)!"
        :style="{
          '--btn-hover-bg': 'var(--bg-very-bright)',

          color: optimizer_working ? 'var(--warning-dark)' : 'var(--text-main)',
          cursor: optimizer_working ? 'not-allowed' : 'pointer',
        }"
      >
        Succeed
      </button>
    </div>
    <AdvancedDetails :upgrade="props.upgrade" />
    <SuccessPopup :upgrade="props.upgrade" v-model="show_success_modal" />
  </div>
  <div
    v-else
    class="col-span-2 flex w-full flex-col text-left"
    :style="{
      backgroundColor: any_overwritten ? 'var(--warning-dark)' : 'transparent',
    }"
  >
    <span> &lt;&lt;&lt; FOR COMPARISON PURPOSE ONLY, DO NOT FOLLOW!</span>
  </div>
</template>

<style scoped></style>
