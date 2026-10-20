<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  ALL_LABELS,
  BAND_COLORS,
  MATERIALS,
  SPECIAL_LEAP_LABELS,
} from "@/Utils/Constants";
import { get_icon_path, metric_to_text } from "@/Utils/Helpers";
import {
  budgets_of,
  DIFF_COLUMNS,
  Snapshot,
  snapshot_diff,
  snapshot_of,
  take_snapshot,
} from "@/Utils/Scans";
import {
  MIN_SPAN,
  WEEK_MS,
  fit,
  owned_at,
  resets,
  value_of,
} from "@/Utils/Income";
import IncomeGraph, { type Series } from "./IncomeGraph.vue";
import { job_for } from "./IncomeJobs";
import {
  jobs_left,
  queue_jobs,
  stop_jobs,
} from "@/WasmInterface/OptimizerJobs";

// The roster's budget snapshots as this character has them over time, and its income per week.
const { roster_config, active_profile } = storeToRefs(useRosterStore());
const profile = active_profile;
const tier = computed(() => profile.value.tier);
const special = computed(() => SPECIAL_LEAP_LABELS[tier.value]);

const BAND_NAMES = ["Char bound", "+ Roster bound", "+ Tradable"];
const TABLE_BANDS = ["Char bound", "Roster bound", "Tradable"];
const band = ref(2);
// free taps are the special leap row, char bound only
const rows = computed(() => [...ALL_LABELS[tier.value], special.value]);
const name = (label: string) => (label === special.value ? "Free taps" : label);
const shown = ref<string[]>([]);
watch(
  tier,
  (tier) => (shown.value = ["Gold", tier ? "Serca Fusion" : "Fusion"]),
  { immediate: true },
);
const toggle_shown = (label: string) =>
  (shown.value = shown.value.includes(label)
    ? shown.value.filter((x) => x !== label)
    : [...shown.value, label]);
const color = (label: string) =>
  label === special.value ? "--free-tap" : MATERIALS[label].color;

// the time, a minute at most behind
const now = ref(Date.now());
const clock = setInterval(() => (now.value = Date.now()), 60_000);

const snapshots = computed(() =>
  roster_config.value.snapshots.filter(
    (snapshot) => snapshot.roster_id === profile.value.roster_id,
  ),
);
const now_snapshot = computed(() => ({
  ...snapshot_of(roster_config.value, profile.value.roster_id, "Manual"),
  at: now.value,
}));
const now_differs = computed(() => {
  const latest = snapshots.value[snapshots.value.length - 1];
  return !latest || budgets_of(latest) !== budgets_of(now_snapshot.value);
});
const include_now = ref(false);
const with_now = computed(() => include_now.value && now_differs.value);

const used = (snapshot: Snapshot) =>
  snapshot.kind !== "Before export" && !snapshot.ignored;

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
  ...(with_now.value ? [now_entry.value] : []),
]);

// ============================================================================
// Hover and lock, as on the slot grid
// ============================================================================

const hovered = ref<Snapshot | null>(null);
const locked = ref<Snapshot | null>(null);
const selected = computed(() => locked.value ?? hovered.value);
const click = (snapshot: Snapshot) =>
  (locked.value = locked.value === snapshot ? null : snapshot);

// the selected one first, then those the estimate uses, newest first
watch(
  [entries, now_entry, with_now, selected],
  () => {
    const order = [
      ...entries.value.filter((e) => e.snapshot === selected.value),
      ...(with_now.value ? [now_entry.value] : []),
      ...[...entries.value].reverse().filter((e) => used(e.snapshot)),
      ...[...entries.value]
        .reverse()
        .filter((e) => e.snapshot.kind === "Before export"),
    ];
    queue_jobs(order.filter((e) => !e.job.result).map((e) => e.job));
  },
  { immediate: true },
);
onBeforeUnmount(() => {
  clearInterval(clock);
  stop_jobs();
});

// ============================================================================
// Rates
// ============================================================================

// the record a band's typed income is kept in
const income_record = (band: number) =>
  band === 0
    ? profile.value.bound_income
    : (roster_config.value.roster_income[profile.value.roster_id] ??= [{}, {}])[
        band - 1
      ];
const manual_rate = (label: string, band: number) =>
  income_record(band)[label] ?? 0;
function set_manual(label: string, band: number, value: string) {
  const amount = Number(value);
  income_record(band)[label] = Number.isFinite(amount) ? amount : 0;
}

const points = (label: string, band_at: number, cumulative: boolean) =>
  series_entries.value.map((entry) => ({
    x: entry.snapshot.at,
    y: value_of(entry.owned, label, band_at, cumulative),
    breaks: entry.snapshot.breaks,
    provisional: !entry.job.result,
  }));
