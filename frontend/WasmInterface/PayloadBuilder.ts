import { ALL_LABELS, BUNDLE_SIZE } from "@/Utils/Constants";
import { TreatmentPlan } from "@/Stores/CharacterProfile";
import { OneMaterialInput, Upgrade, WasmOp } from "@/Utils/Interfaces";
import { toRaw } from "vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import { get_upgrade_map, KeyedUpgrades, OneUpgradeInput } from "@/Utils/KeyedUpgrades";
import { input_column_to_num } from "@/Utils/InputColumn";
import { storeToRefs } from "pinia";

// I don't think it's possible to directly export this struct from rust to javascript because of all the vectors,
// so it's copied & pasted here
export interface Payload {
    material_info: number[][][];
    optimizer_plan?: number[];
    upgrade_info: OneUpgradeInput[];
    special_budget: number;
    special_state?: number[];
    tier: number;
    express_event: boolean;
    min_resolution: number;
    num_threads: number;
    metric_type: number;
}

function keyed_to_array(keyed_upgrades: KeyedUpgrades, upgrade_arr: Upgrade[] | null, tier: number): OneUpgradeInput[] {
    const upgrade_map = get_upgrade_map(upgrade_arr, tier);
    return Object.entries(keyed_upgrades).map(([key, one_upgrade_input]) => {
        const upgrade = upgrade_map.get(key) ?? null;
        let out = structuredClone(toRaw(one_upgrade_input));
        if (upgrade && upgrade.state && upgrade.state.length > 0) {
            out.state = upgrade.state;
        }
        return out;
    });
}

export function build_material_info(): OneMaterialInput[] {
    const { active_profile } = storeToRefs(useRosterStore());
    const { roster_config, active_roster_mats_owned, active_tradable_mats_owned } = storeToRefs(useRosterStore());

    const tier = active_profile.value.tier;
    const bound_budgets = input_column_to_num(active_profile.value.bound_budgets[tier]);
    const enabled = active_profile.value.bound_budgets[tier].enabled;
    const roster_mats_owned = input_column_to_num(active_roster_mats_owned.value[tier]);
    const tradable_mats_owned = input_column_to_num(active_tradable_mats_owned.value[tier]);

    const leftover_price = input_column_to_num(active_profile.value.leftover_price[tier]);
    const effective_price =
        tier == 0
            ? input_column_to_num(roster_config.value.mats_prices[tier])
            : roster_config.value.effective_serca_price;

    const tradable_mats_price = input_column_to_num(roster_config.value.mats_prices[tier]).map(
        (x: number, index: number) =>
            Math.max(Math.min(1, x), Math.floor(x * 0.95)) /
            (ALL_LABELS[active_profile.value.tier][index] == "Shards"
                ? roster_config.value.selected_shard_bag_size
                : BUNDLE_SIZE[index]),
    );
    const mats_prices = effective_price.map(
        (x: number, index: number) =>
            x /
            (ALL_LABELS[active_profile.value.tier][index] == "Shards"
                ? roster_config.value.selected_shard_bag_size
                : BUNDLE_SIZE[index]),
    );
    // console.log()
    return ALL_LABELS[tier].map((_, index) => [
        [0, 0],
        [bound_budgets[index], leftover_price[index]],
        [roster_mats_owned[index], tradable_mats_price[index]],
        [!enabled[index] || index == 5 ? 0 : tradable_mats_owned[index], mats_prices[index]], // disabled mats shouldn't be sold either, disregard tradable gold
    ]);
}

export function build_payload(): Payload {
    const { active_profile } = storeToRefs(useRosterStore());
    const tier = active_profile.value.tier;
    return {
        material_info: build_material_info(),
        optimizer_plan:
            // wasm_op == WasmOp.OptimizeAverage
            active_profile.value.optimizer_treatment_plan === TreatmentPlan.TreatRosterAsBound
                ? [0, 0, 2, 3]
                : active_profile.value.optimizer_treatment_plan === TreatmentPlan.TreatTradableAsBound
                  ? [0, 0, 0, 3]
                  : [0, 1, 2, 3], //this  shouldn't happen
        // : null,
        upgrade_info: keyed_to_array(
            active_profile.value.keyed_upgrades,
            active_profile.value.optimizer_worker_bundle.result?.upgrade_arr,
            tier,
        ),
        special_budget: input_column_to_num(active_profile.value.special_budget)[0],
        express_event: active_profile.value.express_event,
        tier,
        min_resolution: active_profile.value.min_resolution,
        num_threads: 1,
        metric_type: 1,
        special_state: toRaw(active_profile.value.optimizer_worker_bundle.result?.special_state),
    };
}
