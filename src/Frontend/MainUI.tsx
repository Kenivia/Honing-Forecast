import React, { useEffect, useRef, useState, useMemo } from "react"
import "./Sections/UpgradeSelection//CheckboxRow.css"
import { styles } from "./Utils/Styles.ts"
import { INPUT_LABELS, TOP_ROWS, TOP_COLS, BOTTOM_ROWS, BOTTOM_COLS } from "./Utils/Constants.ts"
import { readSettings, writeSettings } from "./Utils/Settings.ts"
import ControlPanel from "./Sections/ControlPanel/ControlPanel.tsx"
import NormalHoningPanel from "./Sections/UpgradeSelection/NormalHoningPanel.tsx"
import AdvancedHoningPanel from "./Sections/UpgradeSelection/AdvancedHoningPanel.tsx"
import ChanceToCostSection from "./Sections/ChanceMode/ChanceModeSection.tsx"
import CostToChanceSection from "./Sections/BudgetMode/BudgetModeSection.tsx"
// const CostToChanceSection = React.lazy(() => import('./CostToChanceSection.tsx'));

import GambaSection from "./Sections/GambaSimulator/GambaSection.tsx"
import LongTermSection from "./Sections/ForecastMode/ForecastModeSection.tsx"
// const GambaSection = React.lazy(() => import('./GambaSection.tsx'));
import Separator from "./Sections/Separator/Separator.tsx"
import { TooltipState, createTooltipHandlers, renderTooltip } from "./Utils/Tooltip.tsx"
import Icon from "./Components/Icon.tsx"
import LabeledCheckbox from "./Components/LabeledCheckbox.tsx"

import { GridMouseDownLogic, mouseMoveLogic, createMouseUpHandler } from "./Sections/UpgradeSelection/Marquee.ts"
import { createClearAll, createFillDemo, createFillDemoIncome } from "./Sections/ControlPanel/ControlPanelFunctions.ts"
import { buildPayload, createCancelableWorkerRunner } from "./WasmInterface/WorkerRunner.ts"
import { ticksToCounts, countsToTicks } from "./Utils/Helpers.ts"

