import React, { useEffect, useRef, useState, useMemo } from 'react'
import CheckboxGrid from '../../components/CheckboxGrid.tsx'
import SpreadsheetGrid from '../../components/SpreadsheetGrid.tsx'
import Graph from '../../components/Graph.tsx'
import Icon from '../../components/Icon.tsx'
import { SpawnWorker } from '../../worker_setup.ts'
import "./CheckboxRow.css"

const INPUT_LABELS = ["Red", "Blue", "Leaps", "Shards", "Oreha", "Gold", "Silver(WIP)", "Red juice", "Blue juice", "Special leaps"]
const OUTPUT_LABELS = ["Red", "Blue", "Leaps", "Shards", "Oreha", "Gold", "Silver(WIP)", "Red juice", "Blue juice",]

function coordsToCell(ref: HTMLDivElement | null, clientX: number, clientY: number, rows: number, cols: number) {
    if (!ref) return { r: 0, c: 0 }
    const rect = ref.getBoundingClientRect()
    const x = Math.max(0, Math.min(clientX - rect.left, rect.width - 1))
    const y = Math.max(0, Math.min(clientY - rect.top, rect.height - 1))
    let cellW = rect.width / cols
    let cellH = rect.height / rows
    const c = Math.floor(x / cellW)
    const r = Math.floor(y / cellH)
    return { r: Math.max(0, Math.min(rows - 1, r)), c: Math.max(0, Math.min(cols - 1, c)) }
}

