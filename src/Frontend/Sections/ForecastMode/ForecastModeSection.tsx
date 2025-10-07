import React, { useState, useMemo, useEffect, useRef, useCallback } from 'react'
import { Slider, styled } from '@mui/material'
import SpreadsheetGrid from '../../Components/SpreadsheetGrid.tsx'
import Graph from '../../Components/Graph.tsx'
import { styles, SMALL_GRAPH_WIDTH, SMALL_GRAPH_HEIGHT, GRAPH_HEIGHT, GRAPH_WIDTH } from '../../Utils/Styles.ts'
import { INPUT_LABELS } from '../../Utils/Constants.ts'
import { SpawnWorker } from '../../WasmInterface/worker_setup.ts'
import { buildPayload } from '../../WasmInterface/WorkerRunner.ts'

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

type LongTermSectionProps = {
    budget_inputs: any
    set_budget_inputs: React.Dispatch<React.SetStateAction<any>>
    userMatsValue: any
    setUserMatsValue: React.Dispatch<React.SetStateAction<any>>
    topGrid: any
    bottomGrid: any
    adv_hone_strategy: any
    express_event: any
    bucketCount: any
    autoOptimization: any
    dataSize: any
    useGridInput: any
    normalCounts: any
    advCounts: any
    incomeArr: number[][]
    setIncomeArr: React.Dispatch<React.SetStateAction<number[][]>>
    // Desired chance props
    desired_chance: string
    uncleaned_desired_chance: string
    onDesiredChange: (_value: string) => void
    onDesiredBlur: () => void
    // Cost result prop for hundred_budgets
    cost_result: any
}
function cost_to_pity(budget: number[], pity_cost: number[], mat_value: number[]): number {
    let sum = 0;
    for (let i = 0; i < 7; i++) {
        if (i == 5) { sum += Math.max(0, pity_cost[i]) * mat_value[i] }
        else {
            sum += Math.max(0, pity_cost[i] - budget[i]) * mat_value[i];
        }
    }
    return sum;
}

function gold_plus_sell_mats(budget: number[], pity_cost: number[], mat_value: number[]): number {
    let sum = budget[5];
    for (let i = 0; i < 7; i++) {
        if (i == 5 || i == 3 || i == 6) { continue; }
        sum += Math.max(0, (budget[i] - pity_cost[i])) * mat_value[i] * 0.95;
    }
    return sum;
}

function cost_to_pity_individual(budget: number[], pity_cost: number[], mat_value: number[], materialIndex: number): number {
    if (materialIndex == 5) { return pity_cost[materialIndex] * mat_value[materialIndex] };
    return Math.max(0, pity_cost[materialIndex] - budget[materialIndex]) * mat_value[materialIndex];
}

