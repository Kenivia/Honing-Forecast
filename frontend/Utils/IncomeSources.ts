import GAME from "@/Data/income.json";
import COMMUNITY from "@/Data/income_community.json";
import {
  ALL_LABELS,
  CONVERTIBLE_MATERIALS,
  SPECIAL_LEAP_LABELS,
} from "./Constants";
import { WEEK_MS } from "./Income";

// The Income page's "Auto" income: what the content a character does gives, from the client
// (scripts/game_files/income.py) and community data (Forecasting/Community.md), as weekly rates
// per band and grants on a date (Crucible stages, event shops).

export type Bag = Record<string, number>;
export interface Item {
  name: string;
  options: { name: string; bag: Bag }[];
}
interface Gate {
  gate: number;
  gold: number[];
  medals: number;
  mats: Bag;
  bonus?: { cost: number; mats: Bag };
}
export interface Raid {
  group: string;
  difficulty: string;
  item_level: number;
  gates: Gate[];
}
interface ShopRow {
  shop: string;
  name: string;
  item_level: number;
  limit: number;
  weekly: boolean;
  cost: number;
  currency: string;
  start?: string;
  end?: string;
  items: Item[];
}
interface Tiered {
  name: string;
  item_level: number;
  bag: Bag;
  gems?: string;
}

export const RAIDS: Raid[] = GAME.raids;
const SHOPS: ShopRow[] = GAME.shops;
const CRUCIBLE = GAME.crucible as {
  end: string;
  stages: { stage: number; opens: string | null; item_level: number; items: Item[] }[];
};
const RAID_SLOTS = 3;
// event tokens per Chaos Dungeon, Kurzan Front, Chaos Rift or guardian clear
const EVENT_TOKENS = 100;

export interface RaidPick {
  group: string;
  difficulty: string;
  bonus: boolean[];
}
export interface SourcesConfig {
  raids: RaidPick[];
  gold_earner: boolean;
  // runs a week, and how many of them are rested (worth double)
  chaos: number;
  chaos_rested: number;
  guardian: number;
  cube: number;
  hourglass: boolean;
  crucible: boolean;
  // the first stages cleared already
  crucible_cleared: number;
  playtime: number;
  weekly_mission: number;
  guild: boolean;
  medal_shop: boolean;
  event_shop: boolean;
  // the option taken per item, by key
  picks: Record<string, number>;
  // shop rows not bought, by shop and name
  off: string[];
}

// the raids an item level enters, newest first, each with its difficulties lowest first
export function raids_for(item_level: number) {
  const out = new Map<string, Raid[]>();
  for (const raid of RAIDS)
    if (raid.item_level <= item_level)
      out.set(raid.group, [...(out.get(raid.group) ?? []), raid]);
  return [...out.values()].map((list) =>
    list.sort((a, b) => a.item_level - b.item_level),
  );
}

// the three newest raids, each on its highest difficulty but Solo
const default_raids = (item_level: number): RaidPick[] =>
  raids_for(item_level)
    .slice(0, RAID_SLOTS)
    .map((list) => {
      const raid =
        list.findLast((x) => x.difficulty !== "Solo") ?? list.at(-1);
      return {
        group: raid.group,
        difficulty: raid.difficulty,
        bonus: raid.gates.map(() => false),
      };
    });

export const config_of = (
  stored: Partial<SourcesConfig>,
  item_level: number,
): SourcesConfig => ({
  raids: default_raids(item_level),
  gold_earner: true,
  chaos: 7,
  chaos_rested: 0,
  guardian: 7,
  cube: 1,
  hourglass: true,
  crucible: true,
  crucible_cleared: 0,
  playtime: 7,
  weekly_mission: 3,
  guild: true,
  medal_shop: true,
  event_shop: true,
  picks: {},
  off: [],
  ...stored,
});

const tier_labels = (tier: number) =>
  new Set([...ALL_LABELS[tier], SPECIAL_LEAP_LABELS[tier]]);

