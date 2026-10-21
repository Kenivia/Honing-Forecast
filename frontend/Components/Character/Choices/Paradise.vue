<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { get_icon_path } from "@/Utils/Helpers";
import type { Snapshot } from "@/Utils/Scans";
import {
  difficulties,
  is_empty,
  PARADISE_CHOICES,
  shown_difficulty,
  worth_color,
  worth_text,
  type Choice,
} from "@/Utils/Choices";
import { jobs_left, queue_jobs } from "@/WasmInterface/OptimizerJobs";
import Spinner from "@/Components/Common/Spinner.vue";
import { option_job, type ChoiceJob } from "./ChoiceJobs";

// What each reward of a Paradise run is worth to this character, best first. The game offers three
// of them; the player takes the best of those.
const props = defineProps<{
  base: ChoiceJob;
  now: Snapshot;
  no_upgrades: boolean;
  item_level: number;
  baseline_text: string;
  // the reward last added
  taken: string | null;
}>();
const emit = defineEmits<{
  take: [key: string, choice: Choice, option: Record<string, number>];
}>();

const { active_profile: profile } = storeToRefs(useRosterStore());

const MODES = ["Destiny", "Netherworld"];
const LEVELS = [
  "Base",
  ...Array.from({ length: 10 }, (_, i) => `Level ${i + 1}`),
];
// mode and level are saved per character beside the raids' difficulties
const mode = computed(
  () => profile.value.choice_levels["Paradise mode"] ?? MODES[0],
);
const level = computed(
  () => profile.value.choice_levels["Paradise level"] ?? LEVELS[0],
);
const pick = (key: string, value: string) =>
  (profile.value.choice_levels[key] = value);
// indices of the options taken with Wealth
const wealth = ref(new Set<number>());

const group = computed(() => `Paradise: ${mode.value}`);
const members = computed(() =>
  PARADISE_CHOICES.filter((choice) => choice.group === group.value),
);
const item_levels = computed(() => difficulties(members.value));
const shown = computed(() =>
  shown_difficulty(
    item_levels.value,
    props.item_level,
    profile.value.choice_levels[group.value],
  ),
);
const choice = computed(() =>
  members.value.find(
    (x) => x.difficulty === shown.value && x.title === level.value,
  ),
);
watch(choice, () => (wealth.value = new Set()));
// Netherworld has no Wealth
const with_wealth = computed(() =>
  choice.value.wealth
    ? { ...choice.value, options: choice.value.wealth }
    : null,
);
const toggle = (index: number) => {
  const next = new Set(wealth.value);
  if (!next.delete(index)) next.add(index);
  wealth.value = next;
};

const worth = (job?: ChoiceJob) =>
  !job
    ? 0
    : job.metric === undefined || props.base.metric === undefined
      ? undefined
      : job.metric - props.base.metric;
// an empty option is the run without
const rows = computed(() =>
  choice.value.options.map((_, index) => {
    const used =
      with_wealth.value && wealth.value.has(index)
        ? with_wealth.value
        : choice.value;
    const option = used.options[index];
    return {
      index,
      name: choice.value.names[index],
      choice: used,
      option,
      job: is_empty(option)
        ? null
        : option_job(profile.value, props.now, props.base, used, index),
    };
  }),
);
// best first, those not run yet last
const sorted = computed(() =>
  rows.value
    .map((row) => ({ ...row, worth: worth(row.job) }))
    .sort((a, b) => (b.worth ?? -Infinity) - (a.worth ?? -Infinity)),
);

// the run without first, then every option, in the data's order
watch(
  () => [props.base, rows.value],
  () => {
    if (props.no_upgrades) return queue_jobs([]);
    if (props.base.metric === undefined) return queue_jobs([props.base]);
    queue_jobs(
      rows.value
        .map((row) => row.job)
        .filter((job) => job && job.metric === undefined),
    );
  },
  { immediate: true },
);
</script>

