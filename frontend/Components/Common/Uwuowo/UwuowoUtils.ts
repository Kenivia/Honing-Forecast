import { WORKER_URL } from "@/Utils/Constants";
import { parse_locale_int } from "@/Utils/InputColumn";

export type UwuowoRegions = "NA" | "CE";

export interface UwuowoResult {
  pieces: [number, number][]; // [+n, ilevel ]
  class_name: string;
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
  let pieces: [number, number][];
  let class_name: string;
  try {
    console.log(
      allDivs.findLast((el) => el.textContent.trim() === "Equipment"),
    );
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
        return [plus_n, ilevel];
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

    class_name = container?.querySelectorAll("p")[2]?.textContent.trim();
  } catch (e) {
    return `Parsing class name failed with message ${e}`;
  }
  return { pieces, class_name };
}
