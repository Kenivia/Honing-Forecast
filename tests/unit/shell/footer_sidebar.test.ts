import { beforeEach, describe, expect, it, vi } from "vitest";
import { mount, RouterLinkStub } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { h, nextTick } from "vue";
import Footer from "@/Components/Common/Footer.vue";
import Sidebar from "@/Components/Common/Sidebar.vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import { LATEST_VERSION, version_equal } from "@/Utils/Changelog";
import { stub_match_media } from "./helpers";

const stubs = { RouterLink: RouterLinkStub };

beforeEach(() => setActivePinia(createPinia()));

function mount_footer(last_seen: string) {
  useRosterStore().roster_config.last_seen_version = last_seen;
  return mount(Footer, { global: { stubs } });
}

describe("Footer", () => {
  it("shows the latest version linking to the change logs", () => {
    const w = mount_footer(LATEST_VERSION);
    const link = w.findComponent(RouterLinkStub);
    expect(link.props("to")).toEqual({ name: "change-logs-root" });
    expect(link.text()).toContain(LATEST_VERSION);
  });

  it("marks new when the last seen version differs", () => {
    expect(mount_footer("v0.0.0").find(".new-tag").exists()).toBe(true);
    expect(mount_footer(LATEST_VERSION).find(".new-tag").exists()).toBe(false);
  });

  it("drops the mark once the last seen version changes", async () => {
    const w = mount_footer("v0.0.0");
    useRosterStore().roster_config.last_seen_version = LATEST_VERSION;
    await nextTick();
    expect(w.find(".new-tag").exists()).toBe(false);
  });

  it("version_equal is plain equality, patch included", () => {
    expect(version_equal("v1.3.0", "v1.3.0")).toBe(true);
    expect(version_equal("1.3.0", "v1.3.0")).toBe(true);
    expect(version_equal("v1.3.0", "v1.3.1")).toBe(false);
    expect(version_equal("v0.0.0", "v1.2.5")).toBe(false);
  });
});

function mount_sidebar(narrow: boolean) {
  stub_match_media(narrow);
  return mount(Sidebar, {
    props: { header: "Head", width: 900 },
    slots: {
      sidebar: (p) =>
        h("button", { class: "slot-close", onClick: p.close }, "x"),
      main: () => h("div", { class: "slot-main" }, "main"),
    },
    global: { stubs: { ...stubs, Footer: true } },
  });
}

describe("Sidebar", () => {
  it("renders header and both slots", () => {
    const w = mount_sidebar(false);
    expect(w.find(".side-bar-header").text()).toBe("Head");
    expect(w.find(".slot-main").exists()).toBe(true);
    expect(w.find(".slot-close").exists()).toBe(true);
  });

  it("queries the media at its width", () => {
    mount_sidebar(false);
    expect(window.matchMedia).toHaveBeenCalledWith("(max-width: 900px)");
  });

  it("wide: no burger, never an open drawer", () => {
    const w = mount_sidebar(false);
    expect(w.find(".burger").exists()).toBe(false);
    expect(w.find("nav").classes()).not.toContain("side-bar--narrow");
  });

  it("narrow: burger opens, backdrop and slot close close it", async () => {
    const w = mount_sidebar(true);
    const nav = () => w.find("nav").classes();
    expect(nav()).toContain("side-bar--narrow");
    expect(nav()).not.toContain("side-bar--open");
    await w.find(".burger").trigger("click");
    expect(nav()).toContain("side-bar--open");
    await w.find(".backdrop").trigger("click");
    expect(nav()).not.toContain("side-bar--open");
    await w.find(".burger").trigger("click");
    await w.find(".slot-close").trigger("click");
    expect(nav()).not.toContain("side-bar--open");
  });
});