function weekly_budget(budget: number[], weekly_income: number[]): number[][] {
    let current = budget;
    let out = [];
    for (let i = 0; i < 53; i++) {
        out.push(current.slice());
        for (let z = 0; z < 7; z++) {
            current[z] += weekly_income[z];
        }


    }
    return out

}
export default function LongTermSection({
    budget_inputs,
    set_budget_inputs,
    userMatsValue,
    setUserMatsValue,
    topGrid,
    bottomGrid,
    adv_hone_strategy,
    express_event,
    bucketCount,
    autoOptimization,
    dataSize,
    useGridInput,
    normalCounts,
    advCounts,
    incomeArr,
    setIncomeArr,
    // Desired chance props
    desired_chance,
    uncleaned_desired_chance,
    onDesiredChange,
    onDesiredBlur,
    // Cost result prop for hundred_budgets
    cost_result,
}: LongTermSectionProps) {
    // Income array is now managed by parent component

    // State for cost_to_chance_arr results
    const [longTermResult, setLongTermResult] = useState<any>(null)
    const [longTermBusy, setLongTermBusy] = useState<boolean>(false)

    // State for pity costs and material values
    const [matValues, setMatValues] = useState<number[]>(() =>
        Object.values(userMatsValue).map(v => parseFloat(v.toString()) || 0)
    )

    // Worker refs and debounce
    const longTermWorkerRef = useRef<Worker | null>(null)
    const debounceTimerRef = useRef<number | null>(null)

    // Function to get pity costs using average_cost_wrapper


    // Update material values when userMatsValue changes
    useEffect(() => {
        setMatValues(Object.values(userMatsValue).map(v => parseFloat(v.toString()) || 0))
    }, [userMatsValue])

    // Income array is now managed by parent component, no local effects needed

    // Column definitions for the long term section - always shows both budget and gold value columns
    const longTermColumnDefs = [
        {
            headerName: "Budget Input",
            field: "budget",
            editable: true,
            flex: 1,
            cellStyle: { background: "var(--grid-cell-bg)", padding: "6px 8px" }
        },
        {
            headerName: "Gold Value",
            field: "matsValue",
            editable: true,
            flex: 1,
            cellStyle: { background: "var(--grid-cell-bg)", padding: "6px 8px" }
        }
    ]

    // Column definitions for income grids (1 column each)
    const incomeColumnDefs = [
        {
            headerName: "Income",
            field: "income",
            editable: true,
            flex: 1,
            cellStyle: { background: "var(--grid-cell-bg)", padding: "6px 8px" }
        }
    ]

    // Column definitions for total weekly income grid (read-only)
    const totalIncomeColumnDefs = [
        {
            headerName: "Total weekly",
            field: "total",
            editable: false,
            flex: 1,
            cellStyle: {
                background: "var(--background-secondary)",
                color: "var(--text-primary)",
                padding: "6px 8px"
            }
        }
    ]

    // Calculate total weekly income (sum of all income grids for each row)
    const totalWeeklyIncome = useMemo(() => {
        return Array.from({ length: 7 }, (_, rowIndex) =>
            incomeArr.reduce((sum, incomeGrid) => sum + (incomeGrid[rowIndex] || 0), 0)
        )
    }, [incomeArr])

    // Debounce keys for cost_to_chance_arr
    const advStrategyKey = useMemo(() => String(adv_hone_strategy), [adv_hone_strategy])
    const expressEventKey = useMemo(() => String(express_event), [express_event])
    const useGridInputKey = useMemo(() => String(useGridInput), [useGridInput])
    const normalCountsKey = useMemo(() => JSON.stringify(normalCounts), [normalCounts])
    const advCountsKey = useMemo(() => JSON.stringify(advCounts), [advCounts])
    const totalWeeklyIncomeKey = useMemo(() => JSON.stringify(totalWeeklyIncome), [totalWeeklyIncome])

    // Function to call cost_to_chance_arr
    const callCostToChanceArr = useCallback(async () => {
        if (!longTermWorkerRef.current) return

        // Create weekly budget array (0-52 weeks)
        const weeklyBudgets = weekly_budget(
            Object.values(budget_inputs).map(v => Math.round(Number(v))),
            totalWeeklyIncome
        )

        // Create payload using the same format as other functions
        const payload = buildPayload({
            topGrid,
            bottomGrid,
            budget_inputs,
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

        // Replace the single budget with the weekly budgets array
        payload.budget_arr = weeklyBudgets

        const id = Math.random().toString(36).substr(2, 9)

        // Create promise for this request
        const p = new Promise(resolve => {
            const handler = (e: MessageEvent) => {
                const { id: responseId, type, result } = e.data
                if (responseId === id && type === 'result') {
                    longTermWorkerRef.current?.removeEventListener('message', handler)
                    resolve(result)
                }
            }
            longTermWorkerRef.current?.addEventListener('message', handler)
        })

        longTermWorkerRef.current.postMessage({
            id,
            payload,
            which_one: "CostToChanceArr"
        })

        p.then((result: any) => {
            // console.log(result, weeklyBudgets, budget_inputs, totalWeeklyIncome)
            setLongTermResult(result)
        }).catch((err) => {
            console.error("Long term worker error", err)
            setLongTermResult(null)
        })
    }, [topGrid, bottomGrid, adv_hone_strategy, express_event, bucketCount, autoOptimization, userMatsValue, dataSize, budget_inputs, useGridInput, normalCounts, advCounts, totalWeeklyIncome])

    // Initialize worker
    useEffect(() => {
        longTermWorkerRef.current = new Worker(new URL("../../js_to_wasm.ts", import.meta.url), { type: "module" })
        return () => {
            if (longTermWorkerRef.current) {
                longTermWorkerRef.current.terminate()
            }
        }
    }, [])

    // Debounce effect for cost_to_chance_arr calls
    useEffect(() => {
        // Clear existing timer
        if (debounceTimerRef.current) {
            window.clearTimeout(debounceTimerRef.current)
            debounceTimerRef.current = null
        }

        // Start new delayed work
        debounceTimerRef.current = window.setTimeout(() => {
            setLongTermBusy(true)
            Promise.all([
                callCostToChanceArr(),
            ]).finally(() => {
                setLongTermBusy(false)
            })
            debounceTimerRef.current = null
        }, 100) // 100ms debounce

        return () => {
            if (debounceTimerRef.current) {
                window.clearTimeout(debounceTimerRef.current)
                debounceTimerRef.current = null
            }
        }
    }, [advStrategyKey, expressEventKey, useGridInputKey, normalCountsKey, advCountsKey, totalWeeklyIncomeKey, callCostToChanceArr])

    // Labels for income grids (7 rows) - use proper labels but hide icons
    const incomeLabels = INPUT_LABELS.slice(0, 7)


    // Convert income grid to SpreadsheetGrid format
    const getIncomeGridData = (gridIndex: number) => {
        return incomeLabels.reduce((acc, label, rowIndex) => {
            acc[label] = incomeArr[gridIndex][rowIndex].toString()
            return acc
        }, {} as Record<string, string>)
    }

    // Convert total income to SpreadsheetGrid format
    const totalIncomeData = incomeLabels.reduce((acc, label, rowIndex) => {
        acc[label] = totalWeeklyIncome[rowIndex].toString()
        return acc
    }, {} as Record<string, string>)

    // Transform longTermResult data for Graph component
    const graphData = useMemo(() => {
        if (!longTermResult || !longTermResult.final_chances) return null

        // const chances = longTermResult.final_chances.map((chance: string) => parseFloat(chance))

        // Create histogram-like data for Graph component
        // We need to create counts array where each week (0-52) represents a "bucket"
        const numWeeks = 53 // 0 to 52 inclusive

        // --- Build overallCounts first (so truncation is driven by overall) ---
        const overallCounts = new Array(numWeeks).fill(0)
        for (let week = 0; week < numWeeks; week++) {
            if (week < longTermResult.final_chances.length) {
                // normalize percent -> fraction
                overallCounts[week] = parseFloat(longTermResult.final_chances[week]) / 100
            }
        }

        // Determine truncation length:
        // If overallCounts stops increasing for 4 consecutive weeks, cut off from the start of that run.
        // Only scan up to the weeks we actually have final_chances for (so trailing zero-filled weeks don't immediately trigger a cutoff).
        const availableWeeks = Math.min(numWeeks, longTermResult.final_chances.length)
        let truncateLen = numWeeks // default: don't truncate
        if (availableWeeks > 0) {
            const EPS = 1e-12
            let lastNonIncreaseStart: number | null = null

            for (let i = 1; i < availableWeeks; i++) {
                if (overallCounts[i] > overallCounts[i - 1] + EPS) {
                    // reset when we see an increase
                    lastNonIncreaseStart = null
                } else {
                    // if run not started, mark it
                    if (lastNonIncreaseStart === null) {
                        lastNonIncreaseStart = i + 1
                    }
                }
            }

            if (lastNonIncreaseStart !== null) {
                // truncate starting at the last non-increase run
                truncateLen = Math.max(6, lastNonIncreaseStart)
            }

            // If availableWeeks is smaller than numWeeks and no plateau found, truncate to availableWeeks
            if (truncateLen === numWeeks && availableWeeks < numWeeks) {
                truncateLen = availableWeeks
            }
        }

        // --- Build counts with overall first, then materials ---
        const counts: number[][] = []
        const mins: number[] = []
        const maxs: number[] = []

        // push overall first (truncated)
        counts.push(overallCounts.slice(0, truncateLen))
        mins.push(1)
        maxs.push(truncateLen)

        // For each material type (7 types), create a histogram truncated to truncateLen
        const numMaterials = 7
        for (let materialIndex = 0; materialIndex < numMaterials; materialIndex++) {
            const materialCounts = new Array(truncateLen).fill(0)

            for (let week = 0; week < truncateLen; week++) {
                // safety: check failure_rates_arr bounds
                if (
                    longTermResult.failure_rates_arr &&
                    week < longTermResult.failure_rates_arr.length &&
                    longTermResult.failure_rates_arr[week] &&
                    materialIndex < longTermResult.failure_rates_arr[week].length
                ) {
                    materialCounts[week] = 1 - longTermResult.failure_rates_arr[week][materialIndex]
                } else {
                    materialCounts[week] = 0
                }
            }

            counts.push(materialCounts)
            mins.push(1)
            maxs.push(truncateLen)
        }

        // debug
        // console.log('graphData: truncateLen=', truncateLen, 'counts lengths=', counts.map(c => c.length))

        return {
            counts,
            mins,
            maxs
        }
    }, [longTermResult])

    // Calculate data for the new graph (cost to pity and gold from selling materials)
    const newGraphData = useMemo(() => {
        if (!longTermResult) return null

        const weeklyBudgets = weekly_budget(
            Object.values(budget_inputs).map(v => Math.round(Number(v))),
            totalWeeklyIncome
        )

        const costToPityData: number[] = []
        const goldFromSellData: number[] = []
        const individualPityCostsData: number[][] = []

        // Initialize individual pity costs arrays
        for (let i = 0; i < 7; i++) {
            individualPityCostsData.push([])
        }

        const pityCost = cost_result?.hundred_budgets?.[parseInt(desired_chance)] || []
        // console.log("Pitycost", pityCost)
        for (let week = 0; week < Math.min(53, weeklyBudgets.length); week++) {
            const this_week_budget = weeklyBudgets[week]
            const costToPity = cost_to_pity(this_week_budget, pityCost, matValues)
            const goldFromSell = gold_plus_sell_mats(this_week_budget, pityCost, matValues)

            costToPityData.push(costToPity)
            goldFromSellData.push(goldFromSell)

            // Calculate individual pity costs for each material
            for (let materialIndex = 0; materialIndex < 7; materialIndex++) {
                const individualCost = cost_to_pity_individual(this_week_budget, pityCost, matValues, materialIndex)
                const clampedCost = Math.max(0, individualCost) // Clamp to 0 if negative
                individualPityCostsData[materialIndex].push(clampedCost)
            }
            // console.log("break", goldFromSell, costToPity)
            if (goldFromSell > costToPity && week > 2) { break }
        }
        // console.log(costToPityData,
        //     goldFromSellData, pityCost)
        return {
            costToPityData,
            goldFromSellData,
            individualPityCostsData,
            weeklyBudgets
        }
    }, [longTermResult, budget_inputs, totalWeeklyIncome, matValues, cost_result, desired_chance])


    return (
        <>

            <div style={{ ...styles.inputSection, flexDirection: "row", maxWidth: "1200px", width: "100%" }}>
                <div style={{ display: 'flex', gap: 20, alignItems: 'center', flexDirection: "column" }}>
                    <div style={{ display: 'flex', gap: 5, alignItems: 'flex-start', alignSelf: "flex-start" }}>
                        {/* Main Budget Input Grid */}
                        <div style={{ display: 'flex', flexDirection: "column", gap: 0, alignItems: 'flex-start', justifyContent: 'start', width: 300 }}>
                            <div style={{ marginBottom: 16, width: 300 }}>
                                <SpreadsheetGrid
                                    columnDefs={longTermColumnDefs}
                                    labels={INPUT_LABELS}
                                    sheet_values={budget_inputs}
                                    set_sheet_values={set_budget_inputs}
                                    secondaryValues={userMatsValue}
                                    setSecondaryValues={setUserMatsValue}
                                />
                            </div>
                        </div>

                        {/* 6 Income Grids */}
                        {Array.from({ length: 6 }, (_, gridIndex) => (
                            <div key={`income-${gridIndex}`} style={{ display: 'flex', flexDirection: "column", gap: 0, alignItems: 'flex-start', justifyContent: 'start', width: 120 }}>
                                {/* <h4 style={{
                                color: 'var(--text-primary)',
                                fontSize: 'var(--font-size-sm)',
                                fontWeight: 'var(--font-weight-medium)',
                                margin: '0 0 8px 0',
                                textAlign: 'center',
                                width: '100%'
                            }}>
                                Income {gridIndex + 1}
                            </h4> */}
                                <div style={{ width: 120 }}>
                                    <SpreadsheetGrid
                                        columnDefs={[{ ...incomeColumnDefs[0], headerName: "Income " + (gridIndex + 1) }]}
                                        labels={incomeLabels}
                                        sheet_values={getIncomeGridData(gridIndex)}
                                        set_sheet_values={(newValues) => {
                                            // Update the specific income grid
                                            setIncomeArr(prev => {
                                                const newArr = prev.map(grid => [...grid])
                                                incomeLabels.forEach((label, rowIndex) => {
                                                    const value = newValues[label] || '0'
                                                    const cleanValue = value.replace(/[^0-9.-]/g, '')
                                                    newArr[gridIndex][rowIndex] = parseFloat(cleanValue) || 0
                                                })
                                                return newArr
                                            })
                                        }}
                                        hideIcons={true}
                                    />
                                </div>
                            </div>
                        ))}

                        {/* Total Weekly Income Grid */}
                        <div style={{ display: 'flex', flexDirection: "column", gap: 0, alignItems: 'flex-start', justifyContent: 'start', width: 10 }}>

                            <div style={{ width: 120 }}>
                                <SpreadsheetGrid
                                    columnDefs={totalIncomeColumnDefs}
                                    labels={incomeLabels}
                                    sheet_values={totalIncomeData}
                                    set_sheet_values={() => { }} // Read-only
                                    readOnly={true}
                                    hideIcons={true}
                                />
                            </div>
                        </div>
                    </div>
                    <div style={{ marginTop: 20, display: 'flex', justifyContent: 'center' }}>
                        <div style={{ width: '100%', maxWidth: '800px' }}>
                            <Graph
                                title={`Chance of Success Over Time (0-${(graphData?.counts?.[0]?.length || 53) - 1} weeks from now)`}
                                labels={["Overall", ...INPUT_LABELS.slice(0, 7).map(label => label)]} // Overall + 7 material types
                                counts={graphData?.counts || null}
                                mins={graphData?.mins || null}
                                maxs={graphData?.maxs || null}
                                width={GRAPH_WIDTH}
                                height={GRAPH_HEIGHT}
                                hasSelection={true}
                                isLoading={longTermBusy}
                                cumulative={false} // Show actual chance values, not cumulative
                                graphType="Raw" // Use Raw mode for this graph
                                xAxisLabel="Week number"
                                yAxisLabel="Chance of success"
                                yMaxOverride={1} // Override yMax to 1 for the first graph
                                customColors={['var(--series-overall)', 'var(--series-red)', 'var(--series-blue)', 'var(--series-leaps)', 'var(--series-shards)', 'var(--series-oreha)', 'var(--series-gold)', 'var(--series-silver)']}
                                showWarning={totalWeeklyIncome.every(income => income === 0)}
                                warningMessage="You are earning nothing per week. Input your income (or click Fill Demo Income) to see your chances in the future."
                            />
                        </div>
                    </div>

                    {/* Desired Chance Slider */}
                    <div style={{ marginTop: 20, display: 'flex', justifyContent: 'center', justifySelf: "left", alignSelf: "flex-start", marginLeft: 100 }}>
                        <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
                            <div style={{ width: 160, fontWeight: 700, textAlign: 'right', paddingRight: 8, color: 'var(--text-primary)', textWrap: "nowrap" }}>Desired chance </div>
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
                                {desired_chance === "0" && (
                                    <span style={{
                                        color: 'var(--text-muted)',
                                        fontSize: 'var(--font-size-sm)',
                                        fontStyle: 'italic'
                                    }}>
                                        0% = luckiest draw in {dataSize} samples
                                    </span>
                                )}
                            </div>
                        </div>
                    </div>

                    {/* New Graph for Cost to Pity and Gold from Selling Materials */}
                    {newGraphData && (
                        <div style={{ marginTop: 20, display: 'flex', justifyContent: 'center' }}>
                            <div style={{ width: '100%', maxWidth: '800px' }}>
                                <Graph
                                    title={`Gold needed to achieve ${parseInt(desired_chance) === 100 ? 'pity' : `${desired_chance}% chance to pass(pessimistic)`} (0-${newGraphData.costToPityData.length - 1} weeks from now)`}
                                    labels={["Cost to Pity", "Gold if sell mats", ...INPUT_LABELS.slice(0, 7)]}
                                    counts={[newGraphData.costToPityData, newGraphData.goldFromSellData, ...newGraphData.individualPityCostsData]}
                                    mins={[0, 0, ...newGraphData.individualPityCostsData.map(() => 0)]}
                                    maxs={[newGraphData.costToPityData.length - 1, newGraphData.goldFromSellData.length - 1, ...newGraphData.individualPityCostsData.map(data => data.length - 1)]}
                                    width={GRAPH_WIDTH}
                                    height={GRAPH_HEIGHT}
                                    hasSelection={true}
                                    isLoading={longTermBusy}
                                    cumulative={false}
                                    graphType="Gold"
                                    xAxisLabel="Week number"
                                    yAxisLabel="Gold value"
                                    // yMaxOverride={newGraphData.costToPityData[0]}
                                    customColors={['var(--series-brown)', 'var(--series-gold)', 'var(--series-red)', 'var(--series-blue)', 'var(--series-leaps)', 'var(--series-shards)', 'var(--series-oreha)', 'var(--series-gold)', 'var(--series-silver)']}
                                    weeklyBudgets={newGraphData.weeklyBudgets}
                                    showWarning={totalWeeklyIncome.every(income => income === 0)}
                                    warningMessage="You are earning nothing per week. Input your income (or click Fill Demo Income) to see your chances in the future."
                                />
                            </div>
                        </div>
                    )}
                </div>
            </div >



        </>
    )
}
