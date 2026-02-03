import React from "react"
import { DEMO_INCOME_1680_ROSTER_BOUND, DEMO_INCOME_1720_CHAR_BOUND, DEMO_UI_VALUES, RESET_UI_DEFAULTS } from "@/Utils/Constants.ts"

const cloneGrid = <T>(grid: T[][]) => grid.map((row) => row.map((cell) => cell))
const cloneStateBundleGrid = (grid: [boolean, number][][][]) =>
    grid.map((row) => row.map((cell) => cell.map((pair) => [pair[0], pair[1]] as [boolean, number])))

export function createClearAll({
    setTopGrid,
    setBottomGrid,
    set_prev_checked_arr,
    set_prev_checked_arr_bottom,
    setUserMatsOwned,
    setUserMatsPrices,
    setUserMatsLeftover,
    setUserWeaponJuiceOwned,
    setUserArmorJuiceOwned,
    setUserWeaponJuicePrices,
    setUserArmorJuicePrices,
    setUserWeaponJuiceLeftover,
    setUserArmorJuiceLeftover,
    set_desired_chance,
    set_adv_hone_strategy_change,
    set_express_event,
    // setAutoGoldValues,
    _setBucketCount,
    setCumulativeGraph,
    setDataSize,
    setLockXAxis,
    setLockedMins,
    setLockedMaxs,
    // setShowAverage,
    setIncomeArr,
    setMinResolution,
    setSpecialState,
    setSucceededGrid,
    setUnlockGrid,
    setStateBundleGrid,
    setProgressGrid,
    setEvaluateAverageResult,
    setBestMetric,
    setBestFlatStateBundle,
    setBestFlatSpecialState: setBestFlatSpecialGrid,
    setBeforeMetric,
}: // setMonteCarloResult,
{
    setTopGrid: React.Dispatch<React.SetStateAction<any>>
    setBottomGrid: React.Dispatch<React.SetStateAction<any>>
    set_prev_checked_arr: React.Dispatch<React.SetStateAction<boolean[]>>
    set_prev_checked_arr_bottom: React.Dispatch<React.SetStateAction<boolean[]>>
    setUserMatsOwned: React.Dispatch<React.SetStateAction<any>>
    setUserMatsPrices: React.Dispatch<React.SetStateAction<any>>
    setUserMatsLeftover: React.Dispatch<React.SetStateAction<any>>
    setUserWeaponJuiceOwned: React.Dispatch<React.SetStateAction<any>>
    setUserArmorJuiceOwned: React.Dispatch<React.SetStateAction<any>>
    setUserWeaponJuicePrices: React.Dispatch<React.SetStateAction<any>>
    setUserArmorJuicePrices: React.Dispatch<React.SetStateAction<any>>
    setUserWeaponJuiceLeftover: React.Dispatch<React.SetStateAction<any>>
    setUserArmorJuiceLeftover: React.Dispatch<React.SetStateAction<any>>
    set_desired_chance: React.Dispatch<React.SetStateAction<string>>
    set_adv_hone_strategy_change: React.Dispatch<React.SetStateAction<string>>
    set_express_event: React.Dispatch<React.SetStateAction<boolean>>
    // setAutoGoldValues: React.Dispatch<React.SetStateAction<boolean>>
    _setBucketCount: React.Dispatch<React.SetStateAction<string>>
    setCumulativeGraph: React.Dispatch<React.SetStateAction<boolean>>
    setDataSize: React.Dispatch<React.SetStateAction<string>>
    setLockXAxis: React.Dispatch<React.SetStateAction<boolean>>
    setLockedMins: React.Dispatch<React.SetStateAction<number[] | null>>
    setLockedMaxs: React.Dispatch<React.SetStateAction<number[] | null>>
    // setShowAverage: React.Dispatch<React.SetStateAction<boolean>>
    setIncomeArr: React.Dispatch<React.SetStateAction<number[][]>>
    setMinResolution: React.Dispatch<React.SetStateAction<number>>
    setSpecialState: React.Dispatch<React.SetStateAction<number[]>>
    setSucceededGrid: React.Dispatch<React.SetStateAction<boolean[][]>>
    setUnlockGrid: React.Dispatch<React.SetStateAction<boolean[][]>>
    setStateBundleGrid: React.Dispatch<React.SetStateAction<[boolean, number][][][]>>
    setProgressGrid: React.Dispatch<React.SetStateAction<number[][]>>
    setEvaluateAverageResult: React.Dispatch<React.SetStateAction<any>>
    setBestMetric: React.Dispatch<React.SetStateAction<number | null>>
    setBestFlatStateBundle: React.Dispatch<React.SetStateAction<[boolean, number][][] | null>>
    setBestFlatSpecialState: React.Dispatch<React.SetStateAction<number[] | null>>
    // setMonteCarloResult: React.Dispatch<React.SetStateAction<any>>
    setBeforeMetric: React.Dispatch<React.SetStateAction<number | null>>
}) {
    return () => {
        // Grids and their column header checkboxes
        setTopGrid(cloneGrid(RESET_UI_DEFAULTS.topGrid))
        setBottomGrid(cloneGrid(RESET_UI_DEFAULTS.bottomGrid))
        set_prev_checked_arr([...RESET_UI_DEFAULTS.prev_checked_arr])
        set_prev_checked_arr_bottom([...RESET_UI_DEFAULTS.prev_checked_arr_bottom])

        // Inputs and toggles to defaults
        setUserMatsOwned({ ...RESET_UI_DEFAULTS.userMatsOwned })
        setUserMatsPrices({ ...RESET_UI_DEFAULTS.userMatsPrices })
        setUserMatsLeftover({ ...RESET_UI_DEFAULTS.userMatsLeftover })
        setUserWeaponJuiceOwned({ ...RESET_UI_DEFAULTS.userWeaponJuiceOwned })
        setUserArmorJuiceOwned({ ...RESET_UI_DEFAULTS.userArmorJuiceOwned })
        setUserWeaponJuicePrices({ ...RESET_UI_DEFAULTS.userWeaponJuicePrices })
        setUserArmorJuicePrices({ ...RESET_UI_DEFAULTS.userArmorJuicePrices })
        setUserWeaponJuiceLeftover({ ...RESET_UI_DEFAULTS.userWeaponJuiceLeftover })
        setUserArmorJuiceLeftover({ ...RESET_UI_DEFAULTS.userArmorJuiceLeftover })
        set_desired_chance(RESET_UI_DEFAULTS.desired_chance)
        set_adv_hone_strategy_change(RESET_UI_DEFAULTS.adv_hone_strategy)
        set_express_event(RESET_UI_DEFAULTS.express_event)
        // setAutoGoldValues(false)
        _setBucketCount(RESET_UI_DEFAULTS.bucketCount)
        setCumulativeGraph(RESET_UI_DEFAULTS.cumulativeGraph)
        setDataSize(RESET_UI_DEFAULTS.dataSize)

        // Reset lock x-axis state
        setLockXAxis(RESET_UI_DEFAULTS.lockXAxis)
        setLockedMins(RESET_UI_DEFAULTS.lockedMins)
        setLockedMaxs(RESET_UI_DEFAULTS.lockedMaxs)

        // Reset show average checkbox
        // setShowAverage(false)

        // Reset income array
        setIncomeArr(cloneGrid(RESET_UI_DEFAULTS.incomeArr))
        setMinResolution(RESET_UI_DEFAULTS.minResolution)
        setSpecialState([...RESET_UI_DEFAULTS.specialState])
        setSucceededGrid(cloneGrid(RESET_UI_DEFAULTS.succeededGrid))
        setUnlockGrid(cloneGrid(RESET_UI_DEFAULTS.unlockGrid))
        setStateBundleGrid(cloneStateBundleGrid(RESET_UI_DEFAULTS.stateBundleGrid))
        setProgressGrid(cloneGrid(RESET_UI_DEFAULTS.progressGrid))
        setEvaluateAverageResult(RESET_UI_DEFAULTS.evaluateAverageResult)
        setBestMetric(null)
        setBestFlatStateBundle(null)
        setBestFlatSpecialGrid(null)
        setBeforeMetric(null)
        // setMonteCarloResult(null)
    }
}

