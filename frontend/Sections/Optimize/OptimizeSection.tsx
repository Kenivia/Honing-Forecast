import SpreadsheetGrid from "@/Components/SpreadsheetGrid.tsx"
import { CELL_H, CELL_W, JUICE_LABELS, MATS_LABELS } from "@/Utils/Constants.ts"
import { createColumnDefs, styles } from "@/Utils/Styles.ts"
import type { InputsBundleWithSetters } from "@/Utils/InputBundles.ts"
import React from "react"
import StateGrid, { StatePair } from "@/Sections/Optimize/StateGrid.tsx"
import StateGridsManager from "@/Sections/Optimize/StateGrid.tsx"
import { SpecialSortable } from "./SpecialSortable.tsx"
import "./OptimizerSection.css"
import InputsSection from "../Inputs/InputsSection.tsx"

type OptimizeSectionProps = {
    curIsBest: boolean
    inputsBundle: InputsBundleWithSetters

    optimizeAvgBusy: boolean
    optimizeAvgWorkerRef: React.MutableRefObject<Worker | null>
    setOptimizeAvgBusy: React.Dispatch<React.SetStateAction<boolean>>
    onCancelOptimizeAverage: () => void
    autoRunOptimizer: boolean
    setAutoRunOptimizer: React.Dispatch<React.SetStateAction<boolean>>
    optimizeAvgError: string | null
    // optimizeAvgResult: any
    setOptimizeButtonPress: React.Dispatch<React.SetStateAction<any>>
    // onGridMouseDown: (_grid: "top" | "bottom", _e: React.MouseEvent) => void

    flatProgressArr: number[]
    setFlatProgressArr: React.Dispatch<React.SetStateAction<any>>

    flatUnlockArr: boolean[]
    setFlatUnlockArr: React.Dispatch<React.SetStateAction<any>>

    flatSucceedArr: boolean[]
    setFlatSucceedArr: React.Dispatch<React.SetStateAction<any>>

    flatStateBundle: StatePair[][]
    setFlatStateBundle: React.Dispatch<React.SetStateAction<any>>
    allowUserChangeState: boolean

    evaluateAverageResult: any
    specialState: number[]
    setSpecialState: React.Dispatch<React.SetStateAction<number[]>>
    // gridRefs: React.RefObject<HTMLDivElement>[]
    // marquee: any
    optimizerProgress: number
    metricType: number
    bestMetric: number | null
    bestFlatStateBundle: StatePair[][] | null
    bestFlatSpecialState: number[] | null
    setMetricType: React.Dispatch<React.SetStateAction<number>>
    ranOutFreeTaps: boolean
    onRanOutFreeTaps: () => void

    beforeMetric: number
    setBeforeMetric: React.Dispatch<React.SetStateAction<number>>
    hasRunOptimizer: boolean
}

function _my_alr_spent_map(already_spent: any, labels: string[], index: number) {
    return already_spent
        ? Object.fromEntries(labels.map((label, lab_index) => [label, String(already_spent[index][lab_index])]))
        : Object.fromEntries(labels.map((label) => [label, "Calculating..."]))
}

function add_comma(inp: number) {
    return Math.round(inp).toLocaleString("en-US", {
        minimumFractionDigits: 0,
        maximumFractionDigits: 0,
    })
}
function breakdown_to_english(input: number) {
    return String(input <= 0 ? "Avg spend " + add_comma(input < 0.0 ? -input : input) + "g" : "Avg surplus of " + add_comma(input) + "g")
}

function _combined_breakdown_to_english(input: number) {
    return String(input <= 0 ? add_comma(input < 0.0 ? -input : input) + "g" : "Avg surplus of " + add_comma(input) + "g")
}
function avg_breakdown_map(avg_breakdown: any, labels: string[], offset: number) {
    return avg_breakdown
        ? Object.fromEntries(
              labels.map((label, lab_index) => {
                  const value = avg_breakdown[offset + lab_index]
                  return [label, value === undefined ? "N/A" : breakdown_to_english(value)]
              }),
          )
        : Object.fromEntries(labels.map((label) => [label, "N/A"]))
}