// what a tier gets of a bag: the lower tier's materials converted at their ratio, the higher's
// dropped, as process_chests does
export function to_tier(bag: Bag, tier: number): Bag {
  const labels = tier_labels(tier);
  const out: Bag = {};
  for (const [label, amount] of Object.entries(bag)) {
    const into = CONVERTIBLE_MATERIALS.find(
      (mat) => mat.from.label === label && labels.has(mat.label),
    );
    if (labels.has(label)) out[label] = (out[label] ?? 0) + amount;
    else if (into)
      out[into.label] = (out[into.label] ?? 0) + amount / into.from.ratio;
  }
  return out;
}

const add = (a: Bag, b: Bag, times = 1) => {
  const out = { ...a };
  for (const [label, amount] of Object.entries(b))
    out[label] = (out[label] ?? 0) + amount * times;
  return out;
};

// by default the option with most of the tier's own materials, the first of ties
export function default_pick(item: Item, tier: number) {
  const labels = tier_labels(tier);
  const score = (bag: Bag) =>
    Object.keys(bag).filter((label) => labels.has(label)).length;
  let best = 0;
  item.options.forEach((option, at) => {
    if (score(option.bag) > score(item.options[best].bag)) best = at;
  });
  return best;
}

// items of different levels share their pick
export const pick_key = (source: string, item: Item) =>
  `${source}/${item.name.replace(/\d+/g, "")}`;

// silver only exists roster bound
const split = (bag: Bag, band: number): Bag[] => {
  const out: Bag[] = [{}, {}, {}];
  for (const [label, amount] of Object.entries(bag))
    out[label === "Silver" ? 1 : band][label] = amount;
  return out;
};

const highest = (list: Tiered[], item_level: number) =>
  list.findLast((entry) => entry.item_level <= item_level);

export interface Pick {
  key: string;
  item: Item;
  at: number;
}
export interface Grant {
  at: number;
  bags: Bag[];
}
export interface Source {
  key: string;
  title: string;
  // what it is at this item level, or why it is not counted
  detail: string;
  // per week by band, in the tier's labels
  weekly: Bag[];
  grants: Grant[];
  picks: Pick[];
}

