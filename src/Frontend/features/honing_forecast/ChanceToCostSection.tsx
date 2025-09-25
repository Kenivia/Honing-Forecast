import React, { useState, useEffect } from 'react'
import SpreadsheetGrid from '../../components/SpreadsheetGrid.tsx'
import Graph from '../../components/Graph.tsx'
import { styles, createColumnDefs, GRAPH_WIDTH, GRAPH_HEIGHT } from './styles.ts'
import { OUTPUT_LABELS } from './constants.ts'
import { CallWorker } from '../../worker_setup.ts'
import { buildPayload } from './Debounce.ts'

type ChanceToCostSectionProps = {
    desired_chance: string
    onDesiredChange: (_: string) => void
    onDesiredBlur: () => void
    cost_result: any
    cachedCostGraphData: { hist_counts?: any, hist_mins?: any, hist_maxs?: any } | null
    AnythingTicked: boolean
    ChanceToCostBusy: boolean
    cumulativeGraph: boolean
    lockXAxis: boolean
    lockedMins: number[] | null
    lockedMaxs: number[] | null
    // Props needed for average_cost calculation
    topGrid: boolean[][]
    bottomGrid: boolean[][]
    adv_hone_strategy: string
    express_event: boolean
    bucketCount: string
    autoOptimization: boolean
    userMatsValue: any
    dataSize: string
    // Show Average checkbox props
    showAverage: boolean
    setShowAverage: React.Dispatch<React.SetStateAction<boolean>>
    // New props for numeric input mode
    useGridInput: boolean
    normalCounts: number[][]
    advCounts: number[][]
}

export default function ChanceToCostSection({
    desired_chance,
    onDesiredChange,
    onDesiredBlur,
    cost_result,
    cachedCostGraphData,
    AnythingTicked,
    ChanceToCostBusy,
    cumulativeGraph,
    lockXAxis,
    lockedMins,
    lockedMaxs,
    // Props needed for average_cost calculation
    topGrid,
    bottomGrid,
    adv_hone_strategy,
    express_event,
    bucketCount,
    autoOptimization,
    userMatsValue,
    dataSize,
    // Show Average checkbox props
    showAverage,
    setShowAverage,
    // New props for numeric input mode
    useGridInput,
    normalCounts,
    advCounts,
}: ChanceToCostSectionProps) {
    const { chanceToCostColumnDefs } = createColumnDefs(false) // autoOptimization not used for this section
    const [averageCosts, setAverageCosts] = useState<number[] | null>(null)

    // Effect to calculate average costs when relevant data changes
    useEffect(() => {
        if (!AnythingTicked) {
            setAverageCosts(null)
            return
        }

        const payload = buildPayload({
            topGrid,
            bottomGrid,
            desired_chance,
            budget_inputs: {}, // Not needed for average_cost
            adv_hone_strategy,
            express_event,
            bucketCount,
            autoOptimization,
            userMatsValue,
            dataSize,
            useGridInput,
            normalCounts,
            advCounts,
        })

        CallWorker(payload, "AverageCost")
            .then((result: any) => {
                if (result && result.average_costs) {
                    setAverageCosts(result.average_costs)
                }
            })
            .catch((error) => {
                console.error("Error calculating average costs:", error)
                setAverageCosts(null)
            })
    }, [topGrid, bottomGrid, adv_hone_strategy, express_event, AnythingTicked, autoOptimization, bucketCount, dataSize, desired_chance, userMatsValue, useGridInput, normalCounts, advCounts])

    return (
        <>
            {/* <h3 style={{ color: 'var(--text-primary)', fontSize: 'var(--font-size-base)', fontWeight: 'var(--font-weight-semibold)', margin: '0 0 -8px 0' }}>Chance to Cost</h3> */}
            <div style={{ ...styles.inputSection, maxWidth: "1200px", width: "100%" }}>
                <div style={{ display: 'flex', gap: 8, alignItems: 'center', marginBottom: 16 }}>
                    <div style={{ width: 160, fontWeight: 700, textAlign: 'right', paddingRight: 8, color: 'var(--text-primary)' }}>Desired chance</div>
                    <div style={{ position: 'relative', display: 'flex', alignItems: 'center' }}>
                        <input
                            type="text"
                            value={desired_chance}
                            onChange={(e) => onDesiredChange(e.target.value)}
                            onBlur={onDesiredBlur}
                            placeholder="0"
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
                        <span style={{ position: 'absolute', right: 10, pointerEvents: 'none', color: "black" }}>%</span>
                    </div>
                </div>

                <div style={{ display: 'flex', gap: 110, alignItems: 'flex-start' }}>
                    <div>
                        <div style={{ marginBottom: 16, width: 210 }}>
                            <SpreadsheetGrid
                                columnDefs={chanceToCostColumnDefs}
                                labels={OUTPUT_LABELS}
                                sheet_values={cost_result ? Object.fromEntries(OUTPUT_LABELS.map(label => [label, cost_result[label] != null ? String(cost_result[label]) : 'No results yet'])) : Object.fromEntries(OUTPUT_LABELS.map(label => [label, 'No results yet']))}
                                set_sheet_values={() => { }} // No-op for read-only
                                readOnly={true}
                            />
                        </div>
                        <div style={{ marginTop: 12, display: 'flex', alignItems: 'center', gap: 8 }}>

                            <label
                                htmlFor="show-average"
                                style={{
                                    color: 'var(--text-primary)',
                                    fontSize: 'var(--font-size-sm)',

                                    cursor: 'pointer',
                                    userSelect: 'none'
                                }}
                            >
                                Show Average
                            </label>
                            <input
                                type="checkbox"
                                id="show-average"
                                checked={showAverage}
                                onChange={(e) => setShowAverage(e.target.checked)}
                                style={{
                                    width: 16,
                                    height: 16,
                                    cursor: 'pointer',
                                    accentColor: "var(--bright-green)"
                                }}
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
                            counts={AnythingTicked ? (cost_result?.hist_counts || cachedCostGraphData?.hist_counts) : null}
                            mins={cost_result?.hist_mins || cachedCostGraphData?.hist_mins}
                            maxs={cost_result?.hist_maxs || cachedCostGraphData?.hist_maxs}
                            width={GRAPH_WIDTH}
                            height={GRAPH_HEIGHT}
                            budgets={cost_result && OUTPUT_LABELS.map(label => Number(cost_result[label]))}
                            additionalBudgets={showAverage ? averageCosts : null}
                            hasSelection={AnythingTicked}
                            isLoading={ChanceToCostBusy}
                            cumulative={cumulativeGraph}
                            lockXAxis={lockXAxis}
                            lockedMins={lockedMins}
                            lockedMaxs={lockedMaxs}
                        />
                    </div>
                </div>
            </div>
        </>
    )
}

