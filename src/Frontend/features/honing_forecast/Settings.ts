import { INPUT_LABELS, OUTPUT_LABELS, STORAGE_KEY, TOP_ROWS, TOP_COLS, BOTTOM_ROWS, BOTTOM_COLS, CELL_W, CELL_H } from "./constants.ts"

export function writeSettings(
    topGrid,
    bottomGrid,
    adv_hone_strategy,
    express_event,
    prev_checked_arr,
    prev_checked_arr_bottom,
    desired_chance,
    budget_inputs,
    autoOptimization,
    userMatsValue,
    cumulativeGraph,
    dataSize
) {
    const toSave = {
        topGrid,
        bottomGrid,
        adv_hone_strategy,
        express_event,
        prev_checked_arr,
        prev_checked_arr_bottom,
        desired_chance,
        budget_inputs,
        autoOptimization,
        userMatsValue,
        cumulativeGraph,
        dataSize,
    }
    localStorage.setItem(STORAGE_KEY, JSON.stringify(toSave))
}
export function readSettings(
    setTopGrid,
    setBottomGrid,
    set_adv_hone_strategy_change,
    set_express_event,
    set_prev_checked_arr,
    set_prev_checked_arr_bottom,
    set_desired_chance,
    set_budget_inputs,
    setAutoOptimization,
    setUserMatsValue,
    setCumulativeGraph,
    setDataSize
) {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return
    const parsed = JSON.parse(raw)
    if (parsed && typeof parsed === "object") {
        if (Array.isArray(parsed.topGrid) && parsed.topGrid.length === TOP_ROWS && parsed.topGrid[0]?.length === TOP_COLS) setTopGrid(parsed.topGrid)
        if (Array.isArray(parsed.bottomGrid) && parsed.bottomGrid.length === BOTTOM_ROWS && parsed.bottomGrid[0]?.length === BOTTOM_COLS)
            setBottomGrid(parsed.bottomGrid)
        if (typeof parsed.adv_hone_strategy === "string") set_adv_hone_strategy_change(parsed.adv_hone_strategy)
        if (typeof parsed.express_event === "boolean") set_express_event(parsed.express_event)
        if (Array.isArray(parsed.prev_checked_arr) && parsed.prev_checked_arr.length === TOP_COLS) set_prev_checked_arr(parsed.prev_checked_arr)
        if (Array.isArray(parsed.prev_checked_arr_bottom) && parsed.prev_checked_arr_bottom.length === BOTTOM_COLS)
            set_prev_checked_arr_bottom(parsed.prev_checked_arr_bottom)
        if (typeof parsed.desired_chance === "string") set_desired_chance(parsed.desired_chance)
        if (parsed.budget_inputs && typeof parsed.budget_inputs === "object") set_budget_inputs(parsed.budget_inputs)
        if (typeof parsed.autoOptimization === "boolean") setAutoOptimization(parsed.autoOptimization)
        if (parsed.userMatsValue && typeof parsed.userMatsValue === "object") setUserMatsValue(parsed.userMatsValue)
        if (typeof parsed.cumulativeGraph === "boolean") setCumulativeGraph(parsed.cumulativeGraph)
        if (typeof parsed.dataSize === "string") setDataSize(parsed.dataSize)
    }
}
