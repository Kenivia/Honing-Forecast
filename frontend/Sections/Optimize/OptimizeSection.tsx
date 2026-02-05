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
import LabeledCheckbox from "@/Components/LabeledCheckbox.tsx"

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
    setAllowUserChangeState: React.Dispatch<React.SetStateAction<boolean>>
    isJuiceInfoCollapsed: boolean
    setIsJuiceInfoCollapsed: React.Dispatch<React.SetStateAction<boolean>>
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
    return String(input <= 0 ? "Avg Eqv Cost " + add_comma(input < 0.0 ? -input : input) + "g" : "Avg Eqv Surplus of " + add_comma(input) + "g")
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
    setAllowUserChangeState,
    isJuiceInfoCollapsed,
    setIsJuiceInfoCollapsed,
    // gridRefs,
    // onGridMouseDown,
    // marquee,
}: OptimizeSectionProps) {
    const [isFreeTapCollapsed, setIsFreeTapCollapsed] = React.useState(true)
    const [isGoldBreakdownCollapsed, setIsGoldBreakdownCollapsed] = React.useState(true)
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

    // const handleOptimizeAverageClick = () => {
    //     if (optimizeAvgBusy) {
    //         optimizeAvgWorkerRef.current?.terminate()
    //         optimizeAvgWorkerRef.current = null
    //         setOptimizeAvgBusy(false)
    //         setAutoRunOptimizer(false)
    //         // setBeforeMetric(null)
    //         onCancelOptimizeAverage()
    //         return
    //     } else {
    //         setBeforeMetric(evaluateAverageResult?.metric ?? null)
    //         setOptimizeButtonPress((prev: number) => prev + 1)
    //     }
    // }
    const result_status = (
        <div
            className="result-status"
            style={{
                color:
                    curIsBest && hasRunOptimizer
                        ? "var(--brighter-optimizer)"
                        : hasRunOptimizer
                          ? "var(--sub-optimal)"
                          : optimizeAvgBusy
                            ? "var(--sub-optimal)"
                            : "red",
            }}
        >
            {curIsBest && hasRunOptimizer
                ? "(Result below is the best known result)"
                : hasRunOptimizer
                  ? "(Current config is not best known result)"
                  : optimizeAvgBusy
                    ? "(Optimizer in progress…)"
                    : "(Optimizer not run yet!!! press optimizer button above)"}
            {/* {!curIsBest && hasRunOptimizer && (
                <button onClick={handleRestoreBest} disabled={!canRestoreBest} className="restore-btn">
                    {hasRunOptimizer ? (canRestoreBest ? "Restore Best" : "Current configuration is the best known one") : "Optimizer not run"}
                </button>
            )} */}
        </div>
    )
    // const optimizeAverageButton = (
    //     <button
    //         onClick={handleOptimizeAverageClick}
    //         style={{
    //             background: optimizeAvgBusy ? "var(--cancel-optimizer-button)" : "var(--optimizer-button)",
    //             color: optimizeAvgBusy ? "var(--text-muted)" : "var(--text)",
    //             padding: "6px 10px",
    //             borderRadius: 4,
    //             border: "1px solid var(--btn-border)",
    //             cursor: "pointer",
    //             height: "50px",
    //             width: "300px",
    //             fontSize: optimizeAvgBusy ? "24px" : "30px",
    //             textWrap: "nowrap",
    //             textAlign: "center",
    //         }}
    //     >
    //         {optimizeAvgBusy ? "Cancel Optimize Average" : " >>> Optimize <<< "}
    //     </button>
    // )

    // console.log("special", specialState)
    // console.log(flatSucceedArr)

    return (
        <div style={{ ...styles.inputSection, flexDirection: "column", maxWidth: "1200px", width: "100%", gap: 12 }}>
            <div>
                <div style={{ position: "relative", flex: 1, display: "flex", flexDirection: "column", gap: 12 }}>
                    <section className="optimizer-section">
                        <span
                            className="optimizer-section-title "
                            aria-expanded={!isJuiceInfoCollapsed}
                            aria-controls="juice-info-section"
                            onClick={() => setIsJuiceInfoCollapsed((prev) => !prev)}
                        >
                            <span>Instructions</span>
                            <span className={`optimizer-section-title-arrow ${isJuiceInfoCollapsed ? "collapsed" : ""}`}>{">"}</span>
                            {result_status}
                        </span>
                        {!isJuiceInfoCollapsed && (
                            <div id="juice-info-section" className="optimizer-section-body">
                                <span>
                                    Go from top to bottom, empty squares mean no book / no juice.<br></br>
                                </span>
                                <div style={{ display: "flex", alignItems: "center", gap: "8px", textWrap: "wrap", lineHeight: "1.0" }}>
                                    <LabeledCheckbox
                                        label="Enable updating progress for even better optimization"
                                        checked={allowUserChangeState}
                                        setChecked={setAllowUserChangeState}
                                    />
                                </div>
                                {allowUserChangeState && (
                                    <div>
                                        <span>
                                            {" "}
                                            - Click the number of taps it took to succeed, and press success. For free taps, just press succeed. How often you
                                            want to update is up to you, you can do it after every fail if you want.
                                            <br></br>- Remember to re-run the optimizer, or toggle "Auto start optimizer"<br></br> - Mats & juice costs are
                                            automatically deducted in the background, but you need to manually update how many special leaps you have left.
                                            <br></br>- Advanced honing progress update is unavailable, you'll have to update your mats manually after succeeding
                                            all 10 levels. As such, you should probably do your advanced honing before all normal honing or after all normal
                                            honing.
                                            <br></br>- Note that the "Avg eqv gold cost" is the costs so far + average cost to come. As such, when you've
                                            succeeded everything, that's how much you spent.
                                        </span>
                                        <div style={{ width: 200 }}>
                                            <SpreadsheetGrid
                                                columnDefs={[
                                                    {
                                                        headerName: "",
                                                        editable: true,
                                                        flex: 1,
                                                        width: "80px",
                                                        background: "var(--grid-cell-bg)",
                                                        backgroundSelected: "var(--grid-cell-selected)",
                                                        color: "var(--grid-cell-text)",
                                                    },
                                                ]}
                                                labels={["Special Leap"]}
                                                sheetValuesArr={[{ "Special Leap": inputsBundle.values.mats.owned["Special Leap"] ?? "0" }]}
                                                setSheetValuesArr={[
                                                    (value) => {
                                                        let cleanValue = value["Special Leap"].replace(/[^0-9]/g, "")
                                                        cleanValue = cleanValue.replace(/^0+(?=\d)/, "")

                                                        cleanValue = String(Math.min(parseFloat(cleanValue), 33333))

                                                        const next = { ...inputsBundle.values.mats.owned }
                                                        next["Special Leap"] = cleanValue
                                                        inputsBundle.setters.mats.setOwned(next)
                                                    },
                                                ]}
                                                hideIcons={false}
                                                fontSizeOverride={"var(--font-size-xs)"}
                                                noHeader={true}
                                            />
                                        </div>
                                    </div>
                                )}
                                {flatStateBundle && flatProgressArr && evaluateAverageResult && specialState && (
                                    <div>
                                        {" "}
                                        {result_status}
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
                                    </div>
                                )}
                            </div>
                        )}
                    </section>{" "}
                </div>{" "}
                <section className="optimizer-section">
                    <span
                        className="optimizer-section-title "
                        aria-expanded={!isFreeTapCollapsed}
                        aria-controls="free-tap-order-section"
                        onClick={() => setIsFreeTapCollapsed((prev) => !prev)}
                    >
                        <span>More Free tap info </span>
                        <span className={`optimizer-section-title-arrow ${isFreeTapCollapsed ? "collapsed" : ""}`}>{">"}</span>
                        {result_status}
                    </span>{" "}
                    {!isFreeTapCollapsed && (
                        <div id="free-tap-order-section" className="optimizer-section-body">
                            <div style={{ display: "flex", alignItems: "center", gap: "8px", textWrap: "wrap", lineHeight: "1.0" }}>
                                <LabeledCheckbox label="Allow updating progress" checked={allowUserChangeState} setChecked={setAllowUserChangeState} />
                            </div>
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
                                    inputsBundle={inputsBundle}
                                    allowUserChangeState={allowUserChangeState}
                                />
                            )}
                        </div>
                    )}
                </section>
                <section className="optimizer-section">
                    <span
                        className="optimizer-section-title "
                        aria-expanded={!isGoldBreakdownCollapsed}
                        aria-controls="gold-breakdown-section"
                        onClick={() => setIsGoldBreakdownCollapsed((prev) => !prev)}
                    >
                        <span>Gold breakdown</span>
                        <span className={`optimizer-section-title-arrow ${isGoldBreakdownCollapsed ? "collapsed" : ""}`}>{">"}</span>
                        {result_status}
                    </span>
                    {!isGoldBreakdownCollapsed && (
                        <div id="gold-breakdown-section" className="optimizer-section-body">
                            {/* <span>The Score is the sum of the "Average equivalent gold spent" for each mat. Without advanced mode, this is calculated as:</span>
                    <span style={{ marginLeft: 50, fontSize: 24, fontFamily: "Times new roman" }}>
                        Average of [(Non-bound mat consumed, if any, otherwise 0) * market price] = Average eqv gold spent.
                    </span>
                    <span>
                        As such, if you have tradable mats, this will not be how much gold you actually spend on buying. This is just a way to convert all the
                        different material types into one metric, so that the optimizer can minimize it.{" "}
                    </span>
                    <br></br>
                    <span>Critically, this is not the same as:</span>
                    <span style={{ marginLeft: 50, fontSize: 24, fontFamily: "Times new roman" }}>[Average of (Mat consumed) - mat owned] * market price,</span>
                    <span>
                        because the cases where we use less than BOUND OWNED drags down the Average of (Mat consumed), but we don't actually gain any gold from
                        having leftover bound mats. More formally this is because Expecation is not commutative with non-linear functions.
                    </span> */}
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

                            <span></span>
                            <span style={{ fontSize: 24, color: "var(--brighter-optimizer)" }}>
                                Combined: {breakdown_to_english(evaluateAverageResult?.metric)}
                            </span>
                            {/* </div>
                    </div> */}
                        </div>
                    )}
                </section>
            </div>
        </div>
    )
}
