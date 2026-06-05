<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { format_char_name } from "@/Utils/Helpers";
import { storeToRefs } from "pinia";
import { computed, ref } from "vue";
const { active_profile, all_profiles } = storeToRefs(useRosterStore());

const char_name = ref(active_profile.value.char_name);
const profile_index = computed(() =>
  all_profiles.value.findIndex(
    (profile) => profile.char_name === active_profile.value.char_name, // a bit cursed but shoul be fine
  ),
);
</script>

<template>
  <div class="flex flex-row items-center gap-2">
    <span class="w-fit"> Char name:</span
    ><input
      v-model="char_name"
      @change="
        () => {
          char_name = format_char_name(char_name, profile_index);
          active_profile.char_name = char_name;
        }
      "
      class="generic-input bg-(--bg-bright)!"
    />
  </div>
</template>
