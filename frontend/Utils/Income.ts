import { ALL_LABELS, SPECIAL_LEAP_LABELS } from "./Constants";
import { chest_gains, process_chests } from "./Chest";
import type { Snapshot, SnapshotResult } from "./Scans";
import { merge_lower_tier } from "@/WasmInterface/PayloadBuilder";

// Income per week from budget snapshots: what one character has per band at each snapshot, and a
// least squares line through it.

export const WEEK_MS = 7 * 24 * 3600 * 1000;
// a weekly reset, Thursday 10:00 UTC; both regions reset together
const RESET = Date.UTC(2026, 0, 1, 10);
// the least past and future the graph shows
export const MIN_SPAN = 5 * WEEK_MS;

export function resets(from: number, to: number) {
  const out: number[] = [];
  for (
    let at = RESET + Math.ceil((from - RESET) / WEEK_MS) * WEEK_MS;
    at <= to;
    at += WEEK_MS
  )
    out.push(at);
  return out;
}

// per label of the tier, what a character owns per band, the lower tier converted up, plus what
// the chests give; free taps under the special leap label, all char bound
export type Owned = Record<string, number[]>;

export function char_chests(snapshot: Snapshot, char_id: string) {
  return [
    ...(snapshot.chars[char_id]?.chests ?? []),
    ...snapshot.roster_chests,
  ];
}

export function owned_at(
  snapshot: Snapshot,
  char_id: string,
  tier: number,
  result?: SnapshotResult,
): Owned {
  const char = snapshot.chars[char_id];
  const merged = merge_lower_tier(
    tier,
    char?.bound ?? {},
    snapshot.roster_owned,
    snapshot.tradable,
  );
  // without an optimizer run, every chest on its first option as Rust does
  const gains =
    result?.gains ??
    chest_gains(process_chests(char_chests(snapshot, char_id), tier));
  const out: Owned = {};
  for (const label of ALL_LABELS[tier]) {
    out[label] = [merged.bound, merged.roster, merged.tradable].map(
      (column, band) => (column[label] ?? 0) + (gains[label]?.[band] ?? 0),
    );
  }
  const special = SPECIAL_LEAP_LABELS[tier];
  out[special] = [
    (merge_lower_tier(tier, char?.special ?? {}, {}, {}).bound[special] ?? 0) +
      (gains[special] ?? []).reduce((a, b) => a + b, 0),
    0,
    0,
  ];
  return out;
}

export const value_of = (owned: Owned, label: string, band: number) =>
  owned[label]?.[band] ?? 0;

export interface Point {
  x: number;
  y: number;
  // starts a new line
  breaks?: boolean;
}

// One least squares slope through runs of points, each run with its own intercept. Null without
// two distinct times in a run.
export function fit(points: Point[]) {
  const runs: Point[][] = [];
  for (const point of points) {
    if (!runs.length || point.breaks) runs.push([]);
    runs[runs.length - 1].push(point);
  }
  let sxx = 0;
  let sxy = 0;
  for (const run of runs) {
    const mx = run.reduce((a, p) => a + p.x, 0) / run.length;
    const my = run.reduce((a, p) => a + p.y, 0) / run.length;
    for (const p of run) {
      sxx += (p.x - mx) ** 2;
      sxy += (p.x - mx) * (p.y - my);
    }
  }
  return sxx ? sxy / sxx : null;
}

// Cells that do not exist in game: silver is roster bound only, free taps char bound only
// (owned_at counts the rare roster-bound chest's special leaps as char bound).
export const has_band = (label: string, band: number) =>
  label === "Silver"
    ? band === 1
    : SPECIAL_LEAP_LABELS.includes(label)
      ? band === 0
      : true;

// what the estimate uses
export const used = (snapshot: Snapshot) =>
  snapshot.kind !== "Before export" && !snapshot.ignored;

// What runs, oldest first, each from the state of the one before: every snapshot but the ignored
// ones (unless selected), then `after` (now and the weeks ahead).
export function chain_of<T extends { snapshot: Snapshot }>(
  entries: T[],
  selected: Snapshot | null,
  after: T[],
): T[] {
  return [
    ...entries
      .filter((entry) => !entry.snapshot.ignored || entry.snapshot === selected)
      .sort((a, b) => a.snapshot.at - b.snapshot.at),
    ...after,
  ];
}

// the first of the chain not done, and the latest state of the session before it
export function next_in_chain<
  T extends { job: { hash: string; result?: unknown } },
>(chain: T[], state_of: (hash: string) => unknown) {
  const at = chain.findIndex((entry) => !entry.job.result);
  if (at < 0) return null;
  const from = chain
    .slice(0, at)
    .map((entry) => state_of(entry.job.hash))
    .findLast(Boolean);
  return { entry: chain[at], from };
}

// `now` with `weeks` of income added, rates per band by label; free taps are char bound
export function projected(
  now: Snapshot,
  char_id: string,
  special: string,
  rates: Record<string, number>[],
  weeks: number,
): Snapshot {
  const out: Snapshot = JSON.parse(JSON.stringify(now));
  const char = out.chars[char_id];
  const add = (record: Record<string, number>, label: string, rate: number) =>
    (record[label] = Math.max(
      0,
      Math.round((record[label] ?? 0) + rate * weeks),
    ));
  for (const [label, rate] of Object.entries(rates[0]))
    add(label === special ? char.special : char.bound, label, rate);
  for (const [label, rate] of Object.entries(rates[1]))
    add(out.roster_owned, label, rate);
  for (const [label, rate] of Object.entries(rates[2]))
    add(out.tradable, label, rate);
  out.at = now.at + weeks * WEEK_MS;
  return out;
}

// the graph's x range: centred on now, the larger of MIN_SPAN and the oldest's age either side
export function range_of(oldest: number | undefined, now: number) {
  const half = Math.max(MIN_SPAN, now - (oldest ?? now));
  return [now - half, now + half] as [number, number];
}

// steps between consecutive points within runs, and the days they span
export function steps_of(points: { at: number; breaks?: boolean }[]) {
  let steps = 0;
  let days = 0;
  points.forEach((point, at) => {
    if (at === 0 || point.breaks) return;
    steps += 1;
    days += (point.at - points[at - 1].at) / 864e5;
  });
  return steps ? { steps, days } : null;
}
