<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { Upgrade } from "@/Utils/KeyedUpgrades";
import { storeToRefs } from "pinia";
import { ref, watch } from "vue";

const { active_profile } = storeToRefs(useRosterStore());

const props = defineProps<{
  upgrade: Upgrade;
}>();

// In Rust start_xp ranges from 0 to 100 (each bar = 10 xp instead of 100 in game)
const current_adv_upgrade = ref(
  props.upgrade.adv_config
    ? Math.floor(props.upgrade.adv_config.start_xp / 10) +
        props.upgrade.upgrade_index * 10
    : 0,
);
const current_adv_xp = ref(
  props.upgrade.adv_config
    ? (props.upgrade.adv_config.start_xp -
        Math.floor(props.upgrade.adv_config.start_xp / 10) * 10) *
        10
    : 0,
);
const current_grace_progress = ref(props.upgrade.adv_config.start_balls);
const next_free = ref(props.upgrade.adv_config?.next_free ?? false);
const next_big = ref(props.upgrade.adv_config?.next_big ?? false);

watch(
  [
    () => props.upgrade.adv_config.start_xp,
    () => props.upgrade.adv_config.start_balls,
    () => props.upgrade.adv_config.next_big,
    () => props.upgrade.adv_config.next_free,
  ],
  () => {
    current_adv_upgrade.value = props.upgrade.adv_config
      ? Math.floor(props.upgrade.adv_config.start_xp / 10) +
        props.upgrade.upgrade_index * 10
      : 0;
    ((current_adv_xp.value = props.upgrade.adv_config
      ? (props.upgrade.adv_config.start_xp -
          Math.floor(props.upgrade.adv_config.start_xp / 10) * 10) *
        10
      : 0),
      (current_grace_progress.value = props.upgrade.adv_config.start_balls));
    next_free.value = props.upgrade.adv_config?.next_free ?? false;
    next_big.value = props.upgrade.adv_config?.next_big ?? false;
  },
);

// function write_adv_progress() {
//     current_adv_upgrade.value = Math.max(props.upgrade.upgrade_index * 10, Math.min((props.upgrade.upgrade_index + 1) * 10 - 1, current_adv_upgrade.value))
//     current_adv_xp.value = Math.floor(Math.max(0, Math.min(90, current_adv_xp.value)) / 10) * 10
//     // console.log(current_grace_progress.value)
//     current_grace_progress.value = Math.min(6, Math.max(0, current_grace_progress.value))

//     active_profile.value.keyed_upgrades[
//         to_upgrade_key(props.upgrade.piece_type, props.upgrade.upgrade_index, props.upgrade.is_normal_honing, active_profile.value.tier)
//     ][7] = [
//         (current_adv_upgrade.value - props.upgrade.upgrade_index * 10) * 10 + current_adv_xp.value / 10,
//         current_grace_progress.value,
//         next_free.value,
//         next_big.value,
//     ]
//     start_all_workers()
// }

const must_show = ref(false);

watch(
  [
    () => props.upgrade.starting_artisan,
    () => props.upgrade.is_normal_honing,
    () => props.upgrade.adv_config.start_balls,
    () => props.upgrade.adv_config.start_xp,
    () => props.upgrade.adv_config.next_big,
    () => props.upgrade.adv_config.next_free,
  ],
  () => {
    if (props.upgrade.is_normal_honing) {
      must_show.value = props.upgrade.starting_artisan > 0;
    } else {
      must_show.value =
        props.upgrade.adv_config.start_balls > 0 ||
        props.upgrade.adv_config.start_xp > 0 ||
        props.upgrade.adv_config.next_big ||
        props.upgrade.adv_config.next_free;
    }
  },
  { immediate: true },
);
</script>
<template>

    
</template>
