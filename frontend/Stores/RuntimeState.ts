import { defineStore } from "pinia";
import { markRaw } from "vue";
import {
  create_worker_bundle,
  WorkerBundle,
} from "@/WasmInterface/WorkerBundle";
import { create_frame_source } from "@/Components/Character/InventoryScanner/FramePassing";
import { useRosterStore } from "./RosterConfig";
import type { MarketRegions } from "@/Utils/MarketDataFetcher";
import type { BudgetSnapshot } from "@/Components/Character/Instructions/Details/NormalDetails/SuccessUtils";

export interface CharRuntime {
  optimizer: WorkerBundle;
  histogram: WorkerBundle;
  marginal: WorkerBundle;
}

// A bundle only spawns a real Worker on its first launch, so idle pairs cost nothing.
function new_char_runtime(): CharRuntime {
  return {
    optimizer: create_worker_bundle(),
    histogram: create_worker_bundle(),
    marginal: create_worker_bundle(),
  };
}

// Everything in here is session-only. Nothing is written to localStorage, which is why
// ConfigStorage no longer needs a list of fields to strip before saving.
export const useRuntimeStore = defineStore("runtime", {
  state: () => ({
    per_char: {} as Record<string, CharRuntime>,
    // keyed by profile id; stand-in for the tick before sync_profiles catches up
    idle: new_char_runtime(),

    cropper: null as WorkerBundle | null,
    frame_source: markRaw(create_frame_source()),

    // the budgets before this run's deductions, by profile id
    budget_snapshots: {} as Record<string, BudgetSnapshot>,
    adv_cache: null as any,

    is_details_update: false,
    // per market region
    is_fetching: {} as Partial<Record<MarketRegions, boolean>>,
    market_fetch_failed: {} as Partial<Record<MarketRegions, boolean>>,
  }),

  getters: {
    active(state): CharRuntime {
      const id = useRosterStore().active_profile.id;
      return state.per_char[id] ?? state.idle;
    },
    optimizer(): WorkerBundle {
      return this.active.optimizer;
    },
    histogram(): WorkerBundle {
      return this.active.histogram;
    },
    marginal(): WorkerBundle {
      return this.active.marginal;
    },
  },

  actions: {
    for_char(id: string): CharRuntime | undefined {
      return this.per_char[id];
    },

    // Driven by a single watch in App.vue, so adds and deletes all land here.
    sync_profiles(ids: string[]) {
      for (const id of ids) {
        if (!this.per_char[id]) {
          this.per_char[id] = new_char_runtime();
        }
      }
      for (const id of Object.keys(this.per_char)) {
        if (!ids.includes(id)) {
          this.per_char[id].optimizer.cancel();
          this.per_char[id].histogram.cancel();
          this.per_char[id].marginal.cancel();
          delete this.per_char[id];
        }
      }
    },

    cancel_active() {
      this.optimizer.cancel();
      this.histogram.cancel();
      this.marginal.cancel();
    },

    ensure_cropper(): WorkerBundle {
      if (!this.cropper) {
        this.cropper = create_worker_bundle();
      }
      return this.cropper;
    },
  },
});
