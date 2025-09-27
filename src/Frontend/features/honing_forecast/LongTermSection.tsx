import React, { useState, useMemo, useEffect, useRef, useCallback } from 'react'
import SpreadsheetGrid from '../../components/SpreadsheetGrid.tsx'
import Graph from '../../components/Graph.tsx'
import { styles, SMALL_GRAPH_WIDTH, SMALL_GRAPH_HEIGHT, GRAPH_HEIGHT, GRAPH_WIDTH } from './styles.ts'
import { INPUT_LABELS } from './constants.ts'
import { SpawnWorker } from '../../worker_setup.ts'
import { buildPayload } from './Debounce.ts'

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
}
function cost_to_pity(budget: number[], pity_cost: number[], mat_value: number[]): number {
    let sum = 0;
    for (let i = 0; i < 7; i++) {
        sum += (pity_cost[i] - budget[i]) * mat_value[i];
    }
    return sum;
}

function gold_from_sell_mats(budget: number[], pity_cost: number[], mat_value: number[]): number {
    let sum = budget[5];
    for (let i = 0; i < 7; i++) {
        sum += Math.max(0, (budget[i] - pity_cost[i])) * Math.floor(mat_value[i] * 0.95);
    }
    return sum;
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
}: LongTermSectionProps) {
    // State for 6 income grids (7 rows each)
    const [Income_arr, setIncome_arr] = useState<number[][]>(() => {
        // Try to load from localStorage first
        const saved = localStorage.getItem('honing_forecast_income_arr')
        if (saved) {
            try {
                const parsed = JSON.parse(saved)
                if (Array.isArray(parsed) && parsed.length === 6 && parsed.every(row => Array.isArray(row) && row.length === 7)) {
                    return parsed
                }
            } catch (e) {
                // If parsing fails, fall back to default
            }
        }
        return Array.from({ length: 6 }, () => Array.from({ length: 7 }, () => 0))
    })

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

    // Save income array to localStorage whenever it changes
    useEffect(() => {
        localStorage.setItem('honing_forecast_income_arr', JSON.stringify(Income_arr))
    }, [Income_arr])

    // Listen for income array updates from fillDemo
    useEffect(() => {
        const handleIncomeArrayUpdate = (event: CustomEvent) => {
            setIncome_arr(event.detail)
        }

        window.addEventListener('incomeArrayUpdated', handleIncomeArrayUpdate as EventListener)

        return () => {
            window.removeEventListener('incomeArrayUpdated', handleIncomeArrayUpdate as EventListener)
        }
    }, [])

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
            Income_arr.reduce((sum, incomeGrid) => sum + (incomeGrid[rowIndex] || 0), 0)
        )
    }, [Income_arr])

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
            console.log(result, weeklyBudgets, budget_inputs, totalWeeklyIncome)
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
            acc[label] = Income_arr[gridIndex][rowIndex].toString()
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
            let nonIncreaseCount = 0
            const EPS = 1e-12
            for (let i = 1; i < availableWeeks; i++) {
                if (overallCounts[i] > overallCounts[i - 1] + EPS) {
                    // we saw an increase — reset counter
                    nonIncreaseCount = 0
                } else {
                    nonIncreaseCount++
                }

                if (nonIncreaseCount >= 1) {
                    // the non-increasing run started at:
                    const startOfRun = i + 1
                    // we will keep weeks [0 .. startOfRun-1] (i.e. truncate starting at startOfRun)
                    truncateLen = Math.max(6, startOfRun) // ensure at least 1 week remains
                    break
                }
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

        // push overall (truncated)


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
        counts.push(overallCounts.slice(0, truncateLen))
        mins.push(1)
        maxs.push(truncateLen)

        // debug
        console.log('graphData: truncateLen=', truncateLen, 'counts lengths=', counts.map(c => c.length))

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

        for (let week = 0; week < Math.min(53, weeklyBudgets.length); week++) {
            const budget = weeklyBudgets[week]
            const costToPity = cost_to_pity(budget, longTermResult.pity_cost, matValues)
            const goldFromSell = gold_from_sell_mats(budget, longTermResult.pity_cost, matValues)

            costToPityData.push(costToPity)
            goldFromSellData.push(goldFromSell)
            if (goldFromSell > costToPity) { break }
        }
        console.log(costToPityData,
            goldFromSellData, longTermResult.pity_cost)
        return {
            costToPityData,
            goldFromSellData
        }
    }, [longTermResult, budget_inputs, totalWeeklyIncome, matValues])


    return (
        <>

            <div style={{ ...styles.inputSection, flexDirection: "row", maxWidth: "1200px", width: "100%" }}>
                <div style={{ display: 'flex', gap: 20, alignItems: 'center', flexDirection: "column" }}>
                    <div style={{ display: 'flex', gap: 5, alignItems: 'flex-start' }}>
                        {/* Main Budget Input Grid */}
                        <div style={{ display: 'flex', flexDirection: "column", gap: 0, alignItems: 'flex-start', justifyContent: 'start', width: 300 }}>
                            <div style={{ marginBottom: 16, width: 310 }}>
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
                                        columnDefs={[{ ...incomeColumnDefs[0], headerName: "Income " + gridIndex }]}
                                        labels={incomeLabels}
                                        sheet_values={getIncomeGridData(gridIndex)}
                                        set_sheet_values={(newValues) => {
                                            // Update the specific income grid
                                            setIncome_arr(prev => {
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
                                title={`Chance of Success Over Time (Weeks 1-${graphData?.counts?.[0]?.length || 52})`}
                                labels={[...INPUT_LABELS.slice(0, 7), "Overall"]} // 7 material types + Overall
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
                            />
                        </div>
                    </div>

                    {/* New Graph for Cost to Pity and Gold from Selling Materials */}
                    {newGraphData && (
                        <div style={{ marginTop: 20, display: 'flex', justifyContent: 'center' }}>
                            <div style={{ width: '100%', maxWidth: '800px' }}>
                                <Graph
                                    title={`Cost to Pity and Gold from Selling Materials (Weeks 1-${newGraphData.costToPityData.length})`}
                                    labels={["Cost to Pity", "Gold if sell mats"]}
                                    counts={[newGraphData.costToPityData, newGraphData.goldFromSellData]}
                                    mins={[1, 1]}
                                    maxs={[newGraphData.costToPityData.length, newGraphData.goldFromSellData.length]}
                                    width={GRAPH_WIDTH}
                                    height={GRAPH_HEIGHT}
                                    hasSelection={true}
                                    isLoading={longTermBusy}
                                    cumulative={false}
                                    graphType="Gold"
                                    xAxisLabel="Week number"
                                    yAxisLabel="Gold value"
                                    customColors={['var(--series-brown)', 'var(--series-gold)']}
                                />
                            </div>
                        </div>
                    )}
                </div>
            </div>



        </>
    )
}
