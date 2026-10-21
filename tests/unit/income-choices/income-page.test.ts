import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick } from "vue";
import { mount, type VueWrapper } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { set_cell } from "@/Utils/InputColumn";
import { WEEK_MS } from "@/Utils/Income";
import { snapshot_of, type Snapshot } from "@/Utils/Scans";
import Income from "@/Components/Character/Income/Income.vue";
import { job_for } from "@/Components/Character/Income/IncomeJobs";
import { queue_jobs } from "@/WasmInterface/OptimizerJobs";
import { fake_out, fake_state, want_upgrade } from "./helpers";

// The Income page: snapshot list, queue order, the table and manual income. Runs are not made.
vi.mock("@/WasmInterface/OptimizerJobs", async (original) => ({
  ...(await original<object>()),
  queue_jobs: vi.fn(),
  stop_jobs: vi.fn(),
}));

const NOW = Date.UTC(2026, 9, 20, 12);
let store: ReturnType<typeof useRosterStore>;
let wrapper: VueWrapper;

beforeEach(() => {
  vi.useFakeTimers();
  vi.setSystemTime(NOW);
  vi.mocked(queue_jobs).mockClear();
  setActivePinia(createPinia());
  store = useRosterStore();
  want_upgrade(store.active_profile);
});
afterEach(() => {
  wrapper?.unmount();
  vi.useRealTimers();
});

// a snapshot of the store with the character's char-bound Red at `red`
function add(at: number, red: number, extra: Partial<Snapshot> = {}): Snapshot {
  const profile = store.active_profile;
  set_cell(profile.bound_budgets, "Red", String(red));
  const snapshot = { ...snapshot_of(store.roster_config, 0, "Manual"), at };
  Object.assign(snapshot, extra);
  store.roster_config.snapshots.push(snapshot);
  return store.roster_config.snapshots.at(-1);
}
const open = () => {
  wrapper = mount(Income, {
    global: { plugins: [], stubs: { IncomeGraph: true } },
  });
  return wrapper;
};
const markers = () =>
  wrapper.findAll('[aria-label="Snapshot timeline"] button');
const text_of = (label: string) =>
  wrapper.find(`[aria-label="${label}"]`).text();
const last_queue = () =>
  vi.mocked(queue_jobs).mock.lastCall[0].map((job) => job.hash);
const hash_of = (snapshot: Snapshot) =>
  job_for(store.active_profile, snapshot, false).hash;
async function band(name: string) {
  const button = wrapper
    .findAll('[aria-label="Bands"] button')
    .find((b) => b.text() === name);
  await button.trigger("click");
}

