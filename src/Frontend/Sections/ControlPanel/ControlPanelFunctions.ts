import React from "react"
import { INPUT_LABELS, TOP_ROWS, TOP_COLS, BOTTOM_ROWS, BOTTOM_COLS } from "@/Frontend/Utils/Constants.ts"

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
    setAutoGoldValues,
    _setBucketCount,
    setCumulativeGraph,
    setDataSize,
    setLockXAxis,
    setLockedMins,
    setLockedMaxs,
    setShowAverage,
    setUseGridInput,
    setNormalCounts,
    setAdvCounts,
    setIncomeArr,
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
    setAutoGoldValues: React.Dispatch<React.SetStateAction<boolean>>
    _setBucketCount: React.Dispatch<React.SetStateAction<string>>
    setCumulativeGraph: React.Dispatch<React.SetStateAction<boolean>>
    setDataSize: React.Dispatch<React.SetStateAction<string>>
    setLockXAxis: React.Dispatch<React.SetStateAction<boolean>>
    setLockedMins: React.Dispatch<React.SetStateAction<number[] | null>>
    setLockedMaxs: React.Dispatch<React.SetStateAction<number[] | null>>
    setShowAverage: React.Dispatch<React.SetStateAction<boolean>>
    setUseGridInput: React.Dispatch<React.SetStateAction<boolean>>
    setNormalCounts: React.Dispatch<React.SetStateAction<number[][]>>
    setAdvCounts: React.Dispatch<React.SetStateAction<number[][]>>
    setIncomeArr: React.Dispatch<React.SetStateAction<number[][]>>
}) {
    return () => {
        // Grids and their column header checkboxes
        setTopGrid(Array.from({ length: TOP_ROWS }, () => Array(TOP_COLS).fill(false)))
        setBottomGrid(Array.from({ length: BOTTOM_ROWS }, () => Array(BOTTOM_COLS).fill(false)))
        set_prev_checked_arr(Array.from({ length: TOP_COLS }, () => false))
        set_prev_checked_arr_bottom(Array.from({ length: BOTTOM_COLS }, () => false))

        // Inputs and toggles to defaults
        set_budget_inputs(Object.fromEntries(INPUT_LABELS.map((l) => [l, "0"])))
        setUserMatsValue(
            Object.fromEntries(
                INPUT_LABELS.slice(0, 7).map((l, index) => {
                    const defaultValues = ["1.0", "0.1", "13.0", "0.2", "90.0", "1.0", "0.0"]
                    return [l, defaultValues[index]]
                })
            )
        )
        set_desired_chance("50")
        set_adv_hone_strategy_change("No juice")
        set_express_event(true)
        setAutoGoldValues(false)
        _setBucketCount("100")
        setCumulativeGraph(false)
        setDataSize("100000")

        // Reset lock x-axis state
        setLockXAxis(false)
        setLockedMins(null)
        setLockedMaxs(null)

        // Reset show average checkbox
        setShowAverage(false)

        // Reset grid input checkbox to ticked (default)
        setUseGridInput(true)

        // Reset numeric input states
        setNormalCounts(Array.from({ length: 2 }, () => Array(TOP_COLS).fill(0)))
        setAdvCounts(Array.from({ length: 2 }, () => Array(BOTTOM_COLS).fill(0)))

        // Reset income array
        setIncomeArr(Array.from({ length: 6 }, () => Array.from({ length: 7 }, () => 0)))
    }
}

