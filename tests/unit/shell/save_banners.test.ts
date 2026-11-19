import { afterEach, describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import LZString from "lz-string";
import { ref } from "vue";
import TopBanners from "@/Components/Common/TopBanners.vue";
import {
  broken_save,
  moved_notice,
  moved_save,
  STORAGE_KEY,
} from "@/Stores/ConfigStorage";

// other test files' "tabs" reach this one through BroadcastChannel
vi.mock("@/Utils/OtherTabs", () => ({ other_tabs: ref([]) }));

afterEach(() => {
  broken_save.value = null;
  moved_save.value = null;
  moved_notice.value = null;
  localStorage.clear();
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

function banners() {
  setActivePinia(createPinia());
  return mount(TopBanners);
}

function broken(kept: boolean) {
  broken_save.value = { key: STORAGE_KEY, version: 8, raw: "garbage", kept };
  localStorage.setItem(`${STORAGE_KEY}_BROKEN`, "garbage");
  return banners();
}

// captures what the Download button hands over
function stub_download() {
  const create = vi.fn((_blob: Blob) => "blob:x");
  vi.stubGlobal("URL", { createObjectURL: create, revokeObjectURL: vi.fn() });
  vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(() => {});
  return create;
}

describe("save banners", () => {
  it("shows nothing by default", () => {
    expect(banners().find('[role="alert"], [role="status"]').exists()).toBe(
      false,
    );
  });

  it("broken: says saving is paused only when no copy could be kept", () => {
    expect(broken(true).text()).not.toContain("paused");
    expect(broken(false).text()).toContain("saving is paused");
  });

  it("broken: Download hands over the save, Discard removes it", async () => {
    const create = stub_download();
    const w = broken(true);
    const [download, discard] = w.findAll('[role="alert"] button');
    await download.trigger("click");
    expect(create.mock.calls[0][0].size).toBe("garbage".length);
    await discard.trigger("click");
    expect(broken_save.value).toBeNull();
    expect(localStorage.getItem(`${STORAGE_KEY}_BROKEN`)).toBeNull();
    expect(w.find('[role="alert"]').exists()).toBe(false);
  });

  it("moved save: Download is a backup, Discard drops it", async () => {
    const create = stub_download();
    const json = JSON.stringify({ version: 13, profiles: [] });
    moved_save.value = { [STORAGE_KEY]: LZString.compressToUTF16(json) };
    localStorage.setItem("hf_moved_in", JSON.stringify(moved_save.value));
    const w = banners();
    expect(w.text()).toContain("was not loaded");
    const [download, , discard] = w.findAll('[role="alert"] button');
    await download.trigger("click");
    const backup = JSON.stringify(JSON.parse(json), null, 2);
    expect(create.mock.calls[0][0].size).toBe(backup.length);
    await discard.trigger("click");
    expect(localStorage.getItem("hf_moved_in")).toBeNull();
    expect(w.find('[role="alert"]').exists()).toBe(false);
  });

  it("moved notice: info until dismissed", async () => {
    moved_notice.value = "moved";
    localStorage.setItem("hf_moved_notice", "moved");
    const w = banners();
    expect(w.find('[role="status"]').text()).toContain("has moved to");
    await w.find('[role="status"] button').trigger("click");
    expect(localStorage.getItem("hf_moved_notice")).toBeNull();
    expect(w.find('[role="status"]').exists()).toBe(false);
  });
});
