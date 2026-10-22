import { WORKER_URL } from "./Constants";
import { LATEST_VERSION } from "./Changelog";
import type { RosterConfig } from "@/Stores/RosterConfig";

// Uploads each roster's budget snapshots not sent yet to the Worker, which looks up the
// characters' recent raids and stores both without the names. Dev builds upload too, tagged
// "dev"; unit tests and browser automation (the e2e tests) never do.

// per request, so the Worker stays within its CPU limit
const BATCH = 20;
let uploading = false;

// real builds only, so an unknown mode (node suites, scripts) never uploads
const enabled = () =>
  ["production", "development"].includes(import.meta.env.MODE) &&
  !navigator.webdriver;

export async function upload_snapshots(config: RosterConfig) {
  if (uploading || !config.analytics || !enabled()) return;
  uploading = true;
  try {
    for (const roster of new Set(config.snapshots.map((s) => s.roster_id))) {
      for (;;) {
        const batch = config.snapshots
          .filter((s) => s.roster_id === roster && !s.sent)
          .slice(0, BATCH);
        if (batch.length === 0 || !config.analytics) break;
        const res = await fetch(`${WORKER_URL}/snapshots`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            roster: String(roster),
            region: config.all_regions[roster],
            version: import.meta.env.DEV ? "dev" : LATEST_VERSION,
            snapshots: batch,
          }),
        }).catch(() => null);
        if (!res?.ok) return;
        for (const snapshot of batch) snapshot.sent = true;
      }
    }
  } finally {
    uploading = false;
  }
}
