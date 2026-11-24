import { WORKER_URL } from "@/Utils/Constants";
import { parse_locale_int } from "@/Utils/InputColumn";
import { MarketRegions } from "@/Utils/MarketDataFetcher";
import { Ref } from "vue";
import {
  BibleCharacter,
  find_linked,
  get_loadouts,
  linked_rosters,
} from "@/Utils/BibleAuth";

export type UwuowoRegions = "NA" | "CE" | null;
export const FETCH_MSG = "Fetching from lostark.bible...";

export function market_to_uwuowo_region(inp: MarketRegions): UwuowoRegions {
  return inp === "nae" ? "NA" : inp === "euc" ? "CE" : null;
}
export interface UwuowoResultBundle {
  result: any;
  status: string;
}
export const DEFAULT_UWUOWO_BUNDLE = {
  result: null,
  status: FETCH_MSG,
};
export function reset_uwuowo_bundle(inp: Ref<any>) {
  inp.value = structuredClone(DEFAULT_UWUOWO_BUNDLE);
}
export interface UwuowoFetchRequest {
  region: UwuowoRegions;
  char_name: string;
  suffix: string;
}
export interface UwuowoPiece {
  plus_n: number;
  ilevel: number;
  tier: number;
  adv: number;
}
// what an import hands its caller to apply
export type ApplyFetch = (
  pieces: UwuowoPiece[],
  force_t4: boolean,
  char_name?: string,
  class_name?: string,
) => void;

// lostark.bible's class ids to the names it shows
const BIBLE_CLASSES: Record<string, string> = {
  warrior: "Warrior",
  berserker: "Berserker",
  destroyer: "Destroyer",
  warlord: "Gunlancer",
  holyknight: "Paladin",
  warrior_female: "Warrior",
  berserker_female: "Slayer",
  holyknight_female: "Valkyrie",
  magician: "Mage",
  arcana: "Arcanist",
  summoner: "Summoner",
  bard: "Bard",
  elemental_master: "Sorceress",
  fighter: "Martial Artist",
  battle_master: "Wardancer",
  infighter: "Scrapper",
  force_master: "Soulfist",
  lance_master: "Glaivier",
  fighter_male: "Martial Artist",
  battle_master_male: "Striker",
  infighter_male: "Breaker",
  delain: "Assassin",
  blade: "Deathblade",
  demonic: "Shadowhunter",
  reaper: "Reaper",
  soul_eater: "Souleater",
  hunter: "Gunner",
  hawk_eye: "Sharpshooter",
  devil_hunter: "Deadeye",
  blaster: "Artillerist",
  scouter: "Machinist",
  hunter_female: "Gunner",
  devil_hunter_female: "Gunslinger",
  specialist: "Specialist",
  yinyangshi: "Artist",
  weather_artist: "Aeromancer",
  alchemist: "Wildsoul",
  specialist_male: "Specialist",
  dimension_master: "Dimensionalist",
  dragon_human: "Guardianknight",
  dragon_knight: "Guardianknight",
};

export interface UwuowoCharResult {
  pieces: UwuowoPiece[]; // [+n, ilevel, tier , adv]; 7th is the vambrace
  class_name: string;
  achieved_ilevel: string;
}

const RATE_LIMIT = 60;
const WINDOW_MS = 60_000;
const request_timestamps: number[] = [];

async function acquire_rate_limit_slot(): Promise<void> {
  while (true) {
    const now = Date.now();
    while (
      request_timestamps.length > 0 &&
      request_timestamps[0] <= now - WINDOW_MS
    ) {
      request_timestamps.shift();
    }

    if (request_timestamps.length < RATE_LIMIT) {
      request_timestamps.push(now);
      return;
    }

    const wait_ms = request_timestamps[0] + WINDOW_MS - now;
    await new Promise((resolve) => setTimeout(resolve, wait_ms));
  }
}

export async function fetch_uwuowo(
  fetch_request: UwuowoFetchRequest,
): Promise<string> {
  await acquire_rate_limit_slot();

  const address = `${WORKER_URL}/character/${fetch_request.region}/${encodeURIComponent(fetch_request.char_name)}/${fetch_request.suffix}`;
  const response = await fetch(address, { method: "GET" });
  if (!response.ok) {
    throw new Error(`Fetch failed, "${await response.text()}"`);
  }
  return await response.text();
}

export async function fetch_and_parse(
  fetch_request: UwuowoFetchRequest,
  parsing_func: (_1: UwuowoFetchRequest, _2: HTMLDivElement[], _3: any) => any,
): Promise<any> {
  let html;
  try {
    html = await fetch_uwuowo(fetch_request);
  } catch (e) {
    return `Fetching failed with message ${e}`;
  }

  const doc = new DOMParser().parseFromString(html, "text/html");

  const allDivs: HTMLDivElement[] = [...doc.querySelectorAll("div")];

  if (
    allDivs.findIndex((el) => el.textContent.includes("Character Not Found")) >=
    0
  ) {
    return `Character: ${fetch_request.char_name} not found (region: ${fetch_request.region})`;
  }
  return parsing_func(fetch_request, allDivs, html);
}

// Signed in and the character is in the user's linked rosters: OAuth API, otherwise public scraping.
export async function fetch_char(
  region: UwuowoRegions,
  char_name: string,
): Promise<string | UwuowoCharResult> {
  const linked = await find_linked(region, char_name);
  if (linked) {
    await acquire_rate_limit_slot();
    try {
      return loadouts_to_result(
        (await get_loadouts(region, linked.character.name)).loadouts,
        linked.character,
      );
    } catch {
      // maybe unlinked meanwhile; falls back to scraping
      linked_rosters(true).catch(() => {});
    }
  }
  return fetch_and_parse({ region, char_name, suffix: "" }, parse_char);
}

