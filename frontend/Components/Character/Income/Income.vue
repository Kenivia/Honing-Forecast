<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { useNow } from "@vueuse/core";
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  ALL_LABELS,
  BAND_COLORS,
  BAND_NAMES,
  SPECIAL_LEAP_LABELS,
} from "@/Utils/Constants";
import { get_icon_path, metric_to_text } from "@/Utils/Helpers";
import { parse_locale_float } from "@/Utils/InputColumn";
import {
  budgets_of,
  DIFF_COLUMNS,
  Snapshot,
  snapshot_diff,
  snapshot_of,
  take_snapshot,
} from "@/Utils/Scans";
import {
  WEEK_MS,
  fit,
  has_band,
  chain_of,
  next_in_chain,
  owned_at,
  projected,
  EXTRA_WEEKS,
  MAX_WEEKS,
  MIN_WEEKS,
  range_of,
  resets,
  steps_of,
  used,
  value_of,
} from "@/Utils/Income";
import IncomeGraph, { type Series } from "./IncomeGraph.vue";
import { job_for, state_of, warm } from "./IncomeJobs";
import { hash, queue_jobs, stop_jobs } from "@/WasmInterface/OptimizerJobs";

// The roster's budget snapshots as this character has them over time, and its income per week.
const { roster_config, active_profile } = storeToRefs(useRosterStore());
const profile = active_profile;
const tier = computed(() => profile.value.tier);
const special = computed(() => SPECIAL_LEAP_LABELS[tier.value]);

const band = ref(2);
// free taps are the special leap row, char bound only
const rows = computed(() => [...ALL_LABELS[tier.value], special.value]);
const name = (label: string) => (label === special.value ? "Free taps" : label);

// the time, a minute at most behind
const clock = useNow({ interval: 60_000 });
const now = computed(() => clock.value.getTime());

const snapshots = computed(() =>
  roster_config.value.snapshots.filter(
    (snapshot) => snapshot.roster_id === profile.value.roster_id,
  ),
);
// Now counts in the estimate at the time its budgets were first seen, so the rates, and with them
// the weeks' payloads, stay as they are (and their runs are found again) while nothing changes.
const current = computed(() =>
  snapshot_of(roster_config.value, profile.value.roster_id, "Manual"),
);
const now_budgets = computed(() => hash(budgets_of(current.value)));
watch(
  [now_budgets, profile],
  ([budgets]) => {
    if (profile.value.income_now?.budgets !== budgets)
      profile.value.income_now = { budgets, at: Date.now() };
  },
  { immediate: true },
);
const now_snapshot = computed(() => ({
  ...current.value,
  at: profile.value.income_now?.at ?? now.value,
}));
// now counts in the estimate unless it is the latest snapshot again
const now_differs = computed(() => {
  const latest = snapshots.value[snapshots.value.length - 1];
  return !latest || budgets_of(latest) !== budgets_of(now_snapshot.value);
});

// each snapshot with its optimizer run under the current upgrades, if done
const entries = computed(() =>
  snapshots.value.map((snapshot) => {
    const job = job_for(profile.value, snapshot, false);
    return {
      snapshot,
      job,
      owned: owned_at(snapshot, profile.value.id, tier.value, job.result),
    };
  }),
);
const now_entry = computed(() => {
  const job = job_for(profile.value, now_snapshot.value, true);
  return {
    snapshot: now_snapshot.value,
    job,
    owned: owned_at(
      now_snapshot.value,
      profile.value.id,
      tier.value,
      job.result,
    ),
  };
});
const series_entries = computed(() => [
  ...entries.value.filter((entry) => used(entry.snapshot)),
  ...(now_differs.value ? [now_entry.value] : []),
]);

// ============================================================================
// Hover and lock, as on the slot grid
// ============================================================================

