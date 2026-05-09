<script setup lang="ts">
import { useRosterStore } from "@/stores/roster_config";
import { LATEST_VERSION, minor_version_equal } from "@/utils/change_log";
import { storeToRefs } from "pinia";
import { ref, watch } from "vue";
import { RouterLink } from "vue-router";

const { roster_config } = storeToRefs(useRosterStore());
const show_new_tag = ref(!minor_version_equal(roster_config.value.last_seen_version, LATEST_VERSION));

watch(
    () => roster_config.value.last_seen_version,
    () => {
        show_new_tag.value = false;
    },
);
</script>

<template>
    <footer class="hf-footer-bar">
        <a href="https://discord.gg/KWDpQyvgzc" class="hf-footer-links">
            <img src="/Icons/Discord.png" alt="Discord" />
            <span>Discord</span>
        </a>
        <RouterLink :to="{ name: 'change-logs-root' }" class="hf-footer-links">
            <img src="/Icons/branch.png" alt="Ko-fi" />
            <span>{{ LATEST_VERSION }}</span>
            <span class="hf-new-tag" v-if="show_new_tag">new!</span>
        </RouterLink>

        <a href="https://ko-fi.com/kenivia" class="hf-footer-links">
            <img src="/Icons/kofi.png" alt="Ko-fi" />
            <span>Donate</span>
        </a>
        <a href="https://github.com/Kenivia/Honing-Forecast" class="hf-footer-links">
            <img src="/Icons/GitHub.png" alt="GitHub" />
            <span>GitHub</span>
        </a>
    </footer>
</template>
<style>
.hf-new-tag {
    background-color: var(--hf-text-muted);
    color: var(--hf-bg-void);
    font-size: 8px;
    border-radius: 3px;
    align-self: flex-start;
    top: -3px;
    left: -4px;
    position: relative;
    padding: 0px 2px;
}
.hf-footer-bar {
    display: grid;
    grid-template-columns: 90px 90px;
    align-items: center;
    padding: 0px 10px;
}

.hf-footer-note {
    color: var(--text-very-muted);
    font-size: 12px;
    text-align: center;
}
.hf-footer-links {
    display: flex;
    gap: 6px;
    align-items: center;
    color: var(--hf-text-muted);

    padding: 2px 12px;
    font-size: 11px;
    height: fit-content;
}
.hf-footer-links:hover {
    color: var(--hf-text-bright);
}

.hf-footer-links img {
    height: 12px;
    filter: brightness(0.8);
}
</style>
