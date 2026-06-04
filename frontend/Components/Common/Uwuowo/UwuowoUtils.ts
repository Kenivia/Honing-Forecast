import { WORKER_URL } from "@/Utils/Constants";

export type AvailRegions = "NA" | "CE";

export type UwuowoResult = [number, number][] | string; // [+n, ilevel ]

export async function fetch_uwuowo(
  region: AvailRegions,
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
  region: AvailRegions,
  char_name: string,
): Promise<UwuowoResult> {
  let html;
  try {
    html = await fetch_uwuowo(region, char_name);
  } catch (e) {
    return `Fetching https://lostark.bible/character/${region}/${char_name} failed with message ${e}`;
  }

  const doc = new DOMParser().parseFromString(html, "text/html");

  const allDivs = [...doc.querySelectorAll("div")];
  const equipment = allDivs.findLast(
    (el) => el.textContent.trim() === "Equipment",
  );

  if (!equipment) {
    return "Equipment div not found";
  }

  const parent = equipment.parentElement;
  const container = parent.children[1];

  if (!container) {
    return "Container not found";
  }

  const out = [];
  [...container.children]
    .filter((_, i) => i % 2 === 0)
    .forEach((oddChild) => {
      const target = oddChild.children[1];
      if (!target) return;

      console.log(target);
    });
}
