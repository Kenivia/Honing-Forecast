import { marked } from "marked";

export const all_change_logs = import.meta.glob(
  "../../public/change-logs/*.md",
);

function sort_versions(versions: string[]): string[] {
  return [...versions].sort((a, b) => {
    const parse = (v: string) => v.replace(/^v/, "").split(".").map(Number);
    const [aMajor, aMinor, aPatch] = parse(a);
    const [bMajor, bMinor, bPatch] = parse(b);

    return bMajor - aMajor || bMinor - aMinor || bPatch - aPatch;
  });
}

export const ALL_VERSIONS = sort_versions(
  Object.entries(all_change_logs).map(([path]) => {
    return path.replace("../../public/change-logs/", "").replace(".md", "");
  }),
);

export const LATEST_VERSION = ALL_VERSIONS[0];

// the user is caught up; any other version, patch included, shows "new!"
export function version_equal(last_seen: string, now: string) {
  return last_seen.replace(/^v/, "") === now.replace(/^v/, "");
}

// A version's change log as html, cached for the session; null when it could not be fetched.
const html_cache = new Map<string, string>();
export async function load_change_log(version: string): Promise<string | null> {
  if (!html_cache.has(version)) {
    const res = await fetch(
      version === "WIP" ? "/WIP.md" : `/change-logs/${version}.md`,
    ).catch(() => null);
    if (!res?.ok) return null;
    html_cache.set(version, await marked.parse(await res.text()));
  }
  return html_cache.get(version);
}
