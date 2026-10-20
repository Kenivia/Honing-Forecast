import { afterEach, describe, expect, it, vi } from "vitest";
import LZString from "lz-string";
import {
  broken_backup,
  broken_save,
  debounced_write_roster_config,
  discard_broken,
  export_config,
  import_config,
  load_roster_config,
  STORAGE_KEY,
  STORAGE_VERSION,
  write_state,
} from "@/Stores/ConfigStorage";
import { to_saved } from "@/Stores/SavedConfig";
import { useRosterStore } from "@/Stores/RosterConfig";
import { createPinia, setActivePinia } from "pinia";
import { nextTick } from "vue";
import { rich_config } from "./fixtures";

const put = (key: string, data: any) =>
  localStorage.setItem(key, LZString.compressToUTF16(JSON.stringify(data)));
const saved_of = (config) => ({
  version: STORAGE_VERSION,
  ...to_saved(config),
});
const named = (name: string) => {
  const config = rich_config();
  config.profiles[0].char_name = name;
  return config;
};
// a minimal V7 payload: one profile, nothing else
const v7 = (name: string) => ({
  profiles: [{ char_name: name, roster_id: 0, tier: 0 }],
  active_profile_index: 0,
});

afterEach(() => {
  vi.useRealTimers();
  vi.restoreAllMocks();
  broken_save.value = null;
});

describe("load: keys", () => {
  it("the current key wins over a legacy one", () => {
    put(STORAGE_KEY, saved_of(named("Current")));
    put("HF_CONFIG_V7_COMPRESSED", v7("Legacy"));
    expect(load_roster_config().profiles[0].char_name).toBe("Current");
  });

  it("a legacy save is migrated, then its key is removed", () => {
    put("HF_CONFIG_V7_COMPRESSED", v7("Legacy"));
    expect(load_roster_config().profiles[0].char_name).toBe("Legacy");
    expect(localStorage.getItem("HF_CONFIG_V7_COMPRESSED")).toBeNull();
  });

  it("a corrupt current key falls through to the legacy save", () => {
    localStorage.setItem(STORAGE_KEY, "garbage");
    put("HF_CONFIG_V7_COMPRESSED", v7("Legacy"));
    expect(load_roster_config().profiles[0].char_name).toBe("Legacy");
  });

  it("abandoned V3 / V4 keys are deleted", () => {
    localStorage.setItem("HF_UI_STATE_V3_roster", "x");
    localStorage.setItem("HF_UI_STATE_V4_roster", "x");
    load_roster_config();
    expect(localStorage.length).toBe(0);
  });

  it("a payload that throws loads the defaults and is kept aside", () => {
    vi.spyOn(console, "error").mockImplementation(() => {});
    put(STORAGE_KEY, { version: STORAGE_VERSION, profiles: "oops" });
    const raw = localStorage.getItem(STORAGE_KEY);
    const loaded = load_roster_config();
    expect(loaded.profiles).toHaveLength(1);
    expect(loaded.profiles[0].char_name).toBe("Newchar");
    write_state({ roster_config: loaded });
    expect(localStorage.getItem(`${STORAGE_KEY}_BROKEN`)).toBe(raw);
  });

  it("a save that does not decompress is kept aside", () => {
    localStorage.setItem(STORAGE_KEY, "garbage");
    load_roster_config();
    expect(localStorage.getItem(`${STORAGE_KEY}_BROKEN`)).toBe("garbage");
  });

  it("a save that decompresses to nothing is kept aside, not taken for no save", () => {
    localStorage.setItem(STORAGE_KEY, "null");
    load_roster_config();
    expect(localStorage.getItem(`${STORAGE_KEY}_BROKEN`)).toBe("null");
    expect(broken_save.value).toMatchObject({ raw: "null", kept: true });
  });

  it("older legacy keys are deleted when a newer save is found", () => {
    put(STORAGE_KEY, saved_of(named("Current")));
    put("HF_CONFIG_V7_COMPRESSED", v7("Legacy"));
    put("HF_CONFIG_V6_COMPRESSED", v7("Older"));
    expect(load_roster_config().profiles[0].char_name).toBe("Current");
    expect(localStorage.getItem("HF_CONFIG_V7_COMPRESSED")).toBeNull();
    expect(localStorage.getItem("HF_CONFIG_V6_COMPRESSED")).toBeNull();
    expect(localStorage.getItem(STORAGE_KEY)).not.toBeNull();
  });
});

describe("write", () => {
  it("writes the version inside the payload under the current key", () => {
    write_state({ roster_config: rich_config() });
    const raw = JSON.parse(
      LZString.decompressFromUTF16(localStorage.getItem(STORAGE_KEY)),
    );
    expect(raw.version).toBe(STORAGE_VERSION);
    expect(raw.profiles).toHaveLength(3);
  });

  it("a failing setItem is logged, not thrown", () => {
    const error = vi.spyOn(console, "error").mockImplementation(() => {});
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new Error("quota");
    });
    expect(() => write_state({ roster_config: rich_config() })).not.toThrow();
    expect(error).toHaveBeenCalled();
  });

  it("the debounced write lands once, 500ms after the last call, with the last state", () => {
    vi.useFakeTimers();
    const set = vi.spyOn(Storage.prototype, "setItem");
    debounced_write_roster_config({ roster_config: named("One") });
    vi.advanceTimersByTime(400);
    debounced_write_roster_config({ roster_config: named("Two") });
    vi.advanceTimersByTime(499);
    expect(set).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(set).toHaveBeenCalledTimes(1);
    expect(load_roster_config().profiles[0].char_name).toBe("Two");
  });
});

