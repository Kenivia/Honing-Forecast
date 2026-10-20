import { afterEach, describe, expect, it, vi } from "vitest";
import { fetch_uwuowo } from "@/Components/Common/Uwuowo/UwuowoUtils";

afterEach(() => {
  vi.unstubAllGlobals();
  vi.useRealTimers();
});

// the limiter's timestamps are module state, so this file holds only this test
describe("lostark.bible rate limit", () => {
  it("allows 60 requests per rolling minute, then waits for the window", async () => {
    vi.useFakeTimers();
    const fn = vi.fn(() =>
      Promise.resolve({ ok: true, text: () => Promise.resolve("") }),
    );
    vi.stubGlobal("fetch", fn);
    const req = { region: "NA" as const, char_name: "A", suffix: "" };

    await Promise.all(Array.from({ length: 60 }, () => fetch_uwuowo(req)));
    expect(fn).toHaveBeenCalledTimes(60);

    await vi.advanceTimersByTimeAsync(30_000);
    const late = [fetch_uwuowo(req), fetch_uwuowo(req)];
    await vi.advanceTimersByTimeAsync(29_999);
    expect(fn).toHaveBeenCalledTimes(60);
    await vi.advanceTimersByTimeAsync(1);
    await Promise.all(late);
    expect(fn).toHaveBeenCalledTimes(62);

    // those two are now in the window with 58 free slots
    await Promise.all(Array.from({ length: 58 }, () => fetch_uwuowo(req)));
    expect(fn).toHaveBeenCalledTimes(120);
    let done = false;
    fetch_uwuowo(req).then(() => (done = true));
    await vi.advanceTimersByTimeAsync(59_999);
    expect(done).toBe(false);
    await vi.advanceTimersByTimeAsync(1);
    expect(done).toBe(true);
  });
});
