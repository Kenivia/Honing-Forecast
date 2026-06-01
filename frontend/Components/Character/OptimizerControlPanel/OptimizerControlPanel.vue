<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { storeToRefs } from "pinia";
import { AdvOverride, NormalOverride } from "@/WasmInterface/PayloadBuilder";
import { start_eval_hist } from "../CharWorkerUtils";
import { computed, ref } from "vue";
import { GridConfig } from "@/Utils/GridStyling";
import { get_optimizer_working } from "../Instructions/InstructionUtils";
import OptimizerRadioButton from "./OptimizerRadioButton.vue";

const store = useRosterStore();
const { active_profile } = storeToRefs(store);

const show_override = ref(false);

const optimizer_working = computed(get_optimizer_working);

const juice_options: { value: NormalOverride; label: string }[] = [
  { value: NormalOverride.Optimizer, label: "Original" },
  { value: NormalOverride.Full, label: "Full juice" },
  { value: NormalOverride.Empty, label: "No juice" },
];

const book_options: { value: NormalOverride; label: string }[] = [
  { value: NormalOverride.Optimizer, label: "Original" },
  { value: NormalOverride.Full, label: "Full Book" },
  { value: NormalOverride.Empty, label: "No Book" },
];

const adv_juice_options: { value: AdvOverride; label: string }[] = [
  { value: AdvOverride.Optimizer, label: "Original" },
  { value: AdvOverride.Full, label: "Full juice" },
  { value: AdvOverride.Grace, label: "Juice on Grace" },
  { value: AdvOverride.Empty, label: "No juice" },
];

const adv_scroll_options: { value: AdvOverride; label: string }[] = [
  { value: AdvOverride.Optimizer, label: "Original" },
  { value: AdvOverride.Full, label: "Full Scroll" },
  { value: AdvOverride.Grace, label: "Scroll on Grace" },
  { value: AdvOverride.Empty, label: "No Scroll" },
];

function on_juice_change(v: NormalOverride) {
  active_profile.value.optimizer_override.normal.juice = v;
  start_eval_hist();
}

function on_book_change(v: NormalOverride) {
  active_profile.value.optimizer_override.normal.book = v;
  start_eval_hist();
}

function on_adv_juice_change(v: AdvOverride) {
  active_profile.value.optimizer_override.advanced.juice = v;
  start_eval_hist();
}

function on_adv_scroll_change(v: AdvOverride) {
  active_profile.value.optimizer_override.advanced.scroll = v;
  start_eval_hist();
}

const grid: GridConfig = {
  grid_template_columns:
    "minmax(110px,0.75fr) minmax(110px,1fr) minmax(110px,1fr)  minmax(110px,1fr)  minmax(110px,1fr)",
};
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
            background: show_override
              ? `var(--warning-dark)`
              : 'var(--bg-main)',
            opacity: optimizer_working ? 0.5 : 1,
            pointerEvents: optimizer_working ? 'none' : 'auto',
          }"
          @click="
            () => {
              active_profile.optimizer_override.normal.juice =
                NormalOverride.Optimizer;
              active_profile.optimizer_override.normal.book =
                NormalOverride.Optimizer;
              active_profile.optimizer_override.special.optimizer = true;
              active_profile.optimizer_override.advanced.juice =
                AdvOverride.Optimizer;
              active_profile.optimizer_override.advanced.scroll =
                AdvOverride.Optimizer;
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
            <span class="h-fit w-full pl-2 text-left">Book Usage</span>
            <span class="h-fit w-full pl-2 text-left">Special Usage</span>
            <span class="h-fit w-full pl-2 text-left">Adv Juice Usage</span>
            <span class="h-fit w-full pl-2 text-left">Adv Scroll Usage</span>
          </div>

          <div class="mats-row h-fit! items-start! border-b-0!">
            <OptimizerRadioButton
              :options="juice_options"
              :model-value="active_profile.optimizer_override.normal.juice"
              :on-change="on_juice_change"
            />

            <OptimizerRadioButton
              :options="book_options"
              :model-value="active_profile.optimizer_override.normal.book"
              :on-change="on_book_change"
            />

            <!-- special is a special case (hah) because there's 2 things that needs to be selected, hiding them when not ticked to avoid confusing -->
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
                  type="radio"
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
                  type="radio"
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
                  type="radio"
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
                  type="radio"
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

            <OptimizerRadioButton
              :options="adv_juice_options"
              :model-value="active_profile.optimizer_override.advanced.juice"
              :on-change="on_adv_juice_change"
            />

            <OptimizerRadioButton
              :options="adv_scroll_options"
              :model-value="active_profile.optimizer_override.advanced.scroll"
              :on-change="on_adv_scroll_change"
            />
          </div>
        </div>
      </div>
      <span v-if="show_override" class="annotation"
        >Comparison with Maxroll is unavailable because I couldn't figure out
        how they calculate their gold values.</span
      >
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
