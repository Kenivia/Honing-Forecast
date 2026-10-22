import fs from "node:fs";
import path from "node:path";
import { nextTick } from "vue";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { import_config } from "@/Stores/ConfigStorage";
import { snapshot_of } from "@/Utils/Scans";
import {
  difficulties,
  is_empty,
  PARADISE_CHOICES,
  PRESET_CHOICES,
  raids_of,
  shown_difficulty,
} from "@/Utils/Choices";
import { ilevel } from "@/Utils/HoningUtil";
import { parse_locale_float } from "@/Utils/InputColumn";
import { base_job, option_job } from "@/Components/Character/Choices/ChoiceJobs";
import Income from "@/Components/Character/Income/Income.vue";
import { queue_jobs } from "@/WasmInterface/OptimizerJobs";

// One-off: the Choices and Income pages' stateless payloads over tests/income-fixture.json, as
// arena scenarios. Run: pnpm exec vitest run --config scripts/arena/vitest.export.config.ts
vi.mock("@/WasmInterface/OptimizerJobs", async (original) => ({
  ...(await original<object>()),
  queue_jobs: vi.fn(),
  stop_jobs: vi.fn(),
}));

const FIXTURE = fs.readFileSync("tests/income-fixture.json", "utf8");
const NOW = Date.UTC(2026, 9, 21, 12);
const OUT = "arena_data/scenarios";

// Maps as plain objects, no cached advanced-honing table
const plain = (payload: any) =>
  JSON.stringify(
    { ...payload, adv_cache: null },
    (_, v) => (v instanceof Map ? Object.fromEntries(v) : v),
  );
const unmap = (payload: any) => JSON.parse(plain(payload));
const write = (dir: string, name: string, body: object) => {
  fs.mkdirSync(path.join(OUT, dir), { recursive: true });
  fs.writeFileSync(
    path.join(OUT, dir, `${name}.json`),
    JSON.stringify({ name, ...body }),
    { encoding: "utf8" },
  );
};
const slug = (s: string) => s.replace(/[^A-Za-z0-9]+/g, "_").replace(/^_|_$/g, "");

let store: ReturnType<typeof useRosterStore>;
beforeEach(() => {
  vi.useFakeTimers();
  vi.setSystemTime(NOW);
  setActivePinia(createPinia());
  store = useRosterStore();
  store.replace_config(import_config(FIXTURE));
});

function group(steps: any[], labels: string[], profile: any) {
  // dedupe identical payloads, base first
  const seen = new Set<string>();
  const out = { steps: [] as any[], labels: [] as string[] };
  steps.forEach((step, i) => {
    const text = JSON.stringify(step);
    if (seen.has(text)) return;
    seen.add(text);
    out.steps.push(step);
    out.labels.push(labels[i]);
  });
  return {
    steps: out.steps,
    meta: {
      star: true,
      character: profile.char_name,
      tier: profile.tier,
      labels: out.labels,
    },
  };
}

describe("export", () => {
  it("choices", () => {
    const profiles = store.roster_config.profiles;
    const names = new Set<string>();
    for (let at = 0; at < profiles.length; at++) {
      store.roster_config.active_profile_index = at;
      const profile = store.active_profile;
      profile.choice_seed = undefined;
      const now = snapshot_of(
        store.roster_config,
        profile.roster_id,
        "Manual",
      );
      const base = base_job(profile, now);
      if (!base.payload.upgrade_info.length) continue;
      const item_level =
        parse_locale_float(ilevel(profile, "achieved")) || 0;
      const base_payload = unmap(base.payload);
      let stem = slug(profile.char_name);
      if (names.has(stem)) stem += `_${at}`;
      names.add(stem);

      const add = (steps: any[], labels: string[], choice: any, title: string) => {
        choice.options.forEach((option: any, i: number) => {
          if (is_empty(option)) return;
          steps.push(
            unmap(option_job(profile, now, base, choice, i).payload),
          );
          labels.push(`${title} ${choice.names?.[i] ?? i}`);
        });
      };

      // bonus chests
      const raids = raids_of(PRESET_CHOICES)
        .filter(
          (r) => !r.difficulties.length || r.difficulties[0].item_level <= item_level,
        )
        .slice(0, 3);
      let steps: any[] = [base_payload];
      let labels = ["base"];
      for (const raid of raids) {
        const shown = shown_difficulty(raid.difficulties, item_level);
        for (const choice of raid.members) {
          if (choice.difficulty && choice.difficulty !== shown) continue;
          add(steps, labels, choice, `${raid.name} ${choice.title} ${shown ?? ""}`.trim());
        }
      }
      if (steps.length > 1)
        write("choices", `${stem}_bonus`, group(steps, labels, profile));

      // paradise
      steps = [base_payload];
      labels = ["base"];
      const cases: [string, string][] = [
        ["Destiny", "Base"],
        ["Destiny", "Level 10"],
        ["Netherworld", "Base"],
      ];
      for (const [mode, level] of cases) {
        const members = PARADISE_CHOICES.filter(
          (c) => c.group === `Paradise: ${mode}`,
        );
        const shown = shown_difficulty(difficulties(members), item_level);
        const choice = members.find(
          (x) => x.difficulty === shown && x.title === level,
        );
        if (choice) add(steps, labels, choice, `Paradise ${mode} ${shown} ${level}`);
      }
      if (steps.length > 1)
        write("choices", `${stem}_paradise`, group(steps, labels, profile));
    }
  });

  it("income", () => {
    const profiles = store.roster_config.profiles;
    const names = new Set<string>();
    for (let at = 0; at < profiles.length; at++) {
      store.roster_config.active_profile_index = at;
      const profile = store.active_profile;
      if (!Object.keys(profile.keyed_upgrades).length) continue;
      if (!store.roster_config.snapshots.some((s) => s.chars[profile.id])) continue;
      const wrapper = mount(Income, { global: { stubs: { IncomeGraph: true } } });
      const vm: any = wrapper.vm;
      vm.weeks_ahead = 12;
      const chain: any[] = vm.chain;
      const snaps = vm.entries.length;
      if (!chain[0]?.job.payload.upgrade_info.length) {
        wrapper.unmount();
        continue;
      }
      const labels = chain.map((e, i) =>
        i < snaps
          ? `snapshot ${new Date(e.snapshot.at).toISOString().slice(0, 10)}`
          : i === snaps
            ? "now"
            : `+${i - snaps}w`,
      );
      let stem = slug(profile.char_name);
      if (names.has(stem)) stem += `_${at}`;
      names.add(stem);
      write("income", stem, {
        steps: chain.map((e) => unmap(e.job.payload)),
        meta: { character: profile.char_name, labels },
      });
      wrapper.unmount();
    }
    expect(queue_jobs).toBeDefined();
  });
});
