# Honing data in the game files

Where the constants JSONs' numbers are in the installed game, and what is still not understood. The tables are the SQLite tables of `EFGame/data2.lpk` (`Screen scanner/Game files.md` has how they are read); `python scripts/game_files/honing.py` is the generator (`../Domain model.md`, game data). The client only holds what the installed version has.

## Normal honing: all there

`ItemEnhanceCommon`, one group per gear set and piece type (411100 Serca weapon, 411101 Serca armour, 411102 Serca vambrace, 410500 T4 weapon, 410501 T4 armour; T4 has no vambrace), `SecondaryKey` 100 plus the level:

- `LevelUpSuccessRate`, `LevelUpFailureBonusRate`, `LevelUpFailureBonusMaxRate`, `LevelUpGuaranteeThreshold` out of 10,000; the juice (`LevelUpAdditiveId0k`, `Rate0k` a unit, `Max0k` units, `MaxRate` the cap over all) out of 100,000. The vambrace takes both juices, up to its max each.
- Shards, silver and gold in `LevelUpCostType/Amount` (type 18 shards, 1 silver, 2 gold).
- Unlock shards as the difference of `AccExp` between levels, unlock silver as that times `FeedSubCost`.
- Free-tap leapstones in `SpecialEnhanceMaterialAmount`.
- Stones, leapstones and fusion in `ItemQualityOption`, the rows whose `EnhanceCommonId` is the group (many items per group, all alike). Books in `ItemEnhanceMaterial` under the level's `SpecialMaterialGroupId`, one row per book item (event, bound and tradable copies), `EffectType00` 1, the chance in `EffectArg00` out of 10,000.

**Advanced honing** is in `ItemAmplificationBase` (unlocks per ten levels) and `ItemAmplificationMaterial` (costs, success and great-success rates, juice, scrolls). Only armour was looked at.

## Open: the discount layer

The base tables hold undiscounted costs; the JSON has the discounted ones.

- T4 normal honing gold in the base table is higher than the JSON's for the low levels (equal from +19). The JSON's number is in `WelcomeBoostEnhance`, which `ItemQualityOption.WelcomeBoostId` points at and which repeats every cost of the level.
- **The rates are not discounted**: `WelcomeBoostEnhance`'s `LevelUpSuccessRateBonus` and `LevelUpFailureBonusMultiply` are 0 and 1 at every T4 level, so the generator takes the base table's T4 rates. The sheet's T4 rates had been Serca's.
- T4 advanced honing stages 1 and 2 are undiscounted in the base table (Guardian Stones, shards). `ItemAmplificationBoost` has discount rates per material and `ItemAmplificationBase` names boost ids, but which row gives the JSON's numbers was not traced.
- Rows are gated by `ClassifyType`/`ClassifyIndex`, and the client tables alone do not say which gates are open. With no event running there is nothing to check the `EVENT_*` rows against.

Until that is understood, generating T4's costs from the game would get the permanent discounts wrong. Serca has no `WelcomeBoostId`, and its game numbers are the JSON's.

## Integer rates

maxroll.gg's calculator (its decompiled wasm and JS fallback) does what the core does: integers out of 10,000, certain at `artisan >= threshold`, the fail bonus and its cap from the tables. The old floats sometimes missed an artisan landing exactly on the threshold by one tap. 100,000 is finer than anything modelled needs: only one unit of a Serca vambrace juice (375 or 25) is not a whole /10,000, and the model never uses part of a stack. T4 +1 to +3 are 10,000 with a bonus of 0, so the bonus is not always a tenth of the base.
