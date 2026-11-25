// Refresh and roster sync for signed-in users; see AI notes "External services", "Character import".
import { reactive, ref } from "vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import { CharProfile } from "@/Stores/CharacterProfile";
import { fresh_profile_id } from "@/Stores/RosterConfig";
import { char_name_valid } from "./Helpers";
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
  if (region === null || !char_name_valid(name)) return;
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

// once per launch: characters fetched before, in a linked roster and still locked (unlocking pauses it)
let auto_done = false;
export async function auto_refresh() {
  if (auto_done || !signed_in.value) return;
  auto_done = true;
  for (const p of [...useRosterStore().roster_config.profiles]) {
    const region = region_of(p);
    // class_name: imported before fetched_at existed
    const fetched = p.fetched_at > 0 || p.class_name !== "";
    if (
      fetched &&
      p.lock_fetched_done &&
      region &&
      char_name_valid(p.char_name) &&
      (await find_linked(region, p.char_name))
    )
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
  // a fresh install's untouched default character is replaced
  const fresh = fresh_profile_id(config);

  // exact names before accentless ones (accented and plain forms may both exist), each local matched once
  const matched = new Set<string>();
  const local_of = new Map<object, (typeof config.profiles)[number]>();
  for (const exact of [true, false])
    for (const b of bibles)
      for (const c of b.characters) {
        if (local_of.has(c)) continue;
        const local = config.profiles.find(
          (p) =>
            config.all_regions[p.roster_id] === REGIONS[b.region] &&
            p.id !== fresh &&
            !matched.has(p.id) &&
            (exact
              ? p.char_name === c.name
              : norm_name(p.char_name) === norm_name(c.name)),
        );
        if (!local) continue;
        local_of.set(c, local);
        matched.add(local.id);
      }

  const per_roster = bibles.map((b) => {
    const region = REGIONS[b.region];
    const unmatched: string[] = [];
    const counts: Record<string, number> = {};
    for (const c of b.characters) {
      const local = local_of.get(c);
      if (!local) {
        unmatched.push(c.name);
        continue;
      }
      store.rename_char(local.id, c.name);
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
          name: store.char_label(p),
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
          store.rename_char(p.id, local.choice);
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
      store.rename_char(p.id, name);
      to_refresh.push(p.id);
    }
  }
  if (fresh && config.profiles.length > 1)
    store.delete_profile(config.profiles.findIndex((p) => p.id === fresh));

  for (const id of to_refresh) await refresh_char(id);
}