const hovered = ref<Snapshot | null>(null);
const locked = ref<Snapshot | null>(null);
const selected = computed(() => locked.value ?? hovered.value);
const click = (snapshot: Snapshot) =>
  (locked.value = locked.value === snapshot ? null : snapshot);

// ============================================================================
// Rates
// ============================================================================

// the record a band's typed income is kept in, made only when written
const income_record = (band: number, create = false) =>
  band === 0
    ? profile.value.bound_income
    : create
      ? (roster_config.value.roster_income[profile.value.roster_id] ??= [
          {},
          {},
        ])[band - 1]
      : roster_config.value.roster_income[profile.value.roster_id]?.[band - 1];
const manual_rate = (label: string, band: number) =>
  has_band(label, band) ? (income_record(band)?.[label] ?? 0) : 0;
// the site's parser has no negatives; income can be one
function set_manual(label: string, band: number, value: string) {
  const text = value.trim();
  const amount = text.startsWith("-")
    ? -parse_locale_float(text.slice(1))
    : parse_locale_float(text);
  income_record(band, true)[label] = Number.isFinite(amount) ? amount : 0;
}
// the table shows only the cells that exist in game
const band_rows = computed(() =>
  rows.value.filter((label) => has_band(label, band.value)),
);

// per week, of one band
const auto_rate = (label: string, band_at: number) => {
  const slope = fit(
    series_entries.value.map((entry) => ({
      x: entry.snapshot.at,
      y: value_of(entry.owned, label, band_at),
      breaks: entry.snapshot.breaks,
    })),
  );
  return slope === null ? null : slope * WEEK_MS;
};

// switching to manual fills what was never typed with the estimate
function set_manual_mode(manual: boolean) {
  if (manual) {
    for (const label of rows.value) {
      for (const at of [0, 1, 2]) {
        if (!has_band(label, at) || income_record(at)?.[label] !== undefined)
          continue;
        income_record(at, true)[label] = Math.round(auto_rate(label, at) ?? 0);
      }
    }
  }
  profile.value.income_manual = manual;
}

// ============================================================================
// The graph
// ============================================================================

// now plus a week of income at a time, as the table has it
const rates = computed(() =>
  [0, 1, 2].map((at) =>
    Object.fromEntries(
      rows.value
        .filter((label) => has_band(label, at))
        .map((label) => [
          label,
          profile.value.income_manual
            ? manual_rate(label, at)
            : (auto_rate(label, at) ?? 0),
        ]),
    ),
  ),
);

// the metrics as the calculator shows them (Rust's are negative), and what pays for the cost
type Entry = typeof now_entry.value;
const cost = (entry: Entry) => -entry.job.result.gold;
const leftover = (entry: Entry) => -entry.job.result.leftover;
const covered = (entry: Entry) =>
  leftover(entry) + value_of(entry.owned, "Gold", 2);
const crosses = (entry: Entry) =>
  !!entry.job.result && covered(entry) >= cost(entry);

// the weeks typed, or by default a week at a time while the last is done until EXTRA_WEEKS past
// the crossing, MIN_WEEKS at least; not saved
const weeks_ahead = ref<number | null>(null);
function set_weeks_ahead(value: string) {
  const weeks = Math.round(parse_locale_float(value));
  weeks_ahead.value =
    value.trim() && Number.isFinite(weeks)
      ? Math.min(MAX_WEEKS, Math.max(MIN_WEEKS, weeks))
      : null;
}
const future = computed(() => {
  const out: Entry[] = [];
  let crossing = crosses(now_entry.value) ? 0 : null;
  for (let weeks = 1; weeks <= (weeks_ahead.value ?? MAX_WEEKS); weeks++) {
    if (weeks_ahead.value === null && weeks > MIN_WEEKS) {
      if (crossing === null && !out.at(-1).job.result) break;
      if (crossing !== null && weeks > crossing + EXTRA_WEEKS) break;
    }
    const snapshot = projected(
      now_snapshot.value,
      profile.value.id,
      special.value,
      rates.value,
      weeks,
    );
    const job = job_for(profile.value, snapshot, true);
    const entry = {
      snapshot,
      job,
      owned: owned_at(snapshot, profile.value.id, tier.value, job.result),
    };
    if (crossing === null && crosses(entry)) crossing = weeks;
    out.push(entry);
  }
  return out;
});
const range = computed(() => range_of(now.value, future.value.length));
const week_lines = computed(() => resets(...range.value));

