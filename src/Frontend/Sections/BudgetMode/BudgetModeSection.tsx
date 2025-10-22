import React from "react"
import SpreadsheetGrid from "@/Frontend/Components/SpreadsheetGrid.tsx"
import Graph from "@/Frontend/Components/Graph.tsx"
import LabeledCheckbox from "@/Frontend/Components/LabeledCheckbox.tsx"
import { styles, createColumnDefs, GRAPH_WIDTH, GRAPH_HEIGHT } from "@/Frontend/Utils/Styles.ts"
import { INPUT_LABELS, OUTPUT_LABELS } from "@/Frontend/Utils/Constants.ts"
import { SliderBundle } from "@/Frontend/Components/SliderBundle.tsx"

type CostToChanceSectionProps = {
    budget_inputs: any
    set_budget_inputs: React.Dispatch<React.SetStateAction<any>>
    userMatsValue: any
    setUserMatsValue: React.Dispatch<React.SetStateAction<any>>

    setAutoGoldValues: React.Dispatch<React.SetStateAction<boolean>>
    chance_result: any
    cachedChanceGraphData: { hist_counts?: any; hist_mins?: any; hist_maxs?: any } | null
    AnythingTicked: boolean
    CostToChanceBusy: boolean
    cumulativeGraph: boolean
    lockXAxis: boolean
    lockedMins: number[] | null
    lockedMaxs: number[] | null
    // cost_result_optimized: any
    desired_chance: string
    uncleaned_desired_chance: string
    onDesiredChange: (_: string) => void
    onDesiredBlur: () => void
    showOptimizedDetails: boolean
    setShowOptimizedDetails: React.Dispatch<React.SetStateAction<boolean>>
    monteCarloResult: any
}