export function createFillRandom({
    setTopGrid,
    setBottomGrid,
    set_desired_chance,
    set_prev_checked_arr,
    set_prev_checked_arr_bottom,
}: {
    setTopGrid: React.Dispatch<React.SetStateAction<any>>
    setBottomGrid: React.Dispatch<React.SetStateAction<any>>
    set_desired_chance: React.Dispatch<React.SetStateAction<string>>
    set_prev_checked_arr: React.Dispatch<React.SetStateAction<boolean[]>>
    set_prev_checked_arr_bottom: React.Dispatch<React.SetStateAction<boolean[]>>
}) {
    return () => {
        // Generate random grids
        const newTopGrid = Array.from({ length: TOP_ROWS }, () => Array.from({ length: TOP_COLS }, () => Math.random() > 0.7))
        const newBottomGrid = Array.from({ length: BOTTOM_ROWS }, () => Array.from({ length: BOTTOM_COLS }, () => Math.random() > 0.7))

        // Check for full columns in top grid and update prev_checked_arr accordingly
        const newPrevCheckedArr = Array.from({ length: TOP_COLS }, (_, colIndex) => {
            return newTopGrid.every((row) => row[colIndex] === true)
        })

        // Check for full columns in bottom grid and update prev_checked_arr_bottom accordingly
        const newPrevCheckedArrBottom = Array.from({ length: BOTTOM_COLS }, (_, colIndex) => {
            return newBottomGrid.every((row) => row[colIndex] === true)
        })

        setTopGrid(newTopGrid)
        setBottomGrid(newBottomGrid)
        set_prev_checked_arr(newPrevCheckedArr)
        set_prev_checked_arr_bottom(newPrevCheckedArrBottom)
        set_desired_chance((Math.random() * 100).toFixed(0).toString())
    }
}

export function createFillDemo({
    setTopGrid,
    setBottomGrid,
    set_budget_inputs,
    set_desired_chance,
    set_prev_checked_arr,
    setUserMatsValue,
}: {
    setTopGrid: React.Dispatch<React.SetStateAction<any>>
    setBottomGrid: React.Dispatch<React.SetStateAction<any>>
    set_budget_inputs: React.Dispatch<React.SetStateAction<any>>
    set_desired_chance: React.Dispatch<React.SetStateAction<string>>
    set_prev_checked_arr: React.Dispatch<React.SetStateAction<boolean[]>>
    setUserMatsValue: React.Dispatch<React.SetStateAction<any>>
}) {
    return () => {
        setTopGrid(
            Array.from({ length: TOP_ROWS }, (_, row_id) =>
                Array.from({ length: TOP_COLS }, (_, ind) => ind == 19 || ind == 20 || ind == 21 || (ind > 21 && row_id == 5))
            )
        )
        setBottomGrid(Array.from({ length: BOTTOM_ROWS }, (_, piece) => Array.from({ length: BOTTOM_COLS }, (_, ind) => ind == 3 && piece < 3)))
        set_budget_inputs({
            Red: "631777",
            Blue: "1064398",
            Leaps: "33748",
            Shards: "12010948",
            Oreha: "25125",
            Gold: "3803792",
            Silver: "999999999",
            "Red juice": "1420",
            "Blue juice": "690",
            "Special leaps": "6767",
        })
        set_desired_chance("50")
        set_prev_checked_arr(Array.from({ length: TOP_COLS }, (_, ind) => ind == 19 || ind == 20 || ind == 21))

        // Set userMatsValue to the specified values
        setUserMatsValue({
            Red: "1.65",
            Blue: "0.03",
            Leaps: "13.0",
            Shards: "0.5",
            Oreha: "95.0",
            Gold: "1.0",
            Silver: "0.0",
        })
    }
}

export function createFillDemoIncome({ setIncomeArr }: { setIncomeArr: React.Dispatch<React.SetStateAction<number[][]>> }) {
    return () => {
        // Set income array with specified values
        const income_1680_roster_bound = [2606, 7751, 133, 0, 0, 90000, 69420]
        const income_1720_char_bound = [13600, 28160, 594, 360279, 1500, 120000, 69420]

        const newIncomeArr = Array.from({ length: 6 }, (_, gridIndex) => {
            if (gridIndex === 0) {
                return income_1720_char_bound
            } else {
                return income_1680_roster_bound
            }
        })
        setIncomeArr(newIncomeArr)
    }
}
