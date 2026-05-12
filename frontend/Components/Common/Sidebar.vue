<script setup lang="ts">
import { useMediaIsNarrow } from "@/Utils/WindowSize";
import { ref } from "vue";
import Footer from "./Footer.vue";

const props = withDefaults(defineProps<{ header?: string }>(), {
  header: "",
});

const isNarrow = useMediaIsNarrow(1174);

const sidebarOpen = ref(false);
const open = () => (sidebarOpen.value = true);
const close = () => (sidebarOpen.value = false);
</script>

<template>
  <div class="flex grow flex-col">
    <div class="flex min-h-full grow flex-row">
      <button
        v-if="isNarrow"
        class="burger"
        @click="open"
        aria-label="Open navigation"
      >
        <span /><span /><span />
      </button>

      <Transition name="backdrop">
        <div v-if="isNarrow && sidebarOpen" class="backdrop" @click="close" />
      </Transition>

      <nav
        class="side-bar"
        :class="{
          'side-bar--narrow': isNarrow,
          'side-bar--open': isNarrow && sidebarOpen,
        }"
      >
        <span
          class="side-bar-header items-center border-b border-(--border-muted)"
          >{{ props.header }}
        </span>
        <slot name="sidebar" :close="close" :is-narrow="isNarrow" />
        <Footer />
      </nav>

      <div class="main-stage cap-width overflow-x-auto overflow-y-hidden">
        <slot name="main" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.cap-width {
  width: calc(min(100vh, 100%));
}

/* ── Sidebar header (title + close button) ───────────────── */
.side-bar-header {
  justify-content: space-between;
  padding: 0px 0px 8px 12px;
  margin-top: -46px;
  margin-left: -8px; /* this is needed to make the bottom border look nice */
  width: 100%;
  justify-self: flex-start;
  position: absolute;
  color: var(--text-muted);
  font-size: 1.1rem;
  border-bottom: 1px solid var(--border-muted);
}

/* ── Nav links ───────────────────────────────────────────── */

.side-bar {
  width: 200px;
  flex-shrink: 0;
  gap: 2px;
  padding: calc(234px + 56px) 2px 12px 2px;
  border-right: 1px solid var(--border-muted);
  height: max(calc(100vh + 234px - 50px), 100%);
  top: calc(50px - 234px);
  justify-content: space-between;
  position: fixed;
  z-index: 1;
  background-color: var(--bg-muted);
  display: flex;
  flex-direction: column;
  overflow-x: hidden;
  overflow-y: auto;
}
.side-bar:nth-child(2) {
  margin-top: 100px;
}
@media (max-width: 1174px) {
  .side-bar {
    height: calc(100vh);
    padding-top: 56px;
    top: 0px;
  }
}
.side-bar--narrow {
  position: fixed;
  top: 0;
  left: 0;
  height: 100vh;
  z-index: 200;
  transform: translateX(-100%);
  transition:
    transform 0.1s cubic-bezier(0.4, 0, 0.2, 1),
    box-shadow 0.1s ease;
  box-shadow: none;
}

.side-bar--open {
  transform: translateX(0);
  box-shadow: 4px 0 24px rgba(0, 0, 0, 0.5);
}

.burger {
  position: fixed;
  top: 70px;
  left: 14px;
  z-index: 100;
  width: 46px;
  height: 46px;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  gap: 5px;
  padding: 6px;
  background: var(--bg-muted);
  border: 2px solid var(--text-bright);
  border-radius: 12px;
  cursor: pointer;
}

.burger:hover {
  background-color: var(--bg-very-bright);
}

.burger span {
  display: block;
  width: 22px;
  height: 2px;
  background: var(--text-bright);
  border-radius: 2px;
}

.backdrop {
  position: fixed;
  inset: 0;
  z-index: 150;
  background: rgba(0, 0, 0, 0.5);
}

.backdrop-enter-active,
.backdrop-leave-active {
  transition: opacity 0.1s ease;
}
.backdrop-enter-from,
.backdrop-leave-to {
  opacity: 0;
}

.main-stage {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 1px;
  justify-content: flex-start;
  align-items: center;
  flex-grow: 1;
  margin-left: 200px;
  max-width: 100vw;
  background-color: var(--bg-very-muted);
  padding-top: 1rem;
}

@media (max-width: 1174px) {
  .main-stage {
    margin-left: 0px;
    margin-right: auto;
  }
}
</style>