export function createResetOptimizerState({
    setMinResolution,
    setSpecialState,
    setSucceededGrid,
    setUnlockGrid,
    setStateBundleGrid,
    setProgressGrid,
    setEvaluateAverageResult,
    setBestMetric,
    setBestFlatStateBundle,
    setBestFlatSpecialState,
    setBeforeMetric,
}: {
    setMinResolution: React.Dispatch<React.SetStateAction<number>>
    setSpecialState: React.Dispatch<React.SetStateAction<number[]>>
    setSucceededGrid: React.Dispatch<React.SetStateAction<boolean[][]>>
    setUnlockGrid: React.Dispatch<React.SetStateAction<boolean[][]>>
    setStateBundleGrid: React.Dispatch<React.SetStateAction<[boolean, number][][][]>>
    setProgressGrid: React.Dispatch<React.SetStateAction<number[][]>>
    setEvaluateAverageResult: React.Dispatch<React.SetStateAction<any>>
    setBestMetric: React.Dispatch<React.SetStateAction<number | null>>
    setBestFlatStateBundle: React.Dispatch<React.SetStateAction<[boolean, number][][] | null>>
    setBestFlatSpecialState: React.Dispatch<React.SetStateAction<number[] | null>>
    setBeforeMetric: React.Dispatch<React.SetStateAction<number | null>>
}) {
    return () => {
        setMinResolution(RESET_UI_DEFAULTS.minResolution)
        setSpecialState([...RESET_UI_DEFAULTS.specialState])
        setSucceededGrid(cloneGrid(RESET_UI_DEFAULTS.succeededGrid))
        setUnlockGrid(cloneGrid(RESET_UI_DEFAULTS.unlockGrid))
        setStateBundleGrid(cloneStateBundleGrid(RESET_UI_DEFAULTS.stateBundleGrid))
        setProgressGrid(cloneGrid(RESET_UI_DEFAULTS.progressGrid))
        setEvaluateAverageResult(RESET_UI_DEFAULTS.evaluateAverageResult)
        setBestMetric(null)
        setBestFlatStateBundle(null)
        setBestFlatSpecialState(null)
        setBeforeMetric(null)
    }
}

