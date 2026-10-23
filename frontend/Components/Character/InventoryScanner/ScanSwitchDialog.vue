<script setup lang="ts">
import { useRouter } from "vue-router";
import { onKeyStroke } from "@vueuse/core";
import { useRosterStore } from "@/Stores/RosterConfig";
import { move_scan, pending_switch, wipe_scan } from "./ScanSwitch";

const router = useRouter();
const store = useRosterStore();

function wipe() {
  const { name } = pending_switch.value;
  pending_switch.value = null;
  wipe_scan();
  router.push({ name: "char-scanner", params: { characterName: name } });
}

// on to the scanner, where the scan goes on
function move() {
  const { name, same_roster } = pending_switch.value;
  pending_switch.value = null;
  move_scan(same_roster);
  router.push({ name: "char-scanner", params: { characterName: name } });
}

onKeyStroke("Escape", () => (pending_switch.value = null));
</script>

<template>
  <div
    v-if="pending_switch"
    class="fixed inset-0 z-50 flex items-center justify-center bg-(--bg-void)/70 p-4"
    @click.self="pending_switch = null"
  >
    <div
      class="card-shell card-body flex max-w-lg flex-col gap-3"
      role="dialog"
      aria-modal="true"
      aria-label="Switch character"
    >
      <span class="font-semibold">Switch to {{ pending_switch.name }}?</span>
      <span class="text-sm text-(--text-muted)">
        The scan on {{ store.active_profile.char_name }} has changes that were
        not exported. If it was meant for {{ pending_switch.name }}, move it
        there and carry on.
      </span>
      <span
        v-if="!pending_switch.same_roster"
        class="text-sm text-(--text-muted)"
      >
        {{ pending_switch.name }} is on another roster: moved, what was read in
        roster storage and the roster-wide currencies count for that roster.
      </span>
      <div class="flex flex-wrap gap-2">
        <button class="generic-button" @click="move">
          Move the scan to {{ pending_switch.name }}
        </button>
        <button class="generic-button text-(--warning)" @click="wipe">
          Wipe the scan
        </button>
        <button class="generic-button" @click="pending_switch = null">
          Cancel
        </button>
      </div>
    </div>
  </div>
</template>
