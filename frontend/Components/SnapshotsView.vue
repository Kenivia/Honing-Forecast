<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { BAND_COLORS } from "@/Utils/Constants";
import { get_icon_path } from "@/Utils/Helpers";
import {
  DIFF_COLUMNS,
  Snapshot,
  snapshot_diff,
  snapshot_of,
  take_snapshot,
} from "@/Utils/Scans";
import Sidebar from "./Common/Sidebar.vue";

// The budget snapshots of a roster, and what changed between two of them (or one and now).
const { roster_config, roster_ids, active_profile } =
  storeToRefs(useRosterStore());

const roster_id = ref(active_profile.value.roster_id);
// null is every character of the roster
const char_id = ref<string | null>(null);
// null is the budgets as they are now
const from = ref<Snapshot | null>(null);
const to = ref<Snapshot | null>(null);

const snapshots = computed(() =>
  roster_config.value.snapshots
    .filter((snapshot) => snapshot.roster_id === roster_id.value)
    .reverse(),
);
// the latest snapshot against now, until the user picks
watch(
  roster_id,
  () => {
    from.value = snapshots.value[0] ?? null;
    to.value = null;
    char_id.value = null;
  },
  { immediate: true },
);

const now = computed(() =>
  snapshot_of(roster_config.value, roster_id.value, "Manual"),
);
// every character the roster has or had in a snapshot
const chars = computed(() => {
  const out = new Map<string, string>();
  for (const snapshot of [...snapshots.value, now.value]) {
    for (const [id, char] of Object.entries(snapshot.chars))
      out.set(id, char.name);
  }
  return [...out];
});

const diff = computed(() =>
  snapshot_diff(from.value ?? now.value, to.value ?? now.value, char_id.value),
);

function remove(snapshot: Snapshot) {
  const list = roster_config.value.snapshots;
  list.splice(list.indexOf(snapshot), 1);
  if (from.value === snapshot) from.value = null;
  if (to.value === snapshot) to.value = null;
}

const when = (at: number) => new Date(at).toLocaleString();
const signed = (amount: number) =>
  amount
    ? (amount > 0 ? "+" : "") +
      amount.toLocaleString("en-US", { maximumFractionDigits: 1 })
    : "";
const color = (amount: number) =>
  amount > 0 ? "var(--achieved)" : amount < 0 ? "var(--warning)" : "";
const BANDS = ["Char bound", "Roster bound", "Tradable"];
</script>

<template>
  <Sidebar :width="1100" header="Snapshots">
    <template #sidebar>
      <div v-if="roster_ids.length > 1" class="side-bar-item">
        <span class="text-nowrap">Roster:</span>
        <select v-model="roster_id" class="selector">
          <option v-for="(id, index) in roster_ids" :key="id" :value="id">
            Roster {{ index + 1 }}
          </option>
        </select>
      </div>
      <div class="side-bar-item">
        <span class="text-nowrap">Character bound of:</span>
        <select v-model="char_id" class="selector" aria-label="Character">
          <option :value="null">Every character</option>
          <option v-for="[id, name] in chars" :key="id" :value="id">
            {{ name }}
          </option>
        </select>
      </div>
      <div class="side-bar-item">
        <button
          class="generic-button"
          @click="take_snapshot(roster_config, roster_id, 'Manual')"
        >
          Take snapshot
        </button>
      </div>
    </template>

    <template #main>
      <div class="flex flex-col gap-3 p-3">
        <div
          class="card-shell card-body text-sm"
          role="group"
          aria-label="Snapshot list"
        >
          <div class="card-title">Snapshots</div>
          <span class="text-(--text-muted)">
            Taken by hand, and before and after every scanner export. Pick two
            to compare.
          </span>
          <div class="grid grid-cols-[3rem_3rem_1fr_auto] items-center gap-x-3">
            <span class="text-(--text-muted)">From</span>
            <span class="text-(--text-muted)">To</span>
            <span></span>
            <span></span>
            <span></span>
            <input
              v-model="to"
              type="radio"
              :value="null"
              aria-label="To now"
            />
            <span>Now</span>
            <span></span>
            <template v-for="snapshot in snapshots" :key="snapshot.at">
              <input
                v-model="from"
                type="radio"
                :value="snapshot"
                :aria-label="`From ${when(snapshot.at)} ${snapshot.kind}`"
              />
              <input
                v-model="to"
                type="radio"
                :value="snapshot"
                :aria-label="`To ${when(snapshot.at)} ${snapshot.kind}`"
              />
              <span>
                {{ when(snapshot.at) }}
                <span class="text-(--text-muted)">{{ snapshot.kind }}</span>
              </span>
              <button
                class="generic-button"
                :aria-label="`Delete ${when(snapshot.at)} ${snapshot.kind}`"
                @click="remove(snapshot)"
              >
                x
              </button>
            </template>
          </div>
          <span v-if="!snapshots.length" class="text-(--text-muted)">
            None yet.
          </span>
        </div>

        <div
          class="card-shell card-body text-sm"
          role="group"
          aria-label="Snapshot changes"
        >
          <div class="card-title">What changed</div>
          <span
            v-if="!diff.rows.length && !diff.chests.length"
            class="text-(--text-muted)"
          >
            Nothing.
          </span>
          <div
            v-if="diff.rows.length"
            class="grid grid-cols-[14rem_repeat(4,8rem)] items-center gap-y-1"
          >
            <span></span>
            <span
              v-for="(name, at) in DIFF_COLUMNS"
              :key="name"
              :style="{ color: at < 3 ? `var(${BAND_COLORS[at]})` : '' }"
            >
              {{ name }}
            </span>
            <template v-for="row in diff.rows" :key="row.label">
              <span
                class="flex items-center gap-1.5 text-(--text-muted)"
                role="rowheader"
              >
                <img
                  :src="get_icon_path(row.label)"
                  class="generic-icon h-6 w-6"
                  :alt="row.label"
                />
                {{ row.label }}
              </span>
              <span
                v-for="(change, at) in row.changes"
                :key="at"
                :aria-label="`${row.label} ${DIFF_COLUMNS[at]}`"
                :style="{ color: color(change) }"
              >
                {{ signed(change) }}
              </span>
            </template>
          </div>
          <template v-if="diff.chests.length">
            <div class="pt-2 text-(--text-muted)">Chests</div>
            <div v-for="chest in diff.chests" :key="chest.title + chest.band">
              {{ chest.title }}
              <span
                class="annotation"
                :style="{ color: `var(${BAND_COLORS[chest.band]})` }"
              >
                {{ BANDS[chest.band] }}
              </span>
              {{ chest.from.toLocaleString() }} →
              {{ chest.to.toLocaleString() }}
            </div>
          </template>
        </div>
      </div>
    </template>
  </Sidebar>
</template>
