import type { CharProfile } from "@/Stores/CharacterProfile";
import type { Snapshot } from "@/Utils/Scans";

// one normal honing upgrade wanted, so payloads have upgrade_info
export function want_upgrade(profile: CharProfile, upgrade_index = 10) {
  const key = `0,${upgrade_index},true,${profile.tier}` as const;
  profile.keyed_upgrades[key] = {
    piece_index: 0,
    upgrade_index,
    is_normal_honing: true,
    starting_artisan: 0,
    starting_num_taps: 0,
    state: null,
    unlocked: false,
    adv_progress: null,
    double_balls: false,
    expanded: false,
    taps_since_last_input: 0,
    used_materials: null,
  };
}

// a bare snapshot of one character's char-bound budget
export function snap(
  at: number,
  char_id: string,
  bound: Record<string, number>,
  extra: Partial<Snapshot> = {},
): Snapshot {
  return {
    at,
    kind: "Manual",
    roster_id: 0,
    roster_owned: {},
    tradable: {},
    roster_chests: [],
    chars: { [char_id]: { name: "A", bound, special: {}, chests: [] } },
    prices: {},
    bundles: {},
    ...extra,
  };
}

// a fake optimizer state and histogram output
export const fake_state = (tag = 0) => ({
  upgrade_arr: [],
  prep_output: { juice_info: null },
  special_state: [tag],
  chest_states: new Map(),
  adv_cache: null,
});
export const fake_out = (gold: number, plan: number) => ({
  metric_per_plan: [gold, plan],
});
