<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { storeToRefs } from "pinia";
import { BUTTON_LABELS, CSS_NAMES } from "@/Utils/Constants";
import {
  SpecialOverride,
  StateOverride,
  Trinary,
} from "@/WasmInterface/PayloadBuilder";
import { start_eval_hist } from "./CharWorkerUtils";
import { ref } from "vue";
import { GridConfig } from "@/Utils/GridStyling";

const store = useRosterStore();
const { active_profile } = storeToRefs(store);

const GRAPH_COLOR_VARS = ["--average", "--bound", "--roster", "--tradable"];
const show_override = ref(false);

const grid: GridConfig = {
  grid_template_columns:
    "minmax(110px,1fr) minmax(110px,1fr) minmax(110px,1fr)",
};
</script>

<template>
  <section class="card-shell min-w-241">
    <div class="card-header">
      <div class="mx-2 flex w-full flex-row items-center gap-3">
        <span class="text-nowrap"
          >Optimizer progress:
          <span
            :style="{
              color:
                active_profile.optimizer_worker_bundle
                  .est_progress_percentage == 100
                  ? 'var(--gold)'
                  : 'var(--warning-dark)',
            }"
          >
            {{
              active_profile.optimizer_worker_bundle.est_progress_percentage.toFixed(
                2,
              )
            }}%</span
          >
        </span>
        <div class="progress-bar">
          <div
            class="progress-fill"
            :style="{
              width: `${active_profile.optimizer_worker_bundle.est_progress_percentage}%`,
            }"
          />
        </div>
        <button
          class="lit-button"
          :style="{
            border: `1px solid var(${show_override ? '--border-main' : '--border-muted'})`,
            background: show_override ? `var(--warning-dark)` : '',
          }"
          @click="
            () => {
              active_profile.optimizer_override.state.juice = Trinary.Optimizer;
              active_profile.optimizer_override.state.book == Trinary.Optimizer;
              active_profile.optimizer_override.special_state.optimizer = true;
              show_override = !show_override;
              start_eval_hist();
            }
          "
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
        }"
      >
        <div v-if="show_override" class="contents">
          <div class="mats-row h-fit! items-end! gap-4">
            <span class="h-fit w-full text-left">Juice Usage</span>
            <span class="h-fit w-full text-left">Book & Scroll Usage</span
            ><span class="h-fit w-full text-left">Special Usage</span>
          </div>

          <!-- just treating everything as one row cos why not -->
          <div class="mats-row h-fit! items-start! border-b-0!">
            <div class="flex grow flex-col">
              <label class="control-panel-checkbox-row">
                <input
                  type="checkbox"
                  :checked="
                    active_profile.optimizer_override.state.juice ===
                    Trinary.Optimizer
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.state.juice =
                        Trinary.Optimizer;
                      (e.target as HTMLInputElement).checked = true;
                      start_eval_hist();
                    }
                  "
                />
                <span>Original</span>
              </label>
              <label class="control-panel-checkbox-row">
                <input
                  type="checkbox"
                  :checked="
                    active_profile.optimizer_override.state.juice ===
                    Trinary.Full
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.state.juice =
                        Trinary.Full;
                      (e.target as HTMLInputElement).checked = true;
                      start_eval_hist();
                    }
                  "
                />
                <span>Full juice</span>
              </label>
              <label class="control-panel-checkbox-row">
                <input
                  type="checkbox"
                  :checked="
                    active_profile.optimizer_override.state.juice ===
                    Trinary.Empty
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.state.juice =
                        Trinary.Empty;
                      (e.target as HTMLInputElement).checked = true;
                      start_eval_hist();
                    }
                  "
                />
                <span>No juice</span>
              </label>
            </div>

            <div class="flex grow flex-col">
              <label class="control-panel-checkbox-row">
                <input
                  type="checkbox"
                  :checked="
                    active_profile.optimizer_override.state.book ===
                    Trinary.Optimizer
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.state.book =
                        Trinary.Optimizer;
                      (e.target as HTMLInputElement).checked = true;
                      start_eval_hist();
                    }
                  "
                />
                <span>Original</span>
              </label>
              <label class="control-panel-checkbox-row">
                <input
                  type="checkbox"
                  :checked="
                    active_profile.optimizer_override.state.book ===
                    Trinary.Full
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.state.book =
                        Trinary.Full;
                      (e.target as HTMLInputElement).checked = true;
                      start_eval_hist();
                    }
                  "
                />
                <span>Full Book</span>
              </label>
              <label class="control-panel-checkbox-row">
                <input
                  type="checkbox"
                  :checked="
                    active_profile.optimizer_override.state.book ===
                    Trinary.Empty
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.state.book =
                        Trinary.Empty;
                      (e.target as HTMLInputElement).checked = true;
                      start_eval_hist();
                    }
                  "
                />
                <span>No Book</span>
              </label>
            </div>

            <div class="flex h-full grow flex-col">
              <label class="control-panel-checkbox-row">
                <input
                  type="checkbox"
                  v-model="
                    active_profile.optimizer_override.special_state.optimizer
                  "
                  @change="start_eval_hist"
                />
                <span>Original</span>
              </label>
              <label
                class="control-panel-checkbox-row"
                v-if="
                  !active_profile.optimizer_override.special_state.optimizer
                "
              >
                <input
                  type="checkbox"
                  :checked="
                    active_profile.optimizer_override.special_state
                      .weapon_first === true
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.special_state.weapon_first = true;
                      (e.target as HTMLInputElement).checked = true;
                      start_eval_hist();
                    }
                  "
                />
                <span>Weapon First</span>
              </label>
              <label
                class="control-panel-checkbox-row"
                v-if="
                  !active_profile.optimizer_override.special_state.optimizer
                "
              >
                <input
                  type="checkbox"
                  :checked="
                    active_profile.optimizer_override.special_state
                      .weapon_first === false
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.special_state.weapon_first = false;
                      (e.target as HTMLInputElement).checked = true;
                      start_eval_hist();
                    }
                  "
                />
                <span>Armor First</span>
              </label>

              <label
                class="control-panel-checkbox-row"
                v-if="
                  !active_profile.optimizer_override.special_state.optimizer
                "
              >
                <input
                  type="checkbox"
                  :checked="
                    active_profile.optimizer_override.special_state
                      .highest_first === true
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.special_state.highest_first = true;
                      (e.target as HTMLInputElement).checked = true;
                      start_eval_hist();
                    }
                  "
                />
                <span>Use on Highest</span>
              </label>
              <label
                class="control-panel-checkbox-row"
                v-if="
                  !active_profile.optimizer_override.special_state.optimizer
                "
              >
                <input
                  type="checkbox"
                  :checked="
                    active_profile.optimizer_override.special_state
                      .highest_first === false
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.special_state.highest_first = false;
                      (e.target as HTMLInputElement).checked = true;
                      start_eval_hist();
                    }
                  "
                />
                <span>Use on Lowest</span>
              </label>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
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