export default function HoningForecastUI() {
    const [topGrid, setTopGrid] = useState(() => Array.from({ length: TOP_ROWS }, () => Array.from({ length: TOP_COLS }, () => false)))
    const [bottomGrid, setBottomGrid] = useState(() => Array.from({ length: BOTTOM_ROWS }, () => Array(BOTTOM_COLS).fill(false)))
    const [budget_inputs, set_budget_inputs] = useState(() => Object.fromEntries(INPUT_LABELS.map((l) => [l, "0"])))
    const [autoGoldValues, setAutoGoldValues] = useState(false)
    const [userMatsValue, setUserMatsValue] = useState(() => {
        const defaultValues = ["1.65", "0.03", "13.0", "0.5", "95.0", "1.0", "0.0"]
        return Object.fromEntries(INPUT_LABELS.slice(0, 7).map((l, index) => [l, defaultValues[index]]))
    })
    const [desired_chance, set_desired_chance] = useState(() => "50")
    const [uncleaned_desired_chance, set_uncleaned_desired_chance] = useState(() => "50")
    const [adv_hone_strategy, set_adv_hone_strategy_change] = useState(() => "No juice")
    const [express_event, set_express_event] = useState(() => true)
    const [bucketCount, _setBucketCount] = useState(() => "100") // leaving the door open for changing bucket count later

    const [prev_checked_arr, set_prev_checked_arr] = useState(() => Array.from({ length: TOP_COLS }, () => false)) // the extra rows on top of the grids
    const [prev_checked_arr_bottom, set_prev_checked_arr_bottom] = useState(() => Array.from({ length: BOTTOM_COLS }, () => false))
    const [cumulativeGraph, setCumulativeGraph] = useState<boolean>(false)
    const [dataSize, setDataSize] = useState<string>(() => "100000")
    const [activePage, setActivePage] = useState<"chance-to-cost" | "cost-to-chance" | "gamba" | "forecast">("chance-to-cost")
    const [mainScale, setMainScale] = useState<number>(1)
    const [zoomCompensation, setZoomCompensation] = useState<number>(1)

    // State for optimized details
    const [showOptimizedDetails, setShowOptimizedDetails] = useState<boolean>(false)

    // Lock x-axis state (shared across all graphs)
    const [lockXAxis, setLockXAxis] = useState<boolean>(false)
    const [lockedMins, setLockedMins] = useState<number[] | null>(null)
    const [lockedMaxs, setLockedMaxs] = useState<number[] | null>(null)
    const [showAverage, setShowAverage] = useState<boolean>(false)
    const [useGridInput, setUseGridInput] = useState<boolean>(true)

    // Numeric input state for when useGridInput is false
    const [normalCounts, setNormalCounts] = useState<number[][]>(() => Array.from({ length: 2 }, () => Array(TOP_COLS).fill(0)))
    const [advCounts, setAdvCounts] = useState<number[][]>(() => Array.from({ length: 2 }, () => Array(BOTTOM_COLS).fill(0)))

    // Income array state (6 grids, 7 rows each)
    const [incomeArr, setIncomeArr] = useState<number[][]>(() => Array.from({ length: 6 }, () => Array.from({ length: 7 }, () => 0)))

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

    // tooltip state & handlers
    const [tooltip, setTooltip] = useState<TooltipState>({
        visible: false,
        type: null,
        x: 0,
        y: 0,
        content: null,
        upgradeData: null,
    })
    const tooltipHandlers = createTooltipHandlers(setTooltip)

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
                set_budget_inputs,
                setAutoGoldValues,
                setUserMatsValue,
                setCumulativeGraph,
                setDataSize,
                setUseGridInput,
                setNormalCounts,
                setAdvCounts,
                setIncomeArr
            )
        } catch (e) {
            // ignore corrupted storage
        }
    }, [])

    // Initialize uncleaned_desired_chance from desired_chance after settings load
    useEffect(() => {
        set_uncleaned_desired_chance(desired_chance)
    }, [desired_chance])

    // Sync topGrid changes to normalCounts
    useEffect(() => {
        const newNormalCounts = ticksToCounts(topGrid)
        setNormalCounts(newNormalCounts)
    }, [topGrid])

    // Sync bottomGrid changes to advCounts
    useEffect(() => {
        const newAdvCounts = ticksToCounts(bottomGrid)
        setAdvCounts(newAdvCounts)
    }, [bottomGrid])

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
                const compensation = 1 / currentPixelRatio
                setZoomCompensation(compensation)
            }
        }
        setZoomCompensation(1 / window.devicePixelRatio)
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
                    budget_inputs,
                    autoGoldValues,
                    userMatsValue,
                    cumulativeGraph,
                    dataSize,
                    useGridInput,
                    normalCounts,
                    advCounts,
                    incomeArr
                )
            } catch (e) {
                // ignore quota or serialization errors
            }
            saveTimerRef.current = null
        }, 250)
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
        budget_inputs,
        autoGoldValues,
        userMatsValue,
        cumulativeGraph,
        dataSize,
        useGridInput,
        normalCounts,
        advCounts,
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

    const onDesiredChange = (value: string) => {
        set_uncleaned_desired_chance(value)

        // Check if the input is immediately valid (integer 0-100 inclusive)
        const cleanValue = value.replace(/[^0-9]/g, "")
        const numValue = parseInt(cleanValue)

        if (cleanValue === value && !isNaN(numValue) && numValue >= 0 && numValue <= 100) {
            set_desired_chance(cleanValue)
        }
    }

    const onDesiredBlur = () => {
        const cleanValue = uncleaned_desired_chance.replace(/[^0-9]/g, "")
        const numValue = parseInt(cleanValue)

        if (cleanValue === "" || isNaN(numValue)) {
            set_uncleaned_desired_chance(desired_chance)
        } else if (numValue > 100) {
            set_desired_chance("100")
            set_uncleaned_desired_chance("100")
        } else if (numValue < 0) {
            set_desired_chance("0")
            set_uncleaned_desired_chance("0")
        } else {
            set_desired_chance(cleanValue)
            set_uncleaned_desired_chance(cleanValue)
        }
    }
    const adv_hone_strategy_change = (value: string) => set_adv_hone_strategy_change(value)

    // Lock x-axis handler
    const onToggleLockXAxis = () => {
        setLockXAxis((prev) => {
            const newVal = !prev
            if (!prev) {
                // we're turning it ON: snapshot current mins/maxs from cached data
                const currentMins = cachedChanceGraphData?.hist_mins || cachedCostGraphData?.hist_mins || null
                const currentMaxs = cachedChanceGraphData?.hist_maxs || cachedCostGraphData?.hist_maxs || null
                setLockedMins(currentMins ? currentMins.slice() : null)
                setLockedMaxs(currentMaxs ? currentMaxs.slice() : null)
            } else {
                // turning it OFF: clear snapshots
                setLockedMins(null)
                setLockedMaxs(null)
            }
            return newVal
        })
    }

    // Handler for numeric input changes
    const handleNumericInputChange = (grid: "top" | "bottom", row: number, col: number, value: number) => {
        if (grid === "top") {
            setNormalCounts((prev) => {
                const newCounts = prev.map((row) => [...row])
                newCounts[row === 4 ? 0 : 1][col] = value
                return newCounts
            })
        } else {
            setAdvCounts((prev) => {
                const newCounts = prev.map((row) => [...row])
                newCounts[row === 4 ? 0 : 1][col] = value
                return newCounts
            })
        }
    }

    // Handler for switching between grid input modes
    const handleUseGridInputChange = (newValue: boolean) => {
        if (newValue !== useGridInput) {
            if (newValue) {
                // Switching to grid input mode - convert counts to ticks
                const newTopGrid = countsToTicks(normalCounts)
                const newBottomGrid = countsToTicks(advCounts)
                setTopGrid(newTopGrid)
                setBottomGrid(newBottomGrid)
            } else {
                // Switching to numeric input mode - convert ticks to counts
                const newNormalCounts = ticksToCounts(topGrid)
                const newAdvCounts = ticksToCounts(bottomGrid)
                setNormalCounts(newNormalCounts)
                setAdvCounts(newAdvCounts)
            }
        }
        setUseGridInput(newValue)
    }

    const clearAll = createClearAll({
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
    })

    const fillDemo = createFillDemo({
        setTopGrid,
        setBottomGrid,
        set_budget_inputs,
        set_desired_chance,
        set_prev_checked_arr,
        setUserMatsValue,
    })

    const fillDemoIncome = createFillDemoIncome({
        setIncomeArr,
    })

    const payloadBuilder = () =>
        buildPayload({
            topGrid,
            bottomGrid,
            budget_inputs,
            adv_hone_strategy,
            express_event,
            bucketCount,
            autoGoldValues,
            userMatsValue,
            dataSize,
            useGridInput,
            normalCounts,
            advCounts,
        })

    const runner = createCancelableWorkerRunner()

    // ---------- Automatic triggers with debounce ----------
    // We'll watch serialized versions of the inputs to detect deep changes
    const budgetKey = useMemo(() => JSON.stringify(budget_inputs), [budget_inputs])
    // const desiredKey = useMemo(() => String(desired_chance), [desired_chance])
    const advStrategyKey = useMemo(() => String(adv_hone_strategy), [adv_hone_strategy])
    const expressEventKey = useMemo(() => String(express_event), [express_event])
    const graphBucketSizeKey = useMemo(() => String(bucketCount), [bucketCount])
    const autoOptKey = useMemo(() => String(autoGoldValues), [autoGoldValues])
    const userMatsKey = useMemo(() => JSON.stringify(userMatsValue), [userMatsValue])
    const dataSizeKey = useMemo(() => String(dataSize), [dataSize])
    // const useGridInputKey = useMemo(() => String(useGridInput), [useGridInput])
    const normalCountsKey = useMemo(() => JSON.stringify(normalCounts), [normalCounts])
    const advCountsKey = useMemo(() => JSON.stringify(advCounts), [advCounts])

    const chanceToCostWorkerRef = useRef<Worker | null>(null)
    const [chanceToCostBusy, setChanceToCostBusy] = useState(false)
    const [chanceToCostResult, setChanceToCostResult] = useState<any>(null)
    const [cachedCostGraphData, setCachedCostGraphData] = useState<{ hist_counts?: any; hist_mins?: any; hist_maxs?: any } | null>(null)

    useEffect(() => {
        runner.start({
            which_one: "ChanceToCost",
            payloadBuilder,
            workerRef: chanceToCostWorkerRef,
            setBusy: setChanceToCostBusy,
            setResult: setChanceToCostResult,
            setCachedGraphData: setCachedCostGraphData,
        })
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [advStrategyKey, expressEventKey, graphBucketSizeKey, dataSizeKey, normalCountsKey, advCountsKey])

    const chanceToCostResultOptimizedWorkerRef = useRef<Worker | null>(null)
    const [_chanceToCostResultOptimizedBusy, setchanceToCostResultOptimizedBusy] = useState(false)
    const [chanceToCostOptimizedResult, setChanceToCostOptimizedResult] = useState<any>(null)
    useEffect(() => {
        runner.start({
            which_one: "ChanceToCostOptimized",
            payloadBuilder,
            workerRef: chanceToCostResultOptimizedWorkerRef,
            setBusy: setchanceToCostResultOptimizedBusy,
            setResult: setChanceToCostOptimizedResult,
        })
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [advStrategyKey, expressEventKey, graphBucketSizeKey, dataSizeKey, normalCountsKey, advCountsKey, budgetKey, userMatsKey])

    const costToChanceWorkerRef = useRef<Worker | null>(null)
    const [costToChanceBusy, setCostToChanceBusy] = useState(false)
    const [costToChanceResult, setCostToChanceResult] = useState<any>(null)
    const [cachedChanceGraphData, setCachedChanceGraphData] = useState<{ hist_counts?: any; hist_mins?: any; hist_maxs?: any } | null>(null)
    useEffect(() => {
        runner.start({
            which_one: "CostToChance",
            payloadBuilder,
            workerRef: costToChanceWorkerRef,
            setBusy: setCostToChanceBusy,
            setResult: setCostToChanceResult,
            setCachedGraphData: setCachedChanceGraphData,
        })
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [budgetKey, advStrategyKey, expressEventKey, graphBucketSizeKey, autoOptKey, userMatsKey, dataSizeKey, normalCountsKey, advCountsKey])

    const averageCostWorkerRef = useRef<Worker | null>(null)
    const [averageCostBusy, setAverageCostBusy] = useState(false)
    const [averageCostsResult, setAverageCostsResult] = useState<{ average_costs?: any } | null>(null)
    useEffect(() => {
        runner.start({
            which_one: "AverageCost",
            payloadBuilder,
            workerRef: averageCostWorkerRef,
            setBusy: setAverageCostBusy,
            setResult: setAverageCostsResult,
        })
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [advStrategyKey, expressEventKey, graphBucketSizeKey, dataSizeKey, normalCountsKey, advCountsKey])

    const parserWorkerRef = useRef<Worker | null>(null)
    const [parserResult, setparserResult] = useState<{ upgradeArr: any; unlocks: any; other_strategy_prob_dists: any } | null>(null)
    const [ParserBusy, setParserBusy] = useState(false)
    useEffect(() => {
        runner.start({
            which_one: "ParserUnified",
            payloadBuilder,
            workerRef: parserWorkerRef,
            setBusy: setParserBusy,
            setResult: setparserResult,
        })
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [advStrategyKey, expressEventKey, graphBucketSizeKey, dataSizeKey, normalCountsKey, advCountsKey])

    // Cleanup on unmount: terminate any running workers and clear timers
    useEffect(() => {
        return () => {
            runner.cancel()
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [])

    // styles and column defs moved to ./styles
    const AnythingTicked = useMemo(
        () => normalCounts[0].some((x) => x > 0) || normalCounts[1].some((x) => x > 0) || advCounts[0].some((x) => x > 0) || advCounts[1].some((x) => x > 0),
        [normalCounts, advCounts]
    )
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
            {renderTooltip(tooltip, mainScale, zoomCompensation)}

            <div
                ref={mainRef}
                style={{
                    ...styles.mainContainer,
                    transform: `scale(${mainScale})`,
                    transformOrigin: "top center",
                }}
            >
                <div style={{ display: "flex", flexDirection: "row", gap: 6 }}>
                    <Icon iconName="Forecast Icon" size={64} style={{ display: "flex", alignItems: "center", gap: "12px" }} display_text="" />
                    <h1 style={styles.heading}>Honing Forecast</h1>
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
                        useGridInput={useGridInput}
                        normalCounts={normalCounts}
                        onNumericInputChange={handleNumericInputChange}
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
                            useGridInput={useGridInput}
                            advCounts={advCounts}
                            onNumericInputChange={handleNumericInputChange}
                        />

                        <ControlPanel
                            controlsLeft={null}
                            mainScale={mainScale}
                            fillDemo={fillDemo}
                            fillDemoIncome={fillDemoIncome}
                            // fillRandom={fillRandom}
                            clearAll={clearAll}
                            express_event={express_event}
                            set_express_event={set_express_event}
                            cumulativeGraph={cumulativeGraph}
                            setCumulativeGraph={setCumulativeGraph}
                            dataSize={dataSize}
                            setDataSize={setDataSize}
                            lockXAxis={lockXAxis}
                            onToggleLockXAxis={onToggleLockXAxis}
                            useGridInput={useGridInput}
                            setUseGridInput={handleUseGridInputChange}
                        />
                    </div>
                </div>

                {/* Page Separator */}
                <Separator activePage={activePage} onPageChange={setActivePage} />

                {/* Always-rendered pages with display toggle */}
                <div className={activePage === "chance-to-cost" ? "page" : "page page--hidden"} aria-hidden={activePage !== "chance-to-cost"}>
                    <ChanceToCostSection
                        desired_chance={desired_chance}
                        uncleaned_desired_chance={uncleaned_desired_chance}
                        onDesiredChange={onDesiredChange}
                        onDesiredBlur={onDesiredBlur}
                        cost_result={chanceToCostResult}
                        cost_result_optimized={chanceToCostOptimizedResult}
                        cachedCostGraphData={cachedCostGraphData}
                        AnythingTicked={AnythingTicked}
                        ChanceToCostBusy={chanceToCostBusy}
                        cumulativeGraph={cumulativeGraph}
                        lockXAxis={lockXAxis}
                        lockedMins={lockedMins}
                        lockedMaxs={lockedMaxs}
                        showAverage={showAverage}
                        setShowAverage={setShowAverage}
                        averageCosts={averageCostsResult?.average_costs}
                        AverageCostBusy={averageCostBusy}
                        dataSize={dataSize}
                        budget_inputs={budget_inputs}
                        set_budget_inputs={set_budget_inputs}
                        userMatsValue={userMatsValue}
                        setUserMatsValue={setUserMatsValue}
                    />
                </div>

                <div className={activePage === "cost-to-chance" ? "page" : "page page--hidden"} aria-hidden={activePage !== "cost-to-chance"}>
                    <CostToChanceSection
                        budget_inputs={budget_inputs}
                        set_budget_inputs={set_budget_inputs}
                        userMatsValue={userMatsValue}
                        setUserMatsValue={setUserMatsValue}
                        autoGoldValues={autoGoldValues}
                        setAutoGoldValues={setAutoGoldValues}
                        chance_result={costToChanceResult}
                        cachedChanceGraphData={cachedChanceGraphData}
                        AnythingTicked={AnythingTicked}
                        CostToChanceBusy={costToChanceBusy}
                        cumulativeGraph={cumulativeGraph}
                        lockXAxis={lockXAxis}
                        lockedMins={lockedMins}
                        lockedMaxs={lockedMaxs}
                        desired_chance={desired_chance}
                        uncleaned_desired_chance={uncleaned_desired_chance}
                        onDesiredChange={onDesiredChange}
                        onDesiredBlur={onDesiredBlur}
                        cost_result_optimized={chanceToCostOptimizedResult}
                        showOptimizedDetails={showOptimizedDetails}
                        setShowOptimizedDetails={setShowOptimizedDetails}
                    />
                </div>

                <div className={activePage === "gamba" ? "page" : "page page--hidden"} aria-hidden={activePage !== "cost-to-chance"}>
                    <GambaSection
                        budget_inputs={budget_inputs}
                        set_budget_inputs={set_budget_inputs}
                        userMatsValue={userMatsValue}
                        setUserMatsValue={setUserMatsValue}
                        topGrid={topGrid}
                        bottomGrid={bottomGrid}
                        adv_hone_strategy={adv_hone_strategy}
                        express_event={express_event}
                        desired_chance={desired_chance}
                        bucketCount={bucketCount}
                        autoGoldValues={autoGoldValues}
                        dataSize={dataSize}
                        tooltipHandlers={tooltipHandlers}
                        chance_result={costToChanceResult}
                        cachedChanceGraphData={cachedChanceGraphData}
                        AnythingTicked={AnythingTicked}
                        CostToChanceBusy={costToChanceBusy}
                        cumulativeGraph={cumulativeGraph}
                        lockXAxis={lockXAxis}
                        lockedMins={lockedMins}
                        lockedMaxs={lockedMaxs}
                        useGridInput={useGridInput}
                        normalCounts={normalCounts}
                        advCounts={advCounts}
                        // Moved worker call results
                        upgradeArr={parserResult ? parserResult.upgradeArr : []}
                        ParserBusy={ParserBusy}
                    />
                </div>

                <div className={activePage === "forecast" ? "page" : "page page--hidden"} aria-hidden={activePage !== "forecast"}>
                    <LongTermSection
                        budget_inputs={budget_inputs}
                        set_budget_inputs={set_budget_inputs}
                        userMatsValue={userMatsValue}
                        setUserMatsValue={setUserMatsValue}
                        topGrid={topGrid}
                        bottomGrid={bottomGrid}
                        adv_hone_strategy={adv_hone_strategy}
                        express_event={express_event}
                        bucketCount={bucketCount}
                        autoGoldValues={autoGoldValues}
                        dataSize={dataSize}
                        useGridInput={useGridInput}
                        normalCounts={normalCounts}
                        advCounts={advCounts}
                        incomeArr={incomeArr}
                        setIncomeArr={setIncomeArr}
                        // Desired chance props
                        desired_chance={desired_chance}
                        uncleaned_desired_chance={uncleaned_desired_chance}
                        onDesiredChange={onDesiredChange}
                        onDesiredBlur={onDesiredBlur}
                        // Cost result prop for hundred_budgets
                        // cost_result={chanceToCostResult}
                        //TODOcost_result_optimized={chanceToCostOptimizedResult}
                        showOptimizedDetails={showOptimizedDetails}
                        setShowOptimizedDetails={setShowOptimizedDetails}
                        chanceToCostOptimizedResult={chanceToCostOptimizedResult}
                    />
                </div>
            </div>
        </div>
    )
}
