import {
  afterEach,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
  vi,
} from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { h, onUnmounted } from "vue";
import { RouterView, type Router } from "vue-router";
import { useRosterStore } from "@/Stores/RosterConfig";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { scanner_shown } from "@/Components/Character/InventoryScanner/ScannerUIutils";
import { load_router, set_profiles, stub_match_media } from "./helpers";

let router: Router;
const calc_log: string[] = [];
// the router is shared, so an old app left mounted would keep reacting to it
const mounted: { unmount: () => void }[] = [];
afterEach(() => mounted.splice(0).forEach((w) => w.unmount()));

beforeAll(async () => {
  for (const [name, path] of [
    ["ControlPanel", "@/Components/Character/ControlPanel.vue"],
    [
      "ScannerControlPanel",
      "@/Components/Character/InventoryScanner/ScannerControlPanel.vue",
    ],
  ])
    vi.doMock(path, () => ({
      default: { name, render: () => h("div", { class: name }) },
    }));
  // records which character each calculator page was set up for
  vi.doMock("@/Components/Character/Calc.vue", () => ({
    default: {
      name: "Calc",
      setup() {
        const id = useRosterStore().active_profile.id;
        calc_log.push(`setup ${id}`);
        onUnmounted(() => calc_log.push(`unmount ${id}`));
        return () => h("div");
      },
    },
  }));
  ({ router } = await load_router(["CharView", "Calc"]));
});

async function mount_at(path: string) {
  const pinia = createPinia();
  setActivePinia(pinia);
  set_profiles([
    ["Alice", 0],
    ["Bob", 0],
    ["Carol", 1],
  ]);
  await router.push("/roster-setup");
  const wrapper = mount(
    { render: () => h(RouterView) },
    { global: { plugins: [pinia, router] } },
  );
  mounted.push(wrapper);
  await router.push(path);
  await flushPromises();
  return { wrapper, store: useRosterStore(), runtime: useRuntimeStore() };
}

const at = () => router.currentRoute.value.fullPath;

beforeEach(() => stub_match_media(false));

describe("CharView", () => {
  it("makes the routed character active", async () => {
    const { wrapper, store } = await mount_at("/chars/Carol/calc");
    expect(store.roster_config.active_profile_index).toBe(2);
    expect(wrapper.find(".side-bar-header").text()).toBe("Carol");
  });

  it("lists the active character's roster beside the sidebar", async () => {
    const { wrapper } = await mount_at("/chars/Bob/calc");
    const chars = wrapper.findAll('nav[aria-label="Characters"] a');
    expect(chars.map((a) => a.text())).toEqual(["Alice", "Bob"]);
    expect(wrapper.findAll("a.selected").map((a) => a.text())).toEqual(["Bob"]);
  });

  it("draws a character's class icon, and a helmet for an unknown class", async () => {
    const { wrapper, store } = await mount_at("/chars/Bob/calc");
    store.all_profiles[0].class_name = "Guardianknight";
    await flushPromises();
    const icons = wrapper.findAll('nav[aria-label="Characters"] a span[style]');
    expect(icons.map((i) => i.attributes("style"))).toEqual([
      expect.stringContaining("guardianknight.svg"),
      expect.stringContaining("unknown.png"),
    ]);
  });

  it("switches and cancels the old runs when the name changes", async () => {
    const { store, runtime } = await mount_at("/chars/Alice/calc");
    const cancel = vi.spyOn(runtime, "cancel_active");
    await router.push("/chars/Bob/choices");
    await flushPromises();
    expect(store.active_profile.char_name).toBe("Bob");
    expect(cancel).toHaveBeenCalled();
  });

  it("an unknown name reached by a param change goes to the first character, same sub-page", async () => {
    const { store } = await mount_at("/chars/Bob/choices");
    await router.push("/chars/Nobody/choices");
    await flushPromises();
    expect(at()).toBe("/chars/Alice/choices");
    expect(store.roster_config.active_profile_index).toBe(0);
  });

  it("leaving for another page stays there and cancels runs", async () => {
    const { runtime } = await mount_at("/chars/Bob/calc");
    const cancel = vi.spyOn(runtime, "cancel_active");
    await router.push("/market-mats");
    await flushPromises();
    expect(at()).toBe("/market-mats");
    expect(cancel).toHaveBeenCalled();
  });

  it("sidebar links are relative to the character", async () => {
    const { wrapper } = await mount_at("/chars/Bob/calc");
    const hrefs = wrapper
      .findAll("a.side-bar-link")
      .map((a) => a.attributes("href"));
    expect(hrefs).toEqual([
      "/chars/Bob/calc",
      "/chars/Bob/income",
      "/chars/Bob/choices",
      "/chars/Bob/scanner",
    ]);
    await wrapper.findAll("a.side-bar-link")[2].trigger("click");
    await flushPromises();
    expect(at()).toBe("/chars/Bob/choices");
  });

  it("control panels follow the sub-page", async () => {
    const { wrapper } = await mount_at("/chars/Bob/calc");
    expect(wrapper.find(".ControlPanel").exists()).toBe(true);
    expect(wrapper.find(".ScannerControlPanel").exists()).toBe(false);
    // jsdom has no screen capture, so the scanner would be hidden as on mobile
    scanner_shown.value = true;
    await router.push("/chars/Bob/scanner");
    await flushPromises();
    expect(wrapper.find(".ControlPanel").exists()).toBe(false);
    expect(wrapper.find(".ScannerControlPanel").exists()).toBe(true);
  });

  it("a character switch remounts the calculator; a rename does not", async () => {
    const { store } = await mount_at("/chars/Alice/calc");
    const [alice, bob] = store.roster_config.profiles.map((p) => p.id);
    calc_log.length = 0;
    await router.push("/chars/Bob/calc");
    await flushPromises();
    // the old page's unmount runs after the new one has started its workers
    expect(calc_log).toEqual([`setup ${bob}`, `unmount ${alice}`]);
    calc_log.length = 0;
    store.active_profile.char_name = "Bobby";
    await flushPromises();
    expect(calc_log).toEqual([]);
  });

  it("the route follows a rename of the active character", async () => {
    const { store } = await mount_at("/chars/Bob/choices");
    store.active_profile.char_name = "Bobby";
    await flushPromises();
    expect(at()).toBe("/chars/Bobby/choices");
    expect(store.active_profile.char_name).toBe("Bobby");
  });
});
