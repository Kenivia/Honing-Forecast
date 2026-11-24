<script setup lang="ts">
// A character's or roster's name: blank shows its default label as the placeholder; the store snaps what was typed
import { nextTick, ref, watch } from "vue";

const props = defineProps<{
  name: string;
  placeholder: string;
  label: string;
  invalid?: boolean;
  locked?: boolean; // shown as text
}>();
const emit = defineEmits<{ rename: [typed: string] }>();

const text = ref(props.name);
watch(
  () => props.name,
  (name) => (text.value = name),
);
async function commit() {
  emit("rename", text.value);
  // the stored name may not change (e.g. only spaces typed), so show it again
  await nextTick();
  text.value = props.name;
}
</script>

<template>
  <input
    v-model="text"
    :aria-label="label"
    :placeholder="placeholder"
    :readonly="locked"
    :tabindex="locked ? -1 : undefined"
    class="generic-input"
    :class="{
      'pointer-events-none border-transparent! bg-transparent!': locked,
      'border-(--warning)! text-(--warning)': invalid,
    }"
    @change="commit"
    @keydown.enter="($event.target as HTMLElement).blur()"
  />
</template>