export function createFillDemo({
    setTopGrid,
    setBottomGrid,
    set_desired_chance,
    set_prev_checked_arr,
    set_prev_checked_arr_bottom,
    setUserMatsOwned,
    setUserMatsPrices,
    setUserMatsLeftover,
    setUserWeaponJuiceOwned,
    setUserArmorJuiceOwned,
    setUserWeaponJuicePrices,
    setUserArmorJuicePrices,
    setUserWeaponJuiceLeftover,
    setUserArmorJuiceLeftover,
    setMinResolution,
    setSpecialState,
    setSucceededGrid,
    setUnlockGrid,
    setStateBundleGrid,
    setProgressGrid,
}: {
    setTopGrid: React.Dispatch<React.SetStateAction<any>>
    setBottomGrid: React.Dispatch<React.SetStateAction<any>>
    set_desired_chance: React.Dispatch<React.SetStateAction<string>>
    set_prev_checked_arr: React.Dispatch<React.SetStateAction<boolean[]>>
    set_prev_checked_arr_bottom: React.Dispatch<React.SetStateAction<boolean[]>>
    setUserMatsOwned: React.Dispatch<React.SetStateAction<any>>
    setUserMatsPrices: React.Dispatch<React.SetStateAction<any>>
    setUserMatsLeftover: React.Dispatch<React.SetStateAction<any>>
    setUserWeaponJuiceOwned: React.Dispatch<React.SetStateAction<any>>
    setUserArmorJuiceOwned: React.Dispatch<React.SetStateAction<any>>
    setUserWeaponJuicePrices: React.Dispatch<React.SetStateAction<any>>
    setUserArmorJuicePrices: React.Dispatch<React.SetStateAction<any>>
    setUserWeaponJuiceLeftover: React.Dispatch<React.SetStateAction<any>>
    setUserArmorJuiceLeftover: React.Dispatch<React.SetStateAction<any>>
    setMinResolution: React.Dispatch<React.SetStateAction<number>>
    setSpecialState: React.Dispatch<React.SetStateAction<number[]>>
    setSucceededGrid: React.Dispatch<React.SetStateAction<boolean[][]>>
    setUnlockGrid: React.Dispatch<React.SetStateAction<boolean[][]>>
    setStateBundleGrid: React.Dispatch<React.SetStateAction<[boolean, number][][][]>>
    setProgressGrid: React.Dispatch<React.SetStateAction<number[][]>>
}) {
    return () => {
        setTopGrid(cloneGrid(DEMO_UI_VALUES.topGrid))
        setBottomGrid(cloneGrid(DEMO_UI_VALUES.bottomGrid))

        set_desired_chance(DEMO_UI_VALUES.desired_chance)
        set_prev_checked_arr([...DEMO_UI_VALUES.prev_checked_arr])
        set_prev_checked_arr_bottom([...DEMO_UI_VALUES.prev_checked_arr_bottom])
        setUserMatsOwned({ ...DEMO_UI_VALUES.userMatsOwned })
        setUserMatsPrices({ ...DEMO_UI_VALUES.userMatsPrices })
        setUserMatsLeftover({ ...DEMO_UI_VALUES.userMatsLeftover })
        setUserWeaponJuiceOwned({ ...DEMO_UI_VALUES.userWeaponJuiceOwned })
        setUserArmorJuiceOwned({ ...DEMO_UI_VALUES.userArmorJuiceOwned })
        setUserWeaponJuicePrices({ ...DEMO_UI_VALUES.userWeaponJuicePrices })
        setUserArmorJuicePrices({ ...DEMO_UI_VALUES.userArmorJuicePrices })
        setUserWeaponJuiceLeftover({ ...DEMO_UI_VALUES.userWeaponJuiceLeftover })
        setUserArmorJuiceLeftover({ ...DEMO_UI_VALUES.userArmorJuiceLeftover })
        setMinResolution(DEMO_UI_VALUES.minResolution)
        setSpecialState([...DEMO_UI_VALUES.specialState])
        setSucceededGrid(cloneGrid(DEMO_UI_VALUES.succeededGrid))
        setUnlockGrid(cloneGrid(DEMO_UI_VALUES.unlockGrid))
        setStateBundleGrid(cloneStateBundleGrid(DEMO_UI_VALUES.stateBundleGrid))
        setProgressGrid(cloneGrid(DEMO_UI_VALUES.progressGrid))
    }
}

export function createFillDemoIncome({ setIncomeArr }: { setIncomeArr: React.Dispatch<React.SetStateAction<number[][]>> }) {
    return () => {
        const newIncomeArr = Array.from({ length: 6 }, (_, gridIndex) => {
            if (gridIndex === 0) {
                return [...DEMO_INCOME_1720_CHAR_BOUND]
            }
            return [...DEMO_INCOME_1680_ROSTER_BOUND]
        })
        setIncomeArr(newIncomeArr)
    }
}
