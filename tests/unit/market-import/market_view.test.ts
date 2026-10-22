import { beforeEach, describe, expect, it, vi } from "vitest";
import { h } from "vue";
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import MarketView from "@/Components/Market/MarketView.vue";
import {
  create_default_owned_input_column,
  useRosterStore,
} from "@/Stores/RosterConfig";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { input_column_to_num } from "@/Utils/InputColumn";
import { start_fetch } from "@/Utils/MarketDataFetcher";

vi.mock("@/Utils/MarketDataFetcher", async (orig) => ({
  ...(await orig<typeof import("@/Utils/MarketDataFetcher")>()),
  start_fetch: vi.fn(),
}));

// renders both slots in place of the responsive sidebar, whose footer would need a router
vi.mock("@/Components/Common/Sidebar.vue", () => ({
  default: {
    setup:
      (_: any, { slots }: any) =>
      () =>
        h("div", [slots.sidebar?.(), slots.main?.()]),
  },
}));

function mount_view() {
  const pinia = createPinia();
  setActivePinia(pinia);
  return mount(MarketView, {
    global: { plugins: [pinia] },
  });
}

const row = (w: any, tier: string, label: string) =>
  w.find(`[aria-label="${tier}"]`).find(`[aria-label="${label}"]`);

beforeEach(() => vi.mocked(start_fetch).mockClear());

describe("convert button", () => {
  it("converts the selected roster's stock", async () => {
    const w = mount_view();
    const cfg = useRosterStore().roster_config;
    cfg.roster_mats_owned[cfg.profiles[0].roster_id].values.Red = "1,000";
    cfg.tradable_mats_owned[cfg.profiles[0].roster_id].values.Red = "502";
    await w.find("button.tier-button").trigger("click");
    const r = input_column_to_num(cfg.roster_mats_owned[cfg.profiles[0].roster_id]);
    const t = input_column_to_num(cfg.tradable_mats_owned[cfg.profiles[0].roster_id]);
    expect([r["Serca Red"], r.Red, t.Red]).toEqual([300, 0, 2]);
  });
});

describe("market sidebar", () => {
  it("changing region stores it and fetches it", async () => {
    const w = mount_view();
    await w.find("select.selector").setValue("euc");
    expect(useRosterStore().roster_config.all_regions[
      useRosterStore().roster_ids[0]
    ]).toBe("euc");
    expect(start_fetch).toHaveBeenCalledWith("euc");
  });

  it("fetch button forces a fetch; custom disables it and hides auto fetch", async () => {
    const w = mount_view();
    const btn = () =>
      w.findAll("button").find((b) => b.text().includes("Fetch Market Data"));
    await btn().trigger("click");
    expect(start_fetch).toHaveBeenCalledWith("nae", true);
    expect(w.text()).toContain("Auto fetch");
    useRosterStore().roster_config.all_regions[
      useRosterStore().roster_ids[0]
    ] = "Custom";
    await flushPromises();
    expect(btn().attributes("disabled")).toBeDefined();
    expect(w.text()).not.toContain("Auto fetch");
  });

  it("auto fetch toggles the flag and fetches", async () => {
    const w = mount_view();
    await w.find('input[type="checkbox"]').setValue(false);
    expect(useRosterStore().roster_config.auto_fetch).toBe(false);
    expect(start_fetch).toHaveBeenCalledWith("nae");
  });

  it("shows fetch status", async () => {
    const w = mount_view();
    const runtime = useRuntimeStore();
    expect(w.text()).toContain("✅");
    runtime.market_fetch_failed.nae = true;
    await flushPromises();
    expect(w.text()).toContain("Failed");
    runtime.market_fetch_failed.euc = false;
    await flushPromises();
    expect(w.text()).toContain("Failed"); // only the shown region counts
    runtime.is_fetching.nae = true;
    await flushPromises();
    expect(w.text()).toContain("Fetching...");
  });

  it("roster picker switches the shown roster", async () => {
    const pinia = createPinia();
    setActivePinia(pinia);
    const store = useRosterStore();
    const second = {
      ...store.roster_config.profiles[0],
      id: "b",
      roster_id: "3",
    };
    store.roster_config.profiles.push(second);
    store.roster_config.roster_mats_owned[3] =
      create_default_owned_input_column();
    store.roster_config.tradable_mats_owned[3] =
      create_default_owned_input_column();
    store.roster_config.all_regions[3] = "euc";
    store.roster_config.roster_mats_owned[3].values.Red = "42";
    const w = mount(MarketView, {
      global: { plugins: [pinia] },
    });
    await w.findAll("select.selector")[1].setValue("3");
    const input = row(w, "T4 materials", "Red").find(
      'input[aria-label="Roster bound owned"]',
    );
    expect((input.element as HTMLInputElement).value).toBe("42");
  });
});

describe("budget grid", () => {
  it("edits the store through the roster cell", async () => {
    const w = mount_view();
    const input = row(w, "T4 materials", "Red").find(
      'input[aria-label="Roster bound owned"]',
    );
    await input.setValue("100+20");
    expect(useRosterStore().roster_config.roster_mats_owned[
      useRosterStore().roster_ids[0]
    ].values.Red).toBe(
      (120).toLocaleString(),
    );
  });

  it("shard price follows the chosen bag", async () => {
    const w = mount_view();
    const cfg = useRosterStore().roster_config;
    cfg.mats_prices.nae.values["Shards x1000"] = "11";
    cfg.mats_prices.nae.values["Shards x3000"] = "33";
    await flushPromises();
    const shards = row(w, "T4 materials", "Shards");
    const price = () =>
      (shards.find('input[aria-label="Market price"]').element as any).value;
    expect(price()).toBe("33");
    await shards.find('select[aria-label="Shards bundle size"]').setValue(1000);
    expect(cfg.selected_bundles.nae.Shards).toBe(1000);
    expect(price()).toBe("11");
    await shards.find('input[aria-label="Market price"]').setValue("12");
    expect(cfg.mats_prices.nae.values["Shards x1000"]).toBe("12");
    expect(cfg.mats_prices.nae.values["Shards x3000"]).toBe("33");
  });

  it("effective Serca price takes the cheaper of buying and converting", async () => {
    const w = mount_view();
    const cfg = useRosterStore().roster_config;
    cfg.mats_prices.nae.values["Red x100"] = "100";
    cfg.mats_prices.nae.values["Serca Red x100"] = "1,000";
    cfg.mats_prices.nae.values["Blue x100"] = "100";
    cfg.mats_prices.nae.values["Serca Blue x100"] = "400";
    await flushPromises();
    const red = row(w, "Serca materials", "Serca Red").text();
    expect(red).toContain("5"); // 1 per unit * 5
    expect(red).toContain("Convert T4");
    const blue = row(w, "Serca materials", "Serca Blue").text();
    expect(blue).toContain("Buy Serca");
    // shared labels show no effective price
    expect(row(w, "Serca materials", "Gold").text()).not.toContain("Buy");
  });
});