export function auto_income(
  config: SourcesConfig,
  item_level: number,
  tier: number,
  now: number,
) {
  const sources: Source[] = [];
  const pick_of = (key: string, item: Item) =>
    config.picks[key] ?? default_pick(item, tier);
  const items_bag = (source: string, items: Item[], picks: Pick[]) => {
    let bag: Bag = {};
    for (const item of items) {
      const key = pick_key(source, item);
      const at = pick_of(key, item);
      if (item.options.length > 1 && !picks.some((p) => p.key === key))
        picks.push({ key, item, at });
      bag = add(bag, item.options[at]?.bag ?? {});
    }
    return bag;
  };
  const source = (
    key: string,
    title: string,
    detail: string,
    weekly: Bag[] = [{}, {}, {}],
    grants: Grant[] = [],
    picks: Pick[] = [],
  ) =>
    sources.push({
      key,
      title,
      detail,
      weekly: weekly.map((bag) => to_tier(bag, tier)),
      grants: grants.map((grant) => ({
        at: grant.at,
        bags: grant.bags.map((bag) => to_tier(bag, tier)),
      })),
      picks,
    });

  // raids: up to three, gold only for a gold earner, the bonus chest paid from the gate's own gold
  let medals = 0;
  config.raids.forEach((pick, slot) => {
    const raid = RAIDS.find(
      (x) => x.group === pick.group && x.difficulty === pick.difficulty,
    );
    if (!raid) return;
    const weekly: Bag[] = [{}, {}, {}];
    raid.gates.forEach((gate, at) => {
      weekly[0] = add(weekly[0], gate.mats);
      medals += gate.medals;
      const gold = config.gold_earner ? [...gate.gold] : [0, 0, 0];
      if (pick.bonus[at] && gate.bonus) {
        weekly[0] = add(weekly[0], gate.bonus.mats);
        let cost = gate.bonus.cost;
        for (const band of [0, 1, 2]) {
          const paid = band === 2 ? cost : Math.min(cost, gold[band]);
          gold[band] -= paid;
          cost -= paid;
        }
      }
      gold.forEach((amount, band) => {
        if (amount) weekly[band] = add(weekly[band], { Gold: amount });
      });
    });
    const silver = weekly[0].Silver ?? 0;
    delete weekly[0].Silver;
    if (silver) weekly[1] = add(weekly[1], { Silver: silver });
    source(
      `raid ${slot}`,
      `${raid.group} (${raid.difficulty})`,
      `Item level ${raid.item_level}`,
      weekly,
    );
  });

  const chaos = highest(COMMUNITY.chaos, item_level);
  source(
    "chaos",
    chaos?.name ?? "Kurzan Front / Chaos Rift",
    chaos ? `Item level ${chaos.item_level}, Tylobic` : "Needs 1640",
    chaos
      ? split(chaos.bag, 0).map((bag) => add({}, bag, config.chaos + config.chaos_rested))
      : undefined,
  );

  const guardian = highest(COMMUNITY.guardian, item_level);
  source(
    "guardian",
    guardian?.name ?? "Guardian raid",
    guardian
      ? `Item level ${guardian.item_level}, Tylobic; gems ${guardian.gems} a run, not counted`
      : "Needs 1640",
    guardian
      ? split(guardian.bag, 0).map((bag) => add({}, bag, config.guardian))
      : undefined,
  );

  const cube = highest(COMMUNITY.cube, item_level);
  source(
    "cube",
    cube?.name ?? "Ebony Cube",
    cube ? `gcalc / LoaGuard; gems ${cube.gems} a ticket, not counted` : "Needs 1640",
    cube ? split(cube.bag, 0).map((bag) => add({}, bag, config.cube)) : undefined,
  );

  // each sand held adds the base reward once more; the rift drops them
  const hourglass = highest(COMMUNITY.hourglass, item_level);
  const sands = Math.min(
    COMMUNITY.max_sand,
    item_level >= 1730
      ? (config.chaos + config.chaos_rested) * COMMUNITY.sand_per_rift_run
      : 0,
  );
  source(
    "hourglass",
    hourglass?.name ?? "Haal's Hourglass",
    hourglass
      ? `gcalc / LoaGuard, ${sands.toFixed(1)} sands a week; gems ${hourglass.gems}, not counted`
      : "Needs 1730",
    hourglass && config.hourglass
      ? split(hourglass.bag, 0).map((bag) => add({}, bag, 1 + sands))
      : undefined,
  );

  // Crucible stages not cleared yet, each once: those open now in the coming week, the rest the
  // week they open, none after the season
  const crucible_picks: Pick[] = [];
  const end = Date.parse(CRUCIBLE.end);
  const stages = CRUCIBLE.stages.filter(
    (stage) =>
      stage.item_level <= item_level && stage.stage > config.crucible_cleared,
  );
  const crucible_grants = config.crucible
    ? stages
        .map((stage) => ({
          at: Math.max(now, stage.opens ? Date.parse(stage.opens) : 0),
          bags: split(items_bag("crucible", stage.items, crucible_picks), 0),
        }))
        .filter((grant) => grant.at < end)
    : [];
  if (!config.crucible)
    for (const stage of stages) items_bag("crucible", stage.items, crucible_picks);
  source(
    "crucible",
    "Paradise Crucible",
    `${stages.length} stages left this season (to ${CRUCIBLE.end}), each cleared once`,
    undefined,
    crucible_grants,
    crucible_picks,
  );

  const playtime_picks: Pick[] = [];
  const playtime = items_bag("playtime", GAME.playtime, playtime_picks);
  source(
    "playtime",
    "Playtime reward",
    "90 minutes a day; once a roster, so count it on one character",
    split(playtime, 1).map((bag) => add({}, bag, config.playtime)),
    [],
    playtime_picks,
  );

  const mission_picks: Pick[] = [];
  const mission = items_bag("mission", GAME.weekly_mission, mission_picks);
  source(
    "mission",
    "Weekly mission reward",
    "100 points a claim",
    split(mission, 0).map((bag) => add({}, bag, config.weekly_mission)),
    [],
    mission_picks,
  );

  // the guild shop's breath chests and the stones of the tier, bloodstones being free
  const guild = SHOPS.filter(
    (row) =>
      row.shop === "Guild shop" &&
      row.item_level <= item_level &&
      (row.name.startsWith("Tier 4 Breath") ||
        row.name.includes("Crystallized") === (tier === 1)),
  );
  source(
    "guild",
    "Guild shop",
    guild.map((row) => `${row.name} x${row.limit}`).join(", ") || "Needs 1640",
    config.guild
      ? split(
          guild.reduce((bag, row) => add(bag, items_bag("guild", row.items, []), row.limit), {}),
          0,
        )
      : undefined,
  );

  // the medal shop, its rows in order while the raids' medals last; only rows that give the tier's
  // own materials, a lower tier's being worth a fraction
  const labels = tier_labels(tier);
  const shop_rows = (shop: (row: ShopRow) => boolean) =>
    SHOPS.filter(
      (row) =>
        shop(row) &&
        row.item_level <= item_level &&
        Object.keys(items_bag("shop", row.items, [])).some((label) =>
          labels.has(label),
        ),
    );
  const buy = (rows: ShopRow[], budget: number, picks: Pick[]) => {
    let bag: Bag = {};
    let left = budget;
    for (const row of rows) {
      if (config.off.includes(`${row.shop}/${row.name}`)) continue;
      const count = Math.min(row.limit, Math.floor(left / row.cost));
      bag = add(bag, items_bag(row.shop, row.items, picks), count);
      left -= count * row.cost;
    }
    return { bag, used: budget - left };
  };
  const medal_rows = shop_rows((row) => row.shop === "Clear Medal shop");
  const medal_picks: Pick[] = [];
  const medal = buy(medal_rows, medals, medal_picks);
  source(
    "medal",
    "Clear Medal shop",
    `${medal.used.toLocaleString("en-US")} of ${medals.toLocaleString("en-US")} medals a week from the raids`,
    config.medal_shop ? split(medal.bag, 0) : undefined,
    [],
    medal_picks,
  );

  // live event shops: a week's tokens spent each week until the shop closes
  const event_rows = shop_rows(
    (row) => !!row.end && Date.parse(row.end) > now,
  );
  const tokens =
    EVENT_TOKENS * (config.chaos + config.guardian);
  const event_picks: Pick[] = [];
  const event = buy(event_rows, tokens, event_picks);
  const event_end = Math.max(0, ...event_rows.map((row) => Date.parse(row.end)));
  const event_grants: Grant[] = [];
  if (config.event_shop)
    for (let at = now + WEEK_MS; at - WEEK_MS < event_end; at += WEEK_MS)
      event_grants.push({ at: Math.min(at, event_end), bags: split(event.bag, 0) });
  source(
    "event",
    event_rows[0]?.shop ?? "Event shop",
    event_rows.length
      ? `${event.used} of ${tokens} tokens a week, to ${event_rows[0].end}`
      : "None open",
    undefined,
    event_grants,
    event_picks,
  );

  const rates: Bag[] = [{}, {}, {}];
  for (const one of sources)
    one.weekly.forEach((bag, band) => (rates[band] = add(rates[band], bag)));
  const grants = sources.flatMap((one) => one.grants);
  return { sources, rates, grants, medal_rows, event_rows };
}

// what the grants give from just after `from` to `to`, per band
export function granted(grants: Grant[], from: number, to: number): Bag[] {
  const out: Bag[] = [{}, {}, {}];
  for (const grant of grants)
    if (grant.at > from - 1 && grant.at <= to && to > from)
      grant.bags.forEach((bag, band) => (out[band] = add(out[band], bag)));
  return out;
}
