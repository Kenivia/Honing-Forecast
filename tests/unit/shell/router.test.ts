import { beforeAll, beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import type { Router } from "vue-router";
import { ALL_VERSIONS, LATEST_VERSION } from "@/Utils/Changelog";
import { useRosterStore } from "@/Stores/RosterConfig";
import { load_router, set_profiles } from "./helpers";

let router: Router;
let guard_switch: ReturnType<typeof import("vitest").vi.fn>;

const go = async (path: string) => {
  await router.push(path);
  return router.currentRoute.value;
};

beforeAll(async () => {
  ({ router, guard_switch } = await load_router());
});

beforeEach(async () => {
  setActivePinia(createPinia());
  set_profiles([
    ["Alice", 0],
    ["Bob", 0],
  ]);
  // park on a non-char page so each test enters the char route afresh
  await router.push("/roster-setup");
});

describe("router", () => {
  it("profile_index_by_name finds by exact name", () => {
    const store = useRosterStore();
    expect(store.profile_index_by_name("Bob")).toBe(1);
    expect(store.profile_index_by_name("bob")).toBe(-1);
  });

  it("/ goes to the first character's calc", async () => {
    expect((await go("/")).fullPath).toBe("/chars/NA/Alice/calc");
  });

  it("a bare character goes to its calc", async () => {
    const r = await go("/chars/NA/Bob");
    expect(r.fullPath).toBe("/chars/NA/Bob/calc");
    expect(r.name).toBe("calc");
  });

  it("known sub-pages resolve by name", async () => {
    expect((await go("/chars/NA/Bob/scanner")).name).toBe("char-scanner");
    expect((await go("/chars/NA/Bob/income")).name).toBe("char-income");
    expect((await go("/chars/NA/Bob/choices")).name).toBe("char-choices");
  });

  it("an unknown character goes to the first one, keeping the sub-page", async () => {
    expect((await go("/chars/NA/Nobody")).fullPath).toBe(
      "/chars/NA/Alice/calc",
    );
    await router.push("/roster-setup");
    expect((await go("/chars/NA/Nobody/choices")).fullPath).toBe(
      "/chars/NA/Alice/choices",
    );
  });

  it("legacy :x/guide and :x/scanner redirect", async () => {
    expect((await go("/chars/NA/Bob/old/guide")).fullPath).toBe("/info/guide");
    expect((await go("/chars/NA/Bob/old/scanner")).fullPath).toBe(
      "/chars/NA/Bob/scanner",
    );
  });

  it("old character urls move under /chars", async () => {
    expect((await go("/Bob")).fullPath).toBe("/chars/NA/Bob/calc");
    expect((await go("/Bob/scanner")).fullPath).toBe("/chars/NA/Bob/scanner");
    expect((await go("/Nobody/choices")).fullPath).toBe(
      "/chars/NA/Alice/choices",
    );
    expect((await go("/chars")).fullPath).toBe("/chars/NA/Alice/calc");
  });

  it("an unknown sub-page goes to calc", async () => {
    expect((await go("/chars/NA/Bob/zzz")).fullPath).toBe("/chars/NA/Bob/calc");
    expect((await go("/chars/NA/Bob/a/b/c")).fullPath).toBe(
      "/chars/NA/Bob/calc",
    );
  });

  it("re-checks the name when only the param changes", async () => {
    await go("/chars/NA/Bob/choices");
    expect((await go("/chars/NA/Nobody/choices")).fullPath).toBe(
      "/chars/NA/Alice/choices",
    );
  });

  it("keeps query and hash on correction", async () => {
    expect((await go("/chars/NA/Nobody/calc?x=1#h")).fullPath).toBe(
      "/chars/NA/Alice/calc?x=1#h",
    );
  });

  it("change logs default to the latest version", async () => {
    expect(LATEST_VERSION).toBe(ALL_VERSIONS[0]);
    expect((await go("/info/change-logs")).fullPath).toBe(
      `/info/change-logs/${LATEST_VERSION}`,
    );
  });

  it("info defaults to the guide, unknown docs too", async () => {
    expect((await go("/info")).name).toBe("info-guide");
    expect((await go("/info/zzz")).name).toBe("info-guide");
  });

  it("old doc urls redirect into info", async () => {
    expect((await go("/chars/NA/Bob/guide")).fullPath).toBe("/info/guide");
    expect((await go("/change-logs")).fullPath).toBe(
      `/info/change-logs/${LATEST_VERSION}`,
    );
    expect((await go("/change-logs/WIP")).fullPath).toBe(
      "/info/change-logs/WIP",
    );
    expect((await go("/change-logs/a/b")).fullPath).toBe(
      `/info/change-logs/${LATEST_VERSION}`,
    );
  });

  it("change log versions are validated", async () => {
    const older = ALL_VERSIONS[ALL_VERSIONS.length - 1];
    expect((await go(`/info/change-logs/${older}`)).params.version).toBe(older);
    expect((await go("/info/change-logs/WIP")).params.version).toBe("WIP");
    await router.push("/roster-setup");
    expect((await go("/info/change-logs/v0.0.1")).params.version).toBe(
      LATEST_VERSION,
    );
    expect((await go("/info/change-logs/a/b")).fullPath).toBe(
      `/info/change-logs/${LATEST_VERSION}`,
    );
  });

  it("re-validates when switching between versions", async () => {
    await go("/info/change-logs/WIP");
    expect((await go("/info/change-logs/v0.0.1")).params.version).toBe(
      LATEST_VERSION,
    );
  });

  it("versions sort newest first, numerically", () => {
    const nums = ALL_VERSIONS.map((v) =>
      v.replace("v", "").split(".").map(Number),
    );
    for (let i = 1; i < nums.length; i++) {
      const [a, b] = [nums[i - 1], nums[i]];
      expect(a[0] * 1e6 + a[1] * 1e3 + a[2]).toBeGreaterThan(
        b[0] * 1e6 + b[1] * 1e3 + b[2],
      );
    }
  });

  it("runs the scan-switch guard on every navigation", async () => {
    guard_switch.mockClear();
    await go("/chars/NA/Bob/calc");
    expect(guard_switch).toHaveBeenCalled();
    expect(guard_switch.mock.calls.at(-1)[0].fullPath).toBe(
      "/chars/NA/Bob/calc",
    );
  });

  it("old region-less character urls go to that name's region", async () => {
    expect((await go("/chars/Bob/scanner")).fullPath).toBe(
      "/chars/NA/Bob/scanner",
    );
    expect((await go("/chars/Nobody/choices")).fullPath).toBe(
      "/chars/NA/Alice/choices",
    );
  });

  it("the same name in two regions is two routes", async () => {
    const store = set_profiles([
      ["Bob", 0],
      ["Bob", 1],
    ]);
    store.roster_config.all_regions["1"] = "euc";
    expect(store.profile_index_by_route("EU", "Bob")).toBe(1);
    expect(store.char_params(store.all_profiles[1])).toEqual({
      region: "EU",
      characterName: "Bob",
    });
    expect((await go("/chars/EU/Bob/calc")).fullPath).toBe(
      "/chars/EU/Bob/calc",
    );
    // a name missing from that region is looked up in any region
    expect((await go("/chars/Custom/Bob/calc")).fullPath).toBe(
      "/chars/NA/Bob/calc",
    );
  });

  it("the guard can cancel a navigation", async () => {
    await go("/chars/NA/Alice/calc");
    guard_switch.mockReturnValueOnce(false);
    await router.push("/chars/NA/Bob/calc");
    expect(router.currentRoute.value.fullPath).toBe("/chars/NA/Alice/calc");
  });
});
