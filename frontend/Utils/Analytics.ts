import { WORKER_URL } from "./Constants";
import { LATEST_VERSION } from "./Changelog";
import type { RosterConfig } from "@/Stores/RosterConfig";
import type { Snapshot } from "./Scans";

// Uploads each roster's budget snapshots not sent yet to the Worker, which looks up the
// characters' recent raids and stores both without the names. Dev builds upload too, tagged
// "dev"; unit tests and browser automation (the e2e tests) never do.

// per request, so the Worker stays within its CPU limit
const BATCH = 20;
// the Worker rejects larger bodies
const MAX_BODY = 300_000;
let uploading = false;

// real builds only, so an unknown mode (node suites, scripts) never uploads
const enabled = () =>
  ["production", "development"].includes(import.meta.env.MODE) &&
  !navigator.webdriver;

const body_of = (config: RosterConfig, roster: string, snapshots: Snapshot[]) =>
  JSON.stringify({
    roster,
    region: config.all_regions[roster],
    version: import.meta.env.DEV ? "dev" : LATEST_VERSION,
    snapshots,
  });

// The oldest unsent snapshots that fit one body. One too large on its own is marked sent
// and never uploaded.
function next_batch(config: RosterConfig, roster: string) {
  const batch: Snapshot[] = [];
  for (const snapshot of config.snapshots) {
    if (snapshot.roster_id !== roster || snapshot.sent) continue;
    if (body_of(config, roster, [...batch, snapshot]).length > MAX_BODY) {
      if (batch.length > 0) break;
      snapshot.sent = true;
      continue;
    }
    batch.push(snapshot);
    if (batch.length === BATCH) break;
  }
  return batch;
}

export async function upload_snapshots(config: RosterConfig) {
  if (uploading || !config.analytics || !enabled()) return;
  uploading = true;
  try {
    for (const roster of new Set(config.snapshots.map((s) => s.roster_id))) {
      for (;;) {
        if (!config.analytics) break;
        const batch = next_batch(config, roster);
        if (batch.length === 0) break;
        const res = await fetch(`${WORKER_URL}/snapshots`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: body_of(config, roster, batch),
        }).catch(() => null);
        if (!res?.ok) return;
        for (const snapshot of batch) snapshot.sent = true;
      }
    }
  } finally {
    uploading = false;
  }
}
