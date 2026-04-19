import { InputColumn, InputType, KeyedUpgrades, StatusGrid } from "@/Utils/Interfaces"
import { createWorkerBundle } from "@/WasmInterface/WorkerBundle"
import { ADV_COLS, ALL_LABELS, NORMAL_COLS, NUM_PIECES, SPECIAL_LEAP_LABEL } from "@/Utils/Constants"
import { create_input_column } from "@/Utils/InputColumn"
import { createStatusGrid } from "@/Utils/StatusGrid"

export interface CharProfile {
    roster_id: number

    optimizer_treatment_plan: TreatmentPlan
    histogram_treatment_plan: TreatmentPlan
    express_event: boolean
    char_name: string

    evaluation_worker_bundle: any
    optimizer_worker_bundle: any
    histogram_worker_bundle: any
    normal_grid: StatusGrid
    adv_grid: StatusGrid

    keyed_upgrades: KeyedUpgrades // see Interface for the definition of these

    special_budget: InputColumn // just a 1 cell column

    bound_budgets: InputColumn[] // bound_budgets[tier].data[row] = "123" (data is string even though we don't directly modify it, should change this to number at some point ig)
    leftover_price: InputColumn[] // The tier distinction is because there's different number of mats (rows) for each tier

    auto_start_optimizer: boolean
    tier: number
    min_resolution: number // currently not used (always 1)
    num_threads: number // currently not used (always 1)
    metric_type: number // currently not used (always 1)
    material_re_render_trigger: boolean // This is here to trigger an update in the special cell in MaterialDist from the change in the confirmation popup in InstructionRow
}

export enum TreatmentPlan {
    TreatRosterAsTradable, // rat alt, treat roster as if we could've sold them
    TreatRosterAsBound, // alt, treat char & roster bound as 0 if there's any leftover, taxed market price if any tradable leftover
    TreatTradableAsBound, // main, treat everything as 0 if any leftover
    TreatAllAsTradable, //special case for gold cost
}

export function create_default_char_profile(): CharProfile {
    return {
        optimizer_treatment_plan: TreatmentPlan.TreatRosterAsBound,
        histogram_treatment_plan: TreatmentPlan.TreatRosterAsTradable,
        express_event: false,
        char_name: "Newchar",

        auto_start_optimizer: true,
        evaluation_worker_bundle: createWorkerBundle(),
        optimizer_worker_bundle: createWorkerBundle(),
        histogram_worker_bundle: createWorkerBundle(),

        normal_grid: createStatusGrid(NUM_PIECES, NORMAL_COLS),
        adv_grid: createStatusGrid(NUM_PIECES, ADV_COLS),

        keyed_upgrades: {},
        special_budget: create_input_column(InputType.Int, [SPECIAL_LEAP_LABEL], ["0"], [33333]),

        bound_budgets: ALL_LABELS.map((this_labels) => create_input_column(InputType.Int, this_labels)),
        leftover_price: ALL_LABELS.map((this_labels) => create_input_column(InputType.Int, this_labels)), // implicit 0 leftover here, currently UI does not allow changing this

        tier: 0,
        min_resolution: 1,
        num_threads: 1,
        metric_type: 1,
        material_re_render_trigger: true,
        roster_id: 0,
    }
}

// Worker bundles are not writable to string(and prolly shouldnt anyway), we re-make them on load
export function recreate_char_profile(parsed): CharProfile {
    return {
        ...parsed,
        evaluation_worker_bundle: createWorkerBundle(),
        optimizer_worker_bundle: createWorkerBundle(),
        histogram_worker_bundle: createWorkerBundle(),
    }
}
