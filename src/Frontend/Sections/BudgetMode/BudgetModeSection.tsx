import React, { useState } from 'react'
import SpreadsheetGrid from '../../Components/SpreadsheetGrid.tsx'
import Graph from '../../Components/Graph.tsx'
import { styles, createColumnDefs, GRAPH_WIDTH, GRAPH_HEIGHT } from '../../Utils/Styles.ts'
import { INPUT_LABELS, OUTPUT_LABELS } from '../../Utils/Constants.ts'

type CostToChanceSectionProps = {
    budget_inputs: any
    set_budget_inputs: React.Dispatch<React.SetStateAction<any>>
    userMatsValue: any
    setUserMatsValue: React.Dispatch<React.SetStateAction<any>>
    autoGoldValues: boolean
    setAutoGoldValues: React.Dispatch<React.SetStateAction<boolean>>
    chance_result: any
    cachedChanceGraphData: { hist_counts?: any, hist_mins?: any, hist_maxs?: any } | null
    AnythingTicked: boolean
    CostToChanceBusy: boolean
    cumulativeGraph: boolean
    lockXAxis: boolean
    lockedMins: number[] | null
    lockedMaxs: number[] | null
}

export default function CostToChanceSection({
    budget_inputs,
    set_budget_inputs,
    userMatsValue,
    setUserMatsValue,
    autoGoldValues,
    setAutoGoldValues: _setAutoGoldValues,
    chance_result,
    cachedChanceGraphData,
    AnythingTicked,
    CostToChanceBusy,
    cumulativeGraph,
    lockXAxis,
    lockedMins,
    lockedMaxs,
}: CostToChanceSectionProps) {
    const { costToChanceColumnDefs } = createColumnDefs(autoGoldValues)
    const [showOptimized, setShowOptimized] = useState<boolean>(() => false);
    const [showOptimizedDetails, setShowOptimizedDetails] = useState<boolean>(() => false);
    // const [showGraph, setShowGraph] = useState<boolean>(() => false);
    return (
        <>
            {/* <h3 style={{ color: 'var(--text-primary)', fontSize: 'var(--font-size-base)', fontWeight: 'var(--font-weight-semibold)', margin: '16px 0 0px 0' }}>Cost to Chance</h3> */}
            <div style={{ ...styles.inputSection, flexDirection: "row", maxWidth: "1200px", width: "100%" }}>
                <div style={{ display: 'flex', gap: autoGoldValues ? 110 : 20, alignItems: 'flex-start' }}>
                    <div style={{ display: 'flex', flexDirection: "column", gap: 0, alignItems: 'flex-start', justifyContent: 'start', width: autoGoldValues ? 210 : 300 }}>
                        <div style={{ display: 'flex', gap: autoGoldValues ? 100 : 0, alignItems: 'flex-start' }}>
                            <div style={{ marginBottom: 16, width: autoGoldValues ? 210 : 310, marginLeft: 10 }}>
                                <SpreadsheetGrid
                                    columnDefs={costToChanceColumnDefs}
                                    labels={INPUT_LABELS}
                                    sheet_values={budget_inputs}
                                    set_sheet_values={set_budget_inputs}
                                    secondaryValues={userMatsValue}
                                    setSecondaryValues={setUserMatsValue}
                                />
                                {/* <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8, marginLeft: 0 }}>

                                    <label htmlFor="custom-gold-values" style={{ color: 'var(--text-primary)', fontSize: 'var(--font-size-sm)', whiteSpace: 'nowrap' }}>
                                        Custom Gold values
                                    </label>
                                    <input
                                        type="checkbox"
                                        id="custom-gold-values"
                                        checked={!autoGoldValues}
                                        onChange={(e) => setAutoGoldValues(!e.target.checked)}
                                        style={{ accentColor: 'var(--control-checked-bg)' }}
                                    />
                                </div> */}
                            </div>
                            <div style={{ display: 'flex', flexDirection: "row" }}>
                                <div style={{ display: 'flex', flexDirection: "row", gap: 0, alignItems: 'flex-start', height: 400 }}>

                                    <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginTop: 150, marginLeft: 60 }}>
                                        <div style={{ ...styles.inputLabelCell, whiteSpace: 'nowrap', color: 'var(--text-success)', textAlign: "right", width: 200, fontSize: 20 }}>Chance of success:</div>
                                        <div style={{ ...styles.inputCell, border: 'none', background: "transparent", color: 'var(--text-success)', fontSize: 28, width: 130, }}>{chance_result ? (String(chance_result.chance) + '%') : '-'}</div>

                                    </div>
                                    {(chance_result) && < div style={{ marginLeft: 80, flexDirection: "column", display: "flex", marginTop: 100, }}>
                                        <div style={{ ...styles.inputLabelCell, marginTop: 0, whiteSpace: 'nowrap', textAlign: "left", color: 'var(--text-success)', }}>Individual chances:</div>
                                        <div style={{ marginTop: 4, color: 'var(--text-muted)', fontSize: 'var(--font-size-sm)', whiteSpace: "wrap", width: 300 }}>
                                            {(chance_result.reasons || []).map((s: string, idx: number) => (
                                                <div key={"Fail reason" + (idx + 1)}> {s}</div>
                                            ))}
                                        </div>
                                    </div>}

                                </div>


                            </div>

                        </div>

                        {chance_result && (Number(chance_result.budgets_red_remaining) < 0 || Number(chance_result.budgets_blue_remaining) < 0) && (
                            <div style={{ marginLeft: 12, color: '#f59e0b', fontSize: 'var(--font-size-sm)', whiteSpace: 'nowrap' }}>
                                Invalid result!  Missing roughly {Number(chance_result.budgets_red_remaining) < 0 ? (Number(chance_result.budgets_red_remaining) * -1).toString() + ' red juice' : ''}
                                {(Number(chance_result.budgets_red_remaining) < 0 && Number(chance_result.budgets_blue_remaining) < 0) ? ' and ' : ''}
                                {Number(chance_result.budgets_blue_remaining) < 0 ? (Number(chance_result.budgets_blue_remaining) * -1).toString() + ' blue juice' : ""} for Advanced Honing, use "No Juice" option or add more juice
                            </div>
                        )}

                        {chance_result && (
                            <div style={{ marginTop: 4, color: 'var(--text-muted)', fontSize: 'var(--font-size-xs)' }}>Run time: {chance_result.run_time}s</div>
                        )}

                        <div style={{ display: 'flex', flexDirection: "row", gap: 50, alignItems: 'flex-start', marginTop: -130, marginLeft: 320 }}>
                            <div style={{ display: "flex", flexDirection: "column" }}>

                                <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginTop: 0 }}>
                                    <div style={{ ...styles.inputLabelCell, whiteSpace: 'nowrap', color: 'var(--text-optimized)', textAlign: "right", width: 200 }}>Optimal chance:</div>
                                    <div style={{ ...styles.inputCell, border: 'none', background: "transparent", color: 'var(--text-optimized)', fontSize: 'var(--font-size-xl)' }}>{chance_result ? (String(chance_result.optimized_chance) + '%') : '-'}</div>

                                </div>

                                <div style={{ ...styles.inputLabelCell, color: '#ffffff', whiteSpace: 'wrap', textAlign: "left", width: 250, fontSize: 14, marginLeft: 80 }}>Your Chance of Success if you use some of your gold to <span style={{ color: "var(--text-optimized)" }}>buy other mats  </span> (excluding juice).</div>


                                <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8, justifySelf: "center", marginLeft: 100, marginTop: 20 }}>

                                    <label htmlFor="show-optimized-details" style={{ color: 'var(--text-primary)', fontSize: 'var(--font-size-sm)', whiteSpace: 'nowrap' }}>
                                        Show details
                                    </label>
                                    <input
                                        type="checkbox"
                                        id="show-optimized-details"
                                        checked={showOptimizedDetails}
                                        onChange={(e) => setShowOptimizedDetails(e.target.checked)}
                                        style={{ accentColor: 'var(--control-checked-bg)' }}
                                    />
                                </div>
                            </div>
                            {showOptimizedDetails &&
                                <div style={{ display: "flex", flexDirection: "column" }}>
                                    {/* <div style={{ ...styles.inputLabelCell, whiteSpace: 'wrap', textAlign: "left", width: 450, fontSize: 14, marginLeft: 0, marginTop: 0 }}>⚠️ Warning! The info below is calculated without knowing how your taps go. You should not pre-emptively buy mats based on this, only buy when you run out! </div> */}
                                    <div style={{ display: "flex", flexDirection: "row" }}>
                                        <div style={{ display: "flex", flexDirection: "column" }}>
                                            <div style={{ ...styles.inputLabelCell, marginTop: 0, whiteSpace: 'nowrap', textAlign: "left", color: 'var(--text-optimized)', paddingTop: 0, marginLeft: 0, width: 200, }}>Buy these:</div>
                                            {chance_result &&
                                                <div style={{ marginTop: 4, color: 'var(--text-muted)', fontSize: 'var(--font-size-sm)', whiteSpace: "wrap", width: 200, marginLeft: 0 }}>
                                                    {(chance_result.buy_arr || []).map((s: string, idx: number) => (
                                                        <div key={"buy arr" + (idx + 1)}>{INPUT_LABELS[idx]}: {Math.max(Number(idx == 5) * (parseInt(s) - budget_inputs[INPUT_LABELS[idx]]),
                                                            parseInt(s) - budget_inputs[INPUT_LABELS[idx]])}
                                                        </div>
                                                    ))}
                                                    <div style={{ ...styles.inputLabelCell, whiteSpace: "wrap", marginLeft: -30, fontSize: 12, width: 200 }}>There are many ways to spend your gold, this gives the best chance of success.</div>
                                                </div>}
                                            <div>
                                            </div>


                                        </div>
                                        <div style={{ display: "flex", flexDirection: "column" }}>
                                            <div style={{ ...styles.inputLabelCell, marginTop: 0, whiteSpace: 'nowrap', textAlign: "left", color: 'var(--text-optimized)', width: 300 }}>Individual chances After buying:</div>
                                            {chance_result &&
                                                <div style={{ marginTop: 4, color: 'var(--text-muted)', fontSize: 'var(--font-size-sm)', whiteSpace: "wrap", width: 300 }}>
                                                    {(chance_result.optimized_reasons || []).map((s: string, idx: number) => (
                                                        <div key={"Fail reason" + (idx + 1)}>{s}</div>
                                                    ))}
                                                </div>
                                            }
                                        </div>
                                    </div>

                                    <div style={{ display: 'flex', flexDirection: "row", alignItems: 'center', gap: 8, marginBottom: 8, marginLeft: 220 }}>

                                        <label htmlFor="show-optimized" style={{ color: 'var(--text-muted)', fontSize: 'var(--font-size-sm)', whiteSpace: 'nowrap' }}>
                                            Show this on graph
                                        </label>
                                        <input
                                            type="checkbox"
                                            id="show-optimized"
                                            checked={showOptimized}
                                            onChange={(e) => setShowOptimized(e.target.checked)}
                                            style={{ accentColor: 'var(--control-checked-bg)' }}
                                        />
                                    </div>
                                </div>


                            }
                        </div>
                        <div style={{ display: 'flex', gap: 0, alignItems: 'flex-start', marginTop: showOptimizedDetails ? 0 : 104, flexDirection: "column", width: "100%" }}>

                            {chance_result && <div style={{ display: "flex", flexDirection: "row", marginTop: 0 }}>
                                <div>
                                    <div style={{ ...styles.inputLabelCell, whiteSpace: 'nowrap', color: 'var(--free-tap)', }}>Free taps value ranking:</div>
                                    <div style={{ marginTop: 4, color: 'var(--text-muted)', fontSize: 'var(--font-size-sm)', width: 250 }}>
                                        {(chance_result.upgrade_strings || []).slice(0, 10).map((upgrade: string, index: number) => (
                                            <div key={"Free tap value" + (index + 1)}>{index + 1}. {upgrade}</div>
                                        ))}
                                    </div>
                                </div>
                                <div>
                                    <div style={{ ...styles.inputLabelCell, whiteSpace: 'nowrap', color: 'var(--series-red)', }}>Red juice (weapon):</div>
                                    <div style={{ marginTop: 4, color: 'var(--text-muted)', fontSize: 'var(--font-size-sm)', width: 330 }}>
                                        {(chance_result.juice_strings_weapon || []).map((s: string, idx: number) => (
                                            <div key={"Red juice value" + (idx + 1)}>{idx + 1}. {s}</div>
                                        ))}
                                    </div>
                                </div>
                                <div>
                                    <div style={{ ...styles.inputLabelCell, whiteSpace: 'nowrap', color: 'var(--series-blue)', }}>Blue juice (armor):</div>
                                    <div style={{ marginTop: 4, color: 'var(--text-muted)', fontSize: 'var(--font-size-sm)', width: 450 }}>
                                        {(chance_result.juice_strings_armor || []).map((s: string, idx: number) => (
                                            <div key={"Blue juice value" + (idx + 1)}>{idx + 1}. {s}</div>
                                        ))}
                                    </div>
                                </div>
                            </div>
                            }
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
                            {(chance_result) && (//&& (chance_result.reasons?.length > 0 || chance_result.upgrade_strings?.length > 0 || chance_result.juice_strings_armor?.length > 0 || chance_result.juice_strings_weapon?.length > 0)) && (

                                <div style={{ display: "flex", marginLeft: 150, marginBottom: 30, marginTop: 30 }}>

                                    <div style={{ flex: 1 }}>
                                        <Graph
                                            title="Cost distribution(uses free tap & juice)"
                                            labels={OUTPUT_LABELS}
                                            counts={AnythingTicked ? (chance_result?.hist_counts || cachedChanceGraphData?.hist_counts) : null}
                                            mins={chance_result?.hist_mins || cachedChanceGraphData?.hist_mins}
                                            maxs={chance_result?.hist_maxs || cachedChanceGraphData?.hist_maxs}
                                            width={GRAPH_WIDTH}
                                            height={GRAPH_HEIGHT}
                                            budgets={OUTPUT_LABELS.map(label => Number(budget_inputs[label]))}
                                            hasSelection={AnythingTicked}
                                            isLoading={CostToChanceBusy}
                                            cumulative={cumulativeGraph}
                                            lockXAxis={lockXAxis}
                                            lockedMins={lockedMins}
                                            lockedMaxs={lockedMaxs}
                                            graphType={"Histogram"}
                                            additionalBudgets={showOptimized ? chance_result?.buy_arr : null}
                                        />
                                    </div>

                                </div>


                            )}
                        </div>
                    </div>

                </div>
            </div >
        </>
    )
}

