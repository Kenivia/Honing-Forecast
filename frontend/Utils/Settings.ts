import {
    MATS_LABELS,
    JUICE_LABELS,
    STORAGE_KEY,
    TOP_ROWS,
    TOP_COLS,
    BOTTOM_ROWS,
    BOTTOM_COLS,
    DEFAULT_MATS_LEFTOVER,
    DEFAULT_JUICE_LEFTOVER,
} from "./Constants.ts"

const hasKeySet = (value: unknown, keys: string[]) => {
    if (!value || typeof value !== "object") return false
    return keys.every((key) => Object.prototype.hasOwnProperty.call(value, key))
}

const isStringOrNumberRecord = (value: unknown, keys: string[]) => {
    if (!hasKeySet(value, keys)) return false
    return keys.every((key) => {
        const entry = (value as Record<string, unknown>)[key]
        return typeof entry === "string" || typeof entry === "number"
    })
}

const isGrid = (value: unknown, rows: number, cols: number, cellCheck: (_cell: unknown) => boolean = () => true) => {
    if (!Array.isArray(value) || value.length !== rows) return false
    return value.every((row) => Array.isArray(row) && row.length === cols && row.every((cell) => cellCheck(cell)))
}

type StatePair = [boolean, number]
const isStatePair = (value: unknown): value is StatePair => {
    return Array.isArray(value) && value.length === 2 && typeof value[0] === "boolean" && typeof value[1] === "number"
}

const isStatePairGrid = (value: unknown, rows: number, cols: number) => {
    if (!Array.isArray(value) || value.length !== rows) return false
    return value.every((row) => Array.isArray(row) && row.length === cols && row.every((cell) => Array.isArray(cell) && cell.every(isStatePair)))
}

const MATS_PRICE_LABELS = MATS_LABELS.slice(0, 7)
const JUICE_WEAPON_LABELS = JUICE_LABELS.map((labels) => labels[0])
const JUICE_ARMOR_LABELS = JUICE_LABELS.map((labels) => labels[1])

