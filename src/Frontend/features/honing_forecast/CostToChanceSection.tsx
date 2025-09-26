import React from 'react'
import SpreadsheetGrid from '../../components/SpreadsheetGrid.tsx'
import Graph from '../../components/Graph.tsx'
import { styles, createColumnDefs, GRAPH_WIDTH, GRAPH_HEIGHT } from './styles.ts'
import { INPUT_LABELS, OUTPUT_LABELS } from './constants.ts'

type CostToChanceSectionProps = {
    budget_inputs: any
    set_budget_inputs: React.Dispatch<React.SetStateAction<any>>
    userMatsValue: any
    setUserMatsValue: React.Dispatch<React.SetStateAction<any>>
    autoOptimization: boolean
    setAutoOptimization: React.Dispatch<React.SetStateAction<boolean>>
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
    autoOptimization,
    setAutoOptimization,
    chance_result,
    cachedChanceGraphData,
    AnythingTicked,
    CostToChanceBusy,
    cumulativeGraph,
    lockXAxis,
    lockedMins,
    lockedMaxs,
}: CostToChanceSectionProps) {
    const { costToChanceColumnDefs } = createColumnDefs(autoOptimization)

    return (
        <>
            {/* <h3 style={{ color: 'var(--text-primary)', fontSize: 'var(--font-size-base)', fontWeight: 'var(--font-weight-semibold)', margin: '16px 0 0px 0' }}>Cost to Chance</h3> */}
            <div style={{ ...styles.inputSection, flexDirection: "row", maxWidth: "1200px", width: "100%" }}>
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

                            <label htmlFor="auto-optimization" style={{ color: 'var(--text-primary)', fontSize: 'var(--font-size-sm)', whiteSpace: 'nowrap' }}>
                                Guesstimate Juice & Special values<br />

                            </label>
                            <input
                                type="checkbox"
                                id="auto-optimization"
                                checked={autoOptimization}
                                onChange={(e) => setAutoOptimization(e.target.checked)}
                                style={{ accentColor: 'var(--control-checked-bg)' }}
                            />
                        </div>

                        <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 8 }}>
                            <div style={{ ...styles.inputLabelCell, whiteSpace: 'nowrap', backgroundColor: 'var(--bg-tertiary)' }}>Chance of Success</div>
                            <div style={{ ...styles.inputCell, border: 'none', background: "transparent", color: 'var(--text-success)', fontSize: 'var(--font-size-xl)' }}>{chance_result ? (String(chance_result.chance) + '%') : '-'}</div>

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
                        {(chance_result) && (//&& (chance_result.reasons?.length > 0 || chance_result.upgrade_strings?.length > 0 || chance_result.juice_strings_armor?.length > 0 || chance_result.juice_strings_weapon?.length > 0)) && (
                            <div style={{ display: 'flex', gap: 0, alignItems: 'flex-start', marginTop: 8 }}>
                                <div>
                                    <div style={{ ...styles.inputLabelCell, marginTop: 0, whiteSpace: 'nowrap', textAlign: "left", }}>Individual chances:</div>
                                    <div style={{ marginTop: 4, color: 'var(--text-muted)', fontSize: 'var(--font-size-sm)', whiteSpace: "wrap", width: 300 }}>
                                        {(chance_result.reasons || []).map((s: string, idx: number) => (
                                            <div key={"Fail reason" + (idx + 1)}>{idx + 1}. {s}</div>
                                        ))}
                                    </div>
                                    {/* <div style={{ ...styles.inputLabelCell, textAlign: "left", fontSize: 'var(--font-size-sm)', whiteSpace: "wrap", width: 350 }}>
                                        Note: These are not what you will run out *first* - Imagine the game lets you keep tapping & go into debt, these are the chances that you will end up with a negative.(I know this is less intuitive, but it is much harder to calculate what you run out *first*)</div> */}

                                </div>
                                <div>
                                    <div style={{ ...styles.inputLabelCell, whiteSpace: 'nowrap' }}>Free taps value ranking:</div>
                                    <div style={{ marginTop: 4, color: 'var(--text-muted)', fontSize: 'var(--font-size-sm)', width: 200 }}>
                                        {(chance_result.upgrade_strings || []).map((upgrade: string, index: number) => (
                                            <div key={"Free tap value" + (index + 1)}>{index + 1}. {upgrade}</div>
                                        ))}
                                    </div>
                                </div>
                                <div>
                                    <div style={{ ...styles.inputLabelCell, whiteSpace: 'nowrap' }}>Red juice (weapon):</div>
                                    <div style={{ marginTop: 4, color: 'var(--text-muted)', fontSize: 'var(--font-size-sm)', width: 280 }}>
                                        {(chance_result.juice_strings_weapon || []).map((s: string, idx: number) => (
                                            <div key={"Red juice value" + (idx + 1)}>{idx + 1}. {s}</div>
                                        ))}
                                    </div>
                                </div>
                                <div>
                                    <div style={{ ...styles.inputLabelCell, whiteSpace: 'nowrap' }}>Blue juice (armor):</div>
                                    <div style={{ marginTop: 4, color: 'var(--text-muted)', fontSize: 'var(--font-size-sm)', width: 400 }}>
                                        {(chance_result.juice_strings_armor || []).map((s: string, idx: number) => (
                                            <div key={"Blue juice value" + (idx + 1)}>{idx + 1}. {s}</div>
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
                        />
                    </div>
                </div>
            </div >
        </>
    )
}

