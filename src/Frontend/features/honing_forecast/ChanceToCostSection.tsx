import React from 'react'
import { Slider, styled } from '@mui/material'
import SpreadsheetGrid from '../../components/SpreadsheetGrid.tsx'
import Graph from '../../components/Graph.tsx'
import { styles, createColumnDefs, GRAPH_WIDTH, GRAPH_HEIGHT } from './styles.ts'
import { OUTPUT_LABELS } from './constants.ts'
import { CallWorker } from '../../worker_setup.ts'

// Styled Material UI Slider with custom colors
const StyledSlider = styled(Slider)(() => ({
    width: 300,
    color: 'var(--slider-track-active)',
    '& .MuiSlider-track': {
        border: 'none',
        backgroundColor: 'var(--slider-track-active)',
        height: 6,
    },
    '& .MuiSlider-rail': {
        backgroundColor: 'var(--slider-track-bg)',
        height: 6,
    },
    '& .MuiSlider-thumb': {
        backgroundColor: 'var(--slider-thumb-bg)',
        border: '2px solid var(--slider-thumb-bg)',
        width: 20,
        height: 20,
        '&:hover, &.Mui-focusVisible': {
            backgroundColor: 'var(--slider-thumb-hover)',
            borderColor: 'var(--slider-thumb-focus)',
            boxShadow: `0 0 0 8px var(--slider-thumb-shadow)`,
        },
        '&.Mui-active': {
            backgroundColor: 'var(--slider-thumb-hover)',
            borderColor: 'var(--slider-thumb-focus)',
        },
    },
    '& .MuiSlider-valueLabel': {
        backgroundColor: 'var(--slider-thumb-bg)',
        color: 'var(--text-primary)',
        fontSize: '12px',
        fontWeight: 'bold',
    },
}))

type ChanceToCostSectionProps = {
    desired_chance: string
    uncleaned_desired_chance: string
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
    // Show Average checkbox props
    showAverage: boolean
    setShowAverage: React.Dispatch<React.SetStateAction<boolean>>
    // Moved worker call results
    averageCosts: number[] | null
    AverageCostBusy: boolean
}

export default function ChanceToCostSection({
    desired_chance,
    uncleaned_desired_chance,
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
    // Show Average checkbox props
    showAverage,
    setShowAverage,
    // Moved worker call results
    averageCosts,
    AverageCostBusy: _AverageCostBusy,
}: ChanceToCostSectionProps) {
    const { chanceToCostColumnDefs } = createColumnDefs(false) // autoOptimization not used for this section

    return (
        <>
            {/* <h3 style={{ color: 'var(--text-primary)', fontSize: 'var(--font-size-base)', fontWeight: 'var(--font-weight-semibold)', margin: '0 0 -8px 0' }}>Chance to Cost</h3> */}
            <div style={{ ...styles.inputSection, maxWidth: "1200px", width: "100%" }}>
                <div style={{ display: 'flex', gap: 8, alignItems: 'center', marginBottom: 16 }}>
                    <div style={{ width: 160, fontWeight: 700, textAlign: 'right', paddingRight: 8, color: 'var(--text-primary)' }}>Desired chance</div>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
                        <StyledSlider
                            value={parseInt(desired_chance) || 0}
                            onChange={(_, value) => {
                                const intValue = Math.round(value as number)
                                onDesiredChange(intValue.toString())
                            }}
                            min={0}
                            max={100}
                            step={1}
                            valueLabelDisplay="off"
                        />
                        <div style={{ position: 'relative', display: 'flex', alignItems: 'center' }}>
                            <input
                                type="text"
                                value={uncleaned_desired_chance}
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
                </div>

                <div style={{ display: 'flex', gap: 110, alignItems: 'flex-start' }}>
                    <div>
                        <div style={{ marginBottom: 16, width: 210 }}>
                            <SpreadsheetGrid
                                columnDefs={chanceToCostColumnDefs}
                                labels={OUTPUT_LABELS}
                                sheet_values={cost_result ?
                                    Object.fromEntries(OUTPUT_LABELS.map((label, lab_index) =>
                                        [label, String(cost_result.hundred_budgets[parseInt(desired_chance)][lab_index])])) :
                                    Object.fromEntries(OUTPUT_LABELS.map(label => [label, 'No results yet']))}
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
                                Show Average(Maxroll)
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
                                Run time: {cost_result.run_time}s{'\nActual chance: '}{cost_result.hundred_chances[parseInt(desired_chance)].toFixed(2)}%
                            </pre>
                        )}
                    </div>
                    <div style={{ flex: 1 }}>
                        <Graph
                            title={cumulativeGraph ? "Cost distribution(Cumulative)" : "Cost distribution"}
                            labels={OUTPUT_LABELS}
                            counts={AnythingTicked ? (cost_result?.hist_counts || cachedCostGraphData?.hist_counts) : null}
                            mins={cost_result?.hist_mins || cachedCostGraphData?.hist_mins}
                            maxs={cost_result?.hist_maxs || cachedCostGraphData?.hist_maxs}
                            width={GRAPH_WIDTH}
                            height={GRAPH_HEIGHT}
                            budgets={cost_result && OUTPUT_LABELS.map((_, lab_index) =>
                                Number(cost_result.hundred_budgets[parseInt(desired_chance)][lab_index]))}
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