describe("store wiring as in App.vue", () => {
  it("a deep mutation is saved 500ms later", async () => {
    vi.useFakeTimers();
    setActivePinia(createPinia());
    const store = useRosterStore();
    store.$subscribe((_m, state) => debounced_write_roster_config(state));
    store.roster_config.profiles[0].bound_income = { Red: 3 };
    store.roster_config.scan_pages.push({ roster_id: 0 } as any);
    await nextTick();
    vi.advanceTimersByTime(500);
    const loaded = load_roster_config();
    expect(loaded.profiles[0].bound_income).toEqual({ Red: 3 });
    expect(loaded.scan_pages).toHaveLength(1);
  });
});

describe("export / import", () => {
  it("import of an export keeps every field and the ids", () => {
    const config = rich_config();
    const back = import_config(export_config(config));
    expect(to_saved(back)).toEqual(
      JSON.parse(JSON.stringify(to_saved(import_config(export_config(back))))),
    );
    expect(back.profiles.map((p) => p.id)).toEqual(
      config.profiles.map((p) => p.id),
    );
    expect(back.snapshots).toEqual(config.snapshots);
    expect(back.scan_pages).toEqual(config.scan_pages);
  });

  it("a backup without a version is read as the current one", () => {
    const text = JSON.stringify(to_saved(named("NoVersion")));
    expect(import_config(text).profiles[0].char_name).toBe("Noversion");
  });

  it("an older backup goes through the migrations", () => {
    const text = JSON.stringify({ ...v7("Old"), version: 7 });
    const back = import_config(text);
    expect(back.profiles[0].char_name).toBe("Old");
    expect(back.profiles[0].chests).toEqual([]);
  });

  it("import does not touch localStorage", () => {
    import_config(export_config(rich_config()));
    expect(localStorage.length).toBe(0);
  });

  it("invalid JSON throws", () => {
    expect(() => import_config("{nope")).toThrow();
  });
});

describe("an unreadable save", () => {
  it("a copy kept by an earlier load is reported until discarded", () => {
    put(STORAGE_KEY, saved_of(named("Current")));
    localStorage.setItem(`${STORAGE_KEY}_BROKEN`, "old");
    const loaded = load_roster_config();
    expect(loaded.profiles[0].char_name).toBe("Current");
    expect(broken_save.value).toMatchObject({ raw: "old", kept: true });
    discard_broken({ roster_config: loaded });
    expect(broken_save.value).toBeNull();
    expect(localStorage.getItem(`${STORAGE_KEY}_BROKEN`)).toBeNull();
  });

  it("with no room for the copy, saving pauses until it is discarded", () => {
    const set = Storage.prototype.setItem;
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(function (k, v) {
      if (k.endsWith("_BROKEN"))
        throw new DOMException("full", "QuotaExceededError");
      set.call(this, k, v);
    });
    localStorage.setItem(STORAGE_KEY, "garbage");
    const loaded = load_roster_config();
    expect(broken_save.value).toMatchObject({ raw: "garbage", kept: false });
    write_state({ roster_config: loaded });
    expect(localStorage.getItem(STORAGE_KEY)).toBe("garbage");
    discard_broken({ roster_config: loaded });
    expect(localStorage.getItem(STORAGE_KEY)).not.toBe("garbage");
    expect(load_roster_config().profiles[0].char_name).toBe("Newchar");
  });

  it("the download is a backup with its version, or the raw string", () => {
    vi.spyOn(console, "error").mockImplementation(() => {});
    put("HF_CONFIG_V7_COMPRESSED", { profiles: "oops" });
    load_roster_config();
    expect(JSON.parse(broken_backup(broken_save.value))).toEqual({
      version: 7,
      profiles: "oops",
    });
    expect(
      broken_backup({
        key: STORAGE_KEY,
        version: 8,
        raw: "garbage",
        kept: true,
      }),
    ).toBe("garbage");
  });

  it("a download of a save the code could not load imports once the code can", () => {
    vi.spyOn(console, "error").mockImplementation(() => {});
    put(STORAGE_KEY, saved_of(named("Kept")));
    // as if this version's loader were broken
    const raw = localStorage.getItem(STORAGE_KEY);
    broken_save.value = {
      key: STORAGE_KEY,
      version: STORAGE_VERSION,
      raw,
      kept: true,
    };
    expect(
      import_config(broken_backup(broken_save.value)).profiles[0].char_name,
    ).toBe("Kept");
  });
});
