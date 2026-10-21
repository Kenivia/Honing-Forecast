<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { get_icon_path, metric_to_text } from "@/Utils/Helpers";
import { snapshot_of } from "@/Utils/Scans";
import {
  affordable,
  is_empty,
  PRESET_CHOICES,
  raids_of,
  shown_difficulty,
  take_option,
  worth_color,
  worth_text,
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
import Paradise from "./Paradise.vue";
import Custom from "./Custom.vue";
import Spinner from "@/Components/Common/Spinner.vue";

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

// Every raid the character can enter, with its difficulties, newest first. Difficulties are locked
// by item level, not tier: a T4 character above Serca's level gets Serca materials it cannot use.
const raids = computed(() =>
  raids_of(PRESET_CHOICES).filter(
    (raid) =>
      !raid.difficulties.length ||
      raid.difficulties[0].item_level <= item_level.value,
  ),
);
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

const TABS = ["Bonus chests", "Paradise", "Custom"];
const tab = ref(TABS[0]);

// the run without any choice first, then the options it seeds, top to bottom; the other tabs queue their own
watch(
  [base, rows, tab],
  () => {
    if (tab.value !== TABS[0]) return;
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

const baseline_text = computed(() =>
  no_upgrades.value
    ? "N/A"
    : base.value.metric === undefined
      ? ""
      : `${metric_to_text(base.value.gold)} gold (${metric_to_text(base.value.gold - base.value.metric)} leftover value)`,
);

const columns = computed(() => [
  profile.value.bound_budgets,
  store.active_roster_mats_owned,
  store.active_tradable_mats_owned,
]);
// the row last added shows so for a moment; keyed by name, as its job changes with the budgets
const taken = ref<string | null>(null);
let taken_timer: ReturnType<typeof setTimeout>;
function take(key: string, choice: Choice, option: Record<string, number>) {
  take_option(choice, option, columns.value, profile.value.special_budget);
  taken.value = key;
  clearTimeout(taken_timer);
  taken_timer = setTimeout(() => (taken.value = null), 2000);
}
</script>

<template>
  <div class="flex w-full min-w-0 flex-col items-center gap-3 p-3">
    <!-- clears the sidebar's menu button, shown below CharView's width of 1201 -->
    <div
      class="flex flex-wrap items-center justify-center gap-1 max-[1201px]:pl-12"
      role="tablist"
      aria-label="Choices"
    >
      <button
        v-for="name in TABS"
        :key="name"
        class="generic-button"
        :class="{ 'bg-(--gold-muted)!': tab === name }"
        role="tab"
        :aria-selected="tab === name"
        @click="tab = name"
      >
        {{ name }}
      </button>
      <span class="self-center px-1 text-sm text-(--text-very-muted)">
        More coming soon...
      </span>
    </div>
    <Custom
      v-if="tab === 'Custom'"
      :base="base"
      :now="now"
      :no_upgrades="no_upgrades"
      :baseline_text="baseline_text"
      :columns="columns"
      :taken="taken"
      @take="take"
    />
    <Paradise
      v-else-if="tab === 'Paradise'"
      :base="base"
      :now="now"
      :no_upgrades="no_upgrades"
      :item_level="item_level"
      :baseline_text="baseline_text"
      :taken="taken"
      @take="take"
    />
    <div v-else class="card-shell w-fit max-w-[min(100%,1055px)] text-sm">
      <div class="card-header gap-4">
        <span class="card-title">Choices</span>
        <span v-if="jobs_left" class="text-(--text-muted)">
          Optimizing {{ jobs_left }} run{{ jobs_left > 1 ? "s" : "" }}
        </span>
      </div>
      <div class="card-body flex flex-col gap-3">
        <span class="text-(--text-main)">
          What buying each raid's bonus chest is worth to this character, how
          much less gold your upgrades cost on average if you buy the chest.
          Only guaranteed honing materials are considered.
        </span>
        <span
          v-if="no_upgrades"
          class="self-center text-center text-(--warning)"
          role="alert"
        >
          No upgrades are selected for this character, so nothing here is worth
          anything.
        </span>
        <!-- scrolls on its own on a narrow screen, so the page does not -->
        <div class="overflow-x-auto">
          <div
            class="grid w-max min-w-full grid-cols-[minmax(10rem,auto)_6rem_auto_minmax(18rem,auto)_auto] items-center gap-x-4 gap-y-1"
            role="table"
            aria-label="Choices"
          >
            <span class="text-(--text-muted)">Gate</span>
            <span class="text-right text-(--text-muted)">Cost</span>
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
            <div class="col-span-full border-b border-(--border-muted)" />
            <template v-for="(group, at) in groups" :key="group.name">
              <!-- a line between raids, the header's under the first -->
              <div
                class="col-span-full flex flex-wrap items-center gap-x-3 gap-y-1 py-1"
                :class="{ 'mt-1 border-t border-(--border-muted) pt-2': at }"
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
                      'bg-(--gold-muted)!': group.shown === difficulty.name,
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
                v-for="{ option, job, choice, key } in group.rows.flatMap(
                  (row) =>
                    row.options.map((x, index) => ({
                      ...x,
                      choice: row.choice,
                      key: `${row.choice.group}/${row.choice.title}/${index}`,
                    })),
                )"
                :key="job.hash"
              >
                <span class="text-(--text-main)">{{ choice.title }}</span>
                <span class="text-right text-(--gold) tabular-nums">
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
                  class="tabular-nums"
                  :aria-label="`${choice.group} ${choice.title} worth`"
                  :style="{
                    color: worth_color(no_upgrades, worth(job.metric)),
                  }"
                >
                  <Spinner
                    v-if="!no_upgrades && worth(job.metric) === undefined"
                    class="inline-block h-4 w-4 align-middle"
                  />
                  {{ worth_text(no_upgrades, worth(job.metric)) }}
                </span>
                <button
                  class="generic-button w-28"
                  :disabled="!affordable(choice, option, columns)"
                  :title="
                    affordable(choice, option, columns)
                      ? 'Add what it gives and take its gold'
                      : 'Not enough gold'
                  "
                  @click="take(key, choice, option)"
                >
                  {{ taken === key ? "Added ✓" : "Add to budget" }}
                </button>
              </template>
            </template>
          </div>
        </div>
        <button
          v-if="hidden"
          class="generic-button w-fit self-center"
          @click="more++"
        >
          Show one more raid ({{ hidden }} hidden)
        </button>
      </div>
    </div>
  </div>
</template>
