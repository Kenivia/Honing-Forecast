import fs from "node:fs";
import { nextTick } from "vue";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { mount, type VueWrapper } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { import_config } from "@/Stores/ConfigStorage";
import type { Snapshot } from "@/Utils/Scans";
import { WEEK_MS } from "@/Utils/Income";
import Income from "@/Components/Character/Income/Income.vue";
import IncomeGraph from "@/Components/Character/Income/IncomeGraph.vue";
import { job_for } from "@/Components/Character/Income/IncomeJobs";
import { queue_jobs } from "@/WasmInterface/OptimizerJobs";
import { set_cell } from "@/Utils/InputColumn";
import { fake_out, fake_state } from "./helpers";

// The Income page over tests/income-fixture.json (scripts/mock income): the real save with five
// weeks of snapshots, rising at fixed rates, a spend with a new line and an ignored outlier.
vi.mock("@/WasmInterface/OptimizerJobs", async (original) => ({
  ...(await original<object>()),
  queue_jobs: vi.fn(),
  stop_jobs: vi.fn(),
}));

const FIXTURE = fs.readFileSync("tests/income-fixture.json", "utf8");
const NOW = Date.UTC(2026, 9, 21, 12);
let store: ReturnType<typeof useRosterStore>;
let wrapper: VueWrapper;

beforeEach(() => {
  vi.useFakeTimers();
  vi.setSystemTime(NOW);
  vi.mocked(queue_jobs).mockClear();
  setActivePinia(createPinia());
  store = useRosterStore();
  store.replace_config(import_config(FIXTURE));
  wrapper = mount(Income, {
    global: { stubs: { IncomeGraph: true } },
  });
});
afterEach(() => {
  wrapper.unmount();
  vi.useRealTimers();
});

const snapshots = () => store.roster_config.snapshots;
const job = (snapshot: Snapshot, is_now = false) =>
  job_for(store.active_profile, snapshot, is_now);
const markers = () => wrapper.findAll('[aria-label="Snapshots"] button');
const graphs = () => wrapper.findAllComponents(IncomeGraph);
const queued = () => vi.mocked(queue_jobs).mock.lastCall[0];
const rate = (label: string) =>
  Number(
    wrapper.find(`[aria-label="${label} income"]`).text().replace(/,/g, ""),
  );
async function band(name: string) {
  await wrapper
    .findAll('[aria-label="Bands"] button')
    .find((b) => b.text() === name)
    .trigger("click");
}

