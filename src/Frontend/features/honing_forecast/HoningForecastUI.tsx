import React, { useEffect, useRef, useState, useMemo } from 'react'
import { SpawnWorker } from '../../worker_setup.ts'
import "./CheckboxRow.css"
import { styles } from './styles.ts'
import { INPUT_LABELS, OUTPUT_LABELS, STORAGE_KEY, TOP_ROWS, TOP_COLS, BOTTOM_ROWS, BOTTOM_COLS, CELL_W, CELL_H } from './constants.ts'
import { readSettings, writeSettings } from './Settings.ts'
import HoningControls from './HoningControls.tsx'
import NormalHoningPanel from './NormalHoningPanel.tsx'
import AdvancedHoningPanel from './AdvancedHoningPanel.tsx'
import ChanceToCostSection from './ChanceToCostSection.tsx'
import CostToChanceSection from './CostToChanceSection.tsx'
import GambaSection from "./GambaSection.tsx"
import Separator from './Separator.tsx'
import { recomputeLayout } from "./Layout.ts"
import { GridMouseDownLogic, mouseMoveLogic, createMouseUpHandler } from "./Marquee.ts"
import { createClearAll, createFillRandom, createFillDemo } from './Control.ts'
import { buildPayload, createStartCancelableWorker, createHandleCallWorker } from './Debounce.ts'
import { TooltipState, createTooltipHandlers, renderTooltip } from './Tooltip.tsx'

// constants and helpers moved to ./constants and ./utils

