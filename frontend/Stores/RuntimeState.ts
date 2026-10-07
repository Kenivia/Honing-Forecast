import { defineStore } from "pinia";
import { markRaw } from "vue";
import {
  create_worker_bundle,
  WorkerBundle,
} from "@/WasmInterface/WorkerBundle";
import { create_frame_source } from "@/Components/Character/InventoryScanner/FramePassing";
import { useRosterStore } from "./RosterConfig";
import type { BudgetSnapshot } from "@/Components/Character/Instructions/Details/NormalDetails/SuccessUtils";

export interface CharRuntime {
  optimizer: WorkerBundle;
  histogram: WorkerBundle;
}

// A bundle only spawns a real Worker on its first launch, so idle pairs cost nothing.
function new_char_runtime(): CharRuntime {
  return {
    optimizer: create_worker_bundle(),
    histogram: create_worker_bundle(),
  };
}

// Everything in here is session-only. Nothing is written to localStorage, which is why
// ConfigStorage no longer needs a list of fields to strip before saving.
export const useRuntimeStore = defineStore("runtime", {
  state: () => ({
    per_char: {} as Record<string, CharRuntime>,
    // stand-in for the tick between a rename and sync_profiles catching up
    idle: new_char_runtime(),

    cropper: null as WorkerBundle | null,
    frame_source: markRaw(create_frame_source()),

    budget_snapshot: null as BudgetSnapshot | null,
    adv_cache: null as any,

    is_details_update: false,
    is_fetching: false,
    market_fetch_failed: true,
  }),

  getters: {
    active(state): CharRuntime {
      const name = useRosterStore().active_profile.char_name;
      return state.per_char[name] ?? state.idle;
    },
    optimizer(): WorkerBundle {
      return this.active.optimizer;
    },
    histogram(): WorkerBundle {
      return this.active.histogram;
    },
  },

  actions: {
    for_char(char_name: string): CharRuntime | undefined {
      return this.per_char[char_name];
    },

    // Driven by a single watch in App.vue, so adds, renames and deletes all land here.
    sync_profiles(names: string[]) {
      for (const name of names) {
        if (!this.per_char[name]) {
          this.per_char[name] = new_char_runtime();
        }
      }
      for (const name of Object.keys(this.per_char)) {
        if (!names.includes(name)) {
          this.per_char[name].optimizer.cancel();
          this.per_char[name].histogram.cancel();
          delete this.per_char[name];
        }
      }
    },

    cancel_active() {
      this.optimizer.cancel();
      this.histogram.cancel();
    },

    ensure_cropper(): WorkerBundle {
      if (!this.cropper) {
        this.cropper = create_worker_bundle();
      }
      return this.cropper;
    },
  },
});
