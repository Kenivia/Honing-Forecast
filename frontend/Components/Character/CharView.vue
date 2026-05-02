<script setup lang="ts">
import Instructions from "@/Components/Character/Instructions/Instructions.vue"
import MaterialDist from "@/Components/Character/MaterialDist/MaterialDist.vue"
import StatusInput from "@/Components/Character/StatusInput/StatusInput.vue"
import { useRosterStore } from "@/Stores/RosterConfig"
import { useMediaIsNarrow } from "@/Utils/WindowSize"
import { storeToRefs } from "pinia"
import { Button } from "primevue"
import { onUnmounted, ref, watch } from "vue"
import { RouterLink, useRoute, useRouter } from "vue-router"
import ControlPanel from "./ControlPanel.vue"

const route = useRoute()
const router = useRouter()

const roster_store = useRosterStore()
const { active_profile, all_profiles } = storeToRefs(roster_store)

const match = all_profiles.value.findIndex((c) => c.char_name === (route.params.characterName as string))
if (match >= 0) {
    roster_store.switchProfile(match)
} else {
    router.replace({ name: "char", params: { characterName: all_profiles.value[0].char_name } })
    roster_store.switchProfile(0)
}
watch(
    () => route.params.characterName as string,
    (name) => {
        const match = all_profiles.value.findIndex((c) => c.char_name === name)
        if (match >= 0) {
            if (roster_store.roster_config.active_profile_index !== match) {
                // this happens one invalid names (routre param written to by the one-off code, triggering the watcher) i believe, idk how to prevent that but this works
                active_profile.value.optimizer_worker_bundle.cancel()
                active_profile.value.histogram_worker_bundle.cancel()
                active_profile.value.evaluation_worker_bundle.cancel()

                roster_store.switchProfile(match)
            }
        } else {
            router.replace({ name: "char", params: { characterName: all_profiles.value[0].char_name } })
            roster_store.switchProfile(0)
        }
    },
)

onUnmounted(() => {
    // kill workers when going to market / roster view
    active_profile.value.optimizer_worker_bundle.cancel()
    active_profile.value.histogram_worker_bundle.cancel()
    active_profile.value.evaluation_worker_bundle.cancel()
})

const { isNarrow: is1200Narrow } = useMediaIsNarrow(1200)

const sidebarOpen = ref(false)

const openSidebar = () => {
    sidebarOpen.value = true
}

const closeSidebar = () => {
    sidebarOpen.value = false
}
</script>

<template>
    <div class="hf-layout">
        <button v-if="is1200Narrow" class="hf-burger" @click="openSidebar" aria-label="Open navigation"><span /><span /><span /></button>

        <Transition name="hf-backdrop">
            <div v-if="is1200Narrow && sidebarOpen" class="hf-backdrop" @click="closeSidebar" />
        </Transition>

        <!-- Sidebar -->
        <nav
            class="hf-side-bar"
            :class="{
                'hf-side-bar--narrow': is1200Narrow,
                'hf-side-bar--open': is1200Narrow && sidebarOpen,
            }"
        >
            <div class="hf-side-bar-header">
                <span class="hf-side-bar-title">{{ route.params.characterName }}</span>
                <button v-if="is1200Narrow" class="hf-close-btn" @click="closeSidebar" aria-label="Close navigation">✕</button>
            </div>

            <RouterLink to="calc" class="hf-side-bar-item" @click="closeSidebar"> Setup & Cost Analysis </RouterLink>
            <RouterLink to="instructions" class="hf-side-bar-item" @click="closeSidebar"> Taps Instructions </RouterLink>
            <ControlPanel v-if="route.path.endsWith('calc')" />
            <footer class="hf-footer-bar">
                <a href="https://ko-fi.com/kenivia" class="hf-header-links">
                    <img src="/Icons/kofi.png" alt="Ko-fi" />
                    <span>Donate</span>
                </a>
                <a href="https://discord.gg/KWDpQyvgzc" class="hf-header-links">
                    <img src="/Icons/Discord.png" alt="Discord" />
                    <span>Discord</span>
                </a>
                <a href="https://github.com/Kenivia/Honing-Forecast" class="hf-header-links">
                    <img src="/Icons/GitHub.png" alt="GitHub" />
                    <span>GitHub</span>
                </a>
                <span class="hf-footer-note">Made with love by Kenivia with help from many awesome people.</span>
            </footer>
        </nav>

        <!-- Main content -->
        <div class="hf-main-stage" :key="active_profile.char_name">
            <StatusInput v-if="route.path.endsWith('calc')" />
            <MaterialDist v-if="route.path.endsWith('calc')" />
            <Instructions v-if="route.path.endsWith('instructions')" />
        </div>
    </div>
</template>

<style scoped>
/* ── Layout shell ────────────────────────────────────────── */
.hf-layout {
    display: flex;
    flex-direction: row;
    min-height: 100%;
    flex-grow: 1;
}

/* ── Sidebar ─────────────────────────────────────────────── */
.hf-side-bar {
    width: 200px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 12px 8px;
    background-color: var(--hf-bg-deep);
    border-right: 1px solid var(--hf-border);
    overflow: hidden;
    min-height: 100%;
}

/* Narrow: slide in from left as a fixed overlay */
.hf-side-bar--narrow {
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

.hf-side-bar--open {
    transform: translateX(0);
    box-shadow: 4px 0 24px rgba(0, 0, 0, 0.5);
}

/* ── Sidebar header (title + close button) ───────────────── */
.hf-side-bar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 8px 12px;
    border-bottom: 1px solid var(--hf-border);
    margin-bottom: 6px;
}

.hf-side-bar-title {
    color: var(--hf-text-muted);
    user-select: none;
    font-size: 1.1rem;
}

/* ── Nav links ───────────────────────────────────────────── */
.hf-side-bar-item {
    display: block;
    padding: 8px 12px;
    border-radius: 6px;
    color: var(--hf-text-bright);
    text-decoration: none;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
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

/* ── Burger button ───────────────────────────────────────── */
.hf-burger {
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

.hf-burger:hover {
    background-color: var(--hf-bg-hover);
}

.hf-burger span {
    display: block;
    width: 22px;
    height: 2px;
    background: var(--hf-text-bright);
    border-radius: 2px;
}

/* ── Close button (inside sidebar on narrow) ─────────────── */
.hf-close-btn {
    background: none;
    border: none;
    color: var(--hf-text-muted);
    cursor: pointer;
    padding: 6px 6px;
    border-radius: 4px;
    transition:
        color 0.1s ease,
        background-color 0.1s ease;
    line-height: 1;
}

.hf-close-btn:hover {
    color: var(--hf-text);
    background-color: var(--hf-bg-hover);
}

/* ── Backdrop ────────────────────────────────────────────── */
.hf-backdrop {
    position: fixed;
    inset: 0;
    z-index: 150;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(2px);
}

.hf-backdrop-enter-active,
.hf-backdrop-leave-active {
    transition: opacity 0.1s ease;
}

.hf-backdrop-enter-from,
.hf-backdrop-leave-to {
    opacity: 0;
}

/* ── Main stage ──────────────────────────────────────────── */
.hf-main-stage {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 8px;
    justify-content: flex-start;
    align-items: center;
    flex-grow: 1;
    margin-left: max(0, calc(50vw - 200px - 1000px));
}

@media (max-width: 1000px) {
    .hf-main-stage {
        margin-left: auto;
    }
}
</style>