describe("Income page", () => {
  it("lists only the roster's snapshots", () => {
    add(NOW - WEEK_MS, 1);
    add(NOW - 2 * WEEK_MS, 2, { roster_id: 1 });
    open();
    expect(markers()).toHaveLength(1);
  });

  it("queues one run at a time, oldest first, each from the state before; ignored only when selected", async () => {
    const a = add(NOW - 3 * WEEK_MS, 1);
    const b = add(NOW - 2 * WEEK_MS, 2, { kind: "Before export" });
    const d = add(NOW - WEEK_MS - 1000, 4, { ignored: true });
    const c = add(NOW - WEEK_MS, 3, { kind: "After export" });
    open();
    expect(last_queue()).toEqual([hash_of(a)]);
    const state = fake_state(7);
    job_for(store.active_profile, a, false).store(fake_out(1, 1), state);
    await nextTick();
    expect(last_queue()).toEqual([hash_of(b)]);
    const queued = vi.mocked(queue_jobs).mock.lastCall[0][0];
    expect(queued.payload.chest_states).toBe(state.chest_states);
    for (const done of [b, c])
      job_for(store.active_profile, done, false).store(
        fake_out(1, 1),
        fake_state(),
      );
    await nextTick();
    const now = { ...snapshot_of(store.roster_config, 0, "Manual"), at: NOW };
    expect(last_queue()).toEqual([
      job_for(store.active_profile, now, true).hash,
    ]);
    await markers()[2].trigger("click");
    expect(last_queue()).toEqual([hash_of(d)]);
  });

  it("estimates per band from used snapshots and now, steps summed", async () => {
    add(NOW - 2 * WEEK_MS, 0);
    add(NOW - WEEK_MS - 1000, 999, { kind: "Before export" });
    add(NOW - WEEK_MS, 70, { kind: "After export" });
    set_cell(store.active_profile.bound_budgets, "Red", "75");
    open();
    await band("Char bound");
    expect(text_of("Red income")).toBe("37.5");
    expect(text_of("Red owned")).toBe("75");
    expect(wrapper.text()).toContain("2 steps over 14.0 days");
    await band("Tradable");
    expect(text_of("Red income")).toBe("0");
  });

  it("one snapshot is not enough", () => {
    add(NOW - WEEK_MS, 5);
    open();
    expect(text_of("Red income")).toBe("-");
    expect(wrapper.text()).toContain("Not enough snapshots to estimate");
  });

  it("ignoring from the dashboard drops the point", async () => {
    add(NOW - 2 * WEEK_MS, 0);
    const b = add(NOW - WEEK_MS, 70);
    open();
    await band("Char bound");
    await markers()[1].trigger("click");
    const ignore = wrapper
      .findAll('[aria-label="Snapshot dashboard"] label')
      .find((l) => l.text().includes("Ignore"));
    await ignore.find("input").trigger("change");
    expect(b.ignored).toBe(true);
    expect(text_of("Red income")).toBe("-");
  });

  it("a break makes two runs with one slope", async () => {
    add(NOW - 3 * WEEK_MS, 0);
    add(NOW - 2 * WEEK_MS, 10);
    add(NOW - WEEK_MS, 500, { breaks: true });
    add(NOW - 1000, 510);
    open();
    await band("Char bound");
    expect(Number(text_of("Red income"))).toBeCloseTo(10, 0);
    expect(wrapper.text()).toContain("2 steps over");
  });

  it("deletes the selected snapshot and clears the dashboard", async () => {
    add(NOW - 2 * WEEK_MS, 1);
    const b = add(NOW - WEEK_MS, 2);
    open();
    await markers()[1].trigger("click");
    expect(wrapper.text()).toContain("Since the snapshot before");
    await wrapper.find('[aria-label^="Delete"]').trigger("click");
    expect(store.roster_config.snapshots).not.toContain(b);
    expect(store.roster_config.snapshots).toHaveLength(1);
    expect(wrapper.text()).toContain("Hover a snapshot on the timeline.");
  });

  it("shows the change from the snapshot before", async () => {
    add(NOW - 2 * WEEK_MS, 1);
    add(NOW - WEEK_MS, 6);
    open();
    await markers()[0].trigger("mouseenter");
    expect(wrapper.text()).toContain("It is the first.");
    await markers()[0].trigger("mouseleave");
    await markers()[1].trigger("mouseenter");
    expect(text_of("Red Char bound")).toBe("+5");
  });

  it("now counts in the estimate only while it differs", async () => {
    add(NOW - 2 * WEEK_MS, 0);
    add(NOW - WEEK_MS, 10);
    open();
    expect(wrapper.text()).not.toContain("Include now");
    await band("Char bound");
    expect(text_of("Red income")).toBe("10");
    set_cell(store.active_profile.bound_budgets, "Red", "30");
    await nextTick();
    expect(Number(text_of("Red income"))).toBeGreaterThan(10);
  });

  it("Take snapshot adds one, not twice for the same budgets", async () => {
    open();
    const take = wrapper
      .findAll("button")
      .find((b) => b.text() === "Take snapshot");
    await take.trigger("click");
    await take.trigger("click");
    expect(store.roster_config.snapshots).toHaveLength(1);
    expect(store.roster_config.snapshots[0].kind).toBe("Manual");
  });

  it("manual mode pre-fills from the estimate and saves per band", async () => {
    add(NOW - 2 * WEEK_MS, 0);
    add(NOW - WEEK_MS, 70);
    open();
    await wrapper.find('[aria-label="Manual income"]').trigger("change");
    const profile = store.active_profile;
    expect(profile.income_manual).toBe(true);
    expect(profile.bound_income.Red).toBe(70);
    expect(store.roster_config.roster_income[0][1].Red).toBe(0);

    // the table shows tradable: typed into the roster's record
    await wrapper.find('input[aria-label="Red income"]').setValue("12");
    expect(store.roster_config.roster_income[0][1].Red).toBe(12);
    await band("Char bound");
    await wrapper.find('input[aria-label="Red income"]').setValue("abc");
    expect(profile.bound_income.Red).toBe(0);
  });

  it("manual income takes a thousands separator and a minus", async () => {
    open();
    await wrapper.find('[aria-label="Manual income"]').trigger("change");
    await wrapper.find('input[aria-label="Red income"]').setValue("1,000");
    expect(store.roster_config.roster_income[0][1].Red).toBe(1000);
    await wrapper.find('input[aria-label="Red income"]').setValue("-5");
    expect(store.roster_config.roster_income[0][1].Red).toBe(-5);
  });

  it("switching to manual keeps what was typed", async () => {
    add(NOW - 2 * WEEK_MS, 0);
    add(NOW - WEEK_MS, 70);
    const profile = store.active_profile;
    profile.bound_income.Red = 3;
    open();
    await wrapper.find('[aria-label="Manual income"]').trigger("change");
    expect(profile.bound_income.Red).toBe(3);
    expect(profile.bound_income.Blue).toBe(0);
  });

  it("rendering writes nothing to the store", async () => {
    store.active_profile.income_manual = true;
    open();
    await band("Roster bound");
    expect(store.roster_config.roster_income).toEqual({});
  });

  it("hides the cells that do not exist in game", async () => {
    open();
    const rows = () =>
      wrapper.findAll('[role="rowheader"]').map((row) => row.text());
    await band("Char bound");
    expect(rows()).not.toContain("Silver");
    expect(rows()).toContain("Free taps");
    await band("Roster bound");
    expect(rows()).toContain("Silver");
    expect(rows()).not.toContain("Free taps");
    await band("Tradable");
    expect(rows()).not.toContain("Silver");
    expect(rows()).not.toContain("Free taps");
    expect(rows()).toContain("Red");
  });

  it("Delete of a snapshot no longer listed removes nothing", async () => {
    add(NOW - 2 * WEEK_MS, 1);
    add(NOW - WEEK_MS, 2);
    open();
    await markers()[0].trigger("click");
    store.roster_config.snapshots.splice(0, 1);
    await nextTick();
    await wrapper.find('[aria-label^="Delete"]').trigger("click");
    expect(store.roster_config.snapshots).toHaveLength(1);
  });

  it("stops the runs on leaving", async () => {
    const { stop_jobs } = await import("@/WasmInterface/OptimizerJobs");
    open().unmount();
    expect(stop_jobs).toHaveBeenCalled();
    wrapper = undefined;
  });
});