export default function HoningForecastUI() {

    const [topGrid, setTopGrid] = useState(() => Array.from({ length: TOP_ROWS }, () => Array.from({ length: TOP_COLS }, () => false)))
    const [bottomGrid, setBottomGrid] = useState(() => Array.from({ length: BOTTOM_ROWS }, () => Array(BOTTOM_COLS).fill(false)))
    const [budget_inputs, set_budget_inputs] = useState(() => Object.fromEntries(INPUT_LABELS.map((l) => [l, '0'])))
    const [autoOptimization, setAutoOptimization] = useState(true)
    const [userMatsValue, setUserMatsValue] = useState(() => Object.fromEntries(INPUT_LABELS.slice(0, 7).map((l) => (l == "Gold") ? [l, "1"] : [l, '0'])))
    const [desired_chance, set_desired_chance] = useState(() => '50')
    const [adv_hone_strategy, set_adv_hone_strategy_change] = useState(() => 'No juice')
    const [express_event, set_express_event] = useState(() => true)
    const [bucketCount, _setBucketCount] = useState(() => "100")
    const [prev_checked_arr, set_prev_checked_arr] = useState(() => Array.from({ length: TOP_COLS }, () => false))
    const [prev_checked_arr_bottom, set_prev_checked_arr_bottom] = useState(() => Array.from({ length: BOTTOM_COLS }, () => false))
    const [cumulativeGraph, setCumulativeGraph] = useState<boolean>(false)
    const [dataSize, setDataSize] = useState<string>(() => '100000')
    const [activePage, setActivePage] = useState<'chance-to-cost' | 'cost-to-chance' | 'gamba'>('chance-to-cost')

    // marquee state & refs (kept here so grids stay presentational)
    const topGridRef = useRef<HTMLDivElement | null>(null)
    const bottomGridRef = useRef<HTMLDivElement | null>(null)
    const mainRef = useRef<HTMLDivElement | null>(null)
    // const controlsRef = useRef<HTMLDivElement | null>(null)
    const [marquee, setMarquee] = useState<any>({ active: false, grid: null, startR: 0, startC: 0, endR: 0, endC: 0, startClientX: 0, startClientY: 0, endClientX: 0, endClientY: 0, initialState: false })
    const [marqueeRect, setMarqueeRect] = useState<any>(null)
    const marqueeRef = useRef(marquee)
    useEffect(() => { marqueeRef.current = marquee }, [marquee])

    // tooltip state & handlers
    const [tooltip, setTooltip] = useState<TooltipState>({
        visible: false,
        type: null,
        x: 0,
        y: 0,
        content: null,
        upgradeData: null
    })
    const tooltipHandlers = createTooltipHandlers(setTooltip)

    // ----- Load saved UI state on mount -----
    useEffect(() => {
        try {
            readSettings(setTopGrid,
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
                setDataSize)

        } catch (e) {
            // ignore corrupted storage
        }
    }, [])

    // ----- Responsive: horizontal scale for main container and anchored controls -----
    const [mainScale, setMainScale] = useState(1)
    const [controlsLeft, setControlsLeft] = useState<number | null>(null)
    useEffect(() => {
        recomputeLayout(mainRef, setMainScale, setControlsLeft)
        const temp_fn = function (_) { recomputeLayout(mainRef, setMainScale, setControlsLeft) }
        window.addEventListener('resize', temp_fn,)
        return () => window.removeEventListener('resize', temp_fn,)
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
                    autoOptimization,
                    userMatsValue,
                    cumulativeGraph,
                    dataSize,)
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
    }, [topGrid, bottomGrid, adv_hone_strategy, express_event, prev_checked_arr, prev_checked_arr_bottom, desired_chance, budget_inputs, autoOptimization, userMatsValue, cumulativeGraph, dataSize])

    const onGridMouseDown = GridMouseDownLogic({
        topGridRef,
        bottomGridRef,
        marqueeRef,
        topGrid,
        bottomGrid,
        setMarquee,
    });

    useEffect(() => {
        const temp_fn = function (ev: MouseEvent) { mouseMoveLogic(ev, marqueeRef, topGridRef, bottomGridRef, setMarquee) }
        window.addEventListener('mousemove', temp_fn)
        return () => window.removeEventListener('mousemove', temp_fn)
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
        window.addEventListener('mouseup', onUp)
        return () => window.removeEventListener('mouseup', onUp)
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
        const cleanValue = value.replace(/[^0-9.]/g, '')
        set_desired_chance(cleanValue)
    }
    const adv_hone_strategy_change = (value: string) => set_adv_hone_strategy_change(value)

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
        setAutoOptimization,
        _setBucketCount,
        setCumulativeGraph,
        setDataSize,
    })

    const fillRandom = createFillRandom({
        setTopGrid,
        setBottomGrid,
        set_desired_chance,
    })

    const fillDemo = createFillDemo({
        setTopGrid,
        setBottomGrid,
        set_budget_inputs,
        set_desired_chance,
        set_prev_checked_arr,
    })

    const [chance_result, set_chance_result] = useState<any>(null)
    const [cost_result, set_cost_result] = useState<any>(null)
    const [CostToChanceBusy, setCostToChanceBusy] = useState(false)
    const [ChanceToCostBusy, setChanceToCostBusy] = useState(false)

    // Cached graph data to preserve during recomputation
    const [cachedChanceGraphData, setCachedChanceGraphData] = useState<{ hist_counts?: any, hist_mins?: any, hist_maxs?: any } | null>(null)
    const [cachedCostGraphData, setCachedCostGraphData] = useState<{ hist_counts?: any, hist_mins?: any, hist_maxs?: any } | null>(null)

    // ---------- New: worker refs & debounce refs ----------
    const costWorkerRef = useRef<Worker | null>(null)
    const chanceWorkerRef = useRef<Worker | null>(null)
    const debounceTimerRef1 = useRef<number | null>(null)
    const debounceTimerRef2 = useRef<number | null>(null)

    const payloadBuilder = () => buildPayload({
        topGrid,
        bottomGrid,
        desired_chance,
        budget_inputs,
        adv_hone_strategy,
        express_event,
        bucketCount,
        autoOptimization,
        userMatsValue,
        dataSize,
    })

    const startCancelableWorker = createStartCancelableWorker({
        costWorkerRef,
        chanceWorkerRef,
        setCostToChanceBusy,
        setChanceToCostBusy,
        set_chance_result,
        set_cost_result,
        setCachedChanceGraphData,
        setCachedCostGraphData,
    })

    const HandleCallWorker = createHandleCallWorker({
        startCancelableWorker,
        buildPayload: payloadBuilder,
    })

    // ---------- Automatic triggers with debounce ----------
    // We'll watch serialized versions of the inputs to detect deep changes
    const topGridKey = useMemo(() => JSON.stringify(topGrid), [topGrid])
    const bottomGridKey = useMemo(() => JSON.stringify(bottomGrid), [bottomGrid])
    const budgetKey = useMemo(() => JSON.stringify(budget_inputs), [budget_inputs])
    const desiredKey = useMemo(() => String(desired_chance), [desired_chance])
    const advStrategyKey = useMemo(() => String(adv_hone_strategy), [adv_hone_strategy])
    const expressEventKey = useMemo(() => String(express_event), [express_event])
    const graphBucketSizeKey = useMemo(() => String(bucketCount), [bucketCount])
    const autoOptKey = useMemo(() => String(autoOptimization), [autoOptimization])
    const userMatsKey = useMemo(() => JSON.stringify(userMatsValue), [userMatsValue])
    const dataSizeKey = useMemo(() => String(dataSize), [dataSize])

    // When budget or grids or strategy change -> run CostToChance (budget -> cost->chance)
    useEffect(() => {
        // clear existing timer
        if (debounceTimerRef1.current) {
            window.clearTimeout(debounceTimerRef1.current)
            debounceTimerRef1.current = null
        }
        // start new delayed work
        debounceTimerRef1.current = window.setTimeout(() => {
            const payload = payloadBuilder()
            startCancelableWorker('CostToChance', payload)
            debounceTimerRef1.current = null
        }, 100) // 100ms debounce
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [budgetKey, topGridKey, bottomGridKey, advStrategyKey, expressEventKey, graphBucketSizeKey, autoOptKey, userMatsKey, dataSizeKey])

    // When desired chance or grids or strategy change -> run ChanceToCost (chance -> cost)
    useEffect(() => {
        if (debounceTimerRef2.current) {
            window.clearTimeout(debounceTimerRef2.current)
            debounceTimerRef2.current = null
        }
        debounceTimerRef2.current = window.setTimeout(() => {
            const payload = payloadBuilder()
            startCancelableWorker('ChanceToCost', payload)
            debounceTimerRef2.current = null
        }, 100)
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [desiredKey, topGridKey, bottomGridKey, advStrategyKey, expressEventKey, graphBucketSizeKey, dataSizeKey])

    // Cleanup on unmount: terminate any running workers and clear timers
    useEffect(() => {
        return () => {
            if (costWorkerRef.current) {
                try { costWorkerRef.current.terminate() } catch (e) { ; }
                costWorkerRef.current = null
            }
            if (chanceWorkerRef.current) {
                try { chanceWorkerRef.current.terminate() } catch (e) { ; }
                chanceWorkerRef.current = null
            }
            if (debounceTimerRef1.current) {
                window.clearTimeout(debounceTimerRef1.current)
                debounceTimerRef1.current = null
            }
            if (debounceTimerRef2.current) {
                window.clearTimeout(debounceTimerRef2.current)
                debounceTimerRef2.current = null
            }
        }
    }, [])

    // styles and column defs moved to ./styles
    const AnythingTicked = useMemo(() => topGrid.some(value => value.some(v => v === true)) || bottomGrid.some(value => value.some(v => v === true)), [topGrid, bottomGrid])
    return (

        <div style={styles.pageContainer}>
            {/* Fixed Demo Controls anchored relative to main container (30px to the right, clamped to viewport) */}
            <HoningControls
                controlsLeft={controlsLeft}
                mainScale={mainScale}
                fillDemo={fillDemo}
                fillRandom={fillRandom}
                clearAll={clearAll}
                express_event={express_event}
                set_express_event={set_express_event}
                cumulativeGraph={cumulativeGraph}
                setCumulativeGraph={setCumulativeGraph}
                dataSize={dataSize}
                setDataSize={setDataSize}
            />
            {
                marqueeRect ? (
                    <div style={{ position: 'fixed', left: marqueeRect.left, top: marqueeRect.top, width: marqueeRect.width, height: marqueeRect.height, background: 'var(--marquee-bg)', border: '2px solid var(--marquee-border)', pointerEvents: 'none', zIndex: 9999 }} />
                ) : null
            }
            {renderTooltip(tooltip)}


            <div ref={mainRef} style={{ ...styles.mainContainer, transform: `scale(${mainScale})`, transformOrigin: 'top', width: "fit-content" }}>
                <h1 style={styles.heading}>Honing Forecast</h1>

                {/* Normal + Advanced Honing side-by-side */}
                <div style={{ display: 'flex', gap: 'var(--spacing-2xl)', alignItems: "flex-start", flexDirection: 'row' }}>
                    <NormalHoningPanel
                        topGrid={topGrid}
                        setTopGrid={setTopGrid}
                        prev_checked_arr={prev_checked_arr}
                        set_prev_checked_arr={set_prev_checked_arr}
                        topGridRef={topGridRef}
                        marquee={marquee}
                        onGridMouseDown={onGridMouseDown}
                    />

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
                </div>

                {/* Page Separator */}
                <Separator activePage={activePage} onPageChange={setActivePage} tooltipHandlers={tooltipHandlers} />

                {/* Conditional Input Sections */}
                {activePage === 'chance-to-cost' && (
                    <ChanceToCostSection
                        desired_chance={desired_chance}
                        onDesiredChange={onDesiredChange}
                        cost_result={cost_result}
                        cachedCostGraphData={cachedCostGraphData}
                        AnythingTicked={AnythingTicked}
                        ChanceToCostBusy={ChanceToCostBusy}
                        cumulativeGraph={cumulativeGraph}
                    />
                )}

                {activePage === 'cost-to-chance' && (
                    <CostToChanceSection
                        budget_inputs={budget_inputs}
                        set_budget_inputs={set_budget_inputs}
                        userMatsValue={userMatsValue}
                        setUserMatsValue={setUserMatsValue}
                        autoOptimization={autoOptimization}
                        setAutoOptimization={setAutoOptimization}
                        chance_result={chance_result}
                        cachedChanceGraphData={cachedChanceGraphData}
                        AnythingTicked={AnythingTicked}
                        CostToChanceBusy={CostToChanceBusy}
                        cumulativeGraph={cumulativeGraph}
                    />
                )}

                {activePage === 'gamba' && (
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
                        autoOptimization={autoOptimization}
                        dataSize={dataSize}
                        tooltipHandlers={tooltipHandlers}
                        chance_result={chance_result}
                        cachedChanceGraphData={cachedChanceGraphData}
                        AnythingTicked={AnythingTicked}
                        CostToChanceBusy={CostToChanceBusy}
                        cumulativeGraph={cumulativeGraph}
                    />
                )}
            </div>

        </div >

    )

}
