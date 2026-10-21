import BONUS_CHESTS from "@/Data/bonus_chests.json";
import PARADISE from "@/Data/paradise.json";
import type { Bag, ManifestChest } from "./Chest";
import { SPECIAL_LEAP_LABELS } from "./Constants";
import { input_column_to_num, set_cell, type InputColumn } from "./InputColumn";

// A choice between options, each what it gives by label, negative for what it costs. The preset
// ones are made from the game files (scanner_setup/game_files/bonus_chests.py); one the user makes
// is the same.
export interface Choice {
  id: string;
  group: string;
  title: string;
  band: number;
  options: Bag[];
  // what it also gives that is no material
  others?: string[];
  // a raid's difficulty and the item level it asks for; a group shows one difficulty at a time
  difficulty?: string;
  item_level?: number;
  // what each option is, when it is not the choice itself
  names?: string[];
  // the options with Paradise's Wealth
  wealth?: Bag[];
}

export const PRESET_CHOICES: Choice[] = BONUS_CHESTS;

// the user's own choice, one per roster: what each option gives, by name
export interface CustomChoice {
  band: number;
  names: string[];
  options: Bag[];
}
export const default_custom_choice = (): CustomChoice => ({
  band: 0,
  names: ["Option 1", "Option 2"],
  options: [{}, {}],
});
// as a choice, without the amounts left at 0
export const as_custom = (custom: CustomChoice): Choice => ({
  id: "custom",
  group: "Custom",
  title: "Custom",
  band: custom.band,
  names: custom.names,
  options: custom.options.map((bag) =>
    Object.fromEntries(Object.entries(bag).filter(([, amount]) => amount)),
  ),
});
// Paradise's rewards (scripts/game_files/paradise.py), one choice per mode, item level and level
export const PARADISE_CHOICES: Choice[] = PARADISE;

export interface Difficulty {
  name: string;
  item_level: number;
}

// the difficulties of a group's choices, lowest item level first
export const difficulties = (choices: Choice[]): Difficulty[] =>
  [
    ...new Map(
      choices
        .filter((choice) => choice.difficulty)
        .map((choice) => [
          choice.difficulty,
          { name: choice.difficulty, item_level: choice.item_level ?? 0 },
        ]),
    ).values(),
  ].sort((a, b) => a.item_level - b.item_level);

// a raid's highest difficulty, which sorts raids by release, newest first
const highest_level = (list: Difficulty[]) => list.at(-1)?.item_level ?? 0;

// the choices by group with their difficulties, newest raid first; ties keep the data's order
export const raids_of = (choices: Choice[]) =>
  [...Map.groupBy(choices, (choice) => choice.group)]
    .map(([name, members]) => ({
      name,
      members,
      difficulties: difficulties(members),
    }))
    .sort(
      (a, b) => highest_level(b.difficulties) - highest_level(a.difficulties),
    );

// a worth rounded to whole gold, signed
export function signed_gold(amount: number) {
  const rounded = Math.round(amount);
  return `${rounded > 0 ? "+" : rounded < 0 ? "−" : ""}${Math.abs(rounded).toLocaleString("en-US")}`;
}

// an option's worth against the run without: "N/A" with nothing to hone, empty until run (a spinner)
export const worth_text = (no_upgrades: boolean, worth?: number) =>
  no_upgrades ? "N/A" : worth === undefined ? "" : signed_gold(worth);
// as shown, so what reads 0 is neither
export const worth_color = (no_upgrades: boolean, worth?: number) =>
  no_upgrades || worth === undefined
    ? "var(--text-muted)"
    : Math.round(worth) > 0
      ? "var(--achieved)"
      : Math.round(worth) < 0
        ? "var(--warning)"
        : "var(--text-main)";

// The one picked by hand if it is still there, else the highest the item level reaches, else the
// lowest.
export function shown_difficulty(
  list: Difficulty[],
  item_level: number,
  picked?: string,
) {
  if (list.some((x) => x.name === picked)) return picked;
  return (
    list.findLast((x) => x.item_level <= item_level)?.name ?? list[0]?.name
  );
}

export const is_empty = (option: Bag) => !Object.keys(option).length;

// an option taken, as a chest for the payload
export const as_chest = (choice: Choice, option: number): ManifestChest => ({
  key: "",
  title: choice.title,
  band: choice.band,
  count: 1,
  options: [choice.options[option]],
  obtain_all: true,
});

// what is owned of a label in the choice's band and the looser ones
const available = (columns: InputColumn[], band: number, label: string) =>
  columns
    .slice(band)
    .reduce(
      (sum, column) => sum + (input_column_to_num(column, true)[label] ?? 0),
      0,
    );

export const affordable = (
  choice: Choice,
  option: Bag,
  columns: InputColumn[],
) =>
  Object.entries(option).every(
    ([label, amount]) =>
      amount >= 0 || available(columns, choice.band, label) >= -amount,
  );

// Adds an option to the budgets (char bound, roster bound, tradable): what it gives to the choice's
// band, free taps to their own budget, and what it costs from its band, then the looser ones.
export function take_option(
  choice: Choice,
  option: Bag,
  columns: InputColumn[],
  special: InputColumn,
) {
  for (const [label, amount] of Object.entries(option)) {
    if (SPECIAL_LEAP_LABELS.includes(label)) {
      const owned = input_column_to_num(special, true)[label] ?? 0;
      set_cell(special, label, String(owned + amount));
      continue;
    }
    let left = amount;
    for (const column of columns.slice(choice.band)) {
      const owned = input_column_to_num(column, true)[label] ?? 0;
      const change = left >= 0 ? left : Math.max(left, -owned);
      set_cell(column, label, String(owned + change));
      left -= change;
      if (!left) break;
    }
  }
}
