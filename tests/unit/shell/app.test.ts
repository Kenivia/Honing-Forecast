import { beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises } from "@vue/test-utils";
import { nextTick } from "vue";
import { new_char_profile } from "@/Stores/CharacterProfile";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { start_fetch } from "@/Utils/MarketDataFetcher";
import { debounced_write_roster_config } from "@/Stores/ConfigStorage";
import App from "@/App.vue";
import { mount_app } from "../mount";
import { names, set_profiles, stub_match_media } from "./helpers";

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

async function mount_shell(path = "/Bob/calc") {
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

  it("header lists characters per roster and marks the routed one", async () => {
    const { wrapper } = await mount_shell("/Bob/calc");
    const groups = wrapper
      .findAll("nav .flex.flex-row")
      .filter((g) => g.find("a.header-button").exists());
    expect(groups.map((g) => g.findAll("a").map((a) => a.text()))).toEqual([
      ["Alice", "Bob"],
      ["Carol"],
    ]);
    const selected = wrapper.findAll("a.header-button.selected");
    expect(selected.map((a) => a.text())).toEqual(["Bob"]);
  });

  it("marks the roster and market buttons by path", async () => {
    const { wrapper, router } = await mount_shell("/roster-setup");
    expect(wrapper.find(".header-button.selected").text()).toBe("Roster setup");
    await router.push("/market-mats");
    await flushPromises();
    expect(wrapper.find(".header-button.selected").text()).toBe(
      "Market & Mats",
    );
  });

  it("narrow screens with several characters get a dropdown that navigates", async () => {
    stub_match_media(true);
    const { wrapper, router } = await mount_shell("/Alice/calc");
    const select = wrapper.find("select");
    expect(select.findAll("option").map((o) => o.text())).toEqual([
      "Character",
      ...names(),
    ]);
    await select.setValue("Carol");
    await flushPromises();
    expect(router.currentRoute.value.params.characterName).toBe("Carol");
  });
});
