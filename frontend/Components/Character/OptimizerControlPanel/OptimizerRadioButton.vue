<script setup lang="ts" generic="T">
defineProps<{
  options: { value: T; label: string }[];
  modelValue: T;
  onChange: (value: T) => void;
}>();
</script>

<template>
  <div class="flex grow flex-col">
    <label
      v-for="option in options"
      :key="String(option.value)"
      class="control-panel-checkbox-row"
    >
      <input
        type="radio"
        :checked="modelValue === option.value"
        @click="onChange(option.value)"
      />
      <span>{{ option.label }}</span>
    </label>
  </div>
</template>
