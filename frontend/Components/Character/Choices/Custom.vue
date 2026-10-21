<script setup lang="ts">
import { computed, watch } from "vue";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { get_icon_path } from "@/Utils/Helpers";
import type { Snapshot } from "@/Utils/Scans";
import type { InputColumn } from "@/Utils/InputColumn";
import { ALL_MATERIAL_LABELS, BAND_NAMES } from "@/Utils/Constants";
import {
  affordable,
  as_custom,
  default_custom_choice,
  is_empty,
  worth_color,
  worth_text,
  type Choice,
} from "@/Utils/Choices";
import { jobs_left, queue_jobs } from "@/WasmInterface/OptimizerJobs";
import Spinner from "@/Components/Common/Spinner.vue";
import { option_job, type ChoiceJob } from "./ChoiceJobs";

// A choice the user makes, shared by the roster: what each option is worth, best first.
const props = defineProps<{
  base: ChoiceJob;
  now: Snapshot;
  no_upgrades: boolean;
  baseline_text: string;
  columns: InputColumn[];
  taken: string | null;
}>();
const emit = defineEmits<{
  take: [key: string, choice: Choice, option: Record<string, number>];
}>();

const { roster_config, active_profile: profile } =
  storeToRefs(useRosterStore());

const roster_id = computed(() => profile.value.roster_id);
const custom = computed(
  () =>
    roster_config.value.custom_choices[roster_id.value] ??
    default_custom_choice(),
);
// written on the first edit
const stored = () =>
  (roster_config.value.custom_choices[roster_id.value] ??=
    default_custom_choice());
const choice = computed(() => as_custom(custom.value));

const set_amount = (index: number, label: string, event: Event) => {
  const input = event.target as HTMLInputElement;
  stored().options[index][label] = Math.round(Number(input.value)) || 0;
  input.value = String(stored().options[index][label]);
};
const add_label = (index: number, event: Event) => {
  const select = event.target as HTMLSelectElement;
  stored().options[index][select.value] = 0;
  select.value = "";
};
const add_option = () => {
  const out = stored();
  out.names.push(`Option ${out.names.length + 1}`);
  out.options.push({});
};
const remove_option = (index: number) => {
  stored().names.splice(index, 1);
  stored().options.splice(index, 1);
};

const worth = (job?: ChoiceJob) =>
  !job
    ? 0
    : job.metric === undefined || props.base.metric === undefined
      ? undefined
      : job.metric - props.base.metric;
const rows = computed(() =>
  custom.value.options.map((bag, index) => ({
    index,
    bag,
    option: choice.value.options[index],
    job: is_empty(choice.value.options[index])
      ? null
      : option_job(profile.value, props.now, props.base, choice.value, index),
  })),
);
const sorted = computed(() =>
  rows.value
    .map((row) => ({ ...row, worth: worth(row.job) }))
    .sort((a, b) => (b.worth ?? -Infinity) - (a.worth ?? -Infinity)),
);

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
      <span class="card-title">Custom</span>
      <span v-if="jobs_left" class="text-(--text-muted)">
        Optimizing {{ jobs_left }} run{{ jobs_left > 1 ? "s" : "" }}
      </span>
    </div>
    <div class="card-body flex flex-col gap-3">
      <span class="text-(--text-muted)">
        What each option of your own choice is worth to this character, best
        first.
      </span>
      <span
        v-if="no_upgrades"
        class="self-center text-center text-(--warning)"
        role="alert"
      >
        No upgrades are selected for this character, so nothing here is worth
        anything.
      </span>
      <div class="flex items-center gap-3">
        <span class="text-(--text-muted)">Band</span>
        <div class="flex gap-1" role="group" aria-label="Custom choice band">
          <button
            v-for="(name, band) in BAND_NAMES"
            :key="band"
            class="generic-button"
            :class="{ 'bg-(--gold-muted)!': custom.band === band }"
            :aria-pressed="custom.band === band"
            @click="stored().band = band"
          >
            {{ name }}
          </button>
        </div>
      </div>
      <!-- scrolls on its own on a narrow screen, so the page does not -->
      <div class="overflow-x-auto">
        <div
          class="grid w-max min-w-full grid-cols-[minmax(10rem,auto)_auto_minmax(18rem,auto)_auto_auto] items-center gap-x-4 gap-y-1"
          role="table"
          aria-label="Custom options"
        >
          <span class="text-(--text-muted)">Option</span>
          <span class="text-(--text-muted)">Gives</span>
          <span class="text-(--text-muted)">
            Worth
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
          <span></span>
          <div class="col-span-full border-b border-(--border-muted)" />
          <template v-for="row in sorted" :key="row.index">
            <input
              type="text"
              class="generic-input h-7! w-40 pl-1"
              :value="custom.names[row.index]"
              :aria-label="`Option ${row.index + 1} name`"
              @change="
                stored().names[row.index] = (
                  $event.target as HTMLInputElement
                ).value
              "
            />
            <span class="flex flex-wrap items-center gap-x-3 gap-y-1">
              <span
                v-for="(amount, label) in row.bag"
                :key="label"
                class="flex items-center gap-1"
              >
                <img
                  :src="get_icon_path(label as string)"
                  class="generic-icon h-5 w-5"
                  :alt="label as string"
                  :title="label as string"
                />
                <input
                  type="number"
                  class="generic-input h-7! w-20 pl-1"
                  :value="amount"
                  :aria-label="`${custom.names[row.index]} ${label}`"
                  @change="set_amount(row.index, label as string, $event)"
                />
                <button
                  class="generic-button"
                  :aria-label="`Remove ${label}`"
                  @click="delete stored().options[row.index][label]"
                >
                  ×
                </button>
              </span>
              <select
                class="selector"
                :aria-label="`${custom.names[row.index]} add material`"
                @change="add_label(row.index, $event)"
              >
                <option value="" selected disabled>+ Material</option>
                <option
                  v-for="label in ALL_MATERIAL_LABELS.filter(
                    (x) => !(x in row.bag),
                  )"
                  :key="label"
                >
                  {{ label }}
                </option>
              </select>
            </span>
            <span
              class="tabular-nums"
              :aria-label="`${custom.names[row.index]} worth`"
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
              :disabled="
                is_empty(row.option) || !affordable(choice, row.option, columns)
              "
              :title="
                affordable(choice, row.option, columns)
                  ? 'Add what it gives and take what it costs'
                  : 'Not enough to pay for it'
              "
              @click="emit('take', `custom/${row.index}`, choice, row.option)"
            >
              {{
                taken === `custom/${row.index}` ? "Added ✓" : "Add to budget"
              }}
            </button>
            <button
              class="generic-button w-fit"
              :disabled="custom.options.length <= 1"
              @click="remove_option(row.index)"
            >
              Remove
            </button>
          </template>
        </div>
      </div>
      <button class="generic-button w-fit" @click="add_option">
        Add option
      </button>
    </div>
  </div>
</template>