// per week, of one band
const auto_rate = (label: string, band_at: number) => {
  const line = fit(points(label, band_at, false));
  return line ? line.slope * WEEK_MS : null;
};

function set_manual_mode(manual: boolean) {
  if (manual) {
    for (const label of rows.value) {
      for (const at of [0, 1, 2]) {
        income_record(at)[label] = Math.round(auto_rate(label, at) ?? 0);
      }
    }
  }
  profile.value.income_manual = manual;
}

// ============================================================================
// The graph
// ============================================================================

const range = computed((): [number, number] => {
  const oldest = snapshots.value[0]?.at ?? now.value;
  const half = Math.max(MIN_SPAN, now.value - oldest);
  return [now.value - half, now.value + half];
});
const series = computed((): Series[] =>
  shown.value
    .filter((label) => rows.value.includes(label))
    .map((label) => {
      const data = points(label, band.value, true);
      const end = range.value[1];
      let line: { x: number; y: number | null }[] = [];
      if (profile.value.income_manual) {
        const rate = [0, 1, 2]
          .slice(0, band.value + 1)
          .reduce((sum, at) => sum + manual_rate(label, at), 0);
        const from = value_of(now_entry.value.owned, label, band.value, true);
        line = [
          { x: now.value, y: from },
          { x: end, y: from + (rate * (end - now.value)) / WEEK_MS },
        ];
      } else {
        // each run over its own points, the last one on to the end
        const fitted = fit(data);
        line = (fitted?.runs ?? []).flatMap((run, at, runs) => [
          { x: run.from, y: run.at(run.from) },
          ...(at === runs.length - 1
            ? [{ x: end, y: run.at(end) }]
            : [
                { x: run.to, y: run.at(run.to) },
                { x: run.to, y: null },
              ]),
        ]);
      }
      return { label: name(label), color: color(label), points: data, line };
    }),
);
const week_lines = computed(() => resets(...range.value));

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
  list.splice(list.indexOf(snapshot), 1);
  if (locked.value === snapshot) locked.value = null;
  if (hovered.value === snapshot) hovered.value = null;
}

// over every run
const steps = computed(() => {
  const list = series_entries.value;
  let steps = 0;
  let days = 0;
  list.forEach((entry, at) => {
    if (at === 0 || entry.snapshot.breaks) return;
    steps += 1;
    days += (entry.snapshot.at - list[at - 1].snapshot.at) / 864e5;
  });
  return steps ? { steps, days } : null;
});

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
        <label v-if="now_differs" class="flex items-center gap-1.5">
          <input v-model="include_now" type="checkbox" />
          Include now
        </label>
        <button
          class="generic-button"
          @click="take_snapshot(roster_config, profile.roster_id, 'Manual')"
        >
          Take snapshot
        </button>
        <span v-if="jobs_left" class="text-(--text-muted)">
          Optimizing {{ jobs_left }} snapshot{{ jobs_left > 1 ? "s" : "" }}
          (hollow points until then)
        </span>
      </div>
      <div class="flex flex-wrap gap-1" role="group" aria-label="Materials">
        <button
          v-for="label in rows"
          :key="label"
          class="barebone-button"
          :class="{ 'opacity-40': !shown.includes(label) }"
          :aria-pressed="shown.includes(label)"
          :aria-label="name(label)"
          :title="name(label)"
          @click="toggle_shown(label)"
        >
          <img
            :src="get_icon_path(label)"
            class="generic-icon h-6 w-6"
            :alt="name(label)"
          />
        </button>
      </div>

      <IncomeGraph
        :series="series"
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
          :key="`break ${snapshot.at}`"
          class="absolute top-0 h-full w-px bg-(--text-muted)"
          :style="{ left: `${place(snapshot.at) - 10}px` }"
        ></div>
        <button
          v-for="snapshot in snapshots"
          :key="snapshot.at"
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
                  {{ TABLE_BANDS[chest.band] }}
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
          {{ TABLE_BANDS[band] }}
        </div>
        <span class="text-(--text-muted)">
          {{
            steps
              ? `${steps.steps} step${steps.steps > 1 ? "s" : ""} over ${steps.days.toFixed(1)} days`
              : "Not enough snapshots to estimate"
          }}
        </span>
        <label v-if="now_differs" class="flex items-center gap-1.5">
          <input v-model="include_now" type="checkbox" />
          Include now
        </label>
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
        <template v-for="label in rows" :key="label">
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
            {{ number(value_of(now_entry.owned, label, band, false)) }}
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
