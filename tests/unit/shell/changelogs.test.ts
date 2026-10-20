import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises } from "@vue/test-utils";
import ChangeLogsView from "@/Components/ChangeLogsView.vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  ALL_VERSIONS,
  LATEST_VERSION,
  load_change_log,
} from "@/Utils/Changelog";
import { mount_app } from "../mount";
import { stub_match_media } from "./helpers";

// fetch answers "# <url>" once released; pending ones resolve in the order the test chooses.
// The html cache is module-wide, so every test uses its own version names.
let pending: { url: string; release: () => void }[] = [];
let auto = true;
let status = 200;
const fetch_mock = vi.fn((url: string) => {
  const res = { ok: status === 200, status, text: async () => `# ${url}` };
  if (status === 0) return Promise.reject(new Error("offline"));
  if (auto) return Promise.resolve(res);
  return new Promise((resolve) =>
    pending.push({ url, release: () => resolve(res) }),
  );
});

beforeEach(() => {
  stub_match_media(false);
  pending = [];
  auto = true;
  status = 200;
  fetch_mock.mockClear();
  vi.stubGlobal("fetch", fetch_mock);
});
afterEach(() => vi.unstubAllGlobals());

const show = async (version: string) => {
  const out = await mount_app(ChangeLogsView, {}, `/change-logs/${version}`);
  await flushPromises();
  return out;
};
const go = async (router, version: string) => {
  await router.push(`/change-logs/${version}`);
  await flushPromises();
};
const body = (w) => w.find(".change-log-html");
const urls = () => fetch_mock.mock.calls.map((c) => c[0]);

describe("load_change_log", () => {
  it("parses, caches, and does not cache failures", async () => {
    expect(await load_change_log("u1")).toContain(
      "<h1>/change-logs/u1.md</h1>",
    );
    expect(await load_change_log("u1")).toContain("u1");
    status = 404;
    expect(await load_change_log("u2")).toBeNull();
    status = 0;
    expect(await load_change_log("u2")).toBeNull();
    status = 200;
    expect(await load_change_log("u2")).toContain("u2");
    expect(urls()).toEqual([
      "/change-logs/u1.md",
      "/change-logs/u2.md",
      "/change-logs/u2.md",
      "/change-logs/u2.md",
    ]);
  });
});

describe("ChangeLogsView", () => {
  it("marks the latest version seen", async () => {
    await show("t-seen");
    expect(useRosterStore().roster_config.last_seen_version).toBe(
      LATEST_VERSION,
    );
  });

  it("renders the version's markdown", async () => {
    const { wrapper } = await show("t-render");
    expect(urls()).toEqual(["/change-logs/t-render.md"]);
    expect(body(wrapper).find("h1").text()).toBe("/change-logs/t-render.md");
  });

  it("WIP reads WIP.md", async () => {
    await show("WIP");
    expect(urls()).toEqual(["/WIP.md"]);
  });

  it("shows loading until the fetch lands", async () => {
    auto = false;
    const { wrapper } = await show("t-load");
    expect(wrapper.text()).toContain("Loading...");
    pending[0].release();
    await flushPromises();
    expect(wrapper.text()).not.toContain("Loading...");
  });

  it("fetches each version once, across visits", async () => {
    const { wrapper, router } = await show("t-c1");
    await go(router, "t-c2");
    await go(router, "t-c1");
    wrapper.unmount();
    await show("t-c2");
    expect(urls()).toEqual(["/change-logs/t-c1.md", "/change-logs/t-c2.md"]);
  });

  it("shows an error when the fetch fails", async () => {
    status = 404;
    const { wrapper } = await show("t-err");
    expect(wrapper.text()).toContain("Could not load this change log.");
    expect(body(wrapper).exists()).toBe(false);
  });

  it("does not fetch when the route leaves the page", async () => {
    const { router } = await show("t-leave");
    await router.push("/roster-setup");
    await flushPromises();
    expect(urls()).toEqual(["/change-logs/t-leave.md"]);
  });

  it("sidebar links WIP then every version", async () => {
    const { wrapper } = await show("t-side");
    const links = wrapper.findAll("a.side-bar-link");
    expect(links.map((a) => a.attributes("href"))).toEqual([
      "/change-logs/WIP",
      ...ALL_VERSIONS.map((v) => `/change-logs/${v}`),
    ]);
  });

  it("shows the last selected version when fetches land out of order", async () => {
    auto = false;
    const { wrapper, router } = await show("t-r1");
    await go(router, "t-r2");
    pending[1].release();
    await flushPromises();
    pending[0].release();
    await flushPromises();
    expect(body(wrapper).text()).toBe("/change-logs/t-r2.md");
  });
});
