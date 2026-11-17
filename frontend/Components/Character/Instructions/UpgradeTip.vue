<script setup lang="ts">
import { computed } from "vue";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { ALL_LABELS, NUM_BASE_MATS } from "@/Utils/Constants";
import { Upgrade } from "@/Utils/KeyedUpgrades";
import ItemList from "./ItemList.vue";

// One attempt of an upgrade: base chance and costs, then each optional juice or book.
const props = defineProps<{ upgrade: Upgrade }>();
const { active_profile } = storeToRefs(useRosterStore());
const runtime = useRuntimeStore();

const percent = (x: number) =>
  `${(x * 100).toLocaleString("en-US", { maximumFractionDigits: 2 })}%`;

const labels = computed(() => ALL_LABELS[active_profile.value.tier]);
const base = computed(() =>
  props.upgrade.costs
    .slice(0, NUM_BASE_MATS)
    .map((amount, i) => ({ label: labels.value[i], amount: Math.ceil(amount) }))
    .filter((item) => item.amount > 0),
);
const optional = computed(() => {
  const u = props.upgrade;
  const info = runtime.histogram.result.juice_info;
  const ids: number[] = (
    u.is_normal_honing ? info.normal_uindex_to_id : info.adv_uindex_to_id
  )[u.piece_type_usize][u.upgrade_index];
  return ids.map((id) => {
    // Rust's HashMap arrives as a Map
    const one = info.all_juices[id][u.piece_type_usize].get(u.upgrade_index);
    return {
      label: labels.value[id + NUM_BASE_MATS],
      amount: u.is_normal_honing ? one.normal_amt_used : one.adv_amt_used,
      effect: u.is_normal_honing
        ? `+${percent(one.normal_chance)} chance`
        : `${percent(one.adv_chances[0])} / ${percent(one.adv_chances[1])}`,
    };
  });
});
</script>

<template>
  <div class="grid grid-cols-[auto_auto] gap-x-6 gap-y-1">
    <div class="flex flex-col gap-1">
      <span class="text-(--text-muted)">Each attempt</span>
      <span v-if="upgrade.is_normal_honing"
        >Base chance <b>{{ percent(upgrade.base_chance) }}</b></span
      >
      <span v-for="item in base" :key="item.label"
        ><ItemList :items="[item]" icon_first
      /></span>
    </div>
    <div v-if="optional.length" class="flex flex-col gap-1">
      <span class="text-(--text-muted)">Optional, per attempt</span>
      <span v-for="item in optional" :key="item.label"
        ><ItemList :items="[item]" icon_first />
        <span class="ml-1.5 text-(--text-muted)">{{ item.effect }}</span></span
      >
    </div>
  </div>
</template>
