import SpreadsheetGrid from "@/Components/SpreadsheetGrid.tsx"
import { CELL_H, CELL_W, JUICE_LABELS, MATS_LABELS } from "@/Utils/Constants.ts"
import { createColumnDefs, styles } from "@/Utils/Styles.ts"
import type { InputsBundleWithSetters } from "@/Utils/InputBundles.ts"
import React from "react"
import StateGrid, { StatePair } from "@/Sections/Optimize/StateGrid.tsx"
import StateGridsManager from "@/Sections/Optimize/StateGrid.tsx"
import { SpecialSortable } from "./SpecialSortable.tsx"

type OptimizeSectionProps = {
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
}

function my_alr_spent_map(already_spent: any, labels: string[], index: number) {
    return already_spent
        ? Object.fromEntries(labels.map((label, lab_index) => [label, String(already_spent[index][lab_index])]))
        : Object.fromEntries(labels.map((label) => [label, "Calculating..."]))
}


function add_comma(inp: number) {
    return Math.round(inp).toLocaleString(
        "en-US",
        {
            minimumFractionDigits: 0,
            maximumFractionDigits: 0,
        },
    )
}
function breakdown_to_english(input: number) {
    return String(
        input <= 0 ? "Avg spend " + add_comma(input < 0.0 ? -input : input) + "g" : "Avg surplus of " + add_comma(input) + "g"
    )
}
function avg_breakdown_map(avg_breakdown: any, labels: string[], offset: number) {
    return avg_breakdown
        ? Object.fromEntries(labels.map((label, lab_index) => [label, breakdown_to_english(avg_breakdown[offset + lab_index])]))
        : Object.fromEntries(labels.map((label) => [label, "Calculating..."]))
}
export default function OptimizeSection({
    inputsBundle: _inputsBundle,
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
    setMetricType,
    ranOutFreeTaps,
    onRanOutFreeTaps,

    // gridRefs,
    // onGridMouseDown,
    // marquee,
}: OptimizeSectionProps) {
    const { wideMatsColumnDefs } = createColumnDefs()
    const already_spent = evaluateAverageResult?.prep_output.already_spent
    const avg_breakdown = evaluateAverageResult?.average_breakdown
    const cloneFlatStateBundle = (bundle: StatePair[][]) => bundle.map((row) => row.map((pair) => [pair[0], pair[1]] as StatePair))
    const canRestoreBest = !optimizeAvgBusy && bestMetric !== null && Boolean(bestFlatStateBundle) && Boolean(bestFlatSpecialGrid) && bestMetric > evaluateAverageResult?.metric

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
            onCancelOptimizeAverage()
            return
        } else {
            setOptimizeButtonPress((prev: number) => prev + 1)
        }
    }

    // console.log("special", specialState)
    // console.log(flatSucceedArr)

    return (
        <div style={{ ...styles.inputSection, flexDirection: "row", maxWidth: "1200px", width: "100%" }}>
            <div>
                <label style={{ display: "inline-flex", alignItems: "center", cursor: "pointer" }}>
                    <input type="checkbox" checked={metricType == 0} onChange={(e) => setMetricType(e.target.checked ? 0 : 1)} style={{ display: "none" }} />
                    <span
                        style={{
                            width: 40,
                            height: 20,
                            background: metricType == 0 ? "var(--prob-mode)" : "var(--average-mode)",
                            borderRadius: 999,
                            position: "relative",
                            transition: "background 0.2s",
                        }}
                    >
                        <span
                            style={{
                                position: "absolute",
                                top: 2,
                                left: metricType == 0 ? 22 : 2,
                                width: 16,
                                height: 16,
                                background: "white",
                                borderRadius: "50%",
                                transition: "left 0.2s",
                            }}
                        />
                    </span>
                </label>
                <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 8, marginBottom: 8 }}>
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
                        {optimizeAvgBusy ? "Cancel Optimize Average" : "Optimize Average"}
                    </button>
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
                        Restore Best
                    </button>
                    <label style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 13 }}>
                        <input type="checkbox" checked={autoRunOptimizer} onChange={(e) => setAutoRunOptimizer(e.target.checked)} />
                        Auto start optimizer
                    </label>

                    {optimizeAvgError && <span style={{ fontSize: 12, color: "red" }}>Error: {optimizeAvgError}</span>}
                </div>
                {optimizeAvgBusy && <span>Optimizer progress: {optimizerProgress.toFixed(2)}%</span>}
                <br />
                Already spent: {evaluateAverageResult?.prep_output.already_spent[3]}
                <br />
                Average cost from now on: {evaluateAverageResult?.metric - evaluateAverageResult?.prep_output.already_spent[3]}
                <br />
                Already spent + more to come: {evaluateAverageResult?.metric}
            </div>
            {avg_breakdown && (
                <div style={{ display: "flex", flexDirection: "row", alignItems: "flex-start", gap: 100, minWidth: 200, flexShrink: 0, marginTop: 0 }}>

                    <SpreadsheetGrid
                        columnDefs={wideMatsColumnDefs}
                        labels={MATS_LABELS.slice(0, 7)}
                        sheetValuesArr={[avg_breakdown_map(avg_breakdown, MATS_LABELS.slice(0, 7), 0)]}
                        colorsArr={[avg_breakdown.slice(0, 7).map((x) => x > -0.5 ? "var(--text-success)" : "var(--deficit)")]}
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
                        colorsArr={[avg_breakdown.slice(7, 7 + evaluateAverageResult?.prep_output.juice_info.num_avail,).map((x) => x > -0.5 ? "var(--text-success)" : "var(--deficit)")]}
                        setSheetValuesArr={[null]}
                    />

                    <SpreadsheetGrid
                        columnDefs={wideMatsColumnDefs}
                        labels={JUICE_LABELS.map((label_row) => label_row[1])}
                        sheetValuesArr={[
                            avg_breakdown_map(
                                avg_breakdown,
                                JUICE_LABELS.map((label_row) => label_row[1]),
                                7 + evaluateAverageResult?.prep_output.juice_info.num_avail,
                            ),
                        ]}
                        colorsArr={[avg_breakdown.slice(7 + evaluateAverageResult?.prep_output.juice_info.num_avail, 7 + 2 * 7 + evaluateAverageResult?.prep_output.juice_info.num_avail).map((x) => x > -0.5 ? "var(--text-success)" : "var(--deficit)")]}
                        setSheetValuesArr={[null]}
                    />
                </div>)
            }
            <div style={{ position: "relative", flex: 1 }}>
                {optimizeAvgBusy && (
                    <div
                        style={{
                            position: "absolute",
                            inset: 0,
                            background: "rgba(0, 0, 0, 0.35)",
                            zIndex: 2,
                        }}
                    />
                )}
                {flatStateBundle && flatProgressArr && evaluateAverageResult && specialState && (
                    <SpecialSortable
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
                {flatStateBundle && flatProgressArr && evaluateAverageResult && specialState && (
                    <StateGridsManager
                        flatProgressArr={flatProgressArr}
                        setFlatProgressArr={setFlatProgressArr}
                        flatUnlockArr={flatUnlockArr}
                        setFlatUnlockArr={setFlatUnlockArr}
                        flatSucceedArr={flatSucceedArr}
                        setFlatSucceedArr={setFlatSucceedArr}
                        flatStateBundle={flatStateBundle}
                        setFlatStateBundle={setFlatStateBundle}
                        allowUserChangeState={allowUserChangeState}
                        upgradeArr={evaluateAverageResult.upgrade_arr}
                        specialState={specialState}
                        juiceInfo={evaluateAverageResult.prep_output.juice_info}
                    />
                )}


            </div>
        </div>
    )
}
