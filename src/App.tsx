


import React, { useEffect, useRef, useState, useMemo } from "react"
import { AgGridReact } from 'ag-grid-react'
import 'ag-grid-community/styles/ag-theme-alpine.css'
import './ag-grid-dark.css'
import { ModuleRegistry, AllCommunityModule } from 'ag-grid-community'
import { themeBalham } from 'ag-grid-community';
// Register all community modules so AG Grid features are available
ModuleRegistry.registerModules([AllCommunityModule])
import { CallWorker } from "./worker_setup.js"
import { clearObjectStore } from "./workers/Cache.js"
const INPUT_LABELS = ["Red", "Blue", "Leaps", "Shards", "Oreha", "Gold", "Silver(WIP)", "Red juice", "Blue juice", "Special leaps"]



// This UI is unbelievably vibe coded, need to split things up into functions for readability TODO
export default function UpgradeCalculatorUI() {
    const TOP_ROWS = 6
    const TOP_COLS = 25
    const BOTTOM_ROWS = 6
    const BOTTOM_COLS = 4


    // Top grid state (6 x 25)
    const [topGrid, setTopGrid] = useState(() => Array.from({ length: TOP_ROWS }, () => Array(TOP_COLS).fill(false)))

    // Bottom grid state (6 x 4)
    const [bottomGrid, setBottomGrid] = useState(() => Array.from({ length: BOTTOM_ROWS }, () => Array(BOTTOM_COLS).fill(false)))

    // Inputs column (10 inputs)
    const [budget_inputs, set_budget_inputs] = useState(() => Object.fromEntries(INPUT_LABELS.map((l) => [l, ""])))

    const [desired_chance, set_desired_chance] = useState(() => "50")
    const [adv_hone_strategy, set_adv_hone_strategy_change] = useState(() => "No juice")

    // Marquee selection for grids (rectangle select)
    const topGridRef = useRef<HTMLDivElement | null>(null)
    const bottomGridRef = useRef<HTMLDivElement | null>(null)
    const [marquee, setMarquee] = useState<{
        active: boolean
        grid: "top" | "bottom" | null
        startR: number
        startC: number
        endR: number
        endC: number
        startClientX: number
        startClientY: number
        endClientX: number
        endClientY: number
        initialState: boolean
    }>({ active: false, grid: null, startR: 0, startC: 0, endR: 0, endC: 0, startClientX: 0, startClientY: 0, endClientX: 0, endClientY: 0, initialState: false })
    const [marqueeRect, setMarqueeRect] = useState<{ left: number; top: number; width: number; height: number } | null>(null)

    // react-data-grid will handle selection, editing and paste behavior for inputs

    // Utility to compute row/col from client coordinates inside a grid ref
    const coordsToCell = (ref: HTMLDivElement | null, clientX: number, clientY: number, rows: number, cols: number) => {
        if (!ref) return { r: 0, c: 0 }
        const rect = ref.getBoundingClientRect()
        const x = Math.max(0, Math.min(clientX - rect.left, rect.width - 1))
        const y = Math.max(0, Math.min(clientY - rect.top, rect.height - 1))
        // Prefer measuring an actual child cell to account for gaps/margins
        let cellW = rect.width / cols
        let cellH = rect.height / rows
        const c = Math.floor(x / cellW)
        const r = Math.floor(y / cellH)
        return { r: Math.max(0, Math.min(rows - 1, r)), c: Math.max(0, Math.min(cols - 1, c)) }
    }

    const onGridMouseDown = (grid: "top" | "bottom", e: React.MouseEvent) => {
        e.preventDefault()
        const ref = grid === "top" ? topGridRef.current : bottomGridRef.current
        const rows = grid === "top" ? TOP_ROWS : BOTTOM_ROWS
        const cols = grid === "top" ? TOP_COLS : BOTTOM_COLS
        const { r, c } = coordsToCell(ref, e.clientX, e.clientY, rows, cols)
        const initialState = grid === "top" ? topGrid[r][c] : bottomGrid[r][c]
        const next = { active: true, grid, startR: r, startC: c, endR: r, endC: c, startClientX: e.clientX, startClientY: e.clientY, endClientX: e.clientX, endClientY: e.clientY, initialState }
        setMarquee(next)
        // ensure the ref is current for global listeners
        marqueeRef.current = next
    }

    // global mousemove: consult marqueeRef.current to avoid stale-closure misses
    useEffect(() => {
        const onMove = (ev: MouseEvent) => {
            const m = marqueeRef.current
            if (!m || !m.active || !m.grid) return
            const grid = m.grid
            const ref = grid === "top" ? topGridRef.current : bottomGridRef.current
            const rows = grid === "top" ? TOP_ROWS : BOTTOM_ROWS
            const cols = grid === "top" ? TOP_COLS : BOTTOM_COLS
            const { r, c } = coordsToCell(ref, ev.clientX, ev.clientY, rows, cols)
            setMarquee((prev) => ({ ...prev, endR: r, endC: c, endClientX: ev.clientX, endClientY: ev.clientY }))
        }
        window.addEventListener("mousemove", onMove)
        return () => window.removeEventListener("mousemove", onMove)
    }, [])

    // use a ref to hold latest marquee so a single global mouseup listener won't miss quick releases
    const marqueeRef = useRef(marquee)
    useEffect(() => {
        marqueeRef.current = marquee
    }, [marquee])

    useEffect(() => {
        const onUp = (ev: MouseEvent) => {
            const m = marqueeRef.current
            if (!m || !m.active || !m.grid) return
            const { grid, startClientX, startClientY, endClientX, endClientY, initialState } = m
            const ref = grid === "top" ? topGridRef.current : bottomGridRef.current
            const rows = grid === "top" ? TOP_ROWS : BOTTOM_ROWS
            const cols = grid === "top" ? TOP_COLS : BOTTOM_COLS
            const startCell = coordsToCell(ref, startClientX, startClientY, rows, cols)
            const endCell = coordsToCell(ref, endClientX || ev.clientX, endClientY || ev.clientY, rows, cols)
            const r1 = Math.min(startCell.r, endCell.r)
            const r2 = Math.max(startCell.r, endCell.r)
            const c1 = Math.min(startCell.c, endCell.c)
            const c2 = Math.max(startCell.c, endCell.c)
            const setter = grid === "top" ? setTopGrid : setBottomGrid
            setter((prev) => {
                const copy = prev.map((row) => row.slice())
                const newState = !initialState
                for (let rr = r1; rr <= r2; rr++) {
                    for (let cc = c1; cc <= c2; cc++) {
                        if (rr < copy.length && cc < copy[rr].length) copy[rr][cc] = newState
                    }
                }
                return copy
            })
            setMarquee({ active: false, grid: null, startR: 0, startC: 0, endR: 0, endC: 0, startClientX: 0, startClientY: 0, endClientX: 0, endClientY: 0, initialState: false })
        }
        window.addEventListener('mouseup', onUp)
        return () => window.removeEventListener('mouseup', onUp)
    }, [])

    // compute marquee overlay rect for visual box
    useEffect(() => {
        if (!marquee.active || !marquee.grid) {
            setMarqueeRect(null)
            return
        }
        // compute overlay directly from client coords so it follows the mouse exactly
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

    // Input change handled by react-data-grid onRowsChange

    const onDesiredChange = (value: string) => {
        set_desired_chance(() => value)
    }
    const adv_hone_strategy_change = (value: string) => {
        set_adv_hone_strategy_change(() => value)
    }

    const clearAll = () => {
        setTopGrid(Array.from({ length: TOP_ROWS }, () => Array(TOP_COLS).fill(false)))
        setBottomGrid(Array.from({ length: BOTTOM_ROWS }, () => Array(BOTTOM_COLS).fill(false)))
        set_budget_inputs(Object.fromEntries(INPUT_LABELS.map((l) => [l, ""])))
        set_desired_chance("50")
    }

    const fillRandom = () => {
        setTopGrid(Array.from({ length: TOP_ROWS }, () => Array.from({ length: TOP_COLS }, () => Math.random() > 0.7)))
        setBottomGrid(Array.from({ length: BOTTOM_ROWS }, () => Array.from({ length: BOTTOM_COLS }, () => Math.random() > 0.7)))
        set_desired_chance((Math.random() * 100).toFixed(2).toString())
    }
    const fillMy = () => {
        setTopGrid(Array.from({ length: TOP_ROWS }, () => Array.from({ length: TOP_COLS }, (_, ind) => ind == 19 || ind == 20)))
        setBottomGrid(Array.from({ length: BOTTOM_ROWS }, () => Array.from({ length: BOTTOM_COLS }, (_, ind) => ind == 2)))
        set_budget_inputs({
            Red: "431777",
            Blue: "1064398",
            Leaps: "23748",
            Shards: "9010948",
            Oreha: "15125",
            Gold: "1803792",
            "Silver(WIP)": "4294967295",
            "Red juice": "0",
            "Blue juice": "0",
            "Special leaps": "0",
        }),
            set_desired_chance("69")
    }

    const [chance_result, set_chance_result] = useState<any>(null)
    const [cost_result, set_cost_result] = useState<any>(null)
    const [CostToChanceBusy, setCostToChanceBusy] = useState(false)
    const [ChanceToCostBusy, setChanceToCostBusy] = useState(false)

    const HandleCallWorker = async (which_one: string) => {
        try {
            if (which_one == "CostToChance") {
                setCostToChanceBusy(true)
                set_chance_result(null)

            } else {
                setChanceToCostBusy(true)
                set_cost_result(null)
            }

            const payload =
            {
                normal_hone_ticks: topGrid,
                adv_hone_ticks: bottomGrid,
                desired_chance: parseFloat(desired_chance),
                budget: (input =>
                    Object.entries(input).map(([, v]) => Math.round(Number(v)))
                )(budget_inputs),
                adv_hone_strategy: adv_hone_strategy,
            }

            const res = await CallWorker(payload, which_one)
            if (which_one == "CostToChance") {
                set_chance_result(res)
            } else {
                set_cost_result(res)
            }
        } catch (err) {
            console.error("Worker error", err)
            if (which_one == "CostToChance") {
                set_chance_result({ error: String(err) })
            } else {
                set_cost_result({ error: String(err) })
            }
        } finally {
            if (which_one == "CostToChance") {
                setCostToChanceBusy(false)
            } else {
                setChanceToCostBusy(false)
            }
        }
    }

    // Simple compact styles (so you don't need Tailwind to see a usable layout)
    const styles = {
        // Constrain width so the UI centers in the browser window
        container: {
            display: "flex",
            flexDirection: "column",
            alignItems: "center",        // <--- use alignItems (not alignContent) to center horizontally
            justifyContent: "center",
            fontFamily: "system-ui, Arial, sans-serif",
            paddingLeft: Math.max((window.innerWidth - 1000) / 2, 0),
            paddingTop: 30,
            width: "min(800px, 100%)",  // keeps layout from spanning full viewport
            margin: "0 auto",

        } as React.CSSProperties,

        // If you want the heading centered like everything else:
        heading: {
            marginBottom: 12,
            marginLeft: 30,
            textAlign: "left",         // center the H2 (change to "left" if you want it left)
            width: "100%",
        } as React.CSSProperties,

        // outer wrapper for the grids
        gridOuter: {
            display: "flex",
            flexDirection: "column",
            gap: 18,
            alignItems: "center",        // center inner blocks
            justifyContent: "center",
            width: "100%",
            maxWidth: 1100,              // matches container max width for consistent centering
        } as React.CSSProperties,

        topGridWrapper: {
            overflowX: "auto" as React.CSSProperties["overflowX"],
            width: "100%",
        },

        grid: (cols: number) => ({
            display: "grid",
            gridTemplateColumns: `repeat(${cols}, 28px)`,
            gap: 0,
        } as React.CSSProperties),

        smallCheckbox: {
            width: 26,
            height: 26,
            margin: 0,
            padding: 0,
            border: "1px solid #bbb",
            boxSizing: "border-box",
        } as React.CSSProperties,

        bottomGridWrapper: { marginTop: 12, width: "100%" } as React.CSSProperties,

        inputsWrapper: {
            marginLeft: 0,
            minWidth: 220,
            display: "flex",
            flexDirection: "column",

        } as React.CSSProperties,

        inputRow: {
            display: "flex",
            alignItems: "center",
            gap: 8,
            marginBottom: 8,
        } as React.CSSProperties,

        labelCol: {
            width: 100,
            textAlign: "right",
            paddingRight: 8,
            whiteSpace: "nowrap",
        } as React.CSSProperties,

        controls: {
            marginTop: 14,
            display: "flex",
            flexWrap: "wrap",
            justifyContent: 'flex-end',
            alignItems: 'center',
        } as React.CSSProperties,

        textarea: {
            width: "100%",
            height: 240,
            marginTop: 12,
            fontFamily: "monospace",
            fontSize: 12,
            padding: 8,
            borderRadius: 6,
            border: "1px solid",
        } as React.CSSProperties,

        inputGridWrap: {
            display: "flex",
            gap: 100,
            marginTop: 12,
            justifyContent: "center",
            width: "100%",
            maxWidth: 1100,
        } as React.CSSProperties,

        inputColumn: {
            minWidth: 320,
            display: "flex",
            flexDirection: "column",
            gap: 0,
            background: "#0f1720",
            paddingLeft: 20,
            paddingTop: 10,
            paddingRight: 10,
            paddingBottom: 20,
            borderRadius: 8,
            alignItems: "stretch", // use "center" if you want each item centered inside column
            width: "100%",
            boxSizing: "border-box",
        } as React.CSSProperties,


        labeledInputRow: {
            display: "flex",
            gap: 8,
            alignItems: "center",
            marginBottom: 8,
            marginTop: 20,
        } as React.CSSProperties,

        percentWrapper: { position: "relative", display: "flex", alignItems: "center" } as React.CSSProperties,

        percentSuffix: { position: "absolute", right: 10, pointerEvents: "none", color: "#aaa" } as React.CSSProperties,

        inputCell: {
            flex: 1,
            padding: "6px 8px",
            border: "1px solid #2b3440",
            background: "transparent",
            color: "#fff",
            borderRadius: 0,
        } as React.CSSProperties,

        inputLabelCell: {
            width: 100,
            textAlign: "right",
            paddingRight: 8,
            color: "#ddd",
            whiteSpace: "nowrap",
            overflow: "visible",
            textOverflow: "ellipsis",
        } as React.CSSProperties,
        primaryButton: {
            background: "#06b6d4",
            color: "#021018",
            padding: "8px 12px",
            borderRadius: 6,
            border: "none",
            cursor: "pointer",
        } as React.CSSProperties,

        // success / green button used for 'Find chance of success'
        successButton: {
            background: '#16a34a',
            color: '#021018',
            padding: '8px 12px',
            borderRadius: 6,
            border: 'none',
            cursor: 'pointer',
        } as React.CSSProperties,

        // box around checkbox grids
        checkboxBox: {
            background: '#071017',
            padding: 12,
            borderRadius: 8,
            width: '100%',
            boxSizing: 'border-box',
        } as React.CSSProperties,
    }

    // ----- NEW: ag-grid-react wiring (memoized) -----
    // Only the input column is managed by the grid; labels are rendered as plain text on the left
    const columnDefs = useMemo(() => [
        { headerName: 'Value', field: 'value', editable: true, flex: 1, cellStyle: { background: '#072f24', color: '#9ff7c7', padding: '6px 8px' } }
    ], [])

    const rows = useMemo(() => INPUT_LABELS.map((label, index) => ({ id: String(index), value: budget_inputs[label] ?? '' })), [budget_inputs])

    // compute grid widths (pixels) to size the checkbox boxes correctly
    const CELL_W = 28
    const topGridWidth = TOP_COLS * CELL_W
    const bottomGridWidth = BOTTOM_COLS * CELL_W



    // --------------------------------------------------
    return (
        <div style={{ padding: 15, }}>
            <h2 style={styles.heading}>Honing Forecast</h2>
            <div style={styles.container}>
                <div style={styles.gridOuter}>
                    {/* marquee overlay */}
                    {marqueeRect ? (
                        <div
                            style={{
                                position: "fixed",
                                left: marqueeRect.left,
                                top: marqueeRect.top,
                                width: marqueeRect.width,
                                height: marqueeRect.height,
                                background: "rgba(16, 185, 129, 0.12)",
                                border: "2px solid rgba(16,185,129,0.6)",
                                pointerEvents: "none",
                                zIndex: 9999,
                            }}
                        />
                    ) : null}
                    {/* Grids area */}
                    <div style={{ display: "flex", gap: 24 }}>
                        <div style={{ flex: 1 }}>
                            {/* Top checkbox grid with +1..+25 header */}
                            <div style={styles.topGridWrapper}>
                                <div style={{ ...styles.checkboxBox, width: Math.min(topGridWidth + 140, 1100) }}>
                                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 8 }}>
                                        <div style={{ fontWeight: 600, color: '#ddd' }}>Normal honing</div>
                                        <div style={{ display: 'grid', gridTemplateColumns: `repeat(${TOP_COLS}, ${CELL_W}px)`, gap: 0 }}>
                                            {Array.from({ length: TOP_COLS }).map((_, i) => (
                                                <div key={i} style={{ width: CELL_W, textAlign: 'center', color: '#9fe7ff', fontSize: 12 }}>+{i + 1}</div>
                                            ))}
                                        </div>
                                    </div>
                                    <div style={{ display: 'flex', gap: 8 }}>
                                        <div style={{ width: 120, display: 'flex', flexDirection: 'column', justifyContent: 'center' }}>
                                            {['Helmet', 'Shoulder', 'Chest', 'Pants', 'Glove', 'Weapon'].map((lab) => (
                                                <div key={lab} style={{ height: 28, color: '#ddd', display: 'flex', alignItems: 'center', justifyContent: 'flex-end', paddingRight: 8 }}>{lab}</div>
                                            ))}
                                        </div>
                                        <div style={{ width: topGridWidth }}>
                                            <div ref={topGridRef} onMouseDown={(e) => onGridMouseDown("top", e)} style={styles.grid(TOP_COLS)}>
                                                {topGrid.flatMap((row, r) =>
                                                    row.map((checked, c) => {
                                                        const key = `t-${r}-${c}`
                                                        let inMarquee = false
                                                        if (marquee.active && marquee.grid === "top") {
                                                            const r1 = Math.min(marquee.startR, marquee.endR)
                                                            const r2 = Math.max(marquee.startR, marquee.endR)
                                                            const c1 = Math.min(marquee.startC, marquee.endC)
                                                            const c2 = Math.max(marquee.startC, marquee.endC)
                                                            inMarquee = r >= r1 && r <= r2 && c >= c1 && c <= c2
                                                        }
                                                        return (
                                                            <div key={key} title={`r${r} c${c}`} style={{ width: CELL_W, height: 28, display: "inline-flex", alignItems: 'center', justifyContent: 'center', margin: 0 }}>
                                                                <input type="checkbox" checked={inMarquee ? !marquee.initialState : checked} readOnly style={styles.smallCheckbox} />
                                                            </div>
                                                        )
                                                    })
                                                )}
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                            <div style={{ height: 30 }} /> {/* small vertical separation between normal and advanced */}
                            {/* Bottom checkbox grid with +10/+20/+30/+40 header */}
                            <div style={{ display: 'flex', gap: 12 }}>
                                <div style={{ ...styles.checkboxBox, width: Math.min(bottomGridWidth + 140, 1100) }}>
                                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 8 }}>
                                        <div style={{ fontWeight: 600, color: '#ddd' }}>Advanced honing</div>
                                        <div style={{ display: 'grid', gridTemplateColumns: `repeat(${BOTTOM_COLS}, ${CELL_W}px)`, gap: 0 }}>
                                            {[10, 20, 30, 40].map((n, i) => (
                                                <div key={i} style={{ width: CELL_W, textAlign: 'center', color: '#9fe7ff', fontSize: 12 }}>+{n}</div>
                                            ))}
                                        </div>
                                    </div>
                                    <div style={{ display: 'flex', gap: 8 }}>
                                        <div style={{ width: 120, display: 'flex', flexDirection: 'column', justifyContent: 'center' }}>
                                            {['Helmet', 'Shoulder', 'Chest', 'Pants', 'Glove', 'Weapon'].map((lab) => (
                                                <div key={lab} style={{ height: 28, color: '#ddd', display: 'flex', alignItems: 'center', justifyContent: 'flex-end', paddingRight: 8 }}>{lab}</div>
                                            ))}
                                        </div>
                                        <div style={{ width: bottomGridWidth }}>
                                            <div ref={bottomGridRef} onMouseDown={(e) => onGridMouseDown("bottom", e)} style={styles.grid(BOTTOM_COLS)}>
                                                {bottomGrid.flatMap((row, r) =>
                                                    row.map((checked, c) => {
                                                        const key = `b-${r}-${c}`
                                                        let inMarquee = false
                                                        if (marquee.active && marquee.grid === "bottom") {
                                                            const r1 = Math.min(marquee.startR, marquee.endR)
                                                            const r2 = Math.max(marquee.startR, marquee.endR)
                                                            const c1 = Math.min(marquee.startC, marquee.endC)
                                                            const c2 = Math.max(marquee.startC, marquee.endC)
                                                            inMarquee = r >= r1 && r <= r2 && c >= c1 && c <= c2
                                                        }
                                                        return (
                                                            <div key={key} style={{ width: CELL_W, height: 28, display: "inline-flex", alignItems: 'center', justifyContent: 'center', margin: 0 }}>
                                                                <input type="checkbox" checked={inMarquee ? !marquee.initialState : checked} readOnly style={styles.smallCheckbox} />
                                                            </div>
                                                        )
                                                    })
                                                )}
                                            </div>
                                        </div>

                                    </div>
                                    <select
                                        id="my-dropdown"
                                        value={adv_hone_strategy}
                                        onChange={(e) => adv_hone_strategy_change(e.target.value)}
                                        className="p-2 border rounded"
                                        style={{ gridColumn: '1 / span 2', justifyContent: "flex-end" }}
                                    >
                                        <option value="No juice">No juice</option>
                                        <option value="Juice on grace">Juice on grace</option>
                                    </select>
                                </div>

                                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: 8, alignItems: 'start', width: 420 }}>

                                    <button onClick={fillMy}>Fill demo</button>
                                    <button onClick={fillRandom}>Fill random demo</button>
                                    <button onClick={clearAll}>Clear all</button>
                                    <button onClick={() => clearObjectStore()}>Clear Cache</button>
                                </div>
                            </div>
                        </div>
                        {/* small vertical separation between normal and advanced */}
                        <div style={{ height: 12 }} />
                    </div>

                    {/* Inputs area below grids - two columns side by side */}
                    <div style={styles.inputGridWrap}>
                        {/* Left column: ChanceToCost (desired chance + button + output read-only column) */}
                        <div style={styles.inputColumn}>
                            <div style={{ fontWeight: 600, color: "#fff" }}>Chance to Cost</div>
                            <div style={styles.labeledInputRow}>
                                <div style={{ ...styles.labelCol, whiteSpace: 'nowrap' }}>{"Desired chance"}</div>
                                <div style={styles.percentWrapper}>
                                    <input
                                        type="text"
                                        value={desired_chance}
                                        onChange={(e) => {
                                            const v = e.target.value
                                            if (/^\d*\.?\d*$/.test(v) || v === "") onDesiredChange(v)
                                        }}
                                        placeholder="0"
                                        style={{ width: 160, fontSize: 15, padding: "6px 8px", borderRadius: 6, background: "#acfffeff", color: "#000", alignSelf: "flex-end" }}
                                    />
                                    <span style={styles.percentSuffix}>%</span>
                                </div>
                            </div>
                            <div style={styles.controls}>
                                <button style={styles.primaryButton} onClick={() => HandleCallWorker("ChanceToCost")} disabled={ChanceToCostBusy}>
                                    {ChanceToCostBusy ? "Running…" : "Find estimated cost for " + desired_chance + "%"}
                                </button>
                            </div>

                            <div style={{ marginTop: 8 }}>
                                <div style={{ fontWeight: 600, color: "#fff", marginBottom: 6 }}>Estimated cost </div>
                                {INPUT_LABELS.map((label, idx) => {
                                    let outLabel = label
                                    let value = cost_result && cost_result[label] != null ? String(cost_result[label]) : "No results yet"
                                    if (idx === INPUT_LABELS.length - 1) {
                                        return null
                                    }
                                    //     outLabel = "Actual prob"
                                    //     value = cost_result && cost_result["Est. Probability"] != null ? String(cost_result["Est. Probability"]) : "-"
                                    // }

                                    return (
                                        <div key={label} style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 0 }}>
                                            <div style={{ ...styles.inputLabelCell, whiteSpace: 'nowrap' }}>{outLabel}</div>
                                            <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                                                <div style={{ ...styles.inputCell, border: 'none', background: 'transparent', color: '#9fe7ff', padding: '6px 8px' }}>{value}</div>
                                                {/* {idx === INPUT_LABELS.length - 1 ? <span style={{ color: '#9fe7ff' }}>%</span> : null} */}
                                            </div>

                                        </div>

                                    )
                                })}
                                <pre>{cost_result ? "\nRun time: " +
                                    cost_result.run_time +
                                    "s"
                                    : ""}</pre>
                            </div>
                        </div>

                        {/* Right column: CostToChance inputs (labels left, grid right) */}
                        <div style={styles.inputColumn}>
                            <div style={{ fontWeight: 600, whiteSpace: 'nowrap' }}>Cost to Chance</div>
                            <div style={{ whiteSpace: 'nowrap', marginTop: 20 }}>Input your budget here</div>
                            <div style={{ width: '100%', marginTop: 8 }}>
                                <div style={{ display: 'flex', padding: 6 }}>
                                    <div style={{ ...styles.inputLabelCell, whiteSpace: 'nowrap' }}>
                                        {INPUT_LABELS.map((lab) => (
                                            <div key={lab} style={{ height: 36, color: '#ddd', display: 'flex', alignItems: 'center', justifyContent: 'flex-end', paddingRight: 8, whiteSpace: 'nowrap' }}>{lab}</div>
                                        ))}
                                    </div>
                                    <div style={{ flex: 1 }}>
                                        <div className="ag-theme-alpine" style={{ width: '100%', position: 'relative', border: '1px solid rgba(255,255,255,0.06)' }}>
                                            <div style={{ position: 'absolute', left: 8, top: 8, zIndex: 5, color: '#9fe7ff', fontSize: 12 }} id="ag-diagnostic-msg">Initializing grid…</div>
                                            <AgGridReact
                                                theme={themeBalham.withParams({
                                                    spacing: 12,
                                                    accentColor: 'red',
                                                })}
                                                rowData={rows}
                                                columnDefs={columnDefs as any}
                                                getRowId={(params: any) => params.data.id}
                                                defaultColDef={{ editable: true, resizable: true }}
                                                headerHeight={0}
                                                singleClickEdit={true}
                                                domLayout={'autoHeight'}
                                                stopEditingWhenCellsLoseFocus={true}
                                                onGridReady={() => {
                                                    const el = document.getElementById('ag-diagnostic-msg')
                                                    if (el) el.style.display = 'none'
                                                }}
                                                onCellValueChanged={(event: any) => {
                                                    // Map grid rows back to INPUT_LABELS using row id
                                                    const api = event.api
                                                    const all: any[] = []
                                                    api.forEachNode((n: any) => all.push(n.data))
                                                    const next: Record<string, string> = { ...budget_inputs }
                                                    all.forEach((r) => {
                                                        const idx = Number(r.id)
                                                        if (!Number.isNaN(idx) && INPUT_LABELS[idx]) {
                                                            next[INPUT_LABELS[idx]] = String(r.value ?? "")
                                                        }
                                                    })
                                                    set_budget_inputs(next)
                                                }}
                                                rowHeight={36}
                                            />
                                        </div>
                                    </div>
                                </div>
                            </div>

                            <div style={styles.controls}>
                                <button style={styles.successButton} onClick={() => HandleCallWorker('CostToChance')} disabled={CostToChanceBusy}>
                                    {CostToChanceBusy ? 'Running…' : 'Find chance of success'}
                                </button>
                            </div>

                            <div style={{ marginTop: 8 }}>
                                <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
                                    <div style={{ ...styles.inputLabelCell, whiteSpace: 'nowrap' }}>Chance of Success</div>
                                    <div style={{ ...styles.inputCell, border: 'none', marginLeft: 40, background: 'transparent', color: '#9ff7c7', fontSize: 20 }}>{chance_result ? (String(chance_result.chance) + '%') : '-'}</div>
                                </div>
                                <div style={{ marginTop: 8, color: '#9fe7ff' }}>{chance_result ? (chance_result.reason || '') : ''}</div>
                                <div style={{ marginTop: 6, color: '#9fe7ff' }}>{chance_result ? ('Run time: ' + chance_result.run_time + 's') : ''}</div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            <div style={{ marginTop: 300 }}></div>
        </div>
    )
}
