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
const used = () =>
  snapshots().filter((s) => s.kind !== "Before export" && !s.ignored);
const job = (snapshot: Snapshot, is_now = false) =>
  job_for(store.active_profile, snapshot, is_now);
const markers = () =>
  wrapper.findAll('[aria-label="Snapshot timeline"] button');
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

  it("draws every snapshot on the timeline", () => {
    expect(markers()).toHaveLength(12);
    const lower = markers().filter((m) =>
      m.attributes("style").includes("top: 1.5rem"),
    );
    expect(lower).toHaveLength(5);
    expect(markers().filter((m) => m.classes("opacity-30"))).toHaveLength(1);
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
    await band("+ Roster bound");
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

  it("the material graph shows one material at a time", async () => {
    expect(
      graphs()[0]
        .props("series")
        .map((s) => s.label),
    ).toEqual(["Gold"]);
    await wrapper.find('[aria-label="Serca Red"]').trigger("click");
    expect(
      graphs()[0]
        .props("series")
        .map((s) => s.label),
    ).toEqual(["Serca Red"]);
    const pressed = wrapper
      .findAll('[aria-label="Materials"] button')
      .filter((b) => b.attributes("aria-pressed") === "true");
    expect(pressed.map((b) => b.attributes("aria-label"))).toEqual([
      "Serca Red",
    ]);
  });

  // one test: now's and the weeks' results are kept for the session, across tests
  it("runs the chain in order, each from the one before, then the metrics graph", async () => {
    const metrics = () => graphs()[1].props("series");
    expect(graphs()[1].props("shared")).toBe(true);
    expect(metrics().map((s) => [s.points, s.line])).toEqual([
      [[], []],
      [[], []],
    ]);

    // the ignored snapshot is left out, then now and the five weeks to the range's end
    const runs = [];
    while (queued().length) {
      const [next] = queued();
      if (runs.length)
        expect(next.payload.chest_states).toBe(runs.at(-1).state.chest_states);
      const state = fake_state(runs.length);
      next.store(fake_out(1000 + runs.length, 100), state);
      runs.push({ next, state });
      await nextTick();
    }
    expect(runs).toHaveLength(11 + 1 + 5);
    expect(runs[0].next.hash).toBe(job(snapshots()[0]).hash);

    // a week of Serca Blue's char bound income on each projected budget
    const blue = (at: number) =>
      runs[at].next.payload.materials["Serca Blue"].bound;
    expect(blue(12) - blue(11)).toBeCloseTo(50_000, -1);
    expect(blue(16) - blue(11)).toBeCloseTo(5 * 50_000, -1);

    const gold_of = (s: Snapshot) =>
      1000 + runs.findIndex((run) => run.next.hash === job(s).hash);
    const [gold, leftover] = metrics();
    expect(gold.points.map((p) => p.x)).toEqual([
      ...used().map((s) => s.at),
      NOW,
    ]);
    expect(gold.points.map((p) => p.y)).toEqual([...used().map(gold_of), 1011]);
    expect(leftover.points.at(-1).y).toBe(911);
    expect(gold.line).toEqual(
      [0, 1, 2, 3, 4, 5].map((k) => ({ x: NOW + k * WEEK_MS, y: 1011 + k })),
    );
  });
});
