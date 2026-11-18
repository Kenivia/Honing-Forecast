// Serca normal honing for the info page demo, mirroring Rust's probability_distribution.
import SERCA from "../../../../crates/core/src/constants/Serca Oct 2026.json";
import { ALL_LABELS, NUM_BASE_MATS, RATE_SCALE } from "@/Utils/Constants";

export const PIECE_TYPES = ["Armor", "Weapon", "Vambrace"];
const FUSION = 4;
const COSTS = [
  SERCA.NORMAL_ARMOR_COST,
  SERCA.NORMAL_WEAPON_COST,
  SERCA.NORMAL_VAMBRACE_COST,
];
export const NUM_LEVELS = SERCA.NORMAL_CHANCES[0].length;

export interface DemoUpgrade {
  piece_type: number;
  upgrade_index: number;
  // juice id -> used on the first n taps
  juice: Record<number, number>;
}

export interface Juice {
  id: number;
  label: string;
  // out of RATE_SCALE, and as a chance
  rate: number;
  chance: number;
}

export const upgrade_name = (u: DemoUpgrade) =>
  `${PIECE_TYPES[u.piece_type]} +${u.upgrade_index + 1}`;

export const fusion_per_tap = (u: DemoUpgrade) =>
  COSTS[u.piece_type][FUSION][u.upgrade_index];

export function juices(piece_type: number, upgrade_index: number): Juice[] {
  return SERCA.JUICE_BOOKS_AVAIL.filter(
    (r) => r[1] === 0 && r[2] === piece_type && r[3] === upgrade_index + 1,
  ).map((r) => ({
    id: r[0],
    label: ALL_LABELS[1][NUM_BASE_MATS + r[0]],
    rate: r[4],
    chance: r[4] / RATE_SCALE,
  }));
}

// dist[n] = chance to succeed on tap n (dist[0] = 0)
export function distribution(u: DemoUpgrade): number[] {
  // the game's tables are out of 10,000
  const at = (table: number[][]) =>
    table[u.piece_type][u.upgrade_index] * (RATE_SCALE / 10000);
  const base = at(SERCA.NORMAL_CHANCES);
  const bonus = at(SERCA.NORMAL_FAIL_BONUS);
  const bonus_max = at(SERCA.NORMAL_FAIL_BONUS_MAX);
  const threshold = at(SERCA.ARTISAN_THRESHOLD);
  const avail = juices(u.piece_type, u.upgrade_index);
  const raw = [0];
  let artisan = 0;
  for (let count = 0; ; count++) {
    if (artisan >= threshold) {
      raw.push(1);
      break;
    }
    const extra = avail.reduce(
      (s, j) => s + (count < (u.juice[j.id] ?? 0) ? j.rate : 0),
      0,
    );
    const rate = Math.min(
      RATE_SCALE,
      base + Math.min(count * bonus, bonus_max) + extra,
    );
    raw.push(rate / RATE_SCALE);
    artisan += rate;
    if (rate === RATE_SCALE) break;
  }
  let left = 1;
  return raw.map((c) => {
    const p = left * c;
    left *= 1 - c;
    return p;
  });
}

// Pity with nothing used, the most taps a juice can be put on.
export const max_taps = (u: DemoUpgrade) =>
  distribution({ ...u, juice: {} }).length - 1;
