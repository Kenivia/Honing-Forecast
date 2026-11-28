<script setup lang="ts">
import { useMediaQuery } from "@vueuse/core";
import { ref } from "vue";
import Footer from "./Footer.vue";
import TopBanners from "./TopBanners.vue";

const props = withDefaults(defineProps<{ header?: string; width: number }>(), {
  header: "",
});

// extra sidebar width for a longer header; keep in sync with .side-bar's --extra
const EXTRA = 20;
const isNarrow = useMediaQuery(
  () => `(max-width: ${props.width + EXTRA}px)`,
);

const sidebarOpen = ref(false);
const open = () => (sidebarOpen.value = true);
const close = () => (sidebarOpen.value = false);
</script>

<template>
  <div class="flex grow flex-col">
    <div class="flex min-h-full grow flex-row">
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
        <span class="side-bar-header" :title="props.header">{{
          props.header
        }}</span>
        <div class="side-bar-body">
          <slot name="sidebar" :close="close" :is-narrow="isNarrow" />
          <Footer />
        </div>
      </nav>

      <div class="flex min-w-0 grow flex-col">
        <slot name="top" />
        <!-- App.vue's header holds it, left of the tabs -->
        <Teleport defer to="#header-burger">
          <button
            v-if="isNarrow"
            class="burger"
            @click="open"
            aria-label="Open navigation"
          >
            <span /><span /><span />
          </button>
        </Teleport>
        <div class="main-stage scroll-x w-full">
          <TopBanners />
          <slot name="main" />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* ── Sidebar header: the same height as the Characters tab row (min-h-11) so the borders line up ── */
.side-bar-header {
  flex-shrink: 0;
  height: 2.75rem;
  line-height: calc(2.75rem - 1px);
  padding: 0 6px;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  color: var(--text-muted);
  font-size: 1.1rem;
  border-bottom: 1px solid var(--border-muted);
}

/* links, controls and footer; each sets its own indent */
.side-bar-body {
  display: flex;
  flex-direction: column;
  flex-grow: 1;
  justify-content: space-between;
  gap: 2px;
  padding: 12px 2px;
}

/* ── Nav links ───────────────────────────────────────────── */

.side-bar {
  --extra: 20px;
  width: calc(200px + var(--extra));
  flex-shrink: 0;
  border-right: 1px solid var(--border-muted);
  height: calc(100vh - var(--top-row-h));
  top: var(--top-row-h);
  align-self: flex-start;
  position: sticky;
  z-index: 1;
  background-color: var(--bg-muted);
  display: flex;
  flex-direction: column;
  overflow-x: hidden;
  overflow-y: auto;
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
  width: 48px;
  align-self: stretch;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  gap: 5px;
  background: var(--bg-muted);
  border: 2px solid var(--text-bright);
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
  background: var(--backdrop);
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
  max-width: 100vw;
  background-color: var(--bg-very-muted);
  padding-top: 1rem;
}
</style>
