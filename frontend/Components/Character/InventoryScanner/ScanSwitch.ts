import { ref } from "vue";
import type { RouteLocationNormalized } from "vue-router";
import { useRosterStore } from "@/Stores/RosterConfig";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { reset_scan, roster_cover, scan_wipes } from "./ScanStore";
import { scan_unexported } from "./Manifest";

// The scan belongs to the active character. Going to another one wipes it, after asking when
// something is not exported; the dialog can also move it there whole, worker included.

// the character asked for while the dialog is up
export const pending_switch = ref<{
  name: string;
  path: string;
  same_roster: boolean;
} | null>(null);
// the next switch keeps the scan
let moving = false;

export function wipe_scan() {
  useRuntimeStore().cropper?.cancel_and_clear_prev_result();
  reset_scan();
  roster_cover.value = null;
  scan_wipes.value++;
}

// the cover was chosen against the old roster's saved reading
export function move_scan(same_roster: boolean) {
  moving = true;
  if (!same_roster) roster_cover.value = null;
}

// a router guard
export function guard_switch(to: RouteLocationNormalized) {
  const name = to.params.characterName as string | undefined;
  if (!name) return;
  const store = useRosterStore();
  const target = store.all_profiles[store.profile_index_by_name(name)];
  if (!target || target.id === store.active_profile.id) return;
  if (moving) {
    moving = false;
    return;
  }
  if (scan_unexported.value) {
    pending_switch.value = {
      name,
      path: to.fullPath,
      same_roster: target.roster_id === store.active_profile.roster_id,
    };
    return false;
  }
  wipe_scan();
}
