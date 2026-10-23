import { check, done, equal } from "./helpers";
// Taking an option of a choice into the budgets, and the bonus chests as the payload sees them.
import { process_chests } from "@/Utils/Chest";
import {
  affordable,
  as_chest,
  difficulties,
  PARADISE_CHOICES,
  PRESET_CHOICES,
  raids_of,
  shown_difficulty,
  take_option,
} from "@/Utils/Choices";
import { MATERIALS } from "@/Utils/Constants";
import {
  create_input_column,
  input_column_to_num,
  InputType,
} from "@/Utils/InputColumn";

const column = (values: Record<string, number>) =>
  create_input_column(InputType.Int, ["Gold", "Red", "Special Leap"], {
    value: (label) => String(values[label] ?? 0),
  });
const choice = {
  id: "test",
  group: "Test",
  title: "G1",
  band: 0,
  options: [{}, { Gold: -4000, Red: 100, "Special Leap": 3 }],
};
const columns = [
  column({ Gold: 3000 }),
  column({ Gold: 500 }),
  column({ Gold: 1000 }),
];
const special = column({ "Special Leap": 2 });

check(
  "affordable with every band",
  affordable(choice, choice.options[1], columns),
);
take_option(choice, choice.options[1], columns, special);
equal(
  "gold from char bound, roster bound, then tradable",
  columns.map((x) => input_column_to_num(x, true).Gold),
  [0, 0, 500],
);
equal("materials to the band", input_column_to_num(columns[0], true).Red, 100);
equal(
  "free taps to their budget",
  input_column_to_num(special, true)["Special Leap"],
  5,
);
check("not affordable after", !affordable(choice, choice.options[1], columns));

const serca = PRESET_CHOICES.find((x) => x.id === "bonus 223013 2");
equal(
  "a Serca chest for a T4 character: its gold and shards only",
  process_chests([as_chest(serca, 1)], 0).map((x) => [x.band, x.options]),
  [[0, [{ Gold: -10560, Shards: 32200 }]]],
);
equal(
  "a Serca chest for a Serca character: as it is, free taps under the Serca label",
  process_chests([as_chest(serca, 1)], 1).map((x) => x.options[0]),
  [
    {
      Gold: -10560,
      "Serca Blue": 2860,
      "Serca Leaps": 60,
      "Serca Red": 1430,
      "Serca Special Leap": 5,
      Shards: 32200,
    },
  ],
);
const t4_leaps = PRESET_CHOICES.find((x) => x.id === "bonus 202124 2");
equal(
  "T4 special leaps for a Serca character: a fifth, as Serca special leaps",
  process_chests([as_chest(t4_leaps, 1)], 1).map((x) => x.options[0]),
  [
    {
      Gold: -5280,
      "Serca Blue": 960,
      "Serca Leaps": 15.6,
      "Serca Red": 480,
      "Serca Special Leap": 2,
      Shards: 20160,
    },
  ],
);
const t4 = PRESET_CHOICES.find((x) => x.id === "bonus 202124 1");
equal(
  "a T4 chest for a Serca character: converted, gold kept, char bound",
  process_chests([as_chest(t4, 1)], 1).map((x) => [x.band, x.options]),
  [
    [
      0,
      [
        {
          Gold: -4320,
          "Serca Blue": 560,
          "Serca Leaps": 8.8,
          "Serca Red": 280,
          Shards: 11880,
        },
      ],
    ],
  ],
);

const serca_levels = difficulties(
  PRESET_CHOICES.filter((x) => x.group === "Witch of Agony, Serca"),
);
equal(
  "a raid's difficulties, lowest first",
  serca_levels.map((x) => [x.name, x.item_level]),
  [
    ["Normal / Solo", 1710],
    ["Hard", 1730],
    ["Nightmare", 1740],
  ],
);
equal("the highest reached", shown_difficulty(serca_levels, 1735), "Hard");
equal(
  "the lowest when none is",
  shown_difficulty(serca_levels, 0),
  "Normal / Solo",
);
equal(
  "a pick by hand wins",
  shown_difficulty(serca_levels, 1745, "Hard"),
  "Hard",
);
equal(
  "a pick that is gone does not",
  shown_difficulty(serca_levels, 1745, "Extreme"),
  "Nightmare",
);

const raids = raids_of(PRESET_CHOICES).map((x) => x.name);
equal("raids by their highest difficulty, newest first", raids, [
  "Belgardin, Arbiter of Death",
  "Horizon Cathedral",
  "Witch of Agony, Serca",
  "Denouement: Final Day",
  "Act 4: Fortress of Destruction",
  "Act 3: Night of Storms and Darkness",
  "Act 2: Requiem of the Floating Nightmare",
  "Act 1: Earth-Shattering Hellfire",
  "Overture: Spiral of the Crimson Midnight Sun",
  "Darkness Legion Commander",
  "Behemoth, the Storm Commander",
]);

// Paradise: every screen has a name, a plain and a Wealth bag per option, and only material labels
for (const mode of ["Destiny", "Netherworld"]) {
  const screens = PARADISE_CHOICES.filter(
    (x) => x.group === `Paradise: ${mode}`,
  );
  equal(
    `${mode}: item levels`,
    difficulties(screens).map((x) => x.item_level),
    [1580, 1640, 1680, 1700, 1730, 1750],
  );
  equal(`${mode}: Base and ten levels each`, screens.length, 6 * 11);
}
check(
  "Paradise options line up with their names and Wealth",
  PARADISE_CHOICES.every(
    (x) =>
      x.names.length === x.options.length &&
      (x.wealth?.length ?? x.options.length) === x.options.length,
  ),
);
check(
  "Paradise gives materials only",
  PARADISE_CHOICES.every((x) =>
    [...x.options, ...(x.wealth ?? [])].every((bag) =>
      Object.keys(bag).every((label) => label in MATERIALS),
    ),
  ),
);
check(
  "Wealth in Destiny only",
  PARADISE_CHOICES.every(
    (x) => !x.wealth === (x.group === "Paradise: Netherworld"),
  ),
);
equal(
  "Netherworld: the main reward alone",
  PARADISE_CHOICES.find((x) => x.id === "paradise 1125 1").options[0],
  { "Lava's Breath": 50, "Glacier's Breath": 150 },
);
const paradise = PARADISE_CHOICES.find((x) => x.id === "paradise 1028 1");
const fusion = paradise.names.indexOf("Superior Abidos Fusion Material x75");
equal(
  "1750 Destiny Base: a reward and the additional rewards",
  paradise.options[fusion],
  {
    "Serca Fusion": 75,
    Shards: 5200,
    "Serca Red": 26,
    "Serca Blue": 170,
    "Serca Leaps": 4,
  },
);
equal("with Wealth instead", paradise.wealth[fusion], {
  "Serca Fusion": 75,
  Shards: 52000,
  "Serca Red": 260,
  "Serca Blue": 1700,
  "Serca Leaps": 40,
});
check(
  "a stone selection chest is one option per stone",
  paradise.names.filter((x) =>
    x.startsWith("Base Level Destruction/Guardian Stone Selection Chest"),
  ).length === 2,
);

done();
