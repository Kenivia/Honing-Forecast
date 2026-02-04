import React, { useEffect, useRef, useState, useMemo, useCallback } from "react"
import "./Sections/UpgradeSelection/CheckboxRow.css"
import { styles } from "./Utils/Styles.ts"
import {
    MATS_LABELS,
    TOP_ROWS,
    TOP_COLS,
    BOTTOM_ROWS,
    BOTTOM_COLS,
    DEFAULT_MATS_PRICES,
    DEFAULT_JUICE_PRICES,
    JUICE_LABELS,
    PIECE_NAMES,
    RESET_UI_DEFAULTS,
} from "./Utils/Constants.ts"
import { readSettings, writeSettings } from "./Utils/Settings.ts"
import ControlPanel from "./Sections/ControlPanel/ControlPanel.tsx"
import NormalHoningPanel from "./Sections/UpgradeSelection/NormalHoningPanel.tsx"
import AdvancedHoningPanel from "./Sections/UpgradeSelection/AdvancedHoningPanel.tsx"
// import ChanceToCostSection from "./Sections/ChanceMode/ChanceModeSection.tsx"
import DistributionSection from "./Sections/BudgetMode/BudgetModeSection.tsx"
// const DistributionSection = React.lazy(() => import('./DistributionSection.tsx'));

import InputsSection from "./Sections/Inputs/InputsSection.tsx"

// import GambaSection from "./Sections/GambaSimulator/GambaSection.tsx"
// import LongTermSection from "./Sections/ForecastMode/ForecastModeSection.tsx"
// const GambaSection = React.lazy(() => import('./GambaSection.tsx'));
import Separator from "./Sections/Separator/Separator.tsx"
import { TooltipState } from "./Utils/Tooltip.tsx"
import Icon from "./Components/Icon.tsx"

import { GridMouseDownLogic, mouseMoveLogic, createMouseUpHandler } from "./Sections/UpgradeSelection/Marquee.ts"
import { createClearAll, createResetOptimizerState } from "./Sections/ControlPanel/ControlPanelFunctions.ts"
import { buildPayload, createCancelableWorkerRunner } from "./WasmInterface/WorkerRunner.ts"
import type { InputsBundleWithSetters, InputsSetters, InputsValues } from "./Utils/InputBundles.ts"
import OptimizeSection from "./Sections/Optimize/OptimizeSection.tsx"
import { StatePair } from "./Sections/Optimize/StateGrid.tsx"
import { applyFlatToGrid } from "./Utils/StateUtils.ts"

