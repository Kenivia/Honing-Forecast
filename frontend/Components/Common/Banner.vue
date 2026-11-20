<script setup lang="ts">
// A strip across the top of the main area (Sidebar.vue); text in the default slot, buttons in #actions.
withDefaults(defineProps<{ tone?: "warning" | "info" }>(), {
  tone: "warning",
});
</script>

<template>
  <div
    :role="tone === 'warning' ? 'alert' : 'status'"
    class="m-auto flex w-fit flex-wrap items-center gap-x-4 gap-y-2 border-x border-b bg-(--bg-muted) px-4 py-2 text-sm [.main-stage>&:first-child]:-mt-4"
    :class="
      tone === 'warning' ? 'border-(--warning-dark)' : 'border-(--bg-bright)'
    "
  >
    <span :class="{ 'text-(--warning)': tone === 'warning' }"><slot /></span>
    <slot name="actions" />
  </div>
</template>
