<script setup lang="ts">
defineProps<{ show_popup: boolean; label?: string }>();
defineEmits<{ close_popup: [] }>();
</script>
<template>
  <Teleport to="body">
    <div v-if="show_popup" class="popup-overlay" @click="$emit('close_popup')">
      <div class="popup" role="dialog" :aria-label="label" @click.stop>
        <slot />
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.popup-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

.popup {
  background: var(--bg-main);
  border: 1px solid var(--border-muted);
  border-radius: 8px;
  padding: 1.5rem;
  max-width: 935px;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 10px 30px var(--backdrop);
  width: fit-content;
  overflow-y: auto;
}
</style>
