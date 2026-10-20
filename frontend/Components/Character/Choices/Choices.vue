<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { get_icon_path } from "@/Utils/Helpers";
import { snapshot_of } from "@/Utils/Scans";
import {
  affordable,
  is_empty,
  PRESET_CHOICES,
  raids_of,
  shown_difficulty,
  signed_gold,
  take_option,
  type Choice,
} from "@/Utils/Choices";
import { ilevel } from "@/Utils/HoningUtil";
import { parse_locale_float } from "@/Utils/InputColumn";
import {
  jobs_left,
  queue_jobs,
  stop_jobs,
} from "@/WasmInterface/OptimizerJobs";
import { base_job, option_job } from "./ChoiceJobs";

// What taking each option of a choice is worth to this character, in average gold under its plan.
const store = useRosterStore();
const { roster_config, active_profile } = storeToRefs(store);
const profile = active_profile;

const now = computed(() =>
  snapshot_of(roster_config.value, profile.value.roster_id, "Manual"),
);
const base = computed(() => base_job(profile.value, now.value));
// with nothing to hone, nothing is worth anything
const no_upgrades = computed(() => !base.value.payload.upgrade_info.length);
const item_level = computed(
  () => parse_locale_float(ilevel(profile.value, "achieved")) || 0,
);

// Every raid with its difficulties, newest first. Difficulties are locked by item level, not tier:
// a T4 character above Serca's level gets Serca materials it cannot use.
const raids = computed(() => raids_of(PRESET_CHOICES));
// the top three, and one more each time the user asks
const SHOWN = 3;
const more = ref(0);
watch(
  () => profile.value.id,
  () => (more.value = 0),
);

// each raid shown with the choices of its difficulty shown
const groups = computed(() =>
  raids.value
    .slice(0, SHOWN + more.value)
    .map(({ name, members, difficulties: list }) => {
      const shown = shown_difficulty(
        list,
        item_level.value,
        profile.value.choice_levels[name],
      );
      return {
        name,
        difficulties: list,
        shown,
        rows: members
          .filter((choice) => !choice.difficulty || choice.difficulty === shown)
          .map((choice) => ({
            choice,
            options: choice.options
              .map((option, index) => ({
                option,
                job: is_empty(option)
                  ? null
                  : option_job(
                      profile.value,
                      now.value,
                      base.value,
                      choice,
                      index,
                    ),
              }))
              .filter(({ job }) => job),
          })),
      };
    }),
);
const hidden = computed(() => raids.value.length - groups.value.length);
const rows = computed(() => groups.value.flatMap((group) => group.rows));
const pick = (group: string, difficulty: string) =>
  (profile.value.choice_levels[group] = difficulty);

// the run without any choice first, then the options it seeds, top to bottom
watch(
  [base, rows],
  () => {
    if (no_upgrades.value) return queue_jobs([]);
    if (base.value.metric === undefined) return queue_jobs([base.value]);
    queue_jobs(
      rows.value
        .flatMap((row) => row.options.map(({ job }) => job))
        .filter((job) => job.metric === undefined),
    );
  },
  { immediate: true },
);
onBeforeUnmount(stop_jobs);

const worth = (metric?: number) =>
  metric === undefined || base.value.metric === undefined
    ? undefined
    : metric - base.value.metric;
const worth_text = (metric?: number) =>
  no_upgrades.value
    ? "N/A"
    : worth(metric) === undefined
      ? "…"
      : signed_gold(worth(metric));
// as shown, so what reads 0 is neither
const worth_color = (amount?: number) =>
  no_upgrades.value || amount === undefined
    ? "var(--text-muted)"
    : Math.round(amount) > 0
      ? "var(--achieved)"
      : Math.round(amount) < 0
        ? "var(--warning)"
        : "var(--text-main)";

const columns = computed(() => [
  profile.value.bound_budgets,
  store.active_roster_mats_owned,
  store.active_tradable_mats_owned,
]);
const take = (choice: Choice, option: Record<string, number>) =>
  take_option(choice, option, columns.value, profile.value.special_budget);
</script>

<template>
  <div class="flex flex-col gap-3 p-3">
    <div class="card-shell card-body flex w-full flex-col gap-2 text-sm">
      <div class="flex flex-wrap items-center gap-x-4 gap-y-2">
        <div class="card-title">Choices</div>
        <span v-if="jobs_left" class="text-(--text-muted)">
          Optimizing {{ jobs_left }} run{{ jobs_left > 1 ? "s" : "" }}
        </span>
      </div>
      <span class="text-(--text-muted)">
        What buying each raid's bonus chest is worth to this character: how much
        less gold its upgrades cost on average, under its plan, with the chest's
        materials and less its gold, each optimized again. Only honing materials
        count. Gold is paid from char bound, then roster bound, then tradable.
      </span>
      <span v-if="no_upgrades" class="text-(--warning)" role="alert">
        No upgrades are selected for this character, so nothing here is worth
        anything. Select some on the calculator to see what each chest is worth.
      </span>
      <div
        class="grid grid-cols-[minmax(10rem,auto)_6rem_auto_6rem_auto] items-center gap-x-4 gap-y-1"
        role="table"
        aria-label="Choices"
      >
        <span class="text-(--text-muted)">Gate</span>
        <span class="text-(--text-muted)">Cost</span>
        <span class="text-(--text-muted)">Gives</span>
        <span class="text-(--text-muted)">Worth</span>
        <span></span>
        <template v-for="group in groups" :key="group.name">
          <div
            class="col-span-5 mt-2 flex flex-wrap items-center gap-x-3 gap-y-1"
            role="rowheader"
          >
            <span class="text-(--text-bright)">{{ group.name }}</span>
            <div
              v-if="group.difficulties.length"
              class="flex gap-1"
              role="group"
              :aria-label="`${group.name} difficulty`"
            >
              <button
                v-for="difficulty in group.difficulties"
                :key="difficulty.name"
                class="generic-button"
                :class="{
                  'bg-(--bg-very-bright)!': group.shown === difficulty.name,
                  'text-(--text-very-muted)':
                    difficulty.item_level > item_level,
                }"
                :aria-pressed="group.shown === difficulty.name"
                :title="`Item level ${difficulty.item_level}`"
                @click="pick(group.name, difficulty.name)"
              >
                {{ difficulty.name }}
              </button>
            </div>
          </div>
          <template
            v-for="{ option, job, choice } in group.rows.flatMap((row) =>
              row.options.map((x) => ({ ...x, choice: row.choice })),
            )"
            :key="job.hash"
          >
            <span class="text-(--text-main)">{{ choice.title }}</span>
            <span class="text-(--gold)">
              {{ (-(option.Gold ?? 0)).toLocaleString("en-US") }}
            </span>
            <span
              class="flex flex-wrap items-center gap-x-3"
              :title="choice.others?.join('\n')"
            >
              <span
                v-for="(amount, label) in option"
                v-show="label !== 'Gold'"
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
              :aria-label="`${choice.group} ${choice.title} worth`"
              :style="{ color: worth_color(worth(job.metric)) }"
            >
              {{ worth_text(job.metric) }}
            </span>
            <button
              class="generic-button w-fit"
              :disabled="!affordable(choice, option, columns)"
              :title="
                affordable(choice, option, columns)
                  ? 'Add what it gives and take its gold'
                  : 'Not enough gold'
              "
              @click="take(choice, option)"
            >
              Add to budget
            </button>
          </template>
        </template>
      </div>
      <button v-if="hidden" class="generic-button w-fit" @click="more++">
        Show one more raid ({{ hidden }} hidden)
      </button>
    </div>
  </div>
</template>