describe("Income page over the fixture", () => {
  it("imports Toneema's snapshots under its id", () => {
    const profile = store.active_profile;
    expect(profile.char_name).toBe("Toneema");
    expect(snapshots()).toHaveLength(12);
    expect(snapshots().every((s) => s.chars[profile.id])).toBe(true);
  });

  it("lists every snapshot, the ignored one faded, a bar above the break", () => {
    expect(markers()).toHaveLength(12);
    expect(markers().filter((m) => m.classes("opacity-30"))).toHaveLength(1);
    const bars = markers().filter((m) => m.classes("border-t"));
    expect(bars.map((m) => m.attributes("aria-label"))).toEqual([
      `${new Date(snapshots().find((s) => s.breaks).at).toLocaleString()} After export`,
    ]);
  });

  it("queues only the oldest snapshot first", () => {
    expect(queued().map((j) => j.hash)).toEqual([job(snapshots()[0]).hash]);
  });

  it("estimates the fixture's rates per band, across the spend", async () => {
    expect(rate("Gold")).toBeCloseTo(40_000, -1);
    expect(rate("Serca Fusion")).toBeCloseTo(100, 0);
    await band("Char bound");
    expect(rate("Gold")).toBeCloseTo(60_000, -1);
    expect(rate("Serca Blue")).toBeCloseTo(50_000, -1);
    expect(rate("Free taps")).toBeCloseTo(20, 0);
    await band("Roster bound");
    expect(rate("Silver")).toBeCloseTo(2_000_000, -2);
  });

  it("without the break or with the outlier the estimate is off", async () => {
    await band("Char bound");
    const spend = snapshots().find((s) => s.breaks);
    spend.breaks = false;
    await wrapper.vm.$nextTick();
    expect(rate("Gold")).toBeLessThan(0);
    spend.breaks = true;
    snapshots().find((s) => s.ignored).ignored = false;
    await wrapper.vm.$nextTick();
    expect(rate("Gold")).toBeGreaterThan(100_000);
  });

  it("weeks ahead can be typed, 5 to 52, empty for the default", async () => {
    const input = wrapper.find('[aria-label="Weeks ahead"]');
    const end = () => (graphs()[0].props("range")[1] - NOW) / WEEK_MS;
    expect(input.attributes("placeholder")).toBe("auto (5)");
    for (const [typed, weeks] of [
      ["8", 8],
      ["2", 5],
      ["99", 52],
      ["", 5],
    ] as const) {
      await input.setValue(typed);
      expect(end()).toBe(weeks);
    }
  });

  it("runs the chain in order, each from the one before, on to the crossing", async () => {
    expect(graphs()).toHaveLength(1);
    const series = () => graphs()[0].props("series");
    expect(series().map((s) => s.points)).toEqual([[], [], []]);

    // every run a cost of 1M and leftovers worth 300k (Rust's metrics are negative); with
    // about 238k tradable gold now and 40k a week, leftovers and gold cover it in week 12,
    // and three weeks past it are shown
    const runs = [];
    while (queued().length) {
      const [next] = queued();
      if (runs.length)
        expect(next.payload.chest_states).toBe(runs.at(-1).state.chest_states);
      const state = fake_state(runs.length);
      next.store(fake_out(-1_000_000, -700_000), state);
      runs.push({ next, state });
      await nextTick();
    }
    // the ignored snapshot is left out, then now and the weeks to the crossing
    expect(runs).toHaveLength(11 + 1 + 15);
    expect(runs[0].next.hash).toBe(job(snapshots()[0]).hash);
    expect(graphs()[0].props("range")).toEqual([NOW, NOW + 15 * WEEK_MS]);

    // a week of Serca Blue's char bound income on each projected budget
    const blue = (at: number) =>
      runs[at].next.payload.materials["Serca Blue"].bound;
    expect(blue(12) - blue(11)).toBeCloseTo(50_000, -1);
    expect(blue(16) - blue(11)).toBeCloseTo(5 * 50_000, -1);

    const [cost, leftover, covered] = series();
    expect(series().map((s) => [s.label, s.color, !!s.dashed])).toEqual([
      ["Avg tradable gold spent", "--gold", false],
      ["Avg value of leftovers", "--text-muted", false],
      ["Leftovers + tradable gold", "--gold", true],
    ]);
    const weeks = Array.from({ length: 16 }, (_, k) => NOW + k * WEEK_MS);
    expect(cost.points.map((p) => p.x)).toEqual(weeks);
    expect(new Set(cost.points.map((p) => p.y))).toEqual(new Set([1_000_000]));
    expect(new Set(leftover.points.map((p) => p.y))).toEqual(
      new Set([300_000]),
    );
    const ahead = covered.points.map((p) => p.y);
    expect(ahead[1] - ahead[0]).toBeCloseTo(40_000, -1);
    expect(ahead[11]).toBeLessThan(1_000_000);
    expect(ahead[12]).toBeGreaterThanOrEqual(1_000_000);
  });

  it("finds every run again after the clock moves and the page is left, until the budgets change", async () => {
    const pinned = store.active_profile.income_now;
    expect(pinned.at).toBe(NOW);
    let runs = 0;
    while (queued().length) {
      queued()[0].store(fake_out(-1_000_000, -700_000), fake_state(runs++));
      await nextTick();
    }
    const left = () => wrapper.text().match(/Optimizing \d+ runs?/);

    vi.advanceTimersByTime(2 * 60_000);
    await nextTick();
    wrapper.unmount();
    wrapper = mount(Income, { global: { stubs: { IncomeGraph: true } } });
    await nextTick();
    expect(left()).toBeNull();
    expect(store.active_profile.income_now).toEqual(pinned);
    // drawn from the time it is, all the same
    expect(graphs()[0].props("series")[0].points[0].x).toBe(NOW + 2 * 60_000);

    set_cell(store.active_profile.bound_budgets, "Serca Red", "1");
    await nextTick();
    expect(store.active_profile.income_now.at).toBe(NOW + 2 * 60_000);
    expect(left()).not.toBeNull();
  });
});
