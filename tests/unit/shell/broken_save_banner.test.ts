import { afterEach, describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import BrokenSaveBanner from "@/Components/Common/BrokenSaveBanner.vue";
import { broken_save, STORAGE_KEY } from "@/Stores/ConfigStorage";

afterEach(() => {
  broken_save.value = null;
  vi.unstubAllGlobals();
});

function banner(kept: boolean) {
  setActivePinia(createPinia());
  broken_save.value = { key: STORAGE_KEY, version: 8, raw: "garbage", kept };
  localStorage.setItem(`${STORAGE_KEY}_BROKEN`, "garbage");
  return mount(BrokenSaveBanner);
}

describe("BrokenSaveBanner", () => {
  it("shows nothing without an unreadable save", () => {
    setActivePinia(createPinia());
    expect(mount(BrokenSaveBanner).html()).toBe("<!--v-if-->");
  });

  it("says saving is paused only when no copy could be kept", () => {
    expect(banner(true).text()).not.toContain("paused");
    expect(banner(false).text()).toContain("saving is paused");
  });

  it("Download hands over the save, Discard removes it", async () => {
    const create = vi.fn((_blob: Blob) => "blob:x");
    vi.stubGlobal("URL", { createObjectURL: create, revokeObjectURL: vi.fn() });
    const click = vi
      .spyOn(HTMLAnchorElement.prototype, "click")
      .mockImplementation(() => {});
    const w = banner(true);
    const [download, discard] = w.findAll("button");
    await download.trigger("click");
    expect(click).toHaveBeenCalledOnce();
    expect(create.mock.calls[0][0].size).toBe("garbage".length);
    await discard.trigger("click");
    expect(broken_save.value).toBeNull();
    expect(localStorage.getItem(`${STORAGE_KEY}_BROKEN`)).toBeNull();
    expect(w.find('[role="alert"]').exists()).toBe(false);
  });
});
