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
import { option_job, type ChoiceJob } from "./ChoiceJobs";

// What each reward of a Paradise run is worth to this character, best first. The game offers three
// of them; the player takes the best of those.
const props = defineProps<{
  base: ChoiceJob;
  now: Snapshot;
  no_upgrades: boolean;
  item_level: number;
  baseline_text: string;
}>();
const emit = defineEmits<{
  take: [choice: Choice, option: Record<string, number>];
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
const wealth = ref(false);

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
const choice = computed(() => {
  const found = members.value.find(
    (x) => x.difficulty === shown.value && x.title === level.value,
  );
  return wealth.value ? { ...found, options: found.wealth } : found;
});

const worth = (job?: ChoiceJob) =>
  !job
    ? 0
    : job.metric === undefined || props.base.metric === undefined
      ? undefined
      : job.metric - props.base.metric;
// an empty option is the run without
const rows = computed(() =>
  choice.value.options.map((option, index) => ({
    index,
    name: choice.value.names[index],
    option,
    job: is_empty(option)
      ? null
      : option_job(profile.value, props.now, props.base, choice.value, index),
  })),
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
  <div class="card-shell card-body flex w-full flex-col gap-2 text-sm">
    <div class="flex flex-wrap items-center gap-x-4 gap-y-2">
      <div class="card-title">Paradise</div>
      <span v-if="jobs_left" class="text-(--text-muted)">
        Optimizing {{ jobs_left }} run{{ jobs_left > 1 ? "s" : "" }}
      </span>
    </div>
    <span class="text-(--text-muted)">
      What each reward of a Paradise run is worth to this character, best first:
      how much less gold its upgrades cost on average, under its plan, with the
      reward and the additional rewards that come with every one. Tick Wealth
      for the Wealth amounts instead. Only honing materials and gold count.
    </span>
    <span v-if="no_upgrades" class="text-(--warning)" role="alert">
      No upgrades are selected for this character, so nothing here is worth
      anything. Select some on the calculator to see what each reward is worth.
    </span>
    <div class="flex flex-wrap items-center gap-x-3 gap-y-1">
      <div class="flex gap-1" role="group" aria-label="Paradise mode">
        <button
          v-for="name in MODES"
          :key="name"
          class="generic-button"
          :class="{ 'bg-(--bg-very-bright)!': mode === name }"
          :aria-pressed="mode === name"
          @click="pick('Paradise mode', name)"
        >
          {{ name }}
        </button>
      </div>
      <div class="flex gap-1" role="group" aria-label="Paradise item level">
        <button
          v-for="difficulty in item_levels"
          :key="difficulty.name"
          class="generic-button"
          :class="{
            'bg-(--bg-very-bright)!': shown === difficulty.name,
            'text-(--text-very-muted)': difficulty.item_level > item_level,
          }"
          :aria-pressed="shown === difficulty.name"
          @click="pick(group, difficulty.name)"
        >
          {{ difficulty.name }}
        </button>
      </div>
    </div>
    <div class="flex flex-wrap gap-1" role="group" aria-label="Paradise level">
      <button
        v-for="name in LEVELS"
        :key="name"
        class="generic-button"
        :class="{ 'bg-(--bg-very-bright)!': level === name }"
        :aria-pressed="level === name"
        @click="pick('Paradise level', name)"
      >
        {{ name }}
      </button>
    </div>
    <div
      class="grid grid-cols-[minmax(10rem,auto)_auto_8rem_auto] items-center gap-x-4 gap-y-1"
      role="table"
      aria-label="Paradise rewards"
    >
      <span class="text-(--text-muted)">Reward</span>
      <span class="text-(--text-muted)">Gives</span>
      <span class="text-(--text-muted)">
        <span class="flex items-center gap-2">
          Worth
          <label class="flex items-center gap-1 text-(--text-main)">
            <input v-model="wealth" type="checkbox" />
            Wealth
          </label>
        </span>
        <span class="block text-(--text-main)" aria-label="Baseline">
          Baseline = {{ baseline_text }}
        </span>
      </span>
      <span></span>
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
        <span
          :aria-label="`${row.name} worth`"
          :style="{ color: worth_color(no_upgrades, row.worth) }"
        >
          {{ worth_text(no_upgrades, row.worth) }}
        </span>
        <button
          class="generic-button w-fit"
          :disabled="is_empty(row.option)"
          title="Add what it gives"
          @click="emit('take', choice, row.option)"
        >
          Add to budget
        </button>
      </template>
    </div>
  </div>
</template>
