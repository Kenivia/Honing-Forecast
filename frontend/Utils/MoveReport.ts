// Reports how the domain move went to the Worker (moves table), always, whatever the analytics
// opt-out: migrate/index.html parks the outcome in localStorage and the loaded app sends it once
// per outcome per browser. See AI notes "Frontend/Domain move".
import { WORKER_URL } from "./Constants";
import { LATEST_VERSION } from "./Changelog";

const REPORT = "hf_move_report";
const REPORTED = "hf_move_reported";

// real builds only; the e2e tests stub the route
const enabled = () =>
  ["production", "development", "staging"].includes(import.meta.env.MODE);

const read = () => JSON.parse(localStorage.getItem(REPORT) ?? "null");

export async function send_move_report(report: Record<string, unknown>) {
  if (!enabled()) return false;
  const res = await fetch(`${WORKER_URL}/moves`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      ...report,
      site: location.host,
      version:
        import.meta.env.MODE === "production"
          ? LATEST_VERSION
          : import.meta.env.MODE,
    }),
  }).catch(() => null);
  return !!res?.ok;
}

// the receiver's "imported" becomes "moved" or "conflict" once the store has decided
export function settle_import(applied: boolean) {
  const report = read();
  if (report?.outcome !== "imported") return;
  report.outcome = applied ? "moved" : "conflict";
  localStorage.setItem(REPORT, JSON.stringify(report));
}

export async function flush_move_report() {
  const report = read();
  if (!report || !(await send_move_report(report))) return;
  localStorage.removeItem(REPORT);
  const done = JSON.parse(localStorage.getItem(REPORTED) ?? "[]");
  localStorage.setItem(REPORTED, JSON.stringify([...done, report.outcome]));
}
