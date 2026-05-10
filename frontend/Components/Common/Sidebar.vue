<script setup lang="ts">
import { useMediaIsNarrow } from "@/Utils/WindowSize";
import { ref } from "vue";
import Footer from "./Footer.vue";

const props = withDefaults(defineProps<{ header?: string }>(), {
    header: "",
});

const { isNarrow } = useMediaIsNarrow(1174);

const sidebarOpen = ref(false);
const open = () => (sidebarOpen.value = true);
const close = () => (sidebarOpen.value = false);
</script>

<template>
    <div class="main-slot">
        <div class="sb-layout">
            <button v-if="isNarrow" class="sb-burger" @click="open" aria-label="Open navigation">
                <span /><span /><span />
            </button>

            <Transition name="sb-backdrop">
                <div v-if="isNarrow && sidebarOpen" class="sb-backdrop" @click="close" />
            </Transition>

            <nav
                class="sb-side-bar"
                :class="{
                    'sb-side-bar--narrow': isNarrow,
                    'sb-side-bar--open': isNarrow && sidebarOpen,
                }"
            >
                <span class="hf-side-bar-header hf-side-bar-title">{{ props.header }} </span>
                <slot name="sidebar" :close="close" :is-narrow="isNarrow" />
                <Footer />
            </nav>

            <div class="sb-main-stage">
                <slot name="main" />
            </div>
        </div>
    </div>
</template>

<style>
.main-slot {
    flex-grow: 1;
    display: flex;
    flex-direction: column;
}

/* ── Sidebar header (title + close button) ───────────────── */
.hf-side-bar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 4px 12px;
    border-bottom: 1px solid var(--hf-border);
    margin-top: -46px;
    width: 100%;
    justify-self: flex-start;
    position: absolute;
}
/* @media (max-width: 1174px) {
    .hf-side-bar-header {
        margin-top: 0;
    }
} */
.hf-side-bar-title {
    color: var(--hf-text-muted);
    user-select: none;
    align-items: center;
    font-size: 1.1rem;
}

/* ── Nav links ───────────────────────────────────────────── */
.hf-side-bar-item {
    /* display: block; */
    padding: 8px 12px;
    border-radius: 6px;
    color: var(--hf-text-bright);
    text-decoration: none;
    white-space: nowrap;
    text-overflow: ellipsis;
    overflow: hidden;
    transition:
        color 0.1s ease,
        background-color 0.1s ease;
    cursor: pointer;
}

.hf-side-bar-item:hover {
    background-color: var(--hf-bg-card);
}

/* RouterLink active state */
.hf-side-bar-item.router-link-active {
    color: var(--hf-gold);
    background-color: var(--hf-bg-card);
}

.sb-layout {
    display: flex;
    flex-direction: row;
    min-height: 100%;
    flex-grow: 1;
}

.sb-side-bar {
    width: 200px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: calc(234px + 56px) 8px 12px 8px;
    background-color: var(--hf-bg-deep);
    border-right: 1px solid var(--hf-border);
    overflow: scroll;

    height: max(calc(100vh + 234px - 50px), 100%);

    top: calc(50px - 234px);
    justify-content: space-between;
    position: fixed;
    z-index: 1;
    overflow-y: auto;
    overflow-x: hidden;
}
.sb-side-bar:nth-child(2) {
    margin-top: 100px;
}
@media (max-width: 1174px) {
    .sb-side-bar {
        height: calc(100vh);
        padding-top: 56px;
        top: 0px;
    }
}
.sb-side-bar--narrow {
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

.sb-side-bar--open {
    transform: translateX(0);
    box-shadow: 4px 0 24px rgba(0, 0, 0, 0.5);
}

.sb-burger {
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
    background: var(--hf-bg-raised);
    border: 2px solid var(--hf-text-bright);
    border-radius: 12px;
    cursor: pointer;
    transition: background-color 0.1s ease;
}

.sb-burger:hover {
    background-color: var(--hf-bg-hover);
}

.sb-burger span {
    display: block;
    width: 22px;
    height: 2px;
    background: var(--hf-text-bright);
    border-radius: 2px;
}

.sb-backdrop {
    position: fixed;
    inset: 0;
    z-index: 150;
    background: rgba(0, 0, 0, 0.5);
}

.sb-backdrop-enter-active,
.sb-backdrop-leave-active {
    transition: opacity 0.1s ease;
}
.sb-backdrop-enter-from,
.sb-backdrop-leave-to {
    opacity: 0;
}

.sb-main-stage {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 8px;
    justify-content: flex-start;
    align-items: center;
    flex-grow: 1;
    margin-left: calc(200px);
    max-width: 100vw;
    background-color: var(--bg-very-dark);
}

@media (max-width: 1174px) {
    .sb-main-stage {
        margin-left: 0px;
        margin-right: auto;
    }
}
</style>
