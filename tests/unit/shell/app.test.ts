import { beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises } from "@vue/test-utils";
import { nextTick } from "vue";
import { new_char_profile } from "@/Stores/CharacterProfile";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { start_fetch } from "@/Utils/MarketDataFetcher";
import { debounced_write_roster_config } from "@/Stores/ConfigStorage";
import App from "@/App.vue";
import { mount_app } from "../mount";
import { set_profiles, stub_match_media } from "./helpers";

vi.mock("@/Utils/MarketDataFetcher", async (orig) => ({
  ...(await orig<any>()),
  start_fetch: vi.fn(),
}));
vi.mock("@/Stores/ConfigStorage", async (orig) => ({
  ...(await orig<any>()),
  debounced_write_roster_config: vi.fn(),
}));
vi.mock("@/Components/Character/InventoryScanner/ScanSwitchDialog.vue", () => ({
  default: { render: () => null },
}));

async function mount_shell(path = "/chars/NA/Bob/calc") {
  const out = await mount_app(App, {}, path);
  const store = set_profiles([
    ["Alice", 0],
    ["Bob", 0],
    ["Carol", 1],
  ]);
  await nextTick();
  return { ...out, store, runtime: useRuntimeStore() };
}

const runtime_ids = () => Object.keys(useRuntimeStore().per_char).sort();
const profile_ids = (store) => store.all_profiles.map((p) => p.id).sort();

beforeEach(() => {
  stub_match_media(false);
  vi.mocked(start_fetch).mockClear();
  vi.mocked(debounced_write_roster_config).mockClear();
});

describe("App", () => {
  it("starts a market fetch per fetchable region on load", async () => {
    await mount_shell();
    expect(vi.mocked(start_fetch).mock.calls).toEqual([["nae"], ["euc"]]);
  });

  it("writes the config on any store mutation", async () => {
    const { store } = await mount_shell();
    vi.mocked(debounced_write_roster_config).mockClear();
    store.roster_config.show_all_rows = true;
    await flushPromises();
    expect(debounced_write_roster_config).toHaveBeenCalled();
  });

  it("keeps runtime bundles in step with the profile ids", async () => {
    const { store, runtime } = await mount_shell();
    expect(runtime_ids()).toEqual(profile_ids(store));

    const bob = store.all_profiles[1];
    const bundle = runtime.per_char[bob.id];
    const cancel = vi.spyOn(bundle.optimizer, "cancel");
    store.add_profile(new_char_profile());
    store.roster_config.profiles.splice(1, 1);
    await nextTick();
    expect(runtime_ids()).toEqual(profile_ids(store));
    expect(runtime.per_char[bob.id]).toBeUndefined();
    expect(cancel).toHaveBeenCalled();
  });

  it("a rename keeps the character's bundle", async () => {
    const { store, runtime } = await mount_shell();
    const id = store.all_profiles[0].id;
    const bundle = runtime.per_char[id];
    store.all_profiles[0].char_name = "Renamed";
    await nextTick();
    expect(runtime.per_char[id]).toBe(bundle);
  });

  it("has a button per roster, marking the routed character's", async () => {
    const { wrapper } = await mount_shell("/chars/NA/Bob/calc");
    const rosters = wrapper.findAll("header a.nav-tab").slice(3);
    expect(rosters.map((a) => a.text())).toEqual(["Roster 1", "Roster 2"]);
    expect(rosters[0].classes()).toContain("selected");
    expect(rosters[1].classes()).not.toContain("selected");
  });

  it("one roster is a Roster 1 button", async () => {
    const { wrapper } = await mount_app(App, {}, "/chars/NA/Alice/calc");
    set_profiles([
      ["Alice", 0],
      ["Bob", 0],
    ]);
    await nextTick();
    expect(wrapper.findAll("header a.nav-tab").at(-1).text()).toBe("Roster 1");
  });

  it("a roster button goes back to the roster's last opened character", async () => {
    const { wrapper, router } = await mount_shell("/chars/NA/Bob/calc");
    await router.push("/chars/NA/Carol/calc");
    await flushPromises();
    const roster_1 = wrapper.findAll("header a.nav-tab")[3];
    expect(roster_1.attributes("href")).toBe("/chars/NA/Bob");
    await roster_1.trigger("click");
    await flushPromises();
    expect(router.currentRoute.value.params.characterName).toBe("Bob");
  });

  it("marks the roster and market buttons by path", async () => {
    const { wrapper, router } = await mount_shell("/roster-setup");
    expect(wrapper.find(".nav-tab.selected").text()).toBe("Roster setup");
    await router.push("/market-mats");
    await flushPromises();
    expect(wrapper.find(".nav-tab.selected").text()).toBe("Market & Mats");
  });
});