export function writeSettings(
    topGrid,
    bottomGrid,
    adv_hone_strategy,
    express_event,
    prev_checked_arr,
    prev_checked_arr_bottom,
    desired_chance,
    userMatsOwned,
    userMatsPrices,
    userMatsLeftover,
    userWeaponJuiceOwned,
    userArmorJuiceOwned,
    userWeaponJuicePrices,
    userArmorJuicePrices,
    userWeaponJuiceLeftover,
    userArmorJuiceLeftover,
    cumulativeGraph,
    dataSize,
    incomeArr,
    minResolution,
    inputToggles,
    // specialState,
    // succeededGrid,
    // unlockGrid,
    // stateBundleGrid,
    // progressGrid,
) {
    // console.log("saving")
    const toSave = {
        topGrid,
        bottomGrid,
        adv_hone_strategy,
        express_event,
        prev_checked_arr,
        prev_checked_arr_bottom,
        desired_chance,
        userMatsOwned,
        userMatsPrices,
        userMatsLeftover,
        userWeaponJuiceOwned,
        userArmorJuiceOwned,
        userWeaponJuicePrices,
        userArmorJuicePrices,
        userWeaponJuiceLeftover,
        userArmorJuiceLeftover,
        cumulativeGraph,
        dataSize,
        incomeArr,
        minResolution,
        inputToggles,
        // specialState,
        // succeededGrid,
        // unlockGrid,
        // stateBundleGrid,
        // progressGrid,
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
    setUserMatsOwned,
    setUserMatsPrices,
    setUserMatsLeftover,
    setUserWeaponJuiceOwned,
    setUserArmorJuiceOwned,
    setUserWeaponJuicePrices,
    setUserArmorJuicePrices,
    setUserWeaponJuiceLeftover,
    setUserArmorJuiceLeftover,
    setCumulativeGraph,
    setDataSize,
    setIncomeArr,
    setMinResolution,
    setInputToggles,
    // setSpecialState,
    // setSucceededGrid,
    // setUnlockGrid,
    // setStateBundleGrid,
    // setProgressGrid,
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
        if (isStringOrNumberRecord(parsed.userMatsOwned, MATS_LABELS)) setUserMatsOwned(parsed.userMatsOwned)
        if (isStringOrNumberRecord(parsed.userMatsPrices, MATS_PRICE_LABELS)) setUserMatsPrices(parsed.userMatsPrices)
        // console.log(parsed.userMatsLeftover)
        // if (isStringOrNumberRecord(parsed.userMatsLeftover, MATS_PRICE_LABELS))
        setUserMatsLeftover(Object.fromEntries(MATS_LABELS.map((label) => [label, DEFAULT_MATS_LEFTOVER[label]])))
        if (isStringOrNumberRecord(parsed.userWeaponJuiceOwned, JUICE_WEAPON_LABELS)) setUserWeaponJuiceOwned(parsed.userWeaponJuiceOwned)
        if (isStringOrNumberRecord(parsed.userArmorJuiceOwned, JUICE_ARMOR_LABELS)) setUserArmorJuiceOwned(parsed.userArmorJuiceOwned)
        if (isStringOrNumberRecord(parsed.userWeaponJuicePrices, JUICE_WEAPON_LABELS)) setUserWeaponJuicePrices(parsed.userWeaponJuicePrices)
        if (isStringOrNumberRecord(parsed.userArmorJuicePrices, JUICE_ARMOR_LABELS)) setUserArmorJuicePrices(parsed.userArmorJuicePrices)
        // if (isStringOrNumberRecord(parsed.userWeaponJuiceLeftover, JUICE_WEAPON_LABELS))
        setUserWeaponJuiceLeftover(Object.fromEntries(JUICE_WEAPON_LABELS.map((row) => [row[0], DEFAULT_JUICE_LEFTOVER[0][row[0]]])))
        // if (isStringOrNumberRecord(parsed.userArmorJuiceLeftover, JUICE_ARMOR_LABELS))
        setUserArmorJuiceLeftover(Object.fromEntries(JUICE_WEAPON_LABELS.map((row) => [row[0], DEFAULT_JUICE_LEFTOVER[1][row[1]]])))
        if (typeof parsed.cumulativeGraph === "boolean") setCumulativeGraph(parsed.cumulativeGraph)
        if (typeof parsed.dataSize === "string") setDataSize(parsed.dataSize)

        if (Array.isArray(parsed.incomeArr) && parsed.incomeArr.length === 6 && parsed.incomeArr.every((row) => Array.isArray(row) && row.length === 7))
            setIncomeArr(parsed.incomeArr)
        if (typeof parsed.minResolution === "number") setMinResolution(parsed.minResolution)
        if (
            parsed.inputToggles &&
            typeof parsed.inputToggles === "object" &&
            Array.isArray(parsed.inputToggles.mats) &&
            Array.isArray(parsed.inputToggles.weapon) &&
            Array.isArray(parsed.inputToggles.juice)
        ) {
            setInputToggles(parsed.inputToggles)
        }
        // if (Array.isArray(parsed.specialState) && parsed.specialState.every((value) => typeof value === "number")) setSpecialState(parsed.specialState)
        // if (isGrid(parsed.succeededGrid, TOP_ROWS, TOP_COLS, (cell) => typeof cell === "boolean")) setSucceededGrid(parsed.succeededGrid)
        // if (isGrid(parsed.unlockGrid, TOP_ROWS, TOP_COLS, (cell) => typeof cell === "boolean")) setUnlockGrid(parsed.unlockGrid)
        // if (isStatePairGrid(parsed.stateBundleGrid, TOP_ROWS, TOP_COLS)) setStateBundleGrid(parsed.stateBundleGrid)
        // if (isGrid(parsed.progressGrid, TOP_ROWS, TOP_COLS, (cell) => typeof cell === "number")) setProgressGrid(parsed.progressGrid)
    }
}