export async function fetch_roster(
  region: UwuowoRegions,
  any_char_name: string,
): Promise<string | string[]> {
  const linked = await find_linked(region, any_char_name);
  if (linked) return linked.roster.characters.map((c) => c.name);
  return fetch_and_parse(
    { region, char_name: any_char_name, suffix: "roster" },
    parse_roster,
  );
}

const API_SLOTS = [
  "head",
  "shoulder",
  "upper_body",
  "lower_body",
  "hand",
  "weapon",
];

// newest loadout; Serca ids start 13462, the vambrace ("bracer") is Serca only
export function loadouts_to_result(
  loadouts: any[],
  character: BibleCharacter,
): UwuowoCharResult {
  const newest = loadouts.reduce((a, b) =>
    b.lastUpdated > a.lastUpdated ? b : a,
  );
  const by_slot = (slot: string) =>
    newest.items.find((item: any) => item.slot === slot);
  const pieces: UwuowoPiece[] = API_SLOTS.map((slot) => {
    const item = by_slot(slot);
    const tier = Math.floor(item.id / 10000) === 13462 ? 1 : 0;
    return {
      plus_n: item.data.honing,
      ilevel: NaN,
      tier,
      adv: tier === 0 ? (item.data.advancedHoning ?? 0) : NaN,
    };
  });
  pieces.push({
    plus_n: by_slot("bracer")?.data.honing ?? 0,
    ilevel: NaN,
    tier: 1,
    adv: NaN,
  });
  return {
    pieces,
    class_name: BIBLE_CLASSES[character.class] ?? character.class,
    achieved_ilevel: character.ilvl.toLocaleString("en-US", {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    }),
  };
}

export function parse_roster(
  fetch_request: UwuowoFetchRequest,
  allDivs: HTMLDivElement[],
  _original: any,
): string | string[] {
  // console.log(allDivs);
  try {
    const roster_cards = allDivs.findLast((el) =>
      el.textContent.trim().startsWith(fetch_request.char_name),
    ).parentElement.parentElement.parentElement.children;

    return [...roster_cards].map((child) => {
      const target = child.children[0].children[0];

      return target.textContent.split(" ")[0];
    });
  } catch (e) {
    return `Parsing roster failed with message ${e}`;
  }
}

export function parse_char(
  _fetch_request: UwuowoFetchRequest,
  allDivs: HTMLDivElement[],
  _original: any,
): string | UwuowoCharResult {
  if (allDivs.findIndex((el) => el.textContent.includes("Missing Data")) >= 0) {
    return `Missing data`;
  }
  let pieces: UwuowoPiece[];
  let class_name: string;
  let achieved_ilevel: string;
  try {
    const container = allDivs.findLast(
      (el) => el.textContent.trim() === "Equipment",
    ).parentElement.children[1];

    // armor alternates with accessories in the first 12 cells
    pieces = [...container.children]
      .slice(0, 12)
      .filter((_, i) => i % 2 === 0)
      .map((odd_child) => {
        const target = odd_child.children[1];

        const [top_row, bottom_row] = target.children;

        let plus_n = parse_locale_int(
          // shouldn't really matter what locale but whatever
          top_row.children[1].textContent.replace("+", ""),
        );
        const ilevel = parse_locale_int(bottom_row.children[1].textContent);
        let tier: number;
        let adv: number;
        // console.log(top_row.children);

        if (top_row.children.length > 3 || ilevel < 1730) {
          tier = 0;
          // console.log(top_row.children, top_row.children?.[3]?.textContent);
          adv = parse_locale_int(
            top_row.children?.[3]?.textContent.replace("+", "") ?? "0",
          );
        } else {
          tier = 1;
          adv = NaN;
        }
        if (top_row.children[2].textContent === "T0") {
          plus_n = 25;
        }

        return { plus_n, ilevel, tier, adv };
      });

    // vambrace (Serca only) is found by label; +0 when not equipped
    const vambrace_row = [...container.children]
      .map((cell) => cell.children[1]?.children[0])
      .find((row) => row?.children[0]?.textContent.trim() === "Vambrace");
    pieces.push({
      plus_n: vambrace_row
        ? parse_locale_int(
            vambrace_row.children[1].textContent.replace("+", ""),
          )
        : 0,
      ilevel: NaN,
      tier: 1,
      adv: NaN,
    });
  } catch (e) {
    return `Parsing equipment failed with message ${e}`;
  }

  try {
    const container = allDivs.find((el) => {
      const firstP = el.querySelector("p");
      return (
        firstP?.textContent.trim() === "North America" ||
        firstP?.textContent.trim() === "Central Europe"
      );
    });

    class_name = container.querySelectorAll("p")[2]?.textContent.trim();
  } catch (e) {
    return `Parsing class name failed with message ${e}`;
  }

  try {
    const container = allDivs.find((el) => {
      const firstP = el.querySelector("p");
      return firstP?.textContent.trim() === "Item Level";
    });
    // console.log(container);

    achieved_ilevel = container.parentElement
      .querySelectorAll("p")[2]
      .textContent.trim();
    // console.log(container.parentElement.children, achieved_ilevel);
  } catch (e) {
    return `Parsing item level failed with message ${e}`;
  }
  return { pieces, class_name, achieved_ilevel };
}
