import { done, equal } from "./helpers";
// The scanned chests as a character of one tier takes them.
import { type Bag, process_chests } from "@/Utils/Chest";

const chest = (title: string, band: number, count: number, options: Bag[]) => ({
  key: title,
  title,
  band,
  count,
  options,
  obtain_all: false,
});
const brief = (chests: ReturnType<typeof process_chests>) =>
  chests.map((x) => [x.title, x.band, x.count, x.options]);

const scanned = [
  chest("Rocks", 2, 3, [{ Red: 250 }, { "Serca Red": 20 }]),
  chest("Leaps", 1, 2, [{ Leaps: 3 }, { Shards: 1000 }]),
  chest("More leaps", 1, 5, [{ Shards: 1000 }, { Leaps: 3 }]),
  chest("Twins", 0, 1, [{ "Serca Blue": 5 }, { "Serca Blue": 5 }]),
  chest("Gold", 2, 1, [{ Gold: 100 }]),
];

equal(
  "T4: nothing of Serca, tradable gold dropped",
  brief(process_chests(scanned, 0)),
  [
    ["Rocks", 2, 3, [{ Red: 250 }]],
    ["Leaps / More leaps", 1, 7, [{ Leaps: 3 }, { Shards: 1000 }]],
  ],
);

equal(
  "Serca: converted, roster bound, the worse option gone",
  brief(process_chests(scanned, 1)),
  [
    ["Rocks", 1, 3, [{ "Serca Red": 50 }]],
    ["Leaps / More leaps", 1, 7, [{ "Serca Leaps": 0.6 }, { Shards: 1000 }]],
    ["Twins", 0, 1, [{ "Serca Blue": 5 }]],
  ],
);

done();