function avg_breakdown_colors(avg_breakdown: any, labels: string[], offset: number) {
    return labels.map((_label, lab_index) => {
        const value = avg_breakdown?.[offset + lab_index]
        if (value === undefined) {
            return "var(--text-muted)"
        }
        return value > -0.5 ? "var(--text-success)" : "var(--deficit)"
    })
}

export default function OptimizeSection({
    curIsBest,
    inputsBundle,
    optimizeAvgBusy,
    // optimizeAvgResult,
    optimizeAvgWorkerRef,
    setOptimizeAvgBusy,
    onCancelOptimizeAverage,
    autoRunOptimizer,
    setAutoRunOptimizer,
    optimizeAvgError,
    setOptimizeButtonPress,
    flatProgressArr,
    setFlatProgressArr,
    flatUnlockArr,
    setFlatUnlockArr,
    flatSucceedArr,
    setFlatSucceedArr,
    flatStateBundle,
    setFlatStateBundle,
    allowUserChangeState,
    evaluateAverageResult,
    specialState,
    setSpecialState,
    optimizerProgress,
    metricType,
    bestMetric,
    bestFlatStateBundle,
    bestFlatSpecialState: bestFlatSpecialGrid,
    setMetricType: _setMetricType,
    ranOutFreeTaps,
    onRanOutFreeTaps,
    beforeMetric,
    setBeforeMetric,
    hasRunOptimizer,
    // gridRefs,
    // onGridMouseDown,
    // marquee,
}: OptimizeSectionProps) {
    const [isFreeTapCollapsed, setIsFreeTapCollapsed] = React.useState(true)
    const { wideMatsColumnDefs } = createColumnDefs()
    // console.log(evaluateAverageResult)
    const _already_spent = evaluateAverageResult?.prep_output.already_spent
    const avg_breakdown = evaluateAverageResult?.average_breakdown
    const cloneFlatStateBundle = (bundle: StatePair[][]) => bundle.map((row) => row.map((pair) => [pair[0], pair[1]] as StatePair))
    const juiceAvail = evaluateAverageResult?.prep_output.juice_info.num_avail ?? 0
    const canRestoreBest =
        !optimizeAvgBusy && bestMetric !== null && Boolean(bestFlatStateBundle) && Boolean(bestFlatSpecialGrid) && bestMetric > evaluateAverageResult?.metric

    // console.log(bestMetric, evaluateAverageResult?.metric)
    const handleRestoreBest = () => {
        if (!canRestoreBest || !bestFlatStateBundle || !bestFlatSpecialGrid) {
            return
        }
        setFlatStateBundle(cloneFlatStateBundle(bestFlatStateBundle))
        setSpecialState([...bestFlatSpecialGrid])
    }

    const handleOptimizeAverageClick = () => {
        if (optimizeAvgBusy) {
            optimizeAvgWorkerRef.current?.terminate()
            optimizeAvgWorkerRef.current = null
            setOptimizeAvgBusy(false)
            setAutoRunOptimizer(false)
            // setBeforeMetric(null)
            onCancelOptimizeAverage()
            return
        } else {
            setBeforeMetric(evaluateAverageResult?.metric ?? null)
            setOptimizeButtonPress((prev: number) => prev + 1)
        }
    }

    const optimizeAverageButton = (
        <button
            onClick={handleOptimizeAverageClick}
            style={{
                background: optimizeAvgBusy ? "var(--cancel-optimizer-button)" : "var(--optimizer-button)",
                color: optimizeAvgBusy ? "var(--text-muted)" : "var(--text)",
                padding: "6px 10px",
                borderRadius: 4,
                border: "1px solid var(--btn-border)",
                cursor: "pointer",
            }}
        >
            {optimizeAvgBusy ? "Cancel Optimize Average" : " >>> Optimize <<< "}
        </button>
    )

    // console.log("special", specialState)
    // console.log(flatSucceedArr)

    return (
        <div style={{ ...styles.inputSection, flexDirection: "column", maxWidth: "1200px", width: "100%", gap: 12 }}>
            <section className="optimizer-section">
                <div className="optimizer-section-title">Inputs</div>
                <div className="optimizer-section-body">
                    <InputsSection inputsBundle={inputsBundle} />
                </div>
            </section>
            <section className="optimizer-section">
                <div className="optimizer-section-title">Optimizer controls</div>
                <div className="optimizer-section-body">
                    <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                        <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
                            {metricType != 0 && optimizeAverageButton}
                            {optimizeAvgError && <span style={{ fontSize: 12, color: "red" }}>Error: {optimizeAvgError}</span>}
                            {beforeMetric !== null && (
                                <span style={{ fontSize: 12, color: "var(--text-success)" }}>{add_comma(evaluateAverageResult?.metric) ?? "N/A"} gold</span>
                            )}{" "}
                            <button
                                onClick={handleRestoreBest}
                                disabled={!canRestoreBest}
                                style={{
                                    background: canRestoreBest ? "var(--optimizer-button)" : "var(--btn-demo)",
                                    color: "var(--btn-demo-text)",
                                    padding: "6px 10px",
                                    borderRadius: 4,
                                    border: "1px solid var(--btn-border)",
                                    cursor: canRestoreBest ? "pointer" : "not-allowed",
                                    opacity: canRestoreBest ? 1 : 0.3,
                                }}
                            >
                                {canRestoreBest ? "Restore Best" : "Current configuration is the best known configuration"}
                            </button>
                        </div>
                        <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
                            <label style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 13 }}>
                                <input type="checkbox" checked={autoRunOptimizer} onChange={(e) => setAutoRunOptimizer(e.target.checked)} />
                                Auto start optimizer whenever something changes
                            </label>
                        </div>
                        <span style={{ fontSize: "0.9em", color: "var(--text-secondary, #666)" }}>
                            Optimizer: {hasRunOptimizer ? "✓ Ran" : "Not run"} | Best Metric:{" "}
                            {hasRunOptimizer && bestMetric !== null ? bestMetric.toFixed(2) : "Unknown"}
                        </span>
                        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                            {optimizeAvgBusy && (
                                <span>Optimizer progress: {optimizerProgress.toFixed(2)}% (this can take a while if you have a lot of upgrades)</span>
                            )}
                        </div>
                    </div>
                </div>
            </section>
            <div style={{ position: "relative", flex: 1, display: "flex", flexDirection: "column", gap: 12 }}>
                {optimizeAvgBusy && (
                    <div
                        style={{
                            position: "absolute",
                            inset: 0,
                            background: "rgba(0, 0, 0, 0.25)",
                            zIndex: 2,
                        }}
                    />
                )}
                <section className="optimizer-section">
                    <div className="optimizer-section-title">Results</div>
                    <div className="optimizer-section-body">
                        {flatStateBundle && flatProgressArr && evaluateAverageResult && specialState && (
                            <StateGridsManager
                                curIsBest={curIsBest}
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
                            />
                        )}
                    </div>
                </section>{" "}
                <section className="optimizer-section">
                    <button
                        type="button"
                        className="optimizer-section-title optimizer-section-title-button"
                        aria-expanded={!isFreeTapCollapsed}
                        aria-controls="free-tap-order-section"
                        onClick={() => setIsFreeTapCollapsed((prev) => !prev)}
                    >
                        <span>More free tap info </span>
                        <span className={`optimizer-section-title-arrow ${isFreeTapCollapsed ? "collapsed" : ""}`}>{">"}</span>
                    </button>
                    {!isFreeTapCollapsed && (
                        <div id="free-tap-order-section" className="optimizer-section-body">
                            Keep attempting free taps on an upgrade until you run out, then move on to the next. <br></br>
                            This is the best order that the algorithm found, you can drag & drop the upgrades to see if you can find a better one.
                            {flatStateBundle && flatProgressArr && evaluateAverageResult && specialState && (
                                <SpecialSortable
                                    curIsBest={curIsBest}
                                    evaluateAverageResult={evaluateAverageResult}
                                    specialState={specialState}
                                    setSpecialState={setSpecialState}
                                    flatSucceedArr={flatSucceedArr}
                                    setFlatSucceedArr={setFlatSucceedArr}
                                    flatUnlockArr={flatUnlockArr}
                                    setFlatUnlockArr={setFlatUnlockArr}
                                    ranOutFreeTaps={ranOutFreeTaps}
                                    onRanOutFreeTaps={onRanOutFreeTaps}
                                />
                            )}
                        </div>
                    )}
                </section>
            </div>
            <section className="optimizer-section">
                <div className="optimizer-section-title">Optimizer breakdown</div>
                <div className="optimizer-section-body">
                    <div
                        style={{
                            display: "flex",
                            flexDirection: "row",
                            alignItems: "flex-start",
                            gap: 100,
                            minWidth: 200,
                            flexShrink: 0,
                            marginTop: 0,
                            flexWrap: "wrap",
                            marginLeft: 100,
                        }}
                    >
                        <SpreadsheetGrid
                            columnDefs={wideMatsColumnDefs}
                            labels={MATS_LABELS.slice(0, 7)}
                            sheetValuesArr={[avg_breakdown_map(avg_breakdown, MATS_LABELS.slice(0, 7), 0)]}
                            colorsArr={[avg_breakdown_colors(avg_breakdown, MATS_LABELS.slice(0, 7), 0)]}
                            setSheetValuesArr={[null]}
                        />
                        <SpreadsheetGrid
                            columnDefs={wideMatsColumnDefs}
                            labels={JUICE_LABELS.map((label_row) => label_row[0])}
                            sheetValuesArr={[
                                avg_breakdown_map(
                                    avg_breakdown,
                                    JUICE_LABELS.map((label_row) => label_row[0]),
                                    7,
                                ),
                            ]}
                            colorsArr={[
                                avg_breakdown_colors(
                                    avg_breakdown,
                                    JUICE_LABELS.map((label_row) => label_row[0]),
                                    7,
                                ),
                            ]}
                            setSheetValuesArr={[null]}
                        />
                        <SpreadsheetGrid
                            columnDefs={wideMatsColumnDefs}
                            labels={JUICE_LABELS.map((label_row) => label_row[1])}
                            sheetValuesArr={[
                                avg_breakdown_map(
                                    avg_breakdown,
                                    JUICE_LABELS.map((label_row) => label_row[1]),
                                    7 + juiceAvail,
                                ),
                            ]}
                            colorsArr={[
                                avg_breakdown_colors(
                                    avg_breakdown,
                                    JUICE_LABELS.map((label_row) => label_row[1]),
                                    7 + juiceAvail,
                                ),
                            ]}
                            setSheetValuesArr={[null]}
                        />
                    </div>
                    {/* <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
                        <div>Already spent: {evaluateAverageResult?.prep_output.already_spent[3]}</div>
                        <div>Average cost from now on: {evaluateAverageResult?.metric - evaluateAverageResult?.prep_output.already_spent[3]}</div>
                        <div>Already spent + more to come: {evaluateAverageResult?.metric}</div>
                    </div> */}
                    {/* <div style={{ display: "grid", gridTemplateColumns: "1fr auto 1fr", alignItems: "center", gap: 12 }}>
                        <div style={{ display: "flex", alignItems: "center", justifyContent: "flex-end", gap: 12 }}> */}
                    <h3>What do the numbers mean?</h3>
                    <span></span>
                    <span style={{ fontSize: 24, color: "var(--optimizer-button)" }}>Combined : {breakdown_to_english(evaluateAverageResult?.metric)}</span>
                    {/* </div>
                    </div> */}
                </div>
            </section>
        </div>
    )
}