export default function HoningForecastUI() {
    const STORAGE_KEY = 'HF_UI_STATE_V1'
    const TOP_ROWS = 6
    const TOP_COLS = 25
    const BOTTOM_ROWS = 6
    const BOTTOM_COLS = 4

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
    const [cumulativeGraph, setCumulativeGraph] = useState<boolean>(true)
    const [dataSize, setDataSize] = useState<string>(() => '100000')

    // marquee state & refs (kept here so grids stay presentational)
    const topGridRef = useRef<HTMLDivElement | null>(null)
    const bottomGridRef = useRef<HTMLDivElement | null>(null)
    const mainRef = useRef<HTMLDivElement | null>(null)
    const controlsRef = useRef<HTMLDivElement | null>(null)
    const [marquee, setMarquee] = useState<any>({ active: false, grid: null, startR: 0, startC: 0, endR: 0, endC: 0, startClientX: 0, startClientY: 0, endClientX: 0, endClientY: 0, initialState: false })
    const [marqueeRect, setMarqueeRect] = useState<any>(null)
    const marqueeRef = useRef(marquee)
    useEffect(() => { marqueeRef.current = marquee }, [marquee])

    // ----- Load saved UI state on mount -----
    useEffect(() => {
        try {
            const raw = localStorage.getItem(STORAGE_KEY)
            if (!raw) return
            const parsed = JSON.parse(raw)
            if (parsed && typeof parsed === 'object') {
                if (Array.isArray(parsed.topGrid) && parsed.topGrid.length === TOP_ROWS && parsed.topGrid[0]?.length === TOP_COLS) setTopGrid(parsed.topGrid)
                if (Array.isArray(parsed.bottomGrid) && parsed.bottomGrid.length === BOTTOM_ROWS && parsed.bottomGrid[0]?.length === BOTTOM_COLS) setBottomGrid(parsed.bottomGrid)
                if (typeof parsed.adv_hone_strategy === 'string') set_adv_hone_strategy_change(parsed.adv_hone_strategy)
                if (typeof parsed.express_event === 'boolean') set_express_event(parsed.express_event)
                if (Array.isArray(parsed.prev_checked_arr) && parsed.prev_checked_arr.length === TOP_COLS) set_prev_checked_arr(parsed.prev_checked_arr)
                if (Array.isArray(parsed.prev_checked_arr_bottom) && parsed.prev_checked_arr_bottom.length === BOTTOM_COLS) set_prev_checked_arr_bottom(parsed.prev_checked_arr_bottom)
                if (typeof parsed.desired_chance === 'string') set_desired_chance(parsed.desired_chance)
                if (parsed.budget_inputs && typeof parsed.budget_inputs === 'object') set_budget_inputs(parsed.budget_inputs)
                if (typeof parsed.autoOptimization === 'boolean') setAutoOptimization(parsed.autoOptimization)
                if (parsed.userMatsValue && typeof parsed.userMatsValue === 'object') setUserMatsValue(parsed.userMatsValue)
                if (typeof parsed.cumulativeGraph === 'boolean') setCumulativeGraph(parsed.cumulativeGraph)
                if (typeof parsed.dataSize === 'string') setDataSize(parsed.dataSize)
            }
        } catch (e) {
            // ignore corrupted storage
        }
    }, [])

    // ----- Responsive: horizontal scale for main container and anchored controls -----
    const [mainScale, setMainScale] = useState(1)
    const [controlsLeft, setControlsLeft] = useState<number | null>(null)

    useEffect(() => {
        function recomputeLayout() {
            const mainEl = mainRef.current
            if (!mainEl) return
            // Compute natural width using scrollWidth (not affected by CSS transform)
            const naturalWidth = mainEl.scrollWidth || 1268
            const vw = window.innerWidth
            const scale = Math.min(1, vw / naturalWidth)
            setMainScale(scale)

            // After scaling, use the element's bounding rect to position controls 30px to the right, clamped to viewport
            const rect = mainEl.getBoundingClientRect()
            const controlsWidth = 200 // matches the fixed panel width
            const desiredLeft = rect.right + 30
            const maxLeft = vw - controlsWidth - 16 // keep a small margin from the right edge
            setControlsLeft(Math.min(desiredLeft, Math.max(0, maxLeft)))
        }
        recomputeLayout()
        window.addEventListener('resize', recomputeLayout)
        return () => window.removeEventListener('resize', recomputeLayout)
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

    // useEffect(() => {
    //     fillDemo()
    // }, [])


    const onGridMouseDown = (grid: 'top' | 'bottom', e: React.MouseEvent) => {
        e.preventDefault()
        const ref = grid === 'top' ? topGridRef.current : bottomGridRef.current
        const rows = grid === 'top' ? TOP_ROWS : BOTTOM_ROWS
        const cols = grid === 'top' ? TOP_COLS : BOTTOM_COLS
        const { r, c } = coordsToCell(ref, e.clientX, e.clientY, rows, cols)
        const initialState = grid === 'top' ? topGrid[r][c] : bottomGrid[r][c]
        const next = { active: true, grid, startR: r, startC: c, endR: r, endC: c, startClientX: e.clientX, startClientY: e.clientY, endClientX: e.clientX, endClientY: e.clientY, initialState }
        setMarquee(next)
        marqueeRef.current = next
    }

    useEffect(() => {
        const onMove = (ev: MouseEvent) => {
            const m = marqueeRef.current
            if (!m || !m.active || !m.grid) return
            const grid = m.grid
            const ref = grid === 'top' ? topGridRef.current : bottomGridRef.current
            const rows = grid === 'top' ? TOP_ROWS : BOTTOM_ROWS
            const cols = grid === 'top' ? TOP_COLS : BOTTOM_COLS
            const { r, c } = coordsToCell(ref, ev.clientX, ev.clientY, rows, cols)
            setMarquee((prev: any) => ({ ...prev, endR: r, endC: c, endClientX: ev.clientX, endClientY: ev.clientY }))
        }
        window.addEventListener('mousemove', onMove)
        return () => window.removeEventListener('mousemove', onMove)
    }, [])

    useEffect(() => {
        const onUp = (ev: MouseEvent) => {
            const m = marqueeRef.current
            if (!m || !m.active || !m.grid) return
            const { grid, startClientX, startClientY, endClientX, endClientY, initialState } = m
            const ref = grid === 'top' ? topGridRef.current : bottomGridRef.current
            const rows = grid === 'top' ? TOP_ROWS : BOTTOM_ROWS
            const cols = grid === 'top' ? TOP_COLS : BOTTOM_COLS
            const startCell = coordsToCell(ref, startClientX, startClientY, rows, cols)
            const endCell = coordsToCell(ref, endClientX || ev.clientX, endClientY || ev.clientY, rows, cols)
            const r1 = Math.min(startCell.r, endCell.r)
            const r2 = Math.max(startCell.r, endCell.r)
            const c1 = Math.min(startCell.c, endCell.c)
            const c2 = Math.max(startCell.c, endCell.c)
            const setter = grid === 'top' ? setTopGrid : setBottomGrid
            setter((prev: any) => {
                const copy = prev.map((row: any) => row.slice())
                const newState = !initialState
                for (let rr = r1; rr <= r2; rr++) {
                    for (let cc = c1; cc <= c2; cc++) {
                        if (rr < copy.length && cc < copy[rr].length) copy[rr][cc] = newState
                    }
                }

                // Update column headers based on new state
                if (grid === 'top') {
                    set_prev_checked_arr(prev => {
                        const newArr = [...prev]
                        for (let cc = c1; cc <= c2; cc++) {
                            let allChecked = true
                            for (let rr = 0; rr < rows; rr++) {
                                if (!copy[rr][cc]) {
                                    allChecked = false
                                    break
                                }
                            }
                            newArr[cc] = allChecked
                        }
                        return newArr
                    })
                } else {
                    set_prev_checked_arr_bottom(prev => {
                        const newArr = [...prev]
                        for (let cc = c1; cc <= c2; cc++) {
                            let allChecked = true
                            for (let rr = 0; rr < rows; rr++) {
                                if (!copy[rr][cc]) {
                                    allChecked = false
                                    break
                                }
                            }
                            newArr[cc] = allChecked
                        }
                        return newArr
                    })
                }

                return copy
            })
            setMarquee({ active: false, grid: null, startR: 0, startC: 0, endR: 0, endC: 0, startClientX: 0, startClientY: 0, endClientX: 0, endClientY: 0, initialState: false })
        }
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

    const clearAll = () => {
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

    const fillRandom = () => {
        setTopGrid(Array.from({ length: TOP_ROWS }, () => Array.from({ length: TOP_COLS }, () => Math.random() > 0.7)))
        setBottomGrid(Array.from({ length: BOTTOM_ROWS }, () => Array.from({ length: BOTTOM_COLS }, () => Math.random() > 0.7)))
        set_desired_chance((Math.random() * 100).toFixed(2).toString())
    }

    const fillDemo = () => {
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
        // set_prev_checked_arr_bottom(Array.from({ length: BOTTOM_COLS }, (_, ind) => ind == 2))
    }

    const [chance_result, set_chance_result] = useState<any>(null)
    const [cost_result, set_cost_result] = useState<any>(null)
    const [CostToChanceBusy, setCostToChanceBusy] = useState(false)
    const [ChanceToCostBusy, setChanceToCostBusy] = useState(false)

    // ---------- New: worker refs & debounce refs ----------
    const costWorkerRef = useRef<Worker | null>(null)
    const chanceWorkerRef = useRef<Worker | null>(null)
    const debounceTimerRef1 = useRef<number | null>(null)
    const debounceTimerRef2 = useRef<number | null>(null)

    // helper to create the payload (keeps it consistent)
    const buildPayload = () => ({
        normal_hone_ticks: topGrid,
        adv_hone_ticks: bottomGrid,
        desired_chance: parseFloat(desired_chance || '0'),
        budget: (input => Object.entries(input).map(([, v]) => Math.round(Number(v))))(budget_inputs),
        adv_hone_strategy: adv_hone_strategy,
        express_event: express_event,
        bucket_count: Math.max(2, Math.min(1000, Math.floor(Number(bucketCount) || 2))),
        user_mats_value: autoOptimization ? null : INPUT_LABELS.slice(0, 7).map(label => parseFloat(userMatsValue[label] || '0')),
        data_size: Math.max(1000, Math.floor(Number(dataSize) || 0)),
    })

    // Helper to start a cancelable worker - it terminates any existing worker for this task
    const startCancelableWorker = (which_one: 'CostToChance' | 'ChanceToCost') => {
        const payload = buildPayload()
        // console.log('hi')
        if (which_one === 'CostToChance') {
            // terminate previous
            if (costWorkerRef.current) {
                try { costWorkerRef.current.terminate() } catch (e) { /* ignore */ }
                costWorkerRef.current = null
            }
            setCostToChanceBusy(true)
            set_chance_result(null)

            const { worker, promise } = SpawnWorker(payload, which_one)
            costWorkerRef.current = worker

            promise.then((res) => {
                // only set if this worker is still the current one

                if (costWorkerRef.current === worker) {

                    set_chance_result(res)
                }
            }).catch((err) => {
                console.error('Worker error', err)
                if (costWorkerRef.current === worker) {
                    set_chance_result({ error: String(err) })
                }
            }).finally(() => {
                // cleanup if this worker is the current one
                if (costWorkerRef.current === worker) {
                    try { worker.terminate() } catch (e) { /* ignore */ }
                    costWorkerRef.current = null
                    setCostToChanceBusy(false)
                }
            })

        } else {
            // ChanceToCost
            if (chanceWorkerRef.current) {
                try { chanceWorkerRef.current.terminate() } catch (e) { /* ignore */ }
                chanceWorkerRef.current = null
            }
            setChanceToCostBusy(true)
            set_cost_result(null)

            const { worker, promise } = SpawnWorker(payload, which_one)
            chanceWorkerRef.current = worker

            promise.then((res) => {
                if (chanceWorkerRef.current === worker) {
                    set_cost_result(res)
                }
            }).catch((err) => {
                console.error('Worker error', err)
                if (chanceWorkerRef.current === worker) {
                    set_cost_result({ error: String(err) })
                }
            }).finally(() => {
                if (chanceWorkerRef.current === worker) {
                    try { worker.terminate() } catch (e) { /* ignore */ }
                    chanceWorkerRef.current = null
                    setChanceToCostBusy(false)
                }
            })
        }
    }

    // ---------- Replace / extend your existing HandleCallWorker to use SpawnWorker so manual buttons also cancel ----------
    const HandleCallWorker = async (which_one: string) => {
        // keep the old behavior for manual button calls but make it cancel previous worker using startCancelableWorker
        if (which_one === 'CostToChance') startCancelableWorker('CostToChance')
        else startCancelableWorker('ChanceToCost')
    }

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
            startCancelableWorker('CostToChance')
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
            startCancelableWorker('ChanceToCost')
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

    const styles: any = {
        pageContainer: {
            minHeight: '100vh',
            background: 'var(--bg-primary)',
            /* grid + place-items centers perfectly both axes and plays nicely with margins */
            display: 'grid',
            placeItems: 'center',
            padding: 'var(--spacing-xl)',
            boxSizing: 'border-box',
            paddingBottom: "100px",
        },


        mainContainer: {
            /* use min so it never exceeds viewport minus padding, but caps at 1400px */
            display: 'flex',
            flexDirection: 'column',
            gap: 'var(--spacing-2xl)',
            boxSizing: 'border-box',
            /* optional visual layout niceties */
            margin: 10,
        },
        heading: {
            color: 'var(--text-primary)',
            fontSize: 'var(--font-size-2xl)',
            fontWeight: 'var(--font-weight-bold)',
            marginBottom: 'var(--spacing-sm)'
        },
        sectionTitle: {
            color: 'var(--text-primary)',
            fontSize: 'var(--font-size-lg)',
            fontWeight: 'var(--font-weight-semibold)',

        },
        gridSection: {
            background: 'var(--bg-secondary)',
            borderRadius: 'var(--border-radius)',
            padding: 'var(--spacing-xl)',
            border: '1px solid var(--border-primary)'
        },
        buttonSection: {
            background: 'var(--bg-secondary)',
            borderRadius: 'var(--border-radius)',
            padding: 'var(--spacing-xl)',
            border: '1px solid var(--border-primary)',
            display: 'flex',
            flexDirection: 'column',
            gap: 'var(--spacing-md)',
            alignItems: 'flex-start'
        },
        inputSection: {
            background: 'var(--bg-secondary)',
            borderRadius: 'var(--border-radius)',
            padding: 'var(--spacing-xl)',
            border: '1px solid var(--border-primary)',

        },
        inputLabelCell: {
            width: 100,
            textAlign: 'right',
            paddingRight: 8,
            color: 'var(--text-secondary)',
            whiteSpace: 'nowrap',
            overflow: 'visible',
            textOverflow: 'ellipsis'
        },
        inputCell: {
            flex: 1,
            padding: '6px 45px',
            border: '1px solid var(--border-accent)',
            background: 'transparent',
            color: 'var(--text-primary)',
            borderRadius: 0
        },
        controls: {
            marginTop: 14,
            display: 'flex',
            flexWrap: 'wrap',
            justifyContent: 'flex-end',
            alignItems: 'center'
        },
        primaryButton: {
            background: 'var(--btn-primary)',
            color: 'var(--btn-primary-text)',
            padding: '8px 12px',
            borderRadius: 'var(--border-radius-small)',
            border: 'none',
            cursor: 'pointer'
        },
        successButton: {
            background: 'var(--btn-success)',
            color: 'var(--btn-success-text)',
            padding: '8px 12px',
            borderRadius: 'var(--border-radius-small)',
            border: 'none',
            cursor: 'pointer'
        },
        demoButton: {
            background: 'var(--btn-demo)',
            color: 'var(--btn-demo-text)',
            padding: '8px 16px',
            borderRadius: 'var(--border-radius-small)',
            border: 'none',
            cursor: 'pointer',
            fontSize: 'var(--font-size-sm)'
        }
    }

    const CELL_W = 28
    const CELL_H = 28

    // Column defs for Chance to Cost (always single column, read-only)
    const chanceToCostColumnDefs = useMemo(() => [
        { headerName: 'Estimated cost', field: 'budget', editable: false, flex: 1, cellStyle: { padding: '6px 8px' } }
    ], [])

    // Column defs for Cost to Chance (conditional second column)
    const costToChanceColumnDefs = useMemo(() => {
        if (autoOptimization) {
            return [{ headerName: 'Budget Input', field: 'budget', editable: true, flex: 1, cellStyle: { background: '#072f24', padding: '6px 8px' } }]
        } else {
            return [
                { headerName: 'Budget Input', field: 'budget', editable: true, flex: 1, cellStyle: { background: '#072f24', padding: '6px 8px' } },
                { headerName: 'Gold Value', field: 'matsValue', editable: true, flex: 1, cellStyle: { background: '#072f24', padding: '6px 8px' } }
            ]
        }
    }, [autoOptimization])
    return (

        <div style={styles.pageContainer}>
            {/* Fixed Demo Controls anchored relative to main container (30px to the right, clamped to viewport) */}
            <div ref={controlsRef} style={{ position: 'fixed', left: controlsLeft ?? undefined, right: controlsLeft == null ? 0 : undefined, top: '50%', transform: 'translateY(-50%)', width: 200, zIndex: 1000 }}>
                <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start', gap: 0, width: 200 }}>
                    <h3 style={{ ...styles.sectionTitle, marginTop: '-8px', alignSelf: 'center' }}>Controls</h3>
                    <div style={{ ...styles.buttonSection, marginTop: '-8px', width: '200px' }}>
                        <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--spacing-sm)', width: '100%' }}>
                            <button style={styles.demoButton} onClick={fillDemo}>Fill Demo</button>
                            <button style={styles.demoButton} onClick={fillRandom}>Fill Random</button>
                            <button style={styles.demoButton} onClick={clearAll}>Clear All</button>

                            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginTop: '8px' }}>
                                <label
                                    htmlFor="express_event"
                                    style={{
                                        color: 'var(--text-primary)',
                                        fontSize: 'var(--font-size-sm)',
                                        cursor: 'pointer'
                                    }}
                                >
                                    Express event
                                </label>
                                <input
                                    type="checkbox"
                                    id="express_event"
                                    checked={express_event}
                                    onChange={(e) => set_express_event(e.target.checked)}
                                    style={{
                                        width: '16px',
                                        height: '16px',
                                        cursor: 'pointer'
                                    }}
                                />

                            </div>
                            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                                <label
                                    htmlFor="cumulative_graph"
                                    style={{
                                        color: 'var(--text-primary)',
                                        fontSize: 'var(--font-size-sm)',
                                        cursor: 'pointer'
                                    }}
                                >
                                    Cumulative Graph
                                </label>
                                <input
                                    type="checkbox"
                                    id="cumulative_graph"
                                    checked={cumulativeGraph}
                                    onChange={(e) => setCumulativeGraph(e.target.checked)}
                                    style={{
                                        width: '16px',
                                        height: '16px',
                                        cursor: 'pointer'
                                    }}
                                />
                            </div>
                            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                                <label
                                    htmlFor="data_size"
                                    style={{
                                        color: 'var(--text-primary)',
                                        fontSize: 'var(--font-size-sm)',
                                        cursor: 'pointer',
                                        textWrap: 'nowrap',
                                    }}
                                >
                                    Trial count
                                </label>
                                <input
                                    type="text"
                                    id="data_size"
                                    value={dataSize}
                                    onChange={(e) => {
                                        // allow only digits; strictly positive integer (no zero), strip leading zeros
                                        let v = e.target.value.replace(/[^0-9]/g, '')
                                        v = v.replace(/^0+(?=\d)/, '')
                                        setDataSize(v)
                                    }}
                                    onBlur={() => {
                                        // clamp to 1000 if empty or < 1000
                                        const n = Math.min(1000000, Math.max(1000, Math.floor(Number(dataSize) || 0)))
                                        setDataSize(String(n))
                                    }}
                                    style={{
                                        width: 80,
                                        fontSize: 14,
                                        padding: '6px 8px',
                                        borderRadius: 6,
                                        background: 'var(--input-bg)',
                                        color: 'var(--input-text)',
                                        border: '1px solid var(--input-border)'
                                    }}
                                    placeholder="100000"
                                />
                            </div>
                            {/* <div style={{ width: 200, display: 'flex', gap: '12px' }}>
                                <div style={{ width: "40%", textAlign: 'right', paddingRight: 8, color: 'var(--text-secondary)' }}>Graph bucket size</div>
                                <input
                                    type="text"
                                    value={String(bucketCount)}
                                    onChange={(e) => {
                                        const cleaned = e.target.value.replace(/[^0-9]/g, '')
                                        const n = cleaned == "" ? "" : Math.max(0, Math.min(2000, Number(cleaned || ''))).toString()
                                        setBucketCount(n)
                                    }}
                                    onBlur={() => {
                                        setBucketCount(bucketCount == "" ? "100" : Math.max(2, Number(bucketCount)).toString())
                                    }}
                                    placeholder="100"
                                    style={{
                                        width: 70,
                                        fontSize: 16,
                                        padding: '6px 8px',
                                        borderRadius: 6,
                                        background: 'var(--input-bg)',
                                        color: 'var(--input-text)',
                                        border: '1px solid var(--input-border)'
                                    }}
                                />
                            </div> */}
                        </div>
                    </div>
                </div>
            </div>
            {
                marqueeRect ? (
                    <div style={{ position: 'fixed', left: marqueeRect.left, top: marqueeRect.top, width: marqueeRect.width, height: marqueeRect.height, background: 'var(--marquee-bg)', border: '2px solid var(--marquee-border)', pointerEvents: 'none', zIndex: 9999 }} />
                ) : null
            }


            <div ref={mainRef} style={{ ...styles.mainContainer, transform: `scale(${mainScale})`, transformOrigin: 'top', width: "fit-content" }}>
                <h1 style={styles.heading}>Honing Forecast</h1>

                {/* Normal + Advanced Honing side-by-side */}
                <div style={{ display: 'flex', gap: 'var(--spacing-2xl)', alignItems: "flex-start", flexDirection: 'row' }}>
                    {/* Normal Honing */}
                    <div>
                        <h2 style={{ ...styles.sectionTitle, marginTop: '-8px' }}>Normal Honing</h2>
                        <div style={styles.gridSection}>
                            <div style={{ display: 'flex', gap: 8 }}>
                                <div style={{ width: 100, display: 'flex', flexDirection: 'column', justifyContent: 'flex-start', textWrap: 'nowrap', gap: 0 }}>
                                    {['', 'Helmet', 'Shoulder', 'Chest', 'Pants', 'Glove', 'Weapon'].map((lab) => (
                                        <div key={lab} style={{ height: 28, color: 'var(--text-secondary)', display: 'flex', alignItems: 'center', justifyContent: 'flex-end', paddingRight: 8 }}>
                                            {lab ? <Icon iconName={lab} size={28} style={{ fontSize: 'var(--font-size-sm)' }} /> : ''}
                                        </div>
                                    ))}
                                </div>
                                <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
                                    <div style={{ display: 'grid', gridTemplateColumns: `repeat(${TOP_COLS}, ${CELL_W}px)`, gap: 0, paddingLeft: 1 }}>
                                        {topGrid[0].map((_col, col_num) => {
                                            const label = `+${col_num + 1}`
                                            return (
                                                <div key={label} className="checkbox-item">
                                                    <input
                                                        id={label}
                                                        type="checkbox"
                                                        className="visually-hidden"
                                                        checked={prev_checked_arr[col_num]}
                                                        onChange={() => setTopGrid((prev: any) => {
                                                            const copy = prev.map((row: any) => row.slice())
                                                            const newState = !prev_checked_arr[col_num]
                                                            for (let rr = 0; rr < TOP_ROWS; rr++) {
                                                                copy[rr][col_num] = newState
                                                            }
                                                            set_prev_checked_arr(prev => {
                                                                const newArr = [...prev]
                                                                newArr[col_num] = newState
                                                                return newArr
                                                            })
                                                            return copy
                                                        })}
                                                    />
                                                    <label htmlFor={label} className="box">
                                                        <span className="box-text">{label}</span>
                                                    </label>
                                                </div>
                                            )
                                        })}
                                    </div>
                                    <CheckboxGrid
                                        grid={topGrid}
                                        rows={TOP_ROWS}
                                        cols={TOP_COLS}
                                        gridRef={topGridRef}
                                        onGridMouseDown={onGridMouseDown}
                                        marquee={marquee}
                                        CELL_W={CELL_W}
                                        CELL_H={CELL_H}
                                        gridName="top"
                                    />
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* Advanced Honing */}
                    <div>
                        <h2 style={{ ...styles.sectionTitle, marginTop: '-8px' }}>Advanced Honing</h2>
                        <div style={styles.gridSection}>
                            <div style={{ display: 'flex', gap: 8 }}>
                                <div style={{ display: 'flex', flexDirection: 'column', justifyContent: 'flex-start', textWrap: 'nowrap', gap: 0 }}>
                                    {['', 'Helmet', 'Shoulder', 'Chest', 'Pants', 'Glove', 'Weapon'].map((lab) => (
                                        <div key={""} style={{ height: 28, color: 'var(--text-secondary)', display: 'flex', alignItems: 'center', justifyContent: 'flex-end', paddingRight: 8 }}>
                                            {lab ? <Icon iconName={lab} display_text="" size={28} style={{ fontSize: 'var(--font-size-sm)' }} /> : ''}
                                        </div>
                                    ))}
                                </div>
                                <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
                                    <div style={{ display: 'grid', gridTemplateColumns: `repeat(${BOTTOM_COLS}, ${CELL_W}px)`, gap: 0, paddingLeft: 1 }}>
                                        {[10, 20, 30, 40].map((n, i) => {
                                            const label = `+${n}`
                                            return (
                                                <div key={label} className="checkbox-item">
                                                    <input
                                                        id={label + ' adv'}
                                                        type="checkbox"
                                                        className="visually-hidden"
                                                        checked={prev_checked_arr_bottom[i]}
                                                        onChange={() => setBottomGrid((prev: any) => {
                                                            const copy = prev.map((row: any) => row.slice())
                                                            const newState = !prev_checked_arr_bottom[i]
                                                            for (let rr = 0; rr < BOTTOM_ROWS; rr++) {
                                                                copy[rr][i] = newState
                                                            }
                                                            set_prev_checked_arr_bottom(prev => {
                                                                const newArr = [...prev]
                                                                newArr[i] = newState
                                                                return newArr
                                                            })
                                                            return copy
                                                        })}
                                                    />
                                                    <label htmlFor={label + ' adv'} className="box">
                                                        <span className="box-text">{label}</span>
                                                    </label>
                                                </div>
                                            )
                                        })}
                                    </div>
                                    <CheckboxGrid
                                        grid={bottomGrid}
                                        rows={BOTTOM_ROWS}
                                        cols={BOTTOM_COLS}
                                        gridRef={bottomGridRef}
                                        onGridMouseDown={onGridMouseDown}
                                        marquee={marquee}
                                        CELL_W={CELL_W}
                                        CELL_H={CELL_H}
                                        gridName="bottom"
                                    />
                                </div>
                            </div>

                            <div style={{ marginTop: 16, display: 'flex', justifyContent: 'flex-end' }}>
                                <select
                                    value={adv_hone_strategy}
                                    onChange={(e) => adv_hone_strategy_change(e.target.value)}
                                    style={{
                                        padding: '8px 12px',
                                        borderRadius: 'var(--border-radius-small)',
                                        background: 'var(--bg-tertiary)',
                                        color: 'var(--text-primary)',
                                        border: '1px solid var(--border-secondary)',
                                        fontSize: 'var(--font-size-sm)'
                                    }}
                                >
                                    <option value="No juice">No juice</option>
                                    <option value="Juice on grace">Juice on grace</option>
                                </select>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Input Sections */}
                {/* Chance to Cost Section */}
                <h3 style={{ color: 'var(--text-primary)', fontSize: 'var(--font-size-base)', fontWeight: 'var(--font-weight-semibold)', margin: '0 0 -8px 0' }}>Chance to Cost</h3>
                <div style={{ ...styles.inputSection, width: 1120 }}>

                    <div style={{ display: 'flex', gap: 8, alignItems: 'center', marginBottom: 16 }}>
                        <div style={{ width: 120, textAlign: 'right', paddingRight: 8, color: 'var(--text-secondary)' }}>Desired chance</div>
                        <div style={{ position: 'relative', display: 'flex', alignItems: 'center' }}>
                            <input
                                type="text"
                                value={desired_chance}
                                onChange={(e) => onDesiredChange(e.target.value)}
                                placeholder="0"
                                style={{
                                    width: 160,
                                    fontSize: 16,
                                    padding: '6px 8px',
                                    borderRadius: 6,
                                    background: 'var(--input-bg)',
                                    color: 'var(--input-text)',
                                    border: '1px solid var(--input-border)'
                                }}
                            />
                            <span style={{ position: 'absolute', right: 10, pointerEvents: 'none', color: "black" }}>%</span>
                        </div>
                    </div>


                    <div style={{ display: 'flex', gap: 110, alignItems: 'flex-start' }}>
                        <div>
                            {/* <div style={{ fontWeight: 'var(--font-weight-semibold)', color: 'var(--text-primary)', marginBottom: 8 }}>Estimated cost</div> */}
                            <div style={{ marginBottom: 16, width: 210 }}>
                                <SpreadsheetGrid
                                    columnDefs={chanceToCostColumnDefs}
                                    labels={OUTPUT_LABELS}
                                    sheet_values={cost_result ? Object.fromEntries(OUTPUT_LABELS.map(label => [label, cost_result[label] != null ? String(cost_result[label]) : 'No results yet'])) : Object.fromEntries(OUTPUT_LABELS.map(label => [label, 'No results yet']))}
                                    set_sheet_values={() => { }} // No-op for read-only
                                    readOnly={true}
                                />
                            </div>
                            {cost_result && (
                                <pre style={{ color: 'var(--text-muted)', fontSize: 'var(--font-size-xs)', marginTop: 8 }}>
                                    Run time: {cost_result.run_time}s{'\n'}{cost_result.actual_prob}
                                </pre>
                            )}
                        </div>
                        <div style={{ flex: 1 }}>
                            <Graph
                                title="Cost distribution"
                                labels={OUTPUT_LABELS}
                                counts={cost_result?.hist_counts}
                                mins={cost_result?.hist_mins}
                                maxs={cost_result?.hist_maxs}
                                width={640}
                                height={320}
                                budgets={cost_result && OUTPUT_LABELS.map(label => Number(cost_result[label]))}
                                hasSelection={topGrid.some(value => value.some(v => v === true))}
                                isLoading={ChanceToCostBusy}
                                cumulative={cumulativeGraph}
                            />
                        </div>
                    </div>
                </div>



                {/* Cost to Chance Section */}
                <h3 style={{ color: 'var(--text-primary)', fontSize: 'var(--font-size-base)', fontWeight: 'var(--font-weight-semibold)', margin: '16px 0 0px 0' }}>Cost to Chance</h3>
                <div style={{ ...styles.inputSection, flexDirection: "row", width: 1120 }}>
                    {/* <div style={{ fontWeight: 'var(--font-weight-semibold)', color: 'var(--text-primary)', marginBottom: 8 }}>Input your budget here</div> */}

                    <div style={{ display: 'flex', gap: autoOptimization ? 110 : 20, alignItems: 'flex-start' }}>

                        <div style={{ display: 'flex', flexDirection: "column", gap: 0, alignItems: 'flex-start', justifyContent: 'start', width: autoOptimization ? 210 : 300 }}>
                            <div style={{ marginBottom: 16, width: autoOptimization ? 210 : 310 }}>
                                <SpreadsheetGrid
                                    columnDefs={costToChanceColumnDefs}
                                    labels={INPUT_LABELS}
                                    sheet_values={budget_inputs}
                                    set_sheet_values={set_budget_inputs}
                                    secondaryValues={userMatsValue}
                                    setSecondaryValues={setUserMatsValue}
                                />
                            </div>


                            <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
                                <input
                                    type="checkbox"
                                    id="auto-optimization"
                                    checked={autoOptimization}
                                    onChange={(e) => setAutoOptimization(e.target.checked)}
                                />
                                <label htmlFor="auto-optimization" style={{ color: 'var(--text-primary)', fontSize: 'var(--font-size-sm)', whiteSpace: 'nowrap' }}>
                                    Automatic juice and special leap optimization(Normal honing only)
                                </label>
                            </div>

                            <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 8 }}>
                                <div style={{ ...styles.inputLabelCell, whiteSpace: 'nowrap', backgroundColor: 'var(--bg-tertiary)' }}>Chance of Success</div>
                                <div style={{ ...styles.inputCell, border: 'none', background: 'transparent', color: 'var(--text-success)', fontSize: 'var(--font-size-xl)' }}>{chance_result ? (String(chance_result.chance) + '%') : '-'}</div>
                                {chance_result && (Number(chance_result.budgets_red_remaining) < 0 || Number(chance_result.budgets_blue_remaining) < 0) && (
                                    <div style={{ marginLeft: 12, color: '#f59e0b', fontSize: 'var(--font-size-sm)', whiteSpace: 'nowrap' }}>
                                        Invalid result!  Missing {Number(chance_result.budgets_red_remaining) < 0 ? (Number(chance_result.budgets_red_remaining) * -1).toString() + ' red juice' : ''}
                                        {(Number(chance_result.budgets_red_remaining) < 0 && Number(chance_result.budgets_blue_remaining) < 0) ? ' and ' : ''}
                                        {Number(chance_result.budgets_blue_remaining) < 0 ? (Number(chance_result.budgets_blue_remaining) * -1).toString() + ' blue juice' : ""} for Advanced honing(on average), use No Juice option or add more juice
                                    </div>
                                )}
                            </div>

                            {chance_result && (
                                <div style={{ marginTop: 4, color: 'var(--text-muted)', fontSize: 'var(--font-size-xs)' }}>Run time: {chance_result.run_time}s</div>
                            )}
                            {(chance_result && (chance_result.upgrade_strings?.length > 0 || chance_result.juice_order_armor?.length > 0 || chance_result.juice_order_weapon?.length > 0)) && (
                                <div style={{ display: 'flex', gap: 0, alignItems: 'flex-start', marginTop: 8 }}>
                                    <div>
                                        <div style={{ marginTop: 0, ...styles.inputLabelCell, whiteSpace: 'nowrap' }}>Reasons for failures/bottlenecks:</div>
                                        <div style={{ marginTop: 4, color: 'var(--text-muted)', fontSize: 'var(--font-size-sm)', whiteSpace: "wrap", width: 300 }}>
                                            {(chance_result.reasons || []).map((s: string, idx: number) => (
                                                <div key={idx}>{idx + 1}. {s}</div>
                                            ))}
                                        </div>
                                    </div>
                                    <div>
                                        <div style={{ ...styles.inputLabelCell, whiteSpace: 'nowrap' }}>Free taps value ranking:</div>
                                        <div style={{ marginTop: 4, color: 'var(--text-muted)', fontSize: 'var(--font-size-sm)', width: 200 }}>
                                            {(chance_result.upgrade_strings || []).map((upgrade: string, index: number) => (
                                                <div key={index}>{index + 1}. {upgrade}</div>
                                            ))}
                                        </div>
                                    </div>
                                    <div>
                                        <div style={{ ...styles.inputLabelCell, whiteSpace: 'nowrap' }}>Red juice (weapon):</div>
                                        <div style={{ marginTop: 4, color: 'var(--text-muted)', fontSize: 'var(--font-size-sm)', width: 200 }}>
                                            {(chance_result.juice_order_weapon || []).map((s: string, idx: number) => (
                                                <div key={idx}>{idx + 1}. {s}</div>
                                            ))}
                                        </div>
                                    </div>
                                    <div>
                                        <div style={{ ...styles.inputLabelCell, whiteSpace: 'nowrap' }}>Blue juice (armor):</div>
                                        <div style={{ marginTop: 4, color: 'var(--text-muted)', fontSize: 'var(--font-size-sm)', width: 200 }}>
                                            {(chance_result.juice_order_armor || []).map((s: string, idx: number) => (
                                                <div key={idx}>{idx + 1}. {s}</div>
                                            ))}
                                        </div>
                                    </div>

                                </div>
                            )}
                        </div>
                        <div style={{ flex: 1 }}>
                            <Graph
                                title="Cost distribution"
                                labels={OUTPUT_LABELS}
                                counts={chance_result?.hist_counts}
                                mins={chance_result?.hist_mins}
                                maxs={chance_result?.hist_maxs}
                                width={640}
                                height={320}
                                // displayMode="cost"
                                budgets={OUTPUT_LABELS.map(label => Number(budget_inputs[label]))}
                                hasSelection={topGrid.some(value => value.some(v => v === true))}
                                isLoading={CostToChanceBusy}
                                cumulative={cumulativeGraph}
                            />
                        </div>

                    </div>

                </div>
            </div>

        </div >

    )

}