export default function CostToChanceSection({
    budget_inputs,
    set_budget_inputs,
    userMatsValue,
    setUserMatsValue,

    setAutoGoldValues: _setAutoGoldValues,
    chance_result,
    cachedChanceGraphData,
    AnythingTicked,
    CostToChanceBusy,
    cumulativeGraph,
    lockXAxis,
    lockedMins,
    lockedMaxs,

    // cost_result_optimized,

    desired_chance,
    uncleaned_desired_chance,
    onDesiredChange,
    onDesiredBlur,
    showOptimizedDetails: noBuyChecked,
    setShowOptimizedDetails,
    monteCarloResult,
}: CostToChanceSectionProps) {
    const { chanceToCostColumnDefs, costToChanceColumnDefs } = createColumnDefs(false) // autoGoldValues not used for this section

    // const [showOptimized, setShowOptimized] = useState<boolean>(() => false)
    // const [showGraph, setShowGraph] = useState<boolean>(() => false);
    /* <h3 style={{ color: 'var(--text-primary)', fontSize: 'var(--font-size-base)', fontWeight: 'var(--font-weight-semibold)', margin: '16px 0 0px 0' }}>Cost to Chance</h3> */
    return (
        <div style={{ ...styles.inputSection, flexDirection: "row", maxWidth: "1200px", width: "100%" }}>
            {/* <div style={{ display: "flex", gap: 110, alignItems: "flex-start" }}> */}
            <div
                style={{
                    display: "flex",
                    flexDirection: "column",
                    gap: 0,
                    alignItems: "flex-start",
                    justifyContent: "start",
                    width: 210,
                }}
            >
                {/* Budget input, chance of success and individual chances */}

                <div style={{ display: "flex", flexDirection: "row", gap: 100, marginTop: 0, marginLeft: 0, width: 1200 }}>
                    <div style={{ marginBottom: 16, width: 310, marginLeft: 0 }}>
                        <SpreadsheetGrid
                            columnDefs={costToChanceColumnDefs}
                            labels={INPUT_LABELS}
                            sheetValuesArr={[budget_inputs, userMatsValue]}
                            setSheetValuesArr={[set_budget_inputs, setUserMatsValue]}
                        />
                    </div>

                    <div style={{ flexDirection: "column", marginTop: 20 }}>
                        {!noBuyChecked && (
                            <div
                                style={{
                                    display: "flex",
                                    flexDirection: "row",
                                    gap: 0,
                                    alignItems: "center",
                                    width: 300,
                                    marginLeft: -155,
                                    marginTop: 20,
                                    marginBottom: -30,
                                }}
                            >
                                <SliderBundle
                                    desiredChance={String(Math.max(Math.floor(chance_result?.optimized_chance ?? 0), Number(desired_chance)))}
                                    uncleanedDesiredChance={uncleaned_desired_chance}
                                    onDesiredChange={(value) =>
                                        onDesiredChange(String(Math.max(Math.floor(chance_result?.optimized_chance ?? 0), Number(value))))
                                    }
                                    onDesiredBlur={onDesiredBlur}
                                />
                            </div>
                        )}
                        {!noBuyChecked && (
                            <div style={{ marginBottom: 16, width: 210, marginLeft: 30, marginTop: 0 }}>
                                <SpreadsheetGrid
                                    columnDefs={chanceToCostColumnDefs}
                                    labels={OUTPUT_LABELS.slice(0, 7)}
                                    sheetValuesArr={
                                        chance_result
                                            ? [
                                                  Object.fromEntries(
                                                      OUTPUT_LABELS.slice(0, 7).map((label, lab_index) => [
                                                          label,
                                                          String(chance_result.typical_costs[parseInt(desired_chance)][lab_index]),
                                                      ])
                                                  ),
                                              ]
                                            : [Object.fromEntries(OUTPUT_LABELS.map((label) => [label, "Calculating..."]))]
                                    }
                                    setSheetValuesArr={[() => {}]} // No-op for read-only
                                />
                            </div>
                        )}
                        {/* {!noBuyChecked && (
                         
                        )} */}
                    </div>
                    <div style={{ flexDirection: "column", marginTop: 70, marginLeft: noBuyChecked ? 240 : 0 }}>
                        <div style={{ display: "flex", flexDirection: "column", width: 300, marginTop: 0 }}>
                            <LabeledCheckbox
                                label="I will buy mats with gold when I run out"
                                checked={!noBuyChecked}
                                setChecked={(x) => setShowOptimizedDetails(!x)}
                                textColor="var(--text-success)"
                                accentColor="var(--text-success)"
                            />

                            <LabeledCheckbox
                                label="I will stop when I run out"
                                checked={noBuyChecked}
                                setChecked={setShowOptimizedDetails}
                                textColor="var(--text-optimized)"
                                accentColor="var(--text-optimized)"
                            />
                        </div>

                        <div style={{ display: "flex", flexDirection: "row" }}>
                            <div
                                style={{
                                    ...styles.inputLabelCell,
                                    whiteSpace: "nowrap",
                                    color: "var(--text-primary)",
                                    fontSize: 20,
                                    marginTop: 16,
                                }}
                            >
                                {"Current chance:"}
                            </div>
                            <div
                                style={{
                                    ...styles.inputCell,
                                    border: "none",
                                    background: "transparent",
                                    color: noBuyChecked ? "var(--text-optimized)" : "var(--text-success)",
                                    fontSize: 28,
                                }}
                            >
                                {chance_result ? (noBuyChecked ? String(chance_result.chance) : String(chance_result.chance_if_buy)) + "%" : "Calculating..."}
                            </div>
                        </div>
                        {!noBuyChecked && (
                            <div style={{ display: "flex", flexDirection: "column" }}>
                                <div style={{ display: "flex", flexDirection: "row", marginTop: 8, marginLeft: 0 }}>
                                    <div style={{ color: "var(--text-primary)", fontSize: 20, marginTop: 8 }}>Extra gold needed for</div>
                                    <div style={{ color: "var(--input-bg)", fontSize: 24, marginLeft: 8, marginTop: 4 }}>{parseInt(desired_chance)}%: </div>
                                </div>
                                <div style={{ color: "var(--text-success)", fontSize: 28, marginRight: 80, textAlign: "center" }}>
                                    {chance_result
                                        ? Math.max(0, chance_result.hundred_gold_costs[parseInt(desired_chance)] - budget_inputs["Gold"]).toLocaleString(
                                              "en-US",
                                              {
                                                  minimumFractionDigits: 0, // show decimals for small K/M/B
                                                  maximumFractionDigits: 0,
                                              }
                                          )
                                        : "Calculating..."}
                                </div>

                                <div
                                    style={{
                                        marginTop: 5,
                                        fontSize: "var(--font-size-sm)",
                                        fontStyle: "italic",
                                        whiteSpace: "wrap",
                                        marginLeft: 0,
                                        width: 320,
                                    }}
                                >
                                    You must balance <span style={{ fontWeight: "bold", fontStyle: "normal" }}>pre-buying</span> (to get a good deal from the
                                    market) and the risk of <span style={{ fontWeight: "bold", fontStyle: "normal" }}>over-buying</span> by yourself.
                                </div>
                                <div
                                    style={{
                                        marginTop: 5,
                                        fontSize: "var(--font-size-xs)",
                                        fontStyle: "italic",
                                        whiteSpace: "wrap",
                                        marginLeft: 0,
                                        width: 320,
                                    }}
                                >
                                    The above Gold value assumes no pre-buying (so 0 over-buying, most efficient) and the left numbers assume you pre-buy almost
                                    everything.
                                </div>
                            </div>
                        )}
                        {noBuyChecked && (
                            <div style={{ marginLeft: 10, flexDirection: "column", display: "flex", marginTop: 0 }}>
                                <div
                                    style={{
                                        ...styles.inputLabelCell,
                                        marginTop: 0,
                                        whiteSpace: "nowrap",
                                        textAlign: "left",
                                        color: "var(--text-optimized)",
                                    }}
                                >
                                    Individual chances:
                                </div>
                                <div
                                    style={{
                                        marginTop: 4,
                                        color: "var(--text-muted)",
                                        fontSize: "var(--font-size-sm)",
                                        whiteSpace: "wrap",
                                        width: 350,
                                    }}
                                >
                                    {(chance_result?.reasons ?? []).map((s: string, idx: number) => (
                                        <div key={"Fail reason" + (idx + 1)}> {s}</div>
                                    ))}
                                </div>
                            </div>
                        )}
                    </div>
                </div>

                {/* Not enough Juice warning*/}
                {chance_result && (Number(chance_result.budgets_red_remaining) < 0 || Number(chance_result.budgets_blue_remaining) < 0) && (
                    <div style={{ marginLeft: 12, color: "#f59e0b", fontSize: "var(--font-size-xl)", whiteSpace: "nowrap" }}>
                        Invalid result! Missing ~
                        {Number(chance_result.budgets_red_remaining) < 0 ? (Number(chance_result.budgets_red_remaining) * -1).toString() + " red juice" : ""}
                        {Number(chance_result.budgets_red_remaining) < 0 && Number(chance_result.budgets_blue_remaining) < 0 ? " and " : ""}
                        {Number(chance_result.budgets_blue_remaining) < 0
                            ? (Number(chance_result.budgets_blue_remaining) * -1).toString() + " blue juice"
                            : ""}{" "}
                        for Advanced Honing, use "No Juice" option or add more juice
                    </div>
                )}

                <div style={{ display: "flex", marginLeft: 150, marginBottom: 30, marginTop: 0, width: GRAPH_WIDTH }}>
                    <div style={{ flex: 1 }}>
                        <Graph
                            title="Cost distribution (uses free tap & juice)"
                            labels={OUTPUT_LABELS}
                            counts={AnythingTicked ? chance_result?.hist_counts || cachedChanceGraphData?.hist_counts : null}
                            mins={chance_result?.hist_mins || cachedChanceGraphData?.hist_mins}
                            maxs={chance_result?.hist_maxs || cachedChanceGraphData?.hist_maxs}
                            width={GRAPH_WIDTH}
                            height={GRAPH_HEIGHT}
                            budgets={OUTPUT_LABELS.map((label) => Number(budget_inputs[label]))}
                            hasSelection={AnythingTicked}
                            isLoading={CostToChanceBusy}
                            cumulative={cumulativeGraph}
                            lockXAxis={lockXAxis}
                            lockedMins={lockedMins}
                            lockedMaxs={lockedMaxs}
                            graphType={"Histogram"}
                            additionalBudgets={!noBuyChecked ? chance_result?.typical_costs[parseInt(desired_chance)] || null : null}
                        />
                    </div>
                </div>

                <div style={{ display: "flex", gap: 0, alignItems: "flex-start", marginTop: 0, flexDirection: "column", width: 1000 }}>
                    {chance_result && (
                        <div style={{ display: "flex", flexDirection: "row", marginTop: 0 }}>
                            <div>
                                <div style={{ ...styles.inputLabelCell, whiteSpace: "nowrap", color: "var(--free-tap)" }}>Free taps value ranking:</div>
                                <div style={{ marginTop: 4, color: "var(--text-muted)", fontSize: "var(--font-size-sm)", width: 250 }}>
                                    {(chance_result.upgrade_strings || []).slice(0, 10).map((upgrade: string, index: number) => (
                                        <div key={"Free tap value" + (index + 1)}>
                                            {index + 1}. {upgrade}
                                        </div>
                                    ))}
                                </div>
                            </div>
                            <div>
                                <div style={{ ...styles.inputLabelCell, whiteSpace: "nowrap", color: "var(--series-red)" }}>Red juice (weapon):</div>
                                <div style={{ marginTop: 4, color: "var(--text-muted)", fontSize: "var(--font-size-sm)", width: 450 }}>
                                    {(chance_result.juice_strings_weapon || []).map((s: string, idx: number) => (
                                        <div key={"Red juice value" + idx}>
                                            {idx ? idx + "." : ""} {s}
                                        </div>
                                    ))}
                                </div>
                            </div>
                            <div>
                                <div style={{ ...styles.inputLabelCell, whiteSpace: "nowrap", color: "var(--series-blue)" }}>Blue juice (armor):</div>
                                <div style={{ marginTop: 4, color: "var(--text-muted)", fontSize: "var(--font-size-sm)", width: 450 }}>
                                    {(chance_result.juice_strings_armor || []).map((s: string, idx: number) => (
                                        <div key={"Blue juice value" + idx}>
                                            {idx ? idx + "." : ""} {s}
                                        </div>
                                    ))}
                                </div>
                            </div>

                            {/* {(chance_result) && <div style={{ display: 'flex', flexDirection: "row", alignItems: 'center', gap: 8, marginBottom: 8, marginLeft: 240, marginTop: 30 }}>

                                <label htmlFor="show-graph" style={{ fontSize: 25, whiteSpace: 'nowrap' }}>
                                    Show cool Graph<br /> {/*  for  Juice &<br /> */}
                            {/* (How much Gold each mat is worth to you) */}

                            {/* </label>
                                <input
                                    type="checkbox"
                                    id="show-graph"
                                    checked={showGraph}
                                    onChange={(e) => setShowGraph(e.target.checked)}
                                    style={{ accentColor: 'var(--control-checked-bg)' }}
                                />
                            </div>}
                             */}
                        </div>
                    )}
                    {chance_result && (
                        <div style={{ marginTop: 10, fontSize: "var(--font-size-xs)", fontStyle: "italic", marginLeft: -0, marginBottom: 5 }}>
                            Currently, these gold values assumes that you are buying ALL mats. This is a work in progress
                        </div>
                    )}
                </div>
            </div>
            {/* Run time*/}

            <div style={{ marginTop: 4, color: "var(--text-muted)", fontSize: "var(--font-size-xs)" }}>
                {chance_result && monteCarloResult ? (
                    <>Run time: {(Number(chance_result.run_time) + Number(monteCarloResult.run_time)).toFixed(2)}s</>
                ) : (
                    "Run time: Calculating..."
                )}
            </div>
        </div>
    )
}