<template>
  <div class="card-shell w-fit max-w-[min(100%,1055px)] text-sm">
    <div class="card-header gap-4">
      <span class="card-title">Paradise</span>
      <span v-if="jobs_left" class="text-(--text-muted)">
        Optimizing {{ jobs_left }} run{{ jobs_left > 1 ? "s" : "" }}
      </span>
    </div>
    <div class="card-body flex flex-col gap-3">
      <span class="text-(--text-main)">
        What each reward of a Paradise run is worth to this character, best
        first (you should pick whichever ranks higher). Only honing materials
        and gold are considered.
      </span>
      <span
        v-if="no_upgrades"
        class="self-center text-center text-(--warning)"
        role="alert"
      >
        No upgrades are selected for this character, so nothing here is worth
        anything.
      </span>
      <div
        class="grid w-fit grid-cols-[auto_auto] items-center gap-x-3 gap-y-1"
      >
        <span class="text-(--text-muted)">Mode</span>
        <div class="flex gap-1" role="group" aria-label="Paradise mode">
          <button
            v-for="name in MODES"
            :key="name"
            class="generic-button"
            :class="{ 'bg-(--gold-muted)!': mode === name }"
            :aria-pressed="mode === name"
            @click="pick('Paradise mode', name)"
          >
            {{ name }}
          </button>
        </div>
        <span class="text-(--text-muted)">Item level</span>
        <div class="flex gap-1" role="group" aria-label="Paradise item level">
          <button
            v-for="difficulty in item_levels"
            :key="difficulty.name"
            class="generic-button"
            :class="{
              'bg-(--gold-muted)!': shown === difficulty.name,
              'text-(--text-very-muted)': difficulty.item_level > item_level,
            }"
            :aria-pressed="shown === difficulty.name"
            @click="pick(group, difficulty.name)"
          >
            {{ difficulty.name }}
          </button>
        </div>
        <span class="text-(--text-muted)">Level</span>
        <div
          class="flex flex-wrap gap-1"
          role="group"
          aria-label="Paradise level"
        >
          <button
            v-for="name in LEVELS"
            :key="name"
            class="generic-button"
            :class="{ 'bg-(--gold-muted)!': level === name }"
            :aria-pressed="level === name"
            @click="pick('Paradise level', name)"
          >
            {{ name }}
          </button>
        </div>
      </div>
      <!-- scrolls on its own on a narrow screen, so the page does not -->
      <div class="overflow-x-auto">
        <div
          class="grid w-max min-w-full grid-cols-[minmax(10rem,auto)_auto_auto_minmax(18rem,auto)_auto] items-center gap-x-4 gap-y-1"
          role="table"
          aria-label="Paradise rewards"
        >
          <span class="text-(--text-muted)">Reward</span>
          <span class="text-(--text-muted)">Gives</span>
          <span></span>
          <span class="text-(--text-muted)">
            <span class="block">Worth</span>
            <span
              class="block whitespace-nowrap text-(--text-main)"
              aria-label="Baseline"
            >
              Baseline =
              <Spinner
                v-if="!baseline_text"
                class="inline-block h-4 w-4 align-middle"
              />
              {{ baseline_text }}
            </span>
          </span>
          <span></span>
          <div class="col-span-full border-b border-(--border-muted)" />
          <template v-for="row in sorted" :key="row.index">
            <span class="text-(--text-main)">{{ row.name }}</span>
            <span class="flex flex-wrap items-center gap-x-3">
              <span
                v-for="(amount, label) in row.option"
                :key="label"
                class="flex items-center gap-1"
              >
                <img
                  :src="get_icon_path(label as string)"
                  class="generic-icon h-5 w-5"
                  :alt="label as string"
                />
                {{ amount.toLocaleString("en-US") }}
              </span>
            </span>
            <label
              v-if="with_wealth"
              class="flex items-center gap-1 text-(--text-muted)"
            >
              <input
                type="checkbox"
                :checked="wealth.has(row.index)"
                @change="toggle(row.index)"
              />
              Wealth
            </label>
            <span v-else></span>
            <span
              class="tabular-nums"
              :aria-label="`${row.name} worth`"
              :style="{ color: worth_color(no_upgrades, row.worth) }"
            >
              <Spinner
                v-if="!no_upgrades && row.worth === undefined"
                class="inline-block h-4 w-4 align-middle"
              />
              {{ worth_text(no_upgrades, row.worth) }}
            </span>
            <button
              class="generic-button w-28"
              :disabled="is_empty(row.option)"
              title="Add what it gives"
              @click="emit('take', row.name, row.choice, row.option)"
            >
              {{ taken === row.name ? "Added ✓" : "Add to budget" }}
            </button>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>
