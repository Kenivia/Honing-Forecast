import { WORKER_URL } from "@/Utils/Constants";
import { parse_locale_int } from "@/Utils/InputColumn";

export type UwuowoRegions = "NA" | "CE";

export interface UwuowoPiece {
  plus_n: number;
  ilevel: number;
  tier: number;
  adv: number;
}
export interface UwuowoResult {
  pieces: UwuowoPiece[]; // [+n, ilevel, tier , adv]
  class_name: string;
  achieved_ilevel: string;
}

export async function fetch_uwuowo(
  region: UwuowoRegions,
  char_name: string,
): Promise<string> {
  const response = await fetch(
    `${WORKER_URL}/character/${region}/${char_name}`,
    { method: "GET" },
  );

  if (!response.ok) {
    throw new Error(`Character fetch failed: ${response.status}`);
  }

  return await response.text();
}
export async function get_parsed_uwuowo(
  region: UwuowoRegions,
  char_name: string,
): Promise<UwuowoResult | string> {
  let html;
  try {
    html = await fetch_uwuowo(region, char_name);
  } catch (e) {
    return `Fetching https://lostark.bible/character/${region}/${char_name} failed with message ${e}`;
  }

  const doc = new DOMParser().parseFromString(html, "text/html");

  const allDivs = [...doc.querySelectorAll("div")];

  if (
    allDivs.findIndex((el) => el.textContent.includes("Character Not Found")) >=
    0
  ) {
    return `Character not found`;
  }

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
      .map((oddChild) => {
        const target = oddChild.children[1];

        const [top_row, bottom_row] = target.children;

        const plus_n = parse_locale_int(
          // shouldn't really matter what locale but whatever
          top_row.children[1].textContent.replace("+", ""),
        );
        const ilevel = parse_locale_int(bottom_row.children[1].textContent);
        let tier: number;
        let adv: number;
        // console.log(top_row.children);
        if (top_row.children.length > 3) {
          tier = 0;
          adv = parse_locale_int(
            top_row.children[3]?.textContent.replace("+", ""),
          );
        } else {
          tier = 1;
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
