import { vi } from "vitest";
import { new_char_profile } from "@/Stores/CharacterProfile";
import {
  create_default_owned_input_column,
  useRosterStore,
} from "@/Stores/RosterConfig";

// jsdom has no matchMedia; every query reports `narrow`.
export function stub_match_media(narrow = false) {
  window.matchMedia = vi.fn().mockImplementation((query: string) => ({
    matches: narrow,
    media: query,
    addEventListener() {},
    removeEventListener() {},
  })) as any;
}

// Replaces the store's characters with `names`, each [name, roster_id].
export function set_profiles(entries: [string, number][]) {
  const store = useRosterStore();
  const config = store.roster_config;
  config.profiles = entries.map(([name, roster_id]) => {
    const p = new_char_profile();
    p.char_name = name;
    p.roster_id = roster_id;
    return p;
  });
  for (const [, roster_id] of entries) {
    config.roster_mats_owned[roster_id] ??= create_default_owned_input_column();
    config.tradable_mats_owned[roster_id] ??=
      create_default_owned_input_column();
    config.all_regions[roster_id] ??= "nae";
  }
  config.active_profile_index = 0;
  return store;
}

export const names = () =>
  useRosterStore().all_profiles.map((p) => p.char_name);

// The real router module with its pages replaced, except those named in `keep`.
export async function load_router(keep: string[] = []) {
  const empty = (name: string) => ({
    default: { name, render: () => null },
  });
  const pages: Record<string, string> = {
    CharView: "@/Components/Character/CharView.vue",
    Calc: "@/Components/Character/Calc.vue",
    Guide: "@/Components/Character/Guide.vue",
    Scanner: "@/Components/Character/InventoryScanner/Scanner.vue",
    MarketView: "@/Components/Market/MarketView.vue",
    RosterView: "@/Components/RosterView.vue",
    ChangeLogsView: "@/Components/ChangeLogsView.vue",
    Income: "@/Components/Character/Income/Income.vue",
    Choices: "@/Components/Character/Choices/Choices.vue",
  };
  for (const [name, path] of Object.entries(pages))
    if (!keep.includes(name)) vi.doMock(path, () => empty(name));
  vi.doMock("vue-router", async () => {
    const actual =
      await vi.importActual<typeof import("vue-router")>("vue-router");
    return { ...actual, createWebHistory: () => actual.createMemoryHistory() };
  });
  const guard_switch = vi.fn();
  vi.doMock("@/Components/Character/InventoryScanner/ScanSwitch", () => ({
    guard_switch,
  }));
  const router = (await import("@/Router")).default;
  return { router, guard_switch };
}
