<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { storeToRefs } from "pinia";
import { Quaternary, Trinary } from "@/WasmInterface/PayloadBuilder";
import { start_eval_hist } from "./CharWorkerUtils";
import { computed, ref } from "vue";
import { GridConfig } from "@/Utils/GridStyling";
import { get_optimizer_working } from "./Instructions/InstructionUtils";

const store = useRosterStore();
const { active_profile } = storeToRefs(store);

const show_override = ref(false);

const grid: GridConfig = {
  grid_template_columns:
    "minmax(110px,0.75fr) minmax(110px,1fr) minmax(110px,1fr)  minmax(110px,1fr)  minmax(110px,1fr)",
};
const optimizer_working = computed(get_optimizer_working);
</script>

<template>
  <section class="card-shell max-w-[min(800px,100%)]! min-w-[min(800px,100%)]!">
    <div>
      <div class="mx-2 flex w-full flex-row items-center gap-3">
        <span class="content-center text-nowrap">Optimizer progress:</span>
        <span
          :style="{
            color:
              active_profile.optimizer_worker_bundle.est_progress_percentage ==
              100
                ? 'var(--gold)'
                : 'var(--warning-dark)',
          }"
        >
          {{
            active_profile.optimizer_worker_bundle.est_progress_percentage.toFixed(
              2,
            )
          }}%
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
          class="lit-button mr-3 min-w-30"
          :style="{
            border: `1px solid var(${show_override ? '--border-main' : '--border-muted'})`,
            background: show_override ? `var(--warning-dark)` : '',
            opacity: optimizer_working ? 0.5 : 1,
            pointerEvents: optimizer_working ? 'none' : 'auto',
          }"
          @click="
            () => {
              active_profile.optimizer_override.normal.juice =
                Trinary.Optimizer;
              active_profile.optimizer_override.normal.book ==
                Trinary.Optimizer;
              active_profile.optimizer_override.special.optimizer = true;
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
          opacity: optimizer_working ? 0.5 : 1,
          pointerEvents: optimizer_working ? 'none' : 'auto',
        }"
      >
        <div v-if="show_override" class="contents">
          <div class="mats-row h-fit! items-end!">
            <span class="h-fit w-full pl-2 text-left">Juice Usage</span>
            <span class="h-fit w-full pl-2 text-left">Book Usage</span
            ><span class="h-fit w-full pl-2 text-left">Special Usage</span>
            <span class="h-fit w-full pl-2 text-left">Adv Juice Usage</span>
            <span class="h-fit w-full pl-2 text-left">Adv Scroll Usage</span>
          </div>

          <!-- just treating everything as one row cos why not -->
          <div class="mats-row h-fit! items-start! border-b-0!">
            <div class="flex grow flex-col">
              <label class="control-panel-checkbox-row">
                <input
                  type="checkbox"
                  :checked="
                    active_profile.optimizer_override.normal.juice ===
                    Trinary.Optimizer
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.normal.juice =
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
                    active_profile.optimizer_override.normal.juice ===
                    Trinary.Full
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.normal.juice =
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
                    active_profile.optimizer_override.normal.juice ===
                    Trinary.Empty
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.normal.juice =
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
                    active_profile.optimizer_override.normal.book ===
                    Trinary.Optimizer
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.normal.book =
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
                    active_profile.optimizer_override.normal.book ===
                    Trinary.Full
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.normal.book =
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
                    active_profile.optimizer_override.normal.book ===
                    Trinary.Empty
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.normal.book =
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
                  v-model="active_profile.optimizer_override.special.optimizer"
                  @change="start_eval_hist"
                />
                <span>Original</span>
              </label>
              <label
                class="control-panel-checkbox-row"
                v-if="!active_profile.optimizer_override.special.optimizer"
              >
                <input
                  type="checkbox"
                  :checked="
                    active_profile.optimizer_override.special.weapon_first ===
                    true
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.special.weapon_first = true;
                      (e.target as HTMLInputElement).checked = true;
                      start_eval_hist();
                    }
                  "
                />
                <span>Weapon First</span>
              </label>
              <label
                class="control-panel-checkbox-row"
                v-if="!active_profile.optimizer_override.special.optimizer"
              >
                <input
                  type="checkbox"
                  :checked="
                    active_profile.optimizer_override.special.weapon_first ===
                    false
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.special.weapon_first = false;
                      (e.target as HTMLInputElement).checked = true;
                      start_eval_hist();
                    }
                  "
                />
                <span>Armor First</span>
              </label>

              <label
                class="control-panel-checkbox-row"
                v-if="!active_profile.optimizer_override.special.optimizer"
              >
                <input
                  type="checkbox"
                  :checked="
                    active_profile.optimizer_override.special.highest_first ===
                    true
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.special.highest_first = true;
                      (e.target as HTMLInputElement).checked = true;
                      start_eval_hist();
                    }
                  "
                />
                <span>Use on Highest</span>
              </label>
              <label
                class="control-panel-checkbox-row"
                v-if="!active_profile.optimizer_override.special.optimizer"
              >
                <input
                  type="checkbox"
                  :checked="
                    active_profile.optimizer_override.special.highest_first ===
                    false
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.special.highest_first = false;
                      (e.target as HTMLInputElement).checked = true;
                      start_eval_hist();
                    }
                  "
                />
                <span>Use on Lowest</span>
              </label>
            </div>

            <div class="flex grow flex-col">
              <label class="control-panel-checkbox-row">
                <input
                  type="checkbox"
                  :checked="
                    active_profile.optimizer_override.advanced.juice ===
                    Quaternary.Optimizer
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.advanced.juice =
                        Quaternary.Optimizer;
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
                    active_profile.optimizer_override.advanced.juice ===
                    Quaternary.Full
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.advanced.juice =
                        Quaternary.Full;
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
                    active_profile.optimizer_override.advanced.juice ===
                    Quaternary.Grace
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.advanced.juice =
                        Quaternary.Grace;
                      (e.target as HTMLInputElement).checked = true;
                      start_eval_hist();
                    }
                  "
                />
                <span>Juice on Grace</span>
              </label>
              <label class="control-panel-checkbox-row">
                <input
                  type="checkbox"
                  :checked="
                    active_profile.optimizer_override.advanced.juice ===
                    Quaternary.Empty
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.advanced.juice =
                        Quaternary.Empty;
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
                    active_profile.optimizer_override.advanced.scroll ===
                    Quaternary.Optimizer
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.advanced.scroll =
                        Quaternary.Optimizer;
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
                    active_profile.optimizer_override.advanced.scroll ===
                    Quaternary.Full
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.advanced.scroll =
                        Quaternary.Full;
                      (e.target as HTMLInputElement).checked = true;
                      start_eval_hist();
                    }
                  "
                />
                <span>Full Scroll</span>
              </label>
              <label class="control-panel-checkbox-row">
                <input
                  type="checkbox"
                  :checked="
                    active_profile.optimizer_override.advanced.scroll ===
                    Quaternary.Grace
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.advanced.scroll =
                        Quaternary.Grace;
                      (e.target as HTMLInputElement).checked = true;
                      start_eval_hist();
                    }
                  "
                />
                <span>Scroll on Grace</span>
              </label>
              <label class="control-panel-checkbox-row">
                <input
                  type="checkbox"
                  :checked="
                    active_profile.optimizer_override.advanced.scroll ===
                    Quaternary.Empty
                  "
                  @click="
                    (e) => {
                      active_profile.optimizer_override.advanced.scroll =
                        Quaternary.Empty;
                      (e.target as HTMLInputElement).checked = true;
                      start_eval_hist();
                    }
                  "
                />
                <span>No Scroll</span>
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
