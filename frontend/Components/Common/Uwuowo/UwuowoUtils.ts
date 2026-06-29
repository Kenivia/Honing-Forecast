import { WORKER_URL } from "@/Utils/Constants";
import { parse_locale_int } from "@/Utils/InputColumn";
import { MarketRegions } from "@/Utils/MarketDataFetcher";
import { Ref } from "vue";

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
export function reset_uwuowo_bundle(inp: Ref<UwuowoResultBundle>) {
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
export interface UwuowoCharResult {
  pieces: UwuowoPiece[]; // [+n, ilevel, tier , adv]
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

export async function fetch_uwuowo(fetch_request): Promise<string> {
  await acquire_rate_limit_slot();

  const address = `${WORKER_URL}/character/${fetch_request.region}/${fetch_request.char_name}/${fetch_request.suffix}`;
  const response = await fetch(address, { method: "GET" });
  if (!response.ok) {
    throw new Error(`Fetch failed, "${await response.text()}"`);
  }
  return await response.text();
}

export async function fetch_and_parse(
  fetch_request: UwuowoFetchRequest,
  parsing_func: (_1: UwuowoFetchRequest, _2: HTMLDivElement[]) => any,
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
  return parsing_func(fetch_request, allDivs);
}

export function parse_roster(
  fetch_request: UwuowoFetchRequest,
  allDivs: HTMLDivElement[],
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

    pieces = [...container.children]
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
          if (top_row.children[2].textContent === "T0") {
            plus_n = 25;
          }
          adv = NaN;
        }

        return { plus_n, ilevel, tier, adv };
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
    return `Parsing class name failed with message ${e}`;
  }
  return { pieces, class_name, achieved_ilevel };
}
