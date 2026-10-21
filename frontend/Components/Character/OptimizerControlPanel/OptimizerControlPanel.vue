<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { storeToRefs } from "pinia";
import { AdvOverride, NormalOverride } from "@/WasmInterface/Overrides";
import { start_eval_hist } from "../CharWorkerUtils";
import { computed, ref } from "vue";
import { GridConfig } from "@/Utils/GridStyling";
import { get_optimizer_working } from "../Instructions/InstructionUtils";
import OptimizerRadioButton from "./OptimizerRadioButton.vue";
import { locale_to_fixed } from "@/Utils/Helpers";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { DEFAULT_CHAR_PROFILE } from "@/Stores/CharacterProfile";
const runtime = useRuntimeStore();

const store = useRosterStore();
const { active_profile } = storeToRefs(store);

const show_override = ref(false);

const optimizer_working = computed(get_optimizer_working);

type Group = "normal" | "special" | "advanced";
const option = <T,>(value: T, label: string) => ({ value, label });

// one radio column per override; the special one sits between book and adv juice
const COLUMNS = [
  {
    title: "Juice Usage",
    group: "normal",
    key: "juice",
    options: [
      option(NormalOverride.Optimizer, "Original"),
      option(NormalOverride.Full, "Full juice"),
      option(NormalOverride.Empty, "No juice"),
    ],
  },
  {
    title: "Book Usage",
    group: "normal",
    key: "book",
    options: [
      option(NormalOverride.Optimizer, "Original"),
      option(NormalOverride.Full, "Full Book"),
      option(NormalOverride.Empty, "No Book"),
    ],
  },
  { title: "Special Usage", group: "special" },
  {
    title: "Adv Juice Usage",
    group: "advanced",
    key: "juice",
    options: [
      option(AdvOverride.Optimizer, "Original"),
      option(AdvOverride.Full, "Full juice"),
      option(AdvOverride.Grace, "Juice on Grace"),
      option(AdvOverride.Empty, "No juice"),
    ],
  },
  {
    title: "Adv Scroll Usage",
    group: "advanced",
    key: "scroll",
    options: [
      option(AdvOverride.Optimizer, "Original"),
      option(AdvOverride.Full, "Full Scroll"),
      option(AdvOverride.Grace, "Scroll on Grace"),
      option(AdvOverride.Empty, "No Scroll"),
    ],
  },
] as { title: string; group: Group; key?: string; options?: any[] }[];

// the special order, shown once "Original" is unticked
const SPECIAL = [
  {
    key: "weapon_first",
    options: [option(true, "Weapon First"), option(false, "Armor First")],
  },
  {
    key: "highest_first",
    options: [option(true, "Highest first"), option(false, "Lowest first")],
  },
];

function set_override(group: Group, key: string, value: unknown) {
  active_profile.value.optimizer_override[group][key] = value;
  start_eval_hist();
}

function toggle_compare() {
  active_profile.value.optimizer_override = structuredClone(
    DEFAULT_CHAR_PROFILE.optimizer_override,
  );
  show_override.value = !show_override.value;
  start_eval_hist();
}

const grid: GridConfig = {
  grid_template_columns:
    "minmax(110px,0.75fr) minmax(110px,1fr) minmax(110px,1fr)  minmax(110px,1fr)  minmax(110px,1fr)",
};
</script>

<template>
  <div class="card-shell max-w-[min(800px,100%)]! min-w-[min(800px,100%)]!">
    <label class="control-panel-checkbox-row">
      <input v-model="active_profile.auto_start_optimizer" type="checkbox" />
      <span>Auto start optimizer </span>
      <span
        class="pl-3 text-(--warning)"
        v-if="!active_profile.auto_start_optimizer"
        >Optimizer will not re-run on updates, results may not be optimal
      </span>
    </label>
    <label class="control-panel-checkbox-row">
      <input
        type="checkbox"
        :checked="active_profile.min_resolution === 10"
        @change="
          active_profile.min_resolution = (
            $event.target as HTMLInputElement
          ).checked
            ? 10
            : 1
        "
      />
      <span>Force x10 streak length</span>
    </label>
    <div>
      <div class="mx-2 flex w-full flex-row items-center gap-3">
        <span class="content-center text-nowrap">Optimizer progress:</span>
        <span
          :style="{
            color:
              runtime.optimizer.est_progress_percentage == 100
                ? 'var(--gold)'
                : 'var(--warning-dark)',
          }"
        >
          {{ locale_to_fixed(runtime.optimizer.est_progress_percentage, 2) }}%
        </span>
        <div class="progress-bar">
          <div
            class="progress-fill"
            :style="{
              width: `${runtime.optimizer.est_progress_percentage}%`,
            }"
          />
        </div>
        <button
          class="lit-button mr-3 min-w-30"
          :style="{
            border: `1px solid var(${show_override ? '--border-main' : '--border-muted'})`,
            background: show_override
              ? `var(--warning-dark)`
              : 'var(--bg-main)',
            opacity: optimizer_working ? 0.5 : 1,
            pointerEvents: optimizer_working ? 'none' : 'auto',
          }"
          @click="toggle_compare"
        >
          Compare with simple strategies
        </button>
      </div>
    </div>
    <div class="card-body pt-0! pb-0!">
      <div
        class="outer-grid mx-auto"
        :style="{
          '--grid-cols': grid.grid_template_columns,
          opacity: optimizer_working ? 0.5 : 1,
          pointerEvents: optimizer_working ? 'none' : 'auto',
        }"
      >
        <div v-if="show_override" class="contents">
          <div class="mats-row h-fit! items-end!">
            <span
              v-for="column in COLUMNS"
              :key="column.title"
              class="h-fit w-full pl-2 text-left"
              >{{ column.title }}</span
            >
          </div>

          <div class="mats-row h-fit! items-start! border-b-0!">
            <template v-for="column in COLUMNS" :key="column.title">
              <!-- two things to pick, hidden while the optimizer's order is used -->
              <div
                v-if="column.group === 'special'"
                class="flex h-full grow flex-col"
              >
                <label class="control-panel-checkbox-row">
                  <input
                    type="checkbox"
                    v-model="
                      active_profile.optimizer_override.special.optimizer
                    "
                    @change="start_eval_hist"
                  />
                  <span>Original</span>
                </label>
                <template
                  v-if="!active_profile.optimizer_override.special.optimizer"
                >
                  <OptimizerRadioButton
                    v-for="pick in SPECIAL"
                    :key="pick.key"
                    :options="pick.options"
                    :model-value="
                      active_profile.optimizer_override.special[pick.key]
                    "
                    :on-change="(v) => set_override('special', pick.key, v)"
                  />
                </template>
              </div>
              <OptimizerRadioButton
                v-else
                :options="column.options"
                :model-value="
                  active_profile.optimizer_override[column.group][column.key]
                "
                :on-change="(v) => set_override(column.group, column.key, v)"
              />
            </template>
          </div>
        </div>
      </div>
      <span v-if="show_override" class="annotation"
        >Comparison with Maxroll is unavailable because I couldn't figure out
        how they calculate their gold values.</span
      >
    </div>
  </div>
</template>
<style>
.progress-bar {
  width: 100%;
  height: 8px;
  background: var(--bg-very-bright);
  border-radius: 4px;
  overflow: hidden;
}
.progress-fill {
  height: 100%;
  background: var(--gold);
  transition: width 0.1s ease;
}
</style>
