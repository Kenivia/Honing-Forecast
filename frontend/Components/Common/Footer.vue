<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { LATEST_VERSION, version_equal } from "@/Utils/Changelog";
import { storeToRefs } from "pinia";
import { ref, watch } from "vue";
import { RouterLink } from "vue-router";

const { roster_config } = storeToRefs(useRosterStore());
const show_new_tag = ref(
  !version_equal(roster_config.value.last_seen_version, LATEST_VERSION),
);

watch(
  () => roster_config.value.last_seen_version,
  () => {
    show_new_tag.value = false;
  },
);
</script>

<template>
  <footer class="footer-bar">
    <a href="https://discord.gg/KWDpQyvgzc" class="footer-links">
      <img src="/Icons/Discord.png" alt="Discord" />
      <span>Discord</span>
    </a>
    <RouterLink :to="{ name: 'change-logs-root' }" class="footer-links">
      <img src="/Icons/branch.png" alt="Ko-fi" />
      <span>{{ LATEST_VERSION }}</span>
      <span class="new-tag" v-if="show_new_tag">new!</span>
    </RouterLink>

    <a href="https://ko-fi.com/kenivia" class="footer-links">
      <img src="/Icons/kofi.png" alt="Ko-fi" />
      <span>Donate</span>
    </a>
    <a href="https://github.com/Kenivia/Honing-Forecast" class="footer-links">
      <img src="/Icons/GitHub.png" alt="GitHub" />
      <span>GitHub</span>
    </a>
  </footer>
</template>
<style scoped>
.new-tag {
  background-color: var(--text-muted);
  color: var(--bg-void);
  font-size: 8px;
  border-radius: 3px;
  align-self: flex-start;
  top: -3px;
  left: -4px;
  position: relative;
  padding: 0px 2px;
}
.footer-bar {
  display: grid;
  grid-template-columns: 90px 90px;
  align-items: center;
  padding: 0px 10px;
}

.footer-note {
  color: var(--text-very-muted);
  font-size: 12px;
  text-align: center;
}
.footer-links {
  display: flex;
  gap: 6px;
  align-items: center;
  color: var(--text-muted);

  padding: 2px 12px;
  font-size: 11px;
  height: fit-content;
}
.footer-links:hover {
  color: var(--text-bright);
}

.footer-links img {
  width: 12px;
  filter: brightness(0.8);
  background-color: transparent;
}

.footer-links:hover img {
  filter: brightness(1);
}
</style>
