import React from "react"
import SpreadsheetGrid from "@/Frontend/Components/SpreadsheetGrid.tsx"
import Graph from "@/Frontend/Components/Graph.tsx"
import { styles, createColumnDefs, GRAPH_WIDTH, GRAPH_HEIGHT } from "@/Frontend/Utils/Styles.ts"
import { OUTPUT_LABELS, INPUT_LABELS } from "@/Frontend/Utils/Constants.ts"
import { SliderBundle } from "@/Frontend/Components/SliderBundle.tsx"
import LabeledCheckbox from "@/Frontend/Components/LabeledCheckbox.tsx"
// import { formatSig } from "@/Frontend/Utils/Helpers.ts"

type ChanceToCostSectionProps = {
    desired_chance: string
    uncleaned_desired_chance: string
    onDesiredChange: (_: string) => void
    onDesiredBlur: () => void
    cost_result: any
    // cost_result_optimized: any
    cachedCostGraphData: { hist_counts?: any; hist_mins?: any; hist_maxs?: any } | null
    AnythingTicked: boolean
    ChanceToCostBusy: boolean

    cumulativeGraph: boolean
    lockXAxis: boolean
    lockedMins: number[] | null
    lockedMaxs: number[] | null
    // Show Average checkbox props
    showAverage: boolean
    setShowAverage: React.Dispatch<React.SetStateAction<boolean>>
    // Moved worker call results
    averageCosts: number[] | null
    AverageCostBusy: boolean
    // Data size for luckiest draw message
    dataSize: string
    budget_inputs: any
    set_budget_inputs: React.Dispatch<React.SetStateAction<any>>
    userMatsValue: any
    setUserMatsValue: React.Dispatch<React.SetStateAction<any>>
    monteCarloResult: any
}

export default function ChanceToCostSection({
    desired_chance,
    uncleaned_desired_chance,
    onDesiredChange,
    onDesiredBlur,
    cost_result,

    // cost_result_optimized,
    cachedCostGraphData,
    AnythingTicked,
    ChanceToCostBusy,
    // ChanceToCostOptimizedBusy,
    cumulativeGraph,
    lockXAxis,
    lockedMins,
    lockedMaxs,
    // Show Average checkbox props
    showAverage,
    setShowAverage,
    // Moved worker call results
    averageCosts,
    AverageCostBusy: _AverageCostBusy,
    // Data size for luckiest draw message
    dataSize,
    monteCarloResult,
}: ChanceToCostSectionProps) {
    const { chanceToCostColumnDefs } = createColumnDefs(false)
    return (
        <>
            {/* <h3 style={{ color: 'var(--text-primary)', fontSize: 'var(--font-size-base)', fontWeight: 'var(--font-weight-semibold)', margin: '0 0 -8px 0' }}>Chance to Cost</h3> */}
            <div style={{ ...styles.inputSection, maxWidth: "1000px", width: "100%" }}>
                {/* <div style={{ display: "flex", flexDirection: "column", paddingLeft: 50, }}> */}
                <div style={{ display: "flex", gap: 8, alignItems: "center", paddingLeft: 50 }}>
                    <SliderBundle
                        desiredChance={desired_chance}
                        uncleanedDesiredChance={uncleaned_desired_chance}
                        onDesiredChange={onDesiredChange}
                        onDesiredBlur={onDesiredBlur}
                        lowThreshold={0}
                        lowText={`0% = luckiest draw in ${dataSize} samples`}
                    />
                    <div style={{ marginLeft: 100 }}>
                        <div style={{ marginBottom: 16, width: 210 }}>
                            <SpreadsheetGrid
                                columnDefs={chanceToCostColumnDefs}
                                labels={OUTPUT_LABELS}
                                sheetValuesArr={
                                    cost_result
                                        ? [
                                              Object.fromEntries(
                                                  OUTPUT_LABELS.map((label, lab_index) => [
                                                      label,
                                                      String(cost_result.hundred_budgets[parseInt(desired_chance)][lab_index]),
                                                  ])
                                              ),
                                          ]
                                        : [Object.fromEntries(OUTPUT_LABELS.map((label) => [label, "Calculating..."]))]
                                }
                                setSheetValuesArr={[() => {}]} // No-op for read-only
                            />
                        </div>
                        <div style={{ marginTop: 12, display: "flex", alignItems: "center", gap: 8 }}>
                            <LabeledCheckbox label="Show Average on graph" checked={showAverage} setChecked={setShowAverage} />
                        </div>
                        <div style={{ height: 40, display: "flex", alignItems: "flex-start", marginTop: 10, marginBottom: 10 }}>
                            <pre
                                style={{
                                    color: "var(--text-muted)",
                                    fontSize: "var(--font-size-xs)",
                                    opacity: cost_result && monteCarloResult ? 1 : 0,
                                    margin: 0,
                                }}
                            >
                                {cost_result && monteCarloResult ? (
                                    <>
                                        Run time: {Number(cost_result.run_time) + Number(monteCarloResult.run_time)}s{"\nActual chance: "}
                                        {cost_result.hundred_chances[parseInt(desired_chance)]?.toFixed(2)}%
                                    </>
                                ) : (
                                    "Run time: Calculating..."
                                )}
                            </pre>
                        </div>
                    </div>
                </div>

                {/* 
                <div style={{ display: 'flex', gap: 8, alignItems: 'center', marginBottom: 16, paddingLeft: 50, }}>
                    <div style={{ marginBottom: 16, width: 210, marginLeft: 10 }}>
                        <SpreadsheetGrid
                            columnDefs={costToChanceColumnDefs}
                            labels={INPUT_LABELS}
                            sheet_values={budget_inputs}
                            set_sheet_values={set_budget_inputs}
                            secondaryValues={userMatsValue}
                            setSecondaryValues={setUserMatsValue}
                        />

                    </div>

                 
                </div> */}

                <div style={{ display: "flex", gap: 110, alignItems: "center", width: 850, justifySelf: "center", paddingLeft: 70 }}>
                    <div style={{ flex: 1 }}>
                        <Graph
                            title={cumulativeGraph ? "Cost distribution(Cumulative)" : "Cost distribution"}
                            labels={OUTPUT_LABELS}
                            counts={AnythingTicked ? cost_result?.hist_counts || cachedCostGraphData?.hist_counts : null}
                            mins={cost_result?.hist_mins || cachedCostGraphData?.hist_mins}
                            maxs={cost_result?.hist_maxs || cachedCostGraphData?.hist_maxs}
                            width={GRAPH_WIDTH}
                            height={GRAPH_HEIGHT}
                            budgets={
                                cost_result && OUTPUT_LABELS.map((_, lab_index) => Number(cost_result.hundred_budgets[parseInt(desired_chance)][lab_index]))
                            }
                            additionalBudgets={showAverage ? averageCosts : null}
                            hasSelection={AnythingTicked}
                            isLoading={ChanceToCostBusy}
                            cumulative={cumulativeGraph}
                            lockXAxis={lockXAxis}
                            lockedMins={lockedMins}
                            lockedMaxs={lockedMaxs}
                            graphType={"Histogram"}
                        />
                    </div>
                </div>
            </div>
        </>
    )
}