const NUM_JUICE_AVAIL = 4
export default function HoningForecastUI() {
    const [topGrid, setTopGrid] = useState(() => Array.from({ length: TOP_ROWS }, () => Array.from({ length: TOP_COLS }, () => false)))
    const [bottomGrid, setBottomGrid] = useState(() => Array.from({ length: BOTTOM_ROWS }, () => Array(BOTTOM_COLS).fill(false)))
    const [userMatsOwned, setUserMatsOwned] = useState(() => Object.fromEntries(MATS_LABELS.map((l) => [l, "0"])))

    const [userMatsPrices, setUserMatsPrices] = useState(() => {
        return Object.fromEntries(MATS_LABELS.slice(0, 7).map((label, index) => [label, DEFAULT_MATS_PRICES[index]]))
    })

    const [userMatsLeftover, setUserMatsLeftover] = useState(() => {
        return Object.fromEntries(MATS_LABELS.map((label, index) => [label, DEFAULT_MATS_PRICES[index]]))
    })

    const [userWeaponJuiceOwned, setUserWeaponJuiceOwned] = useState(() => {
        return Object.fromEntries(JUICE_LABELS.map((label_row) => [label_row[0], "0"]))
    })

    const [userArmorJuiceOwned, setUserArmorJuiceOwned] = useState(() => {
        return Object.fromEntries(JUICE_LABELS.map((label_row) => [label_row[1], "0"]))
    })

    const [userWeaponJuicePrices, setUserWeaponJuicePrices] = useState(() => {
        return Object.fromEntries(JUICE_LABELS.map((label_row, row_index) => [label_row[0], DEFAULT_JUICE_PRICES[row_index][0]]))
    })

    const [userArmorJuicePrices, setUserArmorJuicePrices] = useState(() => {
        return Object.fromEntries(JUICE_LABELS.map((label_row, row_index) => [label_row[1], DEFAULT_JUICE_PRICES[row_index][1]]))
    })

    const [userWeaponJuiceLeftover, setUserWeaponJuiceLeftover] = useState(() => {
        return Object.fromEntries(JUICE_LABELS.map((label_row, row_index) => [label_row[0], DEFAULT_JUICE_PRICES[row_index][0]]))
    })

    const [userArmorJuiceLeftover, setUserArmorJuiceLeftover] = useState(() => {
        return Object.fromEntries(JUICE_LABELS.map((label_row, row_index) => [label_row[1], DEFAULT_JUICE_PRICES[row_index][1]]))
    })

    const inputsValues = useMemo<InputsValues>(
        () => ({
            mats: {
                owned: userMatsOwned,
                prices: userMatsPrices,
                leftover: userMatsLeftover,
            },
            juice: {
                weapon: {
                    owned: userWeaponJuiceOwned,
                    prices: userWeaponJuicePrices,
                    leftover: userWeaponJuiceLeftover,
                },
                armor: {
                    owned: userArmorJuiceOwned,
                    prices: userArmorJuicePrices,
                    leftover: userArmorJuiceLeftover,
                },
            },
        }),
        [
            userMatsOwned,
            userMatsPrices,
            userMatsLeftover,
            userWeaponJuiceOwned,
            userWeaponJuicePrices,
            userWeaponJuiceLeftover,
            userArmorJuiceOwned,
            userArmorJuicePrices,
            userArmorJuiceLeftover,
        ],
    )

    const inputsSetters = useMemo<InputsSetters>(
        () => ({
            mats: {
                setOwned: setUserMatsOwned,
                setPrices: setUserMatsPrices,
                setLeftover: setUserMatsLeftover,
            },
            juice: {
                weapon: {
                    setOwned: setUserWeaponJuiceOwned,
                    setPrices: setUserWeaponJuicePrices,
                    setLeftover: setUserWeaponJuiceLeftover,
                },
                armor: {
                    setOwned: setUserArmorJuiceOwned,
                    setPrices: setUserArmorJuicePrices,
                    setLeftover: setUserArmorJuiceLeftover,
                },
            },
        }),
        [
            setUserMatsOwned,
            setUserMatsPrices,
            setUserMatsLeftover,
            setUserWeaponJuiceOwned,
            setUserWeaponJuicePrices,
            setUserWeaponJuiceLeftover,
            setUserArmorJuiceOwned,
            setUserArmorJuicePrices,
            setUserArmorJuiceLeftover,
        ],
    )

    const inputsBundle = useMemo<InputsBundleWithSetters>(() => ({ values: inputsValues, setters: inputsSetters }), [inputsValues, inputsSetters])

    const [desired_chance, set_desired_chance] = useState(() => "50")
    // const [uncleaned_desired_chance, set_uncleaned_desired_chance] = useState(() => "50")
    const [adv_hone_strategy, set_adv_hone_strategy_change] = useState(() => "x2 grace")
    const [express_event, set_express_event] = useState(() => true)
    const [bucketCount, _setBucketCount] = useState(() => "100") // leaving the door open for changing bucket count later

    const [prev_checked_arr, set_prev_checked_arr] = useState(() => Array.from({ length: TOP_COLS }, () => false)) // the extra rows on top of the grids
    const [prev_checked_arr_bottom, set_prev_checked_arr_bottom] = useState(() => Array.from({ length: BOTTOM_COLS }, () => false))
    const [cumulativeGraph, setCumulativeGraph] = useState<boolean>(RESET_UI_DEFAULTS["cumulativeGraph"])
    const [dataSize, setDataSize] = useState<string>(() => RESET_UI_DEFAULTS["dataSize"])
    const [activePage, setActivePage] = useState<"optimize" | "distribution" | "gamba" | "forecast">("distribution") // "chance-to-cost" |
    const [mainScale, setMainScale] = useState<number>(1)
    // const [zoomCompensation, setZoomCompensation] = useState<number>(1)
    const [optimizeButtonPress, setOptimizeButtonPress] = useState<number>(0)
    const [autoRunOptimizer, setAutoRunOptimizer] = useState<boolean>(false)
    const [optimizeAvgError, setOptimizeAvgError] = useState<string | null>(null)
    const [bestMetric, setBestMetric] = useState<number | null>(null)
    const [beforeMetric, setBeforeMetric] = React.useState<number | null>(null)

    const [bestFlatStateBundle, setBestFlatStateBundle] = useState<StatePair[][] | null>(null)
    const [bestFlatSpecialState, setBestFlatSpecialState] = useState<number[] | null>(null)
    const [ranOutFreeTaps, setRanOutFreeTaps] = useState<boolean>(false)
    const ranOutSkipResetRef = useRef<boolean>(false)
    // const hasRunOptimizerRef = useRef<boolean>(false)
    const [hasRunOptimizer, setHasRunOptimizer] = useState<boolean>(false)
    // State for optimized details
    // const [showOptimizedDetails, setShowOptimizedDetails] = useState<boolean>(false)

    const [flatProgressArr, setFlatProgressArr] = useState<number[]>([])
    const [progressGrid, setProgressGrid] = useState<number[][]>(Array(6).fill(Array(25).fill(0)))

    const [flatStateBundle, setFlatStateBundle] = useState<StatePair[][]>(null)
    const [stateBundleGrid, setStateBundleGrid] = useState<StatePair[][][]>(Array(6).fill(Array(25).fill([])))

    const [flatUnlockArr, setFlatUnlockArr] = useState<boolean[]>([])
    const [unlockGrid, setUnlockGrid] = useState<boolean[][]>(Array(6).fill(Array(25).fill(false)))

    const [flatSucceedArr, setFlatSucceedArr] = useState<boolean[]>([])
    const [succeededGrid, setSucceededGrid] = useState<boolean[][]>(Array(6).fill(Array(25).fill(false)))

    const [specialState, setSpecialState] = useState<number[]>([])
    const [minResolution, setMinResolution] = useState<number>(10)

    const [optimizerProgress, setOptimizerProgress] = useState<number>(0)

    const [allowUserChangeState, setAllowUserChangeState] = useState<boolean>(false)
    // Lock x-axis state (shared across all graphs)
    // const [lockXAxis, setLockXAxis] = useState<boolean>(false)
    // const [lockedMins, setLockedMins] = useState<number[] | null>(null)
    // const [lockedMaxs, setLockedMaxs] = useState<number[] | null>(null)

    // Income array state (6 grids, 7 rows each)
    const [incomeArr, setIncomeArr] = useState<number[][]>(() => Array.from({ length: 6 }, () => Array.from({ length: 7 }, () => 0)))

    const [metricType, setMetricType] = useState<number>(1)
    // marquee state & refs (kept here so grids stay presentational)
    const topGridRef = useRef<HTMLDivElement | null>(null)
    const bottomGridRef = useRef<HTMLDivElement | null>(null)
    const mainRef = useRef<HTMLDivElement | null>(null)
    // const controlsRef = useRef<HTMLDivElement | null>(null)
    const [marquee, setMarquee] = useState<any>({
        active: false,
        grid: null,
        startR: 0,
        startC: 0,
        endR: 0,
        endC: 0,
        startClientX: 0,
        startClientY: 0,
        endClientX: 0,
        endClientY: 0,
        initialState: false,
    })
    const [marqueeRect, setMarqueeRect] = useState<any>(null)
    const marqueeRef = useRef(marquee)
    useEffect(() => {
        marqueeRef.current = marquee
    }, [marquee])

    // // tooltip state & handlers
    // const [tooltip, setTooltip] = useState<TooltipState>({
    //     visible: false,
    //     type: null,
    //     x: 0,
    //     y: 0,
    //     content: null,
    //     upgradeData: null,
    // })
    // const tooltipHandlers = createTooltipHandlers(setTooltip)

    // ----- Load saved UI state on mount -----
    useEffect(() => {
        try {
            readSettings(
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
                // setSpecialState,
                // setSucceededGrid,
                // setUnlockGrid,
                // setStateBundleGrid,
                // setProgressGrid,
            )
        } catch (e) {
            // ignore corrupted storage
        }
    }, [])

    // Initialize uncleaned_desired_chance from desired_chance after settings load
    // useEffect(() => {
    //     set_uncleaned_desired_chance(desired_chance)
    // }, [desired_chance])

    // ----- Responsive scaling based on window width -----
    useEffect(() => {
        const updateScale = () => {
            const width = window.innerWidth
            if (width < 1200) {
                const scale = Math.max(0, width / 1200)
                setMainScale(scale)
            } else {
                setMainScale(1)
            }
        }
        updateScale()
        window.addEventListener("resize", updateScale)
        return () => {
            window.removeEventListener("resize", updateScale)
        }
    }, [])

    // ----- Zoom detection for tooltip compensation -----
    useEffect(() => {
        let previousPixelRatio = window.devicePixelRatio

        const checkZoom = () => {
            const currentPixelRatio = window.devicePixelRatio
            if (currentPixelRatio !== previousPixelRatio) {
                previousPixelRatio = currentPixelRatio
                // Calculate compensation factor to keep tooltips constant size
                // When zoom increases (devicePixelRatio > 1), we scale down tooltips
                // const compensation = 1 / currentPixelRatio
                // setZoomCompensation(compensation)
            }
        }
        // setZoomCompensation(1 / window.devicePixelRatio)
        window.addEventListener("resize", checkZoom)

        return () => {
            window.removeEventListener("resize", checkZoom)
        }
    }, [])

    // ----- Persist UI state (debounced) -----
    const saveTimerRef = useRef<number | null>(null)
    useEffect(() => {
        if (saveTimerRef.current) {
            window.clearTimeout(saveTimerRef.current)
            saveTimerRef.current = null
        }
        saveTimerRef.current = window.setTimeout(() => {
            try {
                writeSettings(
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
                    // specialState,
                    // succeededGrid,
                    // unlockGrid,
                    // stateBundleGrid,
                    // progressGrid,
                )
            } catch (e) {
                // ignore quota or serialization errors
            }
            saveTimerRef.current = null
        }, 670)
        return () => {
            if (saveTimerRef.current) {
                window.clearTimeout(saveTimerRef.current)
                saveTimerRef.current = null
            }
        }
    }, [
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
        minResolution,
        // specialState,
        // succeededGrid,
        // unlockGrid,
        // stateBundleGrid,
        // progressGrid,
        incomeArr,
    ])

    const onGridMouseDown = GridMouseDownLogic({
        topGridRef,
        bottomGridRef,
        marqueeRef,
        topGrid,
        bottomGrid,
        setMarquee,
    })

    useEffect(() => {
        const temp_fn = function (ev: MouseEvent) {
            mouseMoveLogic(ev, marqueeRef, topGridRef, bottomGridRef, setMarquee)
        }

        // Helper function to convert touch event to mouse event
        const createMouseEventFromTouch = (touchEvent: TouchEvent, type: "mousemove" | "mouseup"): MouseEvent => {
            const touch = touchEvent.touches[0] || touchEvent.changedTouches[0]
            return {
                ...touchEvent,
                type,
                clientX: touch.clientX,
                clientY: touch.clientY,
                button: 0,
                buttons: type === "mousemove" ? 1 : 0,
                preventDefault: touchEvent.preventDefault.bind(touchEvent),
                stopPropagation: touchEvent.stopPropagation.bind(touchEvent),
            } as unknown as MouseEvent
        }

        const touchMoveHandler = (ev: TouchEvent) => {
            // Only prevent default if marquee is active (user is dragging on grid)
            if (marqueeRef.current && marqueeRef.current.active) {
                ev.preventDefault()
                const mouseEvent = createMouseEventFromTouch(ev, "mousemove")
                mouseMoveLogic(mouseEvent, marqueeRef, topGridRef, bottomGridRef, setMarquee)
            }
        }

        window.addEventListener("mousemove", temp_fn)
        window.addEventListener("touchmove", touchMoveHandler, { passive: false })
        return () => {
            window.removeEventListener("mousemove", temp_fn)
            window.removeEventListener("touchmove", touchMoveHandler)
        }
    }, [])

    useEffect(() => {
        const onUp = createMouseUpHandler({
            marqueeRef,
            topGridRef,
            bottomGridRef,
            setTopGrid,
            setBottomGrid,
            set_prev_checked_arr,
            set_prev_checked_arr_bottom,
            setMarquee,
        })

        // Helper function to convert touch event to mouse event
        const createMouseEventFromTouch = (touchEvent: TouchEvent, type: "mousemove" | "mouseup"): MouseEvent => {
            const touch = touchEvent.touches[0] || touchEvent.changedTouches[0]
            return {
                ...touchEvent,
                type,
                clientX: touch.clientX,
                clientY: touch.clientY,
                button: 0,
                buttons: type === "mousemove" ? 1 : 0,
                preventDefault: touchEvent.preventDefault.bind(touchEvent),
                stopPropagation: touchEvent.stopPropagation.bind(touchEvent),
            } as unknown as MouseEvent
        }

        const touchEndHandler = (ev: TouchEvent) => {
            // Only prevent default if marquee is active (user is dragging on grid)
            if (marqueeRef.current && marqueeRef.current.active) {
                ev.preventDefault()
                const mouseEvent = createMouseEventFromTouch(ev, "mouseup")
                onUp(mouseEvent)
            }
        }

        window.addEventListener("mouseup", onUp)
        window.addEventListener("touchend", touchEndHandler, { passive: false })
        return () => {
            window.removeEventListener("mouseup", onUp)
            window.removeEventListener("touchend", touchEndHandler)
        }
    }, [])

    useEffect(() => {
        if (!marquee.active || !marquee.grid) {
            setMarqueeRect(null)
            return
        }
        const sx = marquee.startClientX
        const sy = marquee.startClientY
        const ex = marquee.endClientX
        const ey = marquee.endClientY
        const left = Math.min(sx, ex)
        const top = Math.min(sy, ey)
        const width = Math.abs(ex - sx)
        const height = Math.abs(ey - sy)
        setMarqueeRect({ left, top, width, height })
    }, [marquee])

    // const onDesiredChange = (value: string) => {
    //     set_uncleaned_desired_chance(value)

    //     // Check if the input is immediately valid (integer 0-100 inclusive)
    //     const cleanValue = value.replace(/[^0-9]/g, "")
    //     const numValue = parseInt(cleanValue)

    //     if (cleanValue === value && !isNaN(numValue) && numValue >= 0 && numValue <= 100) {
    //         set_desired_chance(cleanValue)
    //     }
    // }

    // const onDesiredBlur = () => {
    //     const cleanValue = uncleaned_desired_chance.replace(/[^0-9]/g, "")
    //     const numValue = parseInt(cleanValue)

    //     if (cleanValue === "" || isNaN(numValue)) {
    //         set_uncleaned_desired_chance(desired_chance)
    //     } else if (numValue > 100) {
    //         set_desired_chance("100")
    //         set_uncleaned_desired_chance("100")
    //     } else if (numValue < 0) {
    //         set_desired_chance("0")
    //         set_uncleaned_desired_chance("0")
    //     } else {
    //         set_desired_chance(cleanValue)
    //         set_uncleaned_desired_chance(cleanValue)
    //     }
    // }
    const adv_hone_strategy_change = (value: string) => set_adv_hone_strategy_change(value)

    // Sync express_event toggle with adv_hone_strategy selector
    useEffect(() => {
        set_adv_hone_strategy_change(express_event ? "x2 grace" : "No x2 grace")
    }, [express_event])

    // Lock x-axis handler
    // const onToggleLockXAxis = () => {
    //     setLockXAxis((prev) => {
    //         const newVal = !prev
    //         // if (!prev) {
    //         //     // we're turning it ON: snapshot current mins/maxs from cached data
    //         //     // const currentMins = cachedAverageGraphData?.hist_mins || null
    //         //     // const currentMaxs = cachedAverageGraphData?.hist_maxs || null
    //         //     // setLockedMins(currentMins ? currentMins.slice() : null)
    //         //     // setLockedMaxs(currentMaxs ? currentMaxs.slice() : null)
    //         // } else
    //         {
    //             // turning it OFF: clear snapshots
    //             setLockedMins(null)
    //             setLockedMaxs(null)
    //         }
    //         return newVal
    //     })
    // }

    const payloadBuilder = () =>
        buildPayload({
            topGrid,
            bottomGrid,
            adv_hone_strategy,
            express_event,
            bucketCount,

            dataSize,
            inputs: inputsValues,
            progressGrid,
            unlockGrid,
            succeededGrid,
            stateBundleGrid,
            specialState,
            minResolution,
            metricType,
            // monteCarloResult,
        })

    const onCopyPayload = async () => {
        const payload = payloadBuilder()
        const payloadJson = JSON.stringify(payload, null, 2)
        try {
            if (navigator.clipboard?.writeText) {
                await navigator.clipboard.writeText(payloadJson)
                return
            }
        } catch (err) {
            // fallback to legacy path
        }
        try {
            const textArea = document.createElement("textarea")
            textArea.value = payloadJson
            textArea.setAttribute("readonly", "")
            textArea.style.position = "fixed"
            textArea.style.opacity = "0"
            textArea.style.pointerEvents = "none"
            document.body.appendChild(textArea)
            textArea.select()
            document.execCommand("copy")
            document.body.removeChild(textArea)
        } catch (err) {
            console.warn("Failed to copy payload", err)
        }
    }

    const runner = useMemo(() => createCancelableWorkerRunner(), [])

    // ---------- Automatic triggers with debounce ----------
    // We'll watch serialized versions of the inputs to detect deep changes
    // const budgetKey = useMemo(() => JSON.stringify(userMatsOwned), [userMatsOwned])
    // const desiredKey = useMemo(() => String(desired_chance), [desired_chance])
    const advStrategyKey = useMemo(() => String(adv_hone_strategy), [adv_hone_strategy])
    const expressEventKey = useMemo(() => String(express_event), [express_event])
    const graphBucketSizeKey = useMemo(() => String(bucketCount), [bucketCount])
    // const autoOptKey = useMemo(() => String(autoGoldValues), [autoGoldValues])
    // const userMatsKey = useMemo(() => JSON.stringify(userMatsPrices), [userMatsPrices])
    const dataSizeKey = useMemo(() => String(dataSize), [dataSize])

    const optimizeButtonPressKey = useMemo(() => String(optimizeButtonPress), [optimizeButtonPress])

    const topGridKey = useMemo(() => String(topGrid), [topGrid])
    const bottomGridKey = useMemo(() => String(bottomGrid), [bottomGrid])
    const progressGridKey = useMemo(() => String(progressGrid), [progressGrid])

    const stateBundleGridKey = useMemo(() => String(stateBundleGrid), [stateBundleGrid])
    const unlockGridKey = useMemo(() => String(unlockGrid), [unlockGrid])
    const succeededGridKey = useMemo(() => String(succeededGrid), [succeededGrid])
    const specialStateKey = useMemo(() => String(specialState), [specialState])
    const minResolutionKey = useMemo(() => String(minResolution), [minResolution])

    const inputBundleKey = useMemo(
        () =>
            JSON.stringify({
                mats: {
                    owned: userMatsOwned,
                    prices: userMatsPrices,
                    leftover: userMatsLeftover,
                },
                juice: {
                    weapon: {
                        owned: userWeaponJuiceOwned,
                        prices: userWeaponJuicePrices,
                        leftover: userWeaponJuiceLeftover,
                    },
                    armor: {
                        owned: userArmorJuiceOwned,
                        prices: userArmorJuicePrices,
                        leftover: userArmorJuiceLeftover,
                    },
                },
            }),
        [
            userMatsOwned,
            userMatsPrices,
            userMatsLeftover,
            userWeaponJuiceOwned,
            userWeaponJuicePrices,
            userWeaponJuiceLeftover,
            userArmorJuiceOwned,
            userArmorJuicePrices,
            userArmorJuiceLeftover,
        ],
    )

    const cloneFlatStateBundle = (bundle: StatePair[][]) => bundle.map((row) => row.map((pair) => [pair[0], pair[1]] as StatePair))

    const updateBestSolution = (res: { metric: number; upgrade_arr: { state: StatePair[] }[]; special_state: number[] }) => {
        setBestMetric((prev) => {
            // if (res === null) {
            //     setBestFlatStateBundle(null)
            //     setBestFlatSpecialState(null)
            //     return null
            // }
            const nextMetric = prev === null ? res.metric : Math.max(res.metric, prev)
            if (prev === null || nextMetric !== prev) {
                setBestFlatStateBundle(cloneFlatStateBundle(res.upgrade_arr.map((upgrade) => upgrade.state)))
                setBestFlatSpecialState(res.special_state ? [...res.special_state] : null)
            }
            return nextMetric
        })
    }

    useEffect(() => {
        const skipRanOutReset = ranOutSkipResetRef.current
        ranOutSkipResetRef.current = false
        // hasRunOptimizerRef.current = false
        setHasRunOptimizer(false)
        setBestMetric(null)
        setBeforeMetric(null)
        setBestFlatStateBundle(null)
        setBestFlatSpecialState(null)

        if (!skipRanOutReset) {
            setRanOutFreeTaps(false)
        }
        // console.log("reest", topGridKey, inputBundleKey, metricType, progressGridKey, succeededGridKey, minResolutionKey, unlockGridKey)
    }, [topGridKey, bottomGridKey, inputBundleKey, metricType, minResolutionKey, progressGridKey, succeededGridKey, unlockGridKey])

    const cancelOptimizeAverage = () => {
        runner.cancel("OptimizeAverage")
        if (optimizeAvgWorkerRef.current) {
            try {
                optimizeAvgWorkerRef.current.terminate()
            } catch (e) {
                /* ignore */
            }
            optimizeAvgWorkerRef.current = null
        }
        setOptimizeAvgBusy(false)
        setOptimizerProgress(0)
        setOptimizeAvgError(null)
    }
    useEffect(() => {
        if (evaluateAverageResult) {
            applyFlatToGrid(
                evaluateAverageResult,
                flatProgressArr,
                progressGrid,
                setProgressGrid,
                flatUnlockArr,
                unlockGrid,
                setUnlockGrid,
                flatSucceedArr,
                succeededGrid,
                setSucceededGrid,
                flatStateBundle,
                stateBundleGrid,
                setStateBundleGrid,
            )
        }

        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [flatProgressArr, flatUnlockArr, flatSucceedArr, flatStateBundle])

    const handleRanOutFreeTaps = () => {
        ranOutSkipResetRef.current = true
        setUserMatsOwned((prev) => ({
            ...prev,
            [MATS_LABELS[7]]: "0",
        }))
        setRanOutFreeTaps(true)
    }
    // const monteCarloWorkerRef = useRef<Worker | null>(null)
    // const [_monteCarloBusy, setMonteCarloBusy] = useState(false)
    // const [monteCarloResult, setMonteCarloResult] = useState<any>(null)
    // useEffect(() => {
    //     runner.start({
    //         which_one: "MonteCarlo",
    //         payloadBuilder,
    //         workerRef: monteCarloWorkerRef,
    //         setBusy: setMonteCarloBusy,
    //         setResult: setMonteCarloResult,
    //     })
    //     // eslint-disable-next-line react-hooks/exhaustive-deps
    // }, [advStrategyKey, expressEventKey, dataSizeKey, normalCountsKey, advCountsKey, budgetKey, userMatsKey])

    // const chanceToCostWorkerRef = useRef<Worker | null>(null)
    // const [chanceToCostBusy, setChanceToCostBusy] = useState(false)
    // const [chanceToCostResult, setChanceToCostResult] = useState<any>(null)
    // const [cachedCostGraphData, setCachedCostGraphData] = useState<{ hist_counts?: any; hist_mins?: any; hist_maxs?: any } | null>(null)

    // useEffect(() => {
    //     runner.start({
    //         which_one: "ChanceToCost",
    //         payloadBuilder,
    //         workerRef: chanceToCostWorkerRef,
    //         setBusy: setChanceToCostBusy,
    //         setResult: setChanceToCostResult,
    //         setCachedGraphData: setCachedCostGraphData,
    //     })
    //     // eslint-disable-next-line react-hooks/exhaustive-deps
    // }, [advStrategyKey, expressEventKey, graphBucketSizeKey, dataSizeKey, normalCountsKey, advCountsKey])

    const evaluateAverageWorkerRef = useRef<Worker | null>(null)
    const [_evaluateAverageBusy, setEvaluateAverageBusy] = useState(false)
    const [evaluateAverageResult, setEvaluateAverageResult] = useState<any>(null)
    // const [cachedAverageGraphData, setCachedAverageGraphData] = useState<{ hist_counts?: any; hist_mins?: any; hist_maxs?: any } | null>(null)
    useEffect(() => {
        runner.start({
            which_one: "EvaluateAverage",
            payloadBuilder,
            workerRef: evaluateAverageWorkerRef,
            setBusy: setEvaluateAverageBusy,
            setResult: setEvaluateAverageResult,
            // setCachedGraphData: setCachedAverageGraphData,
            onSuccess: (res) => {
                // console.log(inputBundleKey)
                setFlatStateBundle(res.upgrade_arr.map((upgrade) => upgrade.state))
                setFlatProgressArr(res.upgrade_arr.map((_, index) => progressGrid[res.upgrade_arr[index].piece_type][res.upgrade_arr[index].upgrade_index]))
                setFlatUnlockArr(res.upgrade_arr.map((_, index) => unlockGrid[res.upgrade_arr[index].piece_type][res.upgrade_arr[index].upgrade_index]))
                setFlatSucceedArr(res.upgrade_arr.map((_, index) => succeededGrid[res.upgrade_arr[index].piece_type][res.upgrade_arr[index].upgrade_index]))
                setSpecialState(res.special_state)
                updateBestSolution(res)
                // applyFlatToGrid(
                //     evaluateAverageResult,
                //     flatProgressArr,
                //     progressGrid,
                //     setProgressGrid,
                //     flatUnlockArr,
                //     unlockGrid,
                //     setUnlockGrid,
                //     flatSucceedArr,
                //     succeededGrid,
                //     setSucceededGrid,
                //     flatStateBundle,
                //     stateBundleGrid,
                //     setStateBundleGrid,
                // )
                // console.log(specialState)
            },
            debounceMs: 10,
        })
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [
        advStrategyKey,
        expressEventKey,
        graphBucketSizeKey,
        dataSizeKey,
        topGridKey,
        bottomGridKey,
        progressGridKey,
        unlockGridKey,
        succeededGridKey,
        stateBundleGridKey,
        inputBundleKey,
        specialStateKey,
        minResolutionKey,
        metricType,
    ])

    const histogramWorkerRef = useRef<Worker | null>(null)
    const [_histogramBusy, setHistogramBusy] = useState(false)
    const [histogramResult, setHistogramResult] = useState<any>(null)
    // const [cachedAverageGraphData, setCachedAverageGraphData] = useState<{ hist_counts?: any; hist_mins?: any; hist_maxs?: any } | null>(null)
    useEffect(() => {
        if (activePage === "distribution") {
            runner.start({
                which_one: "Histogram",
                payloadBuilder,
                workerRef: histogramWorkerRef,
                setBusy: setHistogramBusy,
                setResult: setHistogramResult,
                // setCachedGraphData: setCachedAverageGraphData,
                onSuccess: (res) => {
                    console.log(res)
                    setHistogramResult(res)
                },
                debounceMs: 100,
            })
        }

        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [
        advStrategyKey,
        expressEventKey,
        graphBucketSizeKey,
        dataSizeKey,
        topGridKey,
        bottomGridKey,
        progressGridKey,
        unlockGridKey,
        succeededGridKey,
        stateBundleGridKey,
        inputBundleKey,
        specialStateKey,
        minResolutionKey,
        metricType,
        activePage,
    ])

    const optimizeAvgWorkerRef = useRef<Worker | null>(null)
    const [optimizeAvgBusy, setOptimizeAvgBusy] = useState(false)
    // const [optimizeAvgResult, setOptimizeAvgResult] = useState<{ average_costs?: any } | null>(null)
    const startOptimizeAverage = useCallback((debounce) => {
        setOptimizerProgress(0)
        runner.start({
            which_one: "OptimizeAverage",
            payloadBuilder,
            workerRef: optimizeAvgWorkerRef,
            setBusy: setOptimizeAvgBusy,
            setResult: setEvaluateAverageResult,
            onSuccess: (res) => {
                // console.log(inputBundleKey)
                setHasRunOptimizer(true)
                setFlatStateBundle(res.upgrade_arr.map((upgrade) => upgrade.state))
                setFlatProgressArr(res.upgrade_arr.map((_, index) => progressGrid[res.upgrade_arr[index].piece_type][res.upgrade_arr[index].upgrade_index]))
                setFlatUnlockArr(res.upgrade_arr.map((_, index) => unlockGrid[res.upgrade_arr[index].piece_type][res.upgrade_arr[index].upgrade_index]))
                setFlatSucceedArr(res.upgrade_arr.map((_, index) => succeededGrid[res.upgrade_arr[index].piece_type][res.upgrade_arr[index].upgrade_index]))
                setSpecialState(res.special_state)
                updateBestSolution(res)
                setOptimizeAvgError(null)
                // console.log(specialState)
            },
            onIntermediateMessage: (res_bundle) => {
                // hasRunOptimizerRef.current = true
                setOptimizerProgress(res_bundle.est_progress_percentage)
                let res = res_bundle.state_bundle
                updateBestSolution(res)
                setFlatStateBundle(res.upgrade_arr.map((upgrade) => upgrade.state))
                // setFlatProgressArr(res.upgrade_arr.map((_, index) => progressGrid[res.upgrade_arr[index].piece_type][res.upgrade_arr[index].upgrade_index]))
                // setFlatUnlockArr(res.upgrade_arr.map((_, index) => unlockGrid[res.upgrade_arr[index].piece_type][res.upgrade_arr[index].upgrade_index]))
                // setFlatSucceedArr(res.upgrade_arr.map((_, index) => succeededGrid[res.upgrade_arr[index].piece_type][res.upgrade_arr[index].upgrade_index]))
                setSpecialState(res.special_state)
                setEvaluateAverageResult(res)
                applyFlatToGrid(
                    evaluateAverageResult,
                    flatProgressArr,
                    progressGrid,
                    setProgressGrid,
                    flatUnlockArr,
                    unlockGrid,
                    setUnlockGrid,
                    flatSucceedArr,
                    succeededGrid,
                    setSucceededGrid,
                    flatStateBundle,
                    stateBundleGrid,
                    setStateBundleGrid,
                )
                setOptimizeAvgError(null)
            },
            onError: (err) => {
                setOptimizeAvgError(String(err))
            },
            debounceMs: debounce,
        })
    })
    useEffect(() => {
        cancelOptimizeAverage()
        if (!autoRunOptimizer) {
            return
        }
        setOptimizeAvgError(null)
        setRanOutFreeTaps(false)
        setBeforeMetric(currentMetric)
        startOptimizeAverage(200)
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [
        autoRunOptimizer,
        advStrategyKey,
        expressEventKey,
        graphBucketSizeKey,
        dataSizeKey,
        topGridKey,
        bottomGridKey,
        // progressGridKey,
        // unlockGridKey,
        // succeededGridKey,
        // StateBundleGridKey,
        inputBundleKey,
        // specialStateKey,
        minResolutionKey,
        metricType,
    ])

    useEffect(() => {
        if (optimizeButtonPress > 0) {
            setOptimizeAvgError(null)
            setRanOutFreeTaps(false)
            startOptimizeAverage(0)
        }

        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [optimizeButtonPressKey])

    // const parserWorkerRef = useRef<Worker | null>(null)
    // const [parserResult, setparserResult] = useState<{ upgradeArr: any; unlocks: any; other_strategy_prob_dists: any } | null>(null)
    // const [ParserBusy, setParserBusy] = useState(false)
    // useEffect(() => {
    //     runner.start({
    //         which_one: "ParserUnified",
    //         payloadBuilder,
    //         workerRef: parserWorkerRef,
    //         setBusy: setParserBusy,
    //         setResult: setparserResult,
    //     })
    //     // eslint-disable-next-line react-hooks/exhaustive-deps
    // }, [advStrategyKey, expressEventKey, graphBucketSizeKey, dataSizeKey, minResolutionKey])

    const clearAll = createClearAll({
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

        _setBucketCount,
        setCumulativeGraph,
        setDataSize,
        // setLockXAxis,
        // setLockedMins,
        // setLockedMaxs,
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
        setBestFlatSpecialState,
        setBeforeMetric,
        // setOptimizerMetric,
        // setMonteCarloResult,
        setHasRunOptimizer,
    })

    const resetOptimizerState = createResetOptimizerState({
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
        setHasRunOptimizer,
    })

    // const fillDemo = createFillDemo({
    //     setTopGrid,
    //     setBottomGrid,
    //     setUserMatsOwned,
    //     set_desired_chance,
    //     set_prev_checked_arr,
    //     set_prev_checked_arr_bottom,
    //     setUserMatsPrices,
    //     setUserMatsLeftover,
    //     setUserWeaponJuiceOwned,
    //     setUserArmorJuiceOwned,
    //     setUserWeaponJuicePrices,
    //     setUserArmorJuicePrices,
    //     setUserWeaponJuiceLeftover,
    //     setUserArmorJuiceLeftover,
    //     setMinResolution,
    //     setSpecialState,
    //     setSucceededGrid,
    //     setUnlockGrid,
    //     setStateBundleGrid,
    //     setProgressGrid,
    // })

    // const fillDemoIncome = createFillDemoIncome({
    //     setIncomeArr,
    // })
    // // Cleanup on unmount: terminate any running workers and clear timers
    // useEffect(() => {
    //     return () => {
    //         runner.cancel()
    //     }
    //     // eslint-disable-next-line react-hooks/exhaustive-deps
    // }, [])

    // styles and column defs moved to ./styles
    const AnythingTicked = useMemo(() => topGrid.some((row) => row.some((x) => x)) || bottomGrid.some((row) => row.some((x) => x)), [topGrid, bottomGrid])
    const currentMetric = evaluateAverageResult?.metric
    // const hasRunOptimizer = hasRunOptimizerRef.current
    const curIsBest =
        hasRunOptimizer && bestMetric !== null && currentMetric !== undefined && currentMetric !== null
            ? Math.round(currentMetric) >= Math.round(bestMetric)
            : false

    return (
        <div style={styles.pageContainer}>
            {marqueeRect ? (
                <div
                    style={{
                        position: "fixed",
                        left: marqueeRect.left,
                        top: marqueeRect.top,
                        width: marqueeRect.width,
                        height: marqueeRect.height,
                        background: "var(--marquee-bg)",
                        border: "2px solid var(--marquee-border)",
                        pointerEvents: "none",
                        zIndex: 9999,
                    }}
                />
            ) : null}
            {/* {renderTooltip(tooltip, mainScale, zoomCompensation)} */}

            <div
                ref={mainRef}
                style={{
                    ...styles.mainContainer,
                    transform: `scale(${mainScale})`,
                    transformOrigin: "top center",
                }}
            >
                <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
                    <div style={{ display: "flex", flexDirection: "row", gap: 6 }}>
                        <Icon iconName="Forecast Icon" size={64} style={{ display: "flex", alignItems: "center", gap: "12px" }} display_text="" />
                        <h1 style={styles.heading}>Honing Forecast</h1>
                    </div>
                </div>

                {/* Three panels in a responsive flex layout */}
                <div style={{ display: "flex", gap: "var(--spacing-2xl)", alignItems: "flex-start", flexWrap: "wrap", justifyContent: "flex-start" }}>
                    <NormalHoningPanel
                        topGrid={topGrid}
                        setTopGrid={setTopGrid}
                        prev_checked_arr={prev_checked_arr}
                        set_prev_checked_arr={set_prev_checked_arr}
                        topGridRef={topGridRef}
                        marquee={marquee}
                        onGridMouseDown={onGridMouseDown}
                    />

                    {/* Bundled Advanced Honing Panel and Control Panel */}
                    <div style={{ display: "flex", flexDirection: "row", gap: "var(--spacing-2xl)" }}>
                        <AdvancedHoningPanel
                            bottomGrid={bottomGrid}
                            setBottomGrid={setBottomGrid}
                            prev_checked_arr_bottom={prev_checked_arr_bottom}
                            set_prev_checked_arr_bottom={set_prev_checked_arr_bottom}
                            bottomGridRef={bottomGridRef}
                            marquee={marquee}
                            onGridMouseDown={onGridMouseDown}
                            adv_hone_strategy={adv_hone_strategy}
                            adv_hone_strategy_change={adv_hone_strategy_change}
                        />

                        <ControlPanel
                            controlsLeft={null}
                            mainScale={mainScale}
                            // fillDemo={fillDemo}
                            // fillDemoIncome={fillDemoIncome}
                            clearAll={clearAll}
                            resetOptimizerState={resetOptimizerState}
                            onCopyPayload={onCopyPayload}
                            express_event={express_event}
                            set_express_event={set_express_event}
                            cumulativeGraph={cumulativeGraph}
                            setCumulativeGraph={setCumulativeGraph}
                            dataSize={dataSize}
                            setDataSize={setDataSize}
                            // lockXAxis={lockXAxis}
                            // onToggleLockXAxis={onToggleLockXAxis}
                            minResolution={minResolution}
                            setMinResolution={setMinResolution}
                            allowUserChangeState={allowUserChangeState}
                            setAllowUserChangeState={setAllowUserChangeState}
                        />
                    </div>
                </div>

                {/* Page Separator */}
                {/* <div>
                 
                </div> */}
                <Separator activePage={activePage} onPageChange={setActivePage} setAutoRunOptimizer={setAutoRunOptimizer} />

                {activePage === "optimize" && (
                    <div className={activePage === "optimize" ? "page" : "page page--hidden"}>
                        <OptimizeSection
                            curIsBest={curIsBest}
                            inputsBundle={inputsBundle}
                            optimizeAvgBusy={optimizeAvgBusy}
                            optimizeAvgWorkerRef={optimizeAvgWorkerRef}
                            setOptimizeAvgBusy={setOptimizeAvgBusy}
                            onCancelOptimizeAverage={cancelOptimizeAverage}
                            autoRunOptimizer={autoRunOptimizer}
                            setAutoRunOptimizer={setAutoRunOptimizer}
                            optimizeAvgError={optimizeAvgError}
                            setOptimizeButtonPress={setOptimizeButtonPress}
                            flatProgressArr={flatProgressArr}
                            setFlatProgressArr={setFlatProgressArr}
                            flatUnlockArr={flatUnlockArr}
                            setFlatUnlockArr={setFlatUnlockArr}
                            flatSucceedArr={flatSucceedArr}
                            setFlatSucceedArr={setFlatSucceedArr}
                            flatStateBundle={flatStateBundle}
                            setFlatStateBundle={setFlatStateBundle}
                            allowUserChangeState={allowUserChangeState}
                            evaluateAverageResult={evaluateAverageResult}
                            specialState={specialState}
                            setSpecialState={setSpecialState}
                            optimizerProgress={optimizerProgress}
                            metricType={metricType}
                            bestMetric={bestMetric}
                            bestFlatStateBundle={bestFlatStateBundle}
                            bestFlatSpecialState={bestFlatSpecialState}
                            setMetricType={setMetricType}
                            ranOutFreeTaps={ranOutFreeTaps}
                            onRanOutFreeTaps={handleRanOutFreeTaps}
                            beforeMetric={beforeMetric}
                            setBeforeMetric={setBeforeMetric}
                            hasRunOptimizer={hasRunOptimizer}
                        />
                    </div>
                )}
                {activePage === "distribution" && (
                    <div className={activePage === "distribution" ? "page" : "page page--hidden"}>
                        <DistributionSection cumulativeGraph={cumulativeGraph} histogramResult={histogramResult} setCumulativeGraph={setCumulativeGraph} />
                    </div>
                )}
                {/* {activePage === "gamba" && (
                    <div className={activePage === "gamba" ? "page" : "page page--hidden"}>
                        <GambaSection
                            inputs={inputsBundle}
                            topGrid={topGrid}
                            bottomGrid={bottomGrid}
                            adv_hone_strategy={adv_hone_strategy}
                            express_event={express_event}
                            desired_chance={desired_chance}
                            bucketCount={bucketCount}
                            dataSize={dataSize}
                            progressGrid={progressGrid}
                            tooltipHandlers={tooltipHandlers}
                            chance_result={evaluateAverageResult}
                            cachedChanceGraphData={null}
                            AnythingTicked={AnythingTicked}
                            CostToChanceBusy={evaluateAverageBusy}
                            cumulativeGraph={cumulativeGraph}
                            lockXAxis={lockXAxis}
                            lockedMins={lockedMins}
                            lockedMaxs={lockedMaxs}
                            unlockGrid={unlockGrid}
                            stateBundleGrid={stateBundleGrid}
                            specialState={specialState}
                            upgradeArr={parserResult ? parserResult.upgradeArr : []}
                            ParserBusy={ParserBusy}
                        />
                    </div>
                )}
                {activePage === "forecast" && (
                    <div className={activePage === "forecast" ? "page" : "page page--hidden"}>
                        <LongTermSection
                            budget_inputs={userMatsOwned}
                            set_budget_inputs={setUserMatsOwned}
                            userMatsPrices={userMatsPrices}
                            setUserMatsPrices={setUserMatsPrices}
                            topGrid={topGrid}
                            bottomGrid={bottomGrid}
                            adv_hone_strategy={adv_hone_strategy}
                            express_event={express_event}
                            bucketCount={bucketCount}
                            dataSize={dataSize}
                            incomeArr={incomeArr}
                            setIncomeArr={setIncomeArr}
                            // Desired chance props
                            // desired_chance={desired_chance}
                            // uncleaned_desired_chance={uncleaned_desired_chance}
                            // onDesiredChange={onDesiredChange}
                            // onDesiredBlur={onDesiredBlur}
                            // Cost result prop for hundred_budgets
                            // cost_result={chanceToCostResult}
                            //TODOcost_result_optimized={chanceToCostOptimizedResult}
                            showOptimizedDetails={showOptimizedDetails}
                            setShowOptimizedDetails={setShowOptimizedDetails}
                            payloadBuilder={payloadBuilder}
                            runner={runner}
                            // costToChanceResult={costToChanceResult}
                            monteCarloResult={null}
                        />
                    </div>
                )} */}
            </div>
        </div>
    )
}
