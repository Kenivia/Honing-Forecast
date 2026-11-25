import { beforeEach, describe, expect, it, vi } from "vitest";
import { computed } from "vue";
import { createPinia, setActivePinia } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  auto_refresh,
  close_sync_popup,
  refresh_char,
  sync_pending,
  sync_rosters,
} from "@/Utils/BibleSync";
import { fetch_char } from "@/Components/Common/Uwuowo/UwuowoUtils";
import { bible_auth, bible_rosters, linked_now } from "@/Utils/BibleAuth";

const ROSTERS = vi.hoisted(() => ({ value: [] as any[] }));
vi.mock("@/Utils/BibleAuth", async (original) => ({
  ...(await original<any>()),
  signed_in: computed(() => true),
  linked_rosters: () => Promise.resolve(ROSTERS.value),
  find_linked: (_region: string, name: string) =>
    Promise.resolve(
      ROSTERS.value.some((r) => r.characters.some((c: any) => c.name === name))
        ? {}
        : null,
    ),
}));
vi.mock("@/Components/Common/Uwuowo/UwuowoUtils", async (original) => ({
  ...(await original<any>()),
  fetch_char: vi.fn(),
}));

const piece = (tier: number) => ({ plus_n: 5, ilevel: NaN, tier, adv: 0 });
const result = (tiers: number[]) => ({
  pieces: [...tiers.map(piece), piece(1)],
  class_name: "Bard",
  achieved_ilevel: "1700.00",
});
const roster = (...names: string[]) => ({
  region: "NA",
  world: "W",
  characters: names.map((name) => ({ name, class: "bard", ilvl: 1700 })),
});
const names = () =>
  useRosterStore().roster_config.profiles.map((p) => p.char_name);
const flush = () => new Promise((r) => setTimeout(r, 0));

let store: ReturnType<typeof useRosterStore>;
beforeEach(() => {
  localStorage.clear();
  setActivePinia(createPinia());
  store = useRosterStore();
  store.init();
  vi.mocked(fetch_char).mockResolvedValue(result([0, 0, 0, 0, 0, 0]));
});

describe("sync", () => {
  it("replaces the fresh default character with the linked roster", async () => {
    ROSTERS.value = [roster("Ëlsa", "Bob")];
    await sync_rosters();
    expect(names()).toEqual(["Ëlsa", "Bob"]);
    expect(store.roster_config.profiles.every((p) => p.fetched_at > 0)).toBe(
      true,
    );
  });

  it("matches accentless names, renames them, and asks about the rest", async () => {
    store.active_profile.char_name = "Elsa";
    store.active_profile.fetched_at = 1;
    store.add_char(store.active_profile.roster_id).char_name = "Typo";
    ROSTERS.value = [roster("Ëlsa", "Bob", "Cat")];
    const done = sync_rosters();
    await flush();
    expect(names()).toEqual(["Ëlsa", "Typo"]);
    expect(sync_pending.value[0].bibles).toEqual(["Bob", "Cat"]);
    sync_pending.value[0].locals[0].choice = "Bob";
    close_sync_popup(true);
    await done;
    expect(names()).toEqual(["Ëlsa", "Bob", "Cat"]);
  });

  it("prefers exact names when a roster has both accented and plain", async () => {
    store.active_profile.char_name = "Inngrimschblade";
    store.active_profile.fetched_at = 1;
    ROSTERS.value = [roster("Ínngrimschblade", "Inngrimschblade")];
    await sync_rosters();
    expect(names()).toEqual(["Inngrimschblade", "Ínngrimschblade"]);
  });
});

describe("linked lookup", () => {
  it("prefers the exact name over an accentless match", () => {
    bible_auth.value = { expires_at: Date.now() + 1e6 } as any;
    bible_rosters.value = [roster("Inngrimschblade", "Ínngrimschblade")] as any;
    expect(linked_now("NA", "Ínngrimschblade")?.character.name).toBe(
      "Ínngrimschblade",
    );
    expect(linked_now("NA", "Inngrimschblade")?.character.name).toBe(
      "Inngrimschblade",
    );
    expect(linked_now("NA", "inngrimschblade")?.character.name).toBe(
      "Inngrimschblade",
    );
    bible_auth.value = null;
  });
});

describe("refresh tier", () => {
  it("switches to Serca only once all six pieces are Serca", async () => {
    const id = store.active_profile.id;
    store.active_profile.char_name = "Alpha";
    vi.mocked(fetch_char).mockResolvedValue(result([1, 1, 1, 1, 1, 0]));
    await refresh_char(id);
    expect(store.active_profile.tier).toBe(0);
    vi.mocked(fetch_char).mockResolvedValue(result([1, 1, 1, 1, 1, 1]));
    await refresh_char(id);
    expect(store.active_profile.tier).toBe(1);
  });
});

describe("auto-refresh", () => {
  it("skips characters whose fetched upgrades are unlocked", async () => {
    ROSTERS.value = [roster("Alpha", "Beta")];
    const [a] = store.roster_config.profiles;
    store.roster_config.all_regions[a.roster_id] = "nae";
    a.char_name = "Alpha";
    a.fetched_at = 1;
    store.add_char(a.roster_id);
    const b = store.roster_config.profiles[1];
    b.char_name = "Beta";
    b.fetched_at = 1;
    b.lock_fetched_done = false;
    vi.mocked(fetch_char).mockClear();
    await auto_refresh();
    expect(vi.mocked(fetch_char).mock.calls.map((c) => c[1])).toEqual([
      "Alpha",
    ]);
  });
});
