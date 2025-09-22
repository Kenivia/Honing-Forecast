import React from 'react'
import SpreadsheetGrid from '../../components/SpreadsheetGrid.tsx'
import Graph from '../../components/Graph.tsx'
import { styles, createColumnDefs } from './styles.ts'
import { OUTPUT_LABELS } from './constants.ts'

type ChanceToCostSectionProps = {
    desired_chance: string
    onDesiredChange: (_: string) => void
    cost_result: any
    cachedCostGraphData: { hist_counts?: any, hist_mins?: any, hist_maxs?: any } | null
    AnythingTicked: boolean
    ChanceToCostBusy: boolean
    cumulativeGraph: boolean
}

export default function ChanceToCostSection({
    desired_chance,
    onDesiredChange,
    cost_result,
    cachedCostGraphData,
    AnythingTicked,
    ChanceToCostBusy,
    cumulativeGraph,
}: ChanceToCostSectionProps) {
    const { chanceToCostColumnDefs } = createColumnDefs(false) // autoOptimization not used for this section

    return (
        <>
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
                            counts={AnythingTicked ? (cost_result?.hist_counts || cachedCostGraphData?.hist_counts) : null}
                            mins={cost_result?.hist_mins || cachedCostGraphData?.hist_mins}
                            maxs={cost_result?.hist_maxs || cachedCostGraphData?.hist_maxs}
                            width={640}
                            height={320}
                            budgets={cost_result && OUTPUT_LABELS.map(label => Number(cost_result[label]))}
                            hasSelection={AnythingTicked}
                            isLoading={ChanceToCostBusy}
                            cumulative={cumulativeGraph}
                        />
                    </div>
                </div>
            </div>
        </>
    )
}

