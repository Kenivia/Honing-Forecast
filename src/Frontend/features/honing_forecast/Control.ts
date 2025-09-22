import { INPUT_LABELS, TOP_ROWS, TOP_COLS, BOTTOM_ROWS, BOTTOM_COLS } from './constants.ts'

export function createClearAll({
    setTopGrid,
    setBottomGrid,
    set_prev_checked_arr,
    set_prev_checked_arr_bottom,
    set_budget_inputs,
    setUserMatsValue,
    set_desired_chance,
    set_adv_hone_strategy_change,
    set_express_event,
    setAutoOptimization,
    _setBucketCount,
    setCumulativeGraph,
    setDataSize,
}: {
    setTopGrid: React.Dispatch<React.SetStateAction<any>>
    setBottomGrid: React.Dispatch<React.SetStateAction<any>>
    set_prev_checked_arr: React.Dispatch<React.SetStateAction<boolean[]>>
    set_prev_checked_arr_bottom: React.Dispatch<React.SetStateAction<boolean[]>>
    set_budget_inputs: React.Dispatch<React.SetStateAction<any>>
    setUserMatsValue: React.Dispatch<React.SetStateAction<any>>
    set_desired_chance: React.Dispatch<React.SetStateAction<string>>
    set_adv_hone_strategy_change: React.Dispatch<React.SetStateAction<string>>
    set_express_event: React.Dispatch<React.SetStateAction<boolean>>
    setAutoOptimization: React.Dispatch<React.SetStateAction<boolean>>
    _setBucketCount: React.Dispatch<React.SetStateAction<string>>
    setCumulativeGraph: React.Dispatch<React.SetStateAction<boolean>>
    setDataSize: React.Dispatch<React.SetStateAction<string>>
}) {
    return () => {
        // Grids and their column header checkboxes
        setTopGrid(Array.from({ length: TOP_ROWS }, () => Array(TOP_COLS).fill(false)))
        setBottomGrid(Array.from({ length: BOTTOM_ROWS }, () => Array(BOTTOM_COLS).fill(false)))
        set_prev_checked_arr(Array.from({ length: TOP_COLS }, () => false))
        set_prev_checked_arr_bottom(Array.from({ length: BOTTOM_COLS }, () => false))

        // Inputs and toggles to defaults
        set_budget_inputs(Object.fromEntries(INPUT_LABELS.map((l) => [l, '0'])))
        setUserMatsValue(Object.fromEntries(INPUT_LABELS.slice(0, 7).map((l) => (l == "Gold") ? [l, "1"] : [l, '0'])))
        set_desired_chance('50')
        set_adv_hone_strategy_change('No juice')
        set_express_event(true)
        setAutoOptimization(true)
        _setBucketCount("100")
        setCumulativeGraph(true)
        setDataSize('100000')
    }
}

export function createFillRandom({
    setTopGrid,
    setBottomGrid,
    set_desired_chance,
}: {
    setTopGrid: React.Dispatch<React.SetStateAction<any>>
    setBottomGrid: React.Dispatch<React.SetStateAction<any>>
    set_desired_chance: React.Dispatch<React.SetStateAction<string>>
}) {
    return () => {
        setTopGrid(Array.from({ length: TOP_ROWS }, () => Array.from({ length: TOP_COLS }, () => Math.random() > 0.7)))
        setBottomGrid(Array.from({ length: BOTTOM_ROWS }, () => Array.from({ length: BOTTOM_COLS }, () => Math.random() > 0.7)))
        set_desired_chance((Math.random() * 100).toFixed(2).toString())
    }
}

export function createFillDemo({
    setTopGrid,
    setBottomGrid,
    set_budget_inputs,
    set_desired_chance,
    set_prev_checked_arr,
}: {
    setTopGrid: React.Dispatch<React.SetStateAction<any>>
    setBottomGrid: React.Dispatch<React.SetStateAction<any>>
    set_budget_inputs: React.Dispatch<React.SetStateAction<any>>
    set_desired_chance: React.Dispatch<React.SetStateAction<string>>
    set_prev_checked_arr: React.Dispatch<React.SetStateAction<boolean[]>>
}) {
    return () => {
        setTopGrid(Array.from({ length: TOP_ROWS }, () => Array.from({ length: TOP_COLS }, (_, ind) => ind == 19 || ind == 20 || ind == 21)))
        setBottomGrid(Array.from({ length: BOTTOM_ROWS }, (_, piece) => Array.from({ length: BOTTOM_COLS }, (_, ind) => ind == 3 && piece < 3)))
        set_budget_inputs({
            Red: '431777',
            Blue: '1064398',
            Leaps: '23748',
            Shards: '9010948',
            Oreha: '15125',
            Gold: '1803792',
            'Silver(WIP)': '999999999',
            'Red juice': '420',
            'Blue juice': '690',
            'Special leaps': '6767',
        })
        set_desired_chance('69')
        set_prev_checked_arr(Array.from({ length: TOP_COLS }, (_, ind) => ind == 19 || ind == 20 || ind == 21))
    }
}