// now and the weeks ahead, as far as optimized
const metric_series = computed((): Series[] => {
  // drawn from the time it is, a week apart
  const done = [now_entry.value, ...future.value]
    .map((entry, weeks) => ({ entry, x: now.value + weeks * WEEK_MS }))
    .filter(({ entry }) => entry.job.result);
  const line = (
    label: string,
    color: string,
    y: (entry: Entry) => number,
    dashed = false,
  ) => ({
    label,
    color,
    dashed,
    points: done.map(({ entry, x }) => ({ x, y: y(entry) })),
  });
  return [
    line("Avg tradable gold spent", "--gold", cost),
    line("Avg value of leftovers", "--text-muted", leftover),
    line("Leftovers + tradable gold", "--gold", covered, true),
  ];
});

// one run at a time, oldest first, each from the state of the one before
const chain = computed(() =>
  chain_of(entries.value, selected.value, [now_entry.value, ...future.value]),
);
const runs_left = computed(
  () => chain.value.filter((entry) => !entry.job.result).length,
);
watch(
  chain,
  (chain) => {
    const next = next_in_chain(chain, state_of);
    queue_jobs(next ? [warm(next.entry.job, next.from)] : []);
  },
  { immediate: true },
);
onBeforeUnmount(stop_jobs);

// ============================================================================
// The dashboard
// ============================================================================

const selected_entry = computed(() =>
  entries.value.find((entry) => entry.snapshot === selected.value),
);
const previous = computed(() => {
  const at = snapshots.value.indexOf(selected.value);
  return at > 0 ? snapshots.value[at - 1] : null;
});
const diff = computed(() =>
  selected.value && previous.value
    ? snapshot_diff(previous.value, selected.value, profile.value.id)
    : null,
);
function remove(snapshot: Snapshot) {
  const list = roster_config.value.snapshots;
  const at = list.indexOf(snapshot);
  if (at >= 0) list.splice(at, 1);
  if (locked.value === snapshot) locked.value = null;
  if (hovered.value === snapshot) hovered.value = null;
}

const steps = computed(() =>
  steps_of(series_entries.value.map((entry) => entry.snapshot)),
);

const when = (at: number) => new Date(at).toLocaleString();
const number = (amount: number | null) =>
  amount === null
    ? "-"
    : amount.toLocaleString("en-US", { maximumFractionDigits: 1 });
const signed = (amount: number) =>
  amount
    ? (amount > 0 ? "+" : "") +
      amount.toLocaleString("en-US", { maximumFractionDigits: 1 })
    : "";
const change_color = (amount: number) =>
  amount > 0 ? "var(--achieved)" : amount < 0 ? "var(--warning)" : "";
const KIND_COLORS = {
  "After export": "var(--achieved)",
  Manual: "var(--text-main)",
  "Before export": "var(--text-very-muted)",
};
</script>

