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
  range_of,
  resets,
  steps_of,
  used,
  value_of,
} from "@/Utils/Income";
import IncomeGraph, { type Series } from "./IncomeGraph.vue";
import { job_for, state_of, warm } from "./IncomeJobs";
import { queue_jobs, stop_jobs } from "@/WasmInterface/OptimizerJobs";

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
const now_snapshot = computed(() => ({
  ...snapshot_of(roster_config.value, profile.value.roster_id, "Manual"),
  at: now.value,
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

const range = computed(() => range_of(snapshots.value[0]?.at, now.value));
// the two metrics of each optimized point under the current upgrades, then from now on ahead
const metric_series = computed((): Series[] => {
  const done = series_entries.value.filter((entry) => entry.job.result);
  const ahead = [now_entry.value, ...future.value].filter(
    (entry) => entry.job.result,
  );
  const metric = (label: string, color: string, key: "gold" | "leftover") => ({
    label,
    color,
    points: done.map((entry) => ({
      x: entry.snapshot.at,
      y: entry.job.result[key],
    })),
    line: ahead.map((entry) => ({
      x: entry.snapshot.at,
      y: entry.job.result[key],
    })),
  });
  return [
    metric("Avg tradable gold spent", "--gold", "gold"),
    metric("Avg value of leftovers", "--average", "leftover"),
  ];
});
const week_lines = computed(() => resets(...range.value));

// now plus a week of income at a time, as the table has it, to the end of the range
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
const future = computed(() => {
  const weeks = Math.floor((range.value[1] - now.value) / WEEK_MS);
  return Array.from({ length: weeks }, (_, at) => {
    const snapshot = projected(
      now_snapshot.value,
      profile.value.id,
      special.value,
      rates.value,
      at + 1,
    );
    return { snapshot, job: job_for(profile.value, snapshot, true) };
  });
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

// the plot area, for the timeline to line up with
const area = ref<[number, number]>([0, 0]);
const place = (at: number) => {
  const [from, to] = range.value;
  return (
    area.value[0] +
    ((at - from) / (to - from)) * (area.value[1] - area.value[0])
  );
};

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
        <span v-if="runs_left" class="text-(--text-muted)">
          Optimizing {{ runs_left }} run{{ runs_left > 1 ? "s" : "" }}
        </span>
      </div>
      <span class="text-(--text-bright)">With the current upgrades</span>
      <IncomeGraph
        :series="metric_series"
        :range="range"
        :resets="week_lines"
        :now="now"
        :selected="selected?.at ?? null"
        @area="(left, right) => (area = [left, right])"
      />
      <div
        class="relative h-10 border-t border-(--border-very-muted)"
        role="group"
        aria-label="Snapshot timeline"
      >
        <div
          v-for="snapshot in snapshots.filter((s) => s.breaks && used(s))"
          :key="`break ${snapshot.at} ${snapshot.kind}`"
          class="absolute top-0 h-full w-px bg-(--text-muted)"
          :style="{ left: `${place(snapshot.at) - 10}px` }"
        ></div>
        <button
          v-for="snapshot in snapshots"
          :key="`${snapshot.at} ${snapshot.kind}`"
          class="absolute h-3 w-3 -translate-x-1/2 rounded-full border"
          :class="{
            'opacity-30': snapshot.ignored,
            'outline-2 outline-offset-2 outline-white outline-dotted':
              snapshot === selected,
          }"
          :style="{
            left: `${place(snapshot.at)}px`,
            top: snapshot.kind === 'Before export' ? '1.5rem' : '0.5rem',
            background: KIND_COLORS[snapshot.kind],
            borderColor: KIND_COLORS[snapshot.kind],
          }"
          :aria-label="`${when(snapshot.at)} ${snapshot.kind}`"
          @mouseenter="hovered = snapshot"
          @mouseleave="hovered = null"
          @click="click(snapshot)"
        ></button>
      </div>
      <span class="text-(--text-muted)">
        Snapshots are taken by hand, and before and after every scanner export.
        The estimate uses those after an export and those taken by hand that are
        not ignored, with one slope; a snapshot can start a new line (a bar
        before it), for when something was spent. Hover one to see it, click to
        keep it shown.
      </span>
    </div>

    <div
      class="card-shell card-body flex min-h-32 w-full flex-wrap gap-x-8 gap-y-4 text-sm"
      role="group"
      aria-label="Snapshot dashboard"
    >
      <span v-if="!selected" class="text-(--text-muted)">
        Hover a snapshot on the timeline.
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
