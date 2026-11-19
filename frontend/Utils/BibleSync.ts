// Refresh and roster sync for signed-in users; see AI notes "External services", "Character import".
import { reactive, ref } from "vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import { CharProfile, new_char_profile } from "@/Stores/CharacterProfile";
import { create_default_owned_input_column } from "@/Stores/RosterConfig";
import { format_char_name } from "./Helpers";
import { find_linked, linked_rosters, norm_name, signed_in } from "./BibleAuth";
import {
  fetch_char,
  market_to_uwuowo_region,
} from "@/Components/Common/Uwuowo/UwuowoUtils";
import { apply_results } from "@/Components/Common/Uwuowo/ApplyResults";
import { MarketRegions } from "./MarketDataFetcher";

export const refreshing = reactive(new Set<string>());
export const refresh_errors = reactive<Record<string, string>>({});

const region_of = (p: CharProfile) =>
  market_to_uwuowo_region(
    useRosterStore().roster_config.all_regions[p.roster_id],
  );
const find_profile = (id: string) =>
  useRosterStore().roster_config.profiles.find((p) => p.id === id);

// applied without the confirm panel; dropped if the name or region changed meanwhile
export async function refresh_char(id: string) {
  const profile = find_profile(id);
  if (!profile || refreshing.has(id)) return;
  const region = region_of(profile);
  const name = profile.char_name;
  if (region === null) return;
  refreshing.add(id);
  delete refresh_errors[id];
  const result = await fetch_char(region, name);
  refreshing.delete(id);
  const now = find_profile(id);
  if (!now || now.char_name !== name || region_of(now) !== region) return;
  if (typeof result === "string") {
    refresh_errors[id] = result;
    return;
  }
  apply_results(
    now,
    result.pieces,
    false,
    id !== useRosterStore().active_profile.id,
    result.class_name,
    true,
  );
}

// once per launch: characters fetched before and in a linked roster
let auto_done = false;
export async function auto_refresh() {
  if (auto_done || !signed_in.value) return;
  auto_done = true;
  for (const p of [...useRosterStore().roster_config.profiles]) {
    const region = region_of(p);
    // class_name: imported before fetched_at existed
    const fetched = p.fetched_at > 0 || p.class_name !== "";
    if (fetched && region && (await find_linked(region, p.char_name)))
      await refresh_char(p.id);
  }
}

export interface PendingRoster {
  roster_id: string;
  locals: { id: string; name: string; choice: string }[]; // choice "" = not on lostark.bible
  bibles: string[];
}
export const sync_pending = ref<PendingRoster[]>([]);
export const syncing = ref(false);
let resolve_popup: (ok: boolean) => void = () => {};
export function close_sync_popup(ok: boolean) {
  sync_pending.value = [];
  resolve_popup(ok);
}

// JSON, not deep-equal: values here come from different realms under jsdom
const equal = (a: unknown, b: unknown) =>
  JSON.stringify(a) === JSON.stringify(b);

// the default character with nothing ticked, typed or added, and a roster with no mats
function untouched(p: CharProfile) {
  const config = useRosterStore().roster_config;
  const base = new_char_profile();
  const empty = create_default_owned_input_column();
  return (
    (
      [
        "normal_grid",
        "adv_grid",
        "bound_budgets",
        "special_budget",
        "chests",
        "forbidden_chests",
      ] as const
    ).every((k) => equal(p[k], base[k])) &&
    equal(config.roster_mats_owned[p.roster_id], empty) &&
    equal(config.tradable_mats_owned[p.roster_id], empty) &&
    !config.roster_chests[p.roster_id]?.length
  );
}

// a name typed locally may already hold the bible name in that region
function unique_name(name: string, id: string) {
  const profiles = useRosterStore().roster_config.profiles;
  return format_char_name(
    name,
    profiles.findIndex((p) => p.id === id),
    profiles,
  );
}

const REGIONS: Record<string, MarketRegions> = { NA: "nae", CE: "euc" };

export async function sync_rosters() {
  if (!signed_in.value || syncing.value) return;
  syncing.value = true;
  try {
    await sync_inner();
  } finally {
    syncing.value = false;
  }
}

async function sync_inner() {
  const store = useRosterStore();
  const config = store.roster_config;
  const bibles = await linked_rosters(true);
  const first = config.profiles[0];
  // a fresh install's untouched default character is replaced
  const fresh =
    config.profiles.length === 1 &&
    first.fetched_at === 0 &&
    first.char_name.startsWith("Newchar") &&
    untouched(first)
      ? first.id
      : null;

  const matched = new Set<string>();
  const per_roster = bibles.map((b) => {
    const region = REGIONS[b.region];
    const in_region = config.profiles.filter(
      (p) => config.all_regions[p.roster_id] === region && p.id !== fresh,
    );
    const unmatched: string[] = [];
    const counts: Record<string, number> = {};
    for (const c of b.characters) {
      const local = in_region.find(
        (p) => norm_name(p.char_name) === norm_name(c.name),
      );
      if (!local) {
        unmatched.push(c.name);
        continue;
      }
      local.char_name = unique_name(c.name, local.id);
      matched.add(local.id);
      counts[local.roster_id] = (counts[local.roster_id] ?? 0) + 1;
    }
    const best = Object.keys(counts).sort((x, y) => counts[y] - counts[x])[0];
    return { region, unmatched, best };
  });

  const adds: { roster_id?: string; region: MarketRegions; names: string[] }[] =
    [];
  const pending: PendingRoster[] = reactive([]);
  const claimed = new Set<string>();
  for (const { region, unmatched, best } of per_roster) {
    if (unmatched.length === 0) continue;
    const locals = config.profiles.filter(
      (p) =>
        best !== undefined &&
        p.roster_id === best &&
        !matched.has(p.id) &&
        !claimed.has(p.id),
    );
    locals.forEach((p) => claimed.add(p.id));
    if (locals.length > 0)
      pending.push({
        roster_id: best,
        locals: locals.map((p) => ({
          id: p.id,
          name: p.char_name,
          choice: "",
        })),
        bibles: unmatched,
      });
    else adds.push({ roster_id: best, region, names: unmatched });
  }

  if (pending.length > 0) {
    sync_pending.value = pending;
    const ok = await new Promise<boolean>((r) => (resolve_popup = r));
    if (ok)
      for (const entry of pending) {
        for (const local of entry.locals) {
          const p = find_profile(local.id);
          if (!local.choice || !p) continue;
          p.char_name = unique_name(local.choice, p.id);
          matched.add(p.id);
        }
        const chosen = entry.locals.map((l) => l.choice);
        adds.push({
          roster_id: entry.roster_id,
          region: config.all_regions[entry.roster_id],
          names: entry.bibles.filter((n) => !chosen.includes(n)),
        });
      }
  }

  const to_refresh = [...matched];
  for (const add of adds) {
    for (const name of add.names) {
      const p = add.roster_id
        ? store.add_char(add.roster_id)
        : store.add_roster(add.region);
      add.roster_id = p.roster_id;
      p.char_name = unique_name(name, p.id);
      to_refresh.push(p.id);
    }
  }
  if (fresh && config.profiles.length > 1)
    store.delete_profile(config.profiles.findIndex((p) => p.id === fresh));

  for (const id of to_refresh) await refresh_char(id);
}
