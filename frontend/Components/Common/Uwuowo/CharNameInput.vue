<script setup lang="ts">
import { format_char_name } from "@/Utils/Helpers";

import { ref } from "vue";

const props = defineProps<{
  char_name: string;
  profile_index: number;
}>();

const char_name = ref(props.char_name);

const emit = defineEmits<{
  char_name_change: [new_name: string];
}>();
</script>

<template>
  <div class="flex h-fit w-min basis-full flex-row justify-around gap-1">
    <span class="w-fit text-nowrap"> Char name:</span
    ><input
      v-model="char_name"
      @change="
        () => {
          const new_name = format_char_name(char_name, profile_index);
          // active_profile.char_name = char_name;
          emit('char_name_change', new_name);
        }
      "
      class="generic-input min-w-full"
    />
  </div>
</template>