<template>
  <div class="flex flex-col gap-3 p-3">
    <div class="card-shell card-body flex w-full flex-col gap-2 text-sm">
      <div class="flex flex-wrap items-center gap-x-4 gap-y-2">
        <div class="card-title">Income</div>
        <div class="flex gap-1" role="group" aria-label="Bands">
          <button
            v-for="(name, at) in BAND_NAMES"
            :key="name"
            class="generic-button"
            :class="{ 'bg-(--bg-very-bright)!': band === at }"
            :style="{ color: `var(${BAND_COLORS[at]})` }"
            @click="band = at"
          >
            {{ name }}
          </button>
        </div>
        <button
          class="generic-button"
          @click="take_snapshot(roster_config, profile.roster_id, 'Manual')"
        >
          Take snapshot
        </button>
        <label class="flex items-center gap-1.5">
          Weeks ahead
          <input
            class="generic-input h-6 w-24"
            :value="weeks_ahead ?? ''"
            :placeholder="`auto (${future.length})`"
            aria-label="Weeks ahead"
            @change="set_weeks_ahead(($event.target as HTMLInputElement).value)"
          />
        </label>
        <span v-if="runs_left" class="text-(--text-muted)">
          Optimizing {{ runs_left }} run{{ runs_left > 1 ? "s" : "" }}
        </span>
      </div>
      <IncomeGraph
        :series="metric_series"
        :range="range"
        :resets="week_lines"
        :now="now"
      />
      <span class="text-(--text-muted)">
        Snapshots are taken by hand, and before and after every scanner export.
        The estimate uses those after an export and those taken by hand that are
        not ignored, with one slope; a snapshot can start a new line (a bar
        above it), for when something was spent. Hover one to see it, click to
        keep it shown.
      </span>
      <div
        class="flex max-h-48 flex-col overflow-y-auto"
        role="group"
        aria-label="Snapshots"
      >
        <button
          v-for="snapshot in snapshots"
          :key="`${snapshot.at} ${snapshot.kind}`"
          class="flex gap-3 px-2 py-0.5 text-left hover:bg-(--bg-bright)"
          :class="{
            'opacity-30': snapshot.ignored,
            'bg-(--bg-very-bright)!': snapshot === selected,
            'border-t border-(--text-muted)': snapshot.breaks && used(snapshot),
          }"
          :aria-label="`${when(snapshot.at)} ${snapshot.kind}`"
          @mouseenter="hovered = snapshot"
          @mouseleave="hovered = null"
          @click="click(snapshot)"
        >
          <span class="w-48">{{ when(snapshot.at) }}</span>
          <span :style="{ color: KIND_COLORS[snapshot.kind] }">
            {{ snapshot.kind }}
          </span>
        </button>
      </div>
    </div>

    <div
      class="card-shell card-body flex min-h-32 w-full flex-wrap gap-x-8 gap-y-4 text-sm"
      role="group"
      aria-label="Snapshot dashboard"
    >
      <span v-if="!selected" class="text-(--text-muted)">
        Hover a snapshot in the list.
      </span>
      <template v-else>
        <div class="flex flex-col gap-1">
          <span class="text-base text-(--text-bright)">
            {{ when(selected.at) }}
          </span>
          <span :style="{ color: KIND_COLORS[selected.kind] }">
            {{ selected.kind }}
          </span>
          <label
            v-if="selected.kind !== 'Before export'"
            class="flex items-center gap-1.5"
          >
            <input
              :checked="selected.ignored"
              type="checkbox"
              @change="selected.ignored = !selected.ignored"
            />
            Ignore in the estimate
          </label>
          <label
            v-if="selected.kind !== 'Before export'"
            class="flex items-center gap-1.5"
          >
            <input
              :checked="selected.breaks"
              type="checkbox"
              @change="selected.breaks = !selected.breaks"
            />
            Start a new line here
          </label>
          <button
            class="generic-button w-fit"
            :aria-label="`Delete ${when(selected.at)} ${selected.kind}`"
            @click="remove(selected)"
          >
            Delete
          </button>
        </div>
        <div
          class="grid grid-cols-[auto_auto] content-start gap-x-3 text-(--text-muted)"
        >
          <span class="col-span-2 text-(--text-bright)">
            With the current upgrades
          </span>
          <template v-if="selected_entry?.job.result">
            <span>Average tradable gold spent:</span>
            <span class="text-(--text-main)">
              {{ metric_to_text(selected_entry.job.result.gold) }}
            </span>
            <span>Avg value of leftover mats:</span>
            <span class="text-(--text-main)">
              {{ metric_to_text(selected_entry.job.result.leftover) }}
            </span>
          </template>
          <span v-else class="col-span-2">Not optimized yet.</span>
        </div>
        <div class="flex flex-col gap-1">
          <span class="text-(--text-bright)">Since the snapshot before</span>
          <span v-if="!diff" class="text-(--text-muted)">
            {{ previous ? "Nothing changed." : "It is the first." }}
          </span>
          <div
            v-else
            class="grid grid-cols-[12rem_repeat(4,6rem)] items-center gap-y-1"
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
              <span class="flex items-center gap-1.5 text-(--text-muted)">
                <img
                  :src="get_icon_path(row.label)"
                  class="generic-icon h-5 w-5"
                  :alt="row.label"
                />
                {{ row.label }}
              </span>
              <span
                v-for="(change, at) in row.changes"
                :key="at"
                :aria-label="`${row.label} ${DIFF_COLUMNS[at]}`"
                :style="{ color: change_color(change) }"
              >
                {{ signed(change) }}
              </span>
            </template>
            <template
              v-for="chest in diff.chests"
              :key="chest.title + chest.band"
            >
              <span class="col-span-5 text-(--text-muted)">
                {{ chest.title }}
                <span :style="{ color: `var(${BAND_COLORS[chest.band]})` }">
                  {{ BAND_NAMES[chest.band] }}
                </span>
                {{ chest.from.toLocaleString() }} →
                {{ chest.to.toLocaleString() }}
              </span>
            </template>
          </div>
        </div>
      </template>
    </div>

    <div
      class="card-shell card-body flex flex-col gap-2 text-sm"
      role="group"
      aria-label="Income table"
    >
      <div class="flex flex-wrap items-center gap-x-4 gap-y-1">
        <div class="card-title" :style="{ color: `var(${BAND_COLORS[band]})` }">
          {{ BAND_NAMES[band] }}
        </div>
        <span class="text-(--text-muted)">
          {{
            steps
              ? `${steps.steps} step${steps.steps > 1 ? "s" : ""} over ${steps.days.toFixed(1)} days`
              : "Not enough snapshots to estimate"
          }}
        </span>
        <label class="flex items-center gap-1.5">
          <input
            :checked="profile.income_manual"
            type="checkbox"
            aria-label="Manual income"
            @change="set_manual_mode(!profile.income_manual)"
          />
          Type the income by hand
        </label>
      </div>
      <div
        class="grid grid-cols-[14rem_8rem_auto] items-center gap-x-3 gap-y-1"
      >
        <span></span>
        <span class="text-(--text-muted)">Owned now</span>
        <span class="text-(--text-muted)">Income / week</span>
        <template v-for="label in band_rows" :key="label">
          <span
            class="flex items-center gap-1.5 text-(--text-muted)"
            role="rowheader"
          >
            <img
              :src="get_icon_path(label)"
              class="generic-icon h-6 w-6"
              :alt="name(label)"
            />
            {{ name(label) }}
          </span>
          <span :aria-label="`${name(label)} owned`">
            {{ number(value_of(now_entry.owned, label, band)) }}
          </span>
          <span class="flex items-center gap-2">
            <template v-if="profile.income_manual">
              <input
                class="generic-input h-6 w-24"
                :value="manual_rate(label, band)"
                :aria-label="`${name(label)} income`"
                @change="
                  set_manual(
                    label,
                    band,
                    ($event.target as HTMLInputElement).value,
                  )
                "
              />
              <span class="text-(--text-muted)">
                auto {{ number(auto_rate(label, band)) }}
              </span>
            </template>
            <span v-else :aria-label="`${name(label)} income`">
              {{ number(auto_rate(label, band)) }}
            </span>
          </span>
        </template>
      </div>
    </div>
  </div>
</template>
