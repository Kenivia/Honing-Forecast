import React, { useState, useMemo, useEffect, useRef } from "react"
import SpreadsheetGrid from "@/Frontend/Components/SpreadsheetGrid.tsx"
import Graph from "@/Frontend/Components/Graph.tsx"
import { styles, StyledSlider, GRAPH_HEIGHT, GRAPH_WIDTH, ColumnDef } from "@/Frontend/Utils/Styles.ts"
import { INPUT_LABELS } from "@/Frontend/Utils/Constants.ts"

import LabeledCheckbox from "@/Frontend/Components/LabeledCheckbox.tsx"

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
    autoGoldValues: any
    dataSize: any
    useGridInput: any
    normalCounts: any
    advCounts: any
    showOptimizedDetails: boolean
    setShowOptimizedDetails: React.Dispatch<React.SetStateAction<boolean>>
    incomeArr: number[][]
    setIncomeArr: React.Dispatch<React.SetStateAction<number[][]>>
    // Desired chance props
    desired_chance: string
    uncleaned_desired_chance: string
    onDesiredChange: (_value: string) => void
    onDesiredBlur: () => void
    // Cost result prop for hundred_budgets
    // cost_result: any
    // chanceToCostOptimizedResult: any
    payloadBuilder: any
    runner: any

    costToChanceResult: any
    monteCarloResult: any
}

function weekly_budget(budget: number[], weekly_income: number[]): number[][] {
    let current = budget
    let out = []
    for (let i = 0; i < 53; i++) {
        out.push(current.slice())
        for (let z = 0; z < 7; z++) {
            current[z] += weekly_income[z]
        }
    }
    return out
}
export default function LongTermSection({
    budget_inputs,
    set_budget_inputs,
    userMatsValue,
    setUserMatsValue,
    adv_hone_strategy,
    express_event,
    dataSize,
    useGridInput,
    normalCounts,
    advCounts,
    showOptimizedDetails,
    setShowOptimizedDetails,
    incomeArr,
    setIncomeArr,
    // Desired chance props
    desired_chance,
    uncleaned_desired_chance,
    onDesiredChange,
    onDesiredBlur,
    // Cost result prop for hundred_budgets
    // cost_result,
    // chanceToCostOptimizedResult,
    payloadBuilder,
    runner,
    costToChanceResult,
    monteCarloResult,
}: LongTermSectionProps) {
    // Income array is now managed by parent component

    // State for cost_to_chance_arr results

    // State for pity costs and material values
    const [_, setMatValues] = useState<number[]>(() => Object.values(userMatsValue).map((v) => parseFloat(v.toString()) || 0))

    // Worker refs and debounce

    // Function to get pity costs using average_cost_wrapper

    // Update material values when userMatsValue changes
    useEffect(() => {
        setMatValues(Object.values(userMatsValue).map((v) => parseFloat(v.toString()) || 0))
    }, [userMatsValue])

    // Income array is now managed by parent component, no local effects needed

    // Column definitions for the long term section - always shows both budget and gold value columns
    const longTermColumnDefs: ColumnDef[] = [
        {
            headerName: "Owned now",
            editable: true,
            flex: 1,
            background: "var(--grid-cell-bg)",
            backgroundSelected: "var(--grid-cell-selected)",
            color: "var(--grid-cell-text)",
        },
        {
            headerName: "Gold Price",
            editable: true,
            flex: 1,
            background: "var(--grid-cell-bg)",
            backgroundSelected: "var(--grid-cell-selected)",
            color: "var(--grid-cell-text)",
        },
    ]

    // Column definitions for income grids (1 column each)
    const incomeColumnDefs: ColumnDef[] = [
        {
            headerName: "Income",
            editable: true,
            flex: 1,
            background: "var(--grid-cell-bg)",
            backgroundSelected: "var(--grid-cell-selected)",
            color: "var(--grid-cell-text)",
        },
    ]

    // Column definitions for total weekly income grid (read-only)
    const totalIncomeColumnDefs: ColumnDef[] = [
        {
            headerName: "Total weekly",
            editable: false,
            flex: 1,
            background: "var(--grid-cell-bg-readonly)",
            backgroundSelected: "var(--grid-cell-selected-readonly)",
            color: "var(--grid-cell-text-readonly)",
        },
    ]

    // Calculate total weekly income (sum of all income grids for each row)
    const totalWeeklyIncome = useMemo(() => {
        return Array.from({ length: 7 }, (_, rowIndex) => incomeArr.reduce((sum, incomeGrid) => sum + (incomeGrid[rowIndex] || 0), 0))
    }, [incomeArr])

    const weeklyBudgets = useMemo(() => {
        return weekly_budget(
            Object.values(budget_inputs).map((v) => Math.round(Number(v))),
            totalWeeklyIncome
        )
    }, [budget_inputs, totalWeeklyIncome])
    // Debounce keys for cost_to_chance_arr
    const advStrategyKey = useMemo(() => String(adv_hone_strategy), [adv_hone_strategy])
    const expressEventKey = useMemo(() => String(express_event), [express_event])
    const useGridInputKey = useMemo(() => String(useGridInput), [useGridInput])
    const normalCountsKey = useMemo(() => JSON.stringify(normalCounts), [normalCounts])
    const advCountsKey = useMemo(() => JSON.stringify(advCounts), [advCounts])
    const totalWeeklyIncomeKey = useMemo(() => JSON.stringify(totalWeeklyIncome), [totalWeeklyIncome])

    // Function to call cost_to_chance_arr
    const chanceToCostArrWorkerRef = useRef<Worker | null>(null)
    const [longTermBusy, setLongTermBusy] = useState(false)
    const [longTermResult, setLongTermResult] = useState<any>(null)

    useEffect(() => {
        runner.start({
            which_one: "CostToChanceArr",
            payloadBuilder: () => {
                let x = payloadBuilder()
                x.budget_arr = weeklyBudgets
                // console.log(x)
                return x
            },
            workerRef: chanceToCostArrWorkerRef,
            setBusy: setLongTermBusy,
            setResult: setLongTermResult,
            dependency: monteCarloResult != null,
        })
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [advStrategyKey, expressEventKey, useGridInputKey, normalCountsKey, advCountsKey, totalWeeklyIncomeKey, weeklyBudgets, monteCarloResult])

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
        if (!longTermResult) return null
        let relevant_chance_result
        let relevant_fail_arr
        if (showOptimizedDetails) {
            if (!longTermResult.final_chances) {
                return null
            } else {
                relevant_chance_result = longTermResult.final_chances
                relevant_fail_arr = longTermResult.failure_rates_arr
            }
        } else {
            if (!longTermResult.buy_chances) {
                return null
            } else {
                relevant_chance_result = longTermResult.buy_chances
                // relevant_fail_arr = longTermResult.buy_cost
            }
        } // const chances = longTermResult.final_chances.map((chance: string) => parseFloat(chance))

        // Create histogram-like data for Graph component
        // We need to create counts array where each week (0-52) represents a "bucket"
        const numWeeks = 53 // 0 to 52 inclusive

        // --- Build overallCounts first (so truncation is driven by overall) ---
        const overallCounts = new Array(numWeeks).fill(0)
        for (let week = 0; week < numWeeks; week++) {
            if (week < relevant_chance_result.length) {
                // normalize percent -> fraction
                overallCounts[week] = relevant_chance_result[week] / 100
            }
        }

        // Determine truncation length:
        // If overallCounts stops increasing for 4 consecutive weeks, cut off from the start of that run.
        // Only scan up to the weeks we actually have final_chances for (so trailing zero-filled weeks don't immediately trigger a cutoff).
        const availableWeeks = Math.min(numWeeks, relevant_chance_result.length)
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
        if (showOptimizedDetails) {
            for (let materialIndex = 0; materialIndex < numMaterials; materialIndex++) {
                const materialCounts = new Array(truncateLen).fill(0)

                for (let week = 0; week < truncateLen; week++) {
                    // safety: check failure_rates_arr bounds
                    if (relevant_fail_arr && week < relevant_fail_arr.length && relevant_fail_arr[week] && materialIndex < relevant_fail_arr[week].length) {
                        materialCounts[week] = relevant_fail_arr[week][materialIndex]
                    } else {
                        materialCounts[week] = 0
                    }
                }

                counts.push(materialCounts)
                mins.push(1)
                maxs.push(truncateLen)
            }
        }
        // debug
        // console.log('graphData: truncateLen=', truncateLen, 'counts lengths=', counts.map(c => c.length))
        // console.log(longTermResult)
        return {
            counts,
            mins,
            maxs,
        }
    }, [longTermResult, showOptimizedDetails])

    // Calculate data for the new graph (cost to pity and gold from selling materials)
    const goldGraphData = useMemo(() => {
        if (!costToChanceResult) return null

        const costToPityData: number[] = []
        const goldFromSellData: number[] = []
        const individualPityCostsData: number[][] = []

        // Initialize individual pity costs arrays
        for (let i = 0; i < 7; i++) {
            individualPityCostsData.push([])
        }

        // const needed_budget = costToChanceResult.hundred_budgets[parseInt(desired_chance)] || []
        // console.log("Pitycost", pityCost)
        for (let week = 0; week < Math.min(53, weeklyBudgets.length); week++) {
            const costToAchieve = costToChanceResult.hundred_gold_costs[parseInt(desired_chance)]
            const goldFromSell = weeklyBudgets[week][5] // gold_plus_sell_mats(this_week_budget, goldCost, matValues)

            costToPityData.push(costToAchieve)
            goldFromSellData.push(weeklyBudgets[week][5]) // til i sort out selling shinanigan

            // // Calculate individual pity costs for each material
            // for (let materialIndex = 0; materialIndex < 7; materialIndex++) {
            //     const individualCost = cost_to_pity_individual(weeklyBudgets[week], needed_budget, matValues, materialIndex)
            //     const clampedCost = Math.max(0, individualCost) // Clamp to 0 if negative
            //     individualPityCostsData[materialIndex].push(clampedCost)
            // }
            // // console.log("break", goldFromSell, costToPity)
            if (goldFromSell > costToAchieve && week > 2) {
                break
            }
        }

        return {
            costToPityData,
            goldFromSellData,
            individualPityCostsData,
            weeklyBudgets,
        }
    }, [costToChanceResult, desired_chance, weeklyBudgets])

    return (
        <>
            <div style={{ ...styles.inputSection, flexDirection: "row", maxWidth: "1200px", width: "100%" }}>
                <div style={{ display: "flex", gap: 20, alignItems: "center", flexDirection: "column" }}>
                    <div style={{ display: "flex", gap: 5, alignItems: "flex-start", alignSelf: "flex-start" }}>
                        {/* Main Owned now Grid */}
                        <div style={{ display: "flex", flexDirection: "column", gap: 0, alignItems: "flex-start", justifyContent: "start", width: 300 }}>
                            <div style={{ marginBottom: 16, width: 300 }}>
                                <SpreadsheetGrid
                                    columnDefs={longTermColumnDefs}
                                    labels={INPUT_LABELS}
                                    sheetValuesArr={[budget_inputs, userMatsValue]}
                                    setSheetValuesArr={[set_budget_inputs, setUserMatsValue]}
                                />
                            </div>
                        </div>

                        {/* 6 Income Grids */}
                        {Array.from({ length: 6 }, (_, gridIndex) => (
                            <div
                                key={`income-${gridIndex}`}
                                style={{ display: "flex", flexDirection: "column", gap: 0, alignItems: "flex-start", justifyContent: "start", width: 120 }}
                            >
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
                                        sheetValuesArr={[getIncomeGridData(gridIndex)]}
                                        setSheetValuesArr={[
                                            (newValues) => {
                                                // Update the specific income grid
                                                setIncomeArr((prev) => {
                                                    const newArr = prev.map((grid) => [...grid])
                                                    incomeLabels.forEach((label, rowIndex) => {
                                                        const value = newValues[label] || "0"
                                                        const cleanValue = value.replace(/[^0-9.-]/g, "")
                                                        newArr[gridIndex][rowIndex] = parseFloat(cleanValue) || 0
                                                    })
                                                    return newArr
                                                })
                                            },
                                        ]}
                                        hideIcons={true}
                                    />
                                </div>
                            </div>
                        ))}

                        {/* Total Weekly Income Grid */}
                        <div style={{ display: "flex", flexDirection: "column", gap: 0, alignItems: "flex-start", justifyContent: "start", width: 10 }}>
                            <div style={{ width: 120 }}>
                                <SpreadsheetGrid
                                    columnDefs={totalIncomeColumnDefs}
                                    labels={incomeLabels}
                                    sheetValuesArr={[totalIncomeData]}
                                    setSheetValuesArr={[() => {}]} // Read-only
                                    hideIcons={true}
                                />
                            </div>
                        </div>
                    </div>
                    <div style={{ display: "flex", alignItems: "flex-start", gap: 8, marginBottom: 8, justifySelf: "left", marginLeft: 30, marginTop: 20 }}>
                        <LabeledCheckbox
                            label="I don't want to buy anything"
                            checked={showOptimizedDetails}
                            setChecked={setShowOptimizedDetails}
                            textColor="var(--text-optimized)"
                            accentColor="var(--text-optimized)"
                        />
                    </div>
                    <div style={{ marginTop: 20, display: "flex", justifyContent: "center" }}>
                        <div style={{ width: "100%", maxWidth: "800px" }}>
                            <Graph
                                title={`Chance of Success Over Time (0-${(graphData?.counts?.[0]?.length || 53) - 1} weeks from now)`}
                                labels={[showOptimizedDetails ? "Overall no buy" : "Overall", ...INPUT_LABELS.slice(0, 7).map((label) => label)]} // Overall + 7 material types
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
                                customColors={[
                                    showOptimizedDetails ? "var(--text-optimized)" : "var(--series-overall)",
                                    "var(--series-red)",
                                    "var(--series-blue)",
                                    "var(--series-leaps)",
                                    "var(--series-shards)",
                                    "var(--series-oreha)",
                                    "var(--series-gold)",
                                    "var(--series-silver)",
                                ]}
                                showWarning={totalWeeklyIncome.every((income) => income === 0)}
                                warningMessage="You are earning nothing per week. Input your income (or click Fill Demo Income) to see your chances in the future."
                            />
                        </div>
                    </div>

                    {/* Desired Chance Slider */}
                    <div style={{ marginTop: 20, display: "flex", justifyContent: "center", justifySelf: "left", alignSelf: "flex-start", marginLeft: 100 }}>
                        <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
                            <div style={{ width: 160, fontWeight: 700, textAlign: "right", paddingRight: 8, color: "var(--text-primary)", textWrap: "nowrap" }}>
                                Desired chance{" "}
                            </div>
                            <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
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
                                <div style={{ position: "relative", display: "flex", alignItems: "center" }}>
                                    <input
                                        type="text"
                                        value={uncleaned_desired_chance}
                                        onChange={(e) => onDesiredChange(e.target.value)}
                                        onBlur={onDesiredBlur}
                                        placeholder="0"
                                        style={{
                                            width: 70,
                                            fontSize: 16,
                                            padding: "6px 8px",
                                            borderRadius: 6,
                                            background: "var(--input-bg)",
                                            color: "var(--input-text)",
                                            border: "1px solid var(--input-border)",
                                        }}
                                    />
                                    <span style={{ position: "absolute", right: 10, pointerEvents: "none", color: "black" }}>%</span>
                                </div>
                                {desired_chance === "0" && (
                                    <span
                                        style={{
                                            color: "var(--text-muted)",
                                            fontSize: "var(--font-size-sm)",
                                            fontStyle: "italic",
                                        }}
                                    >
                                        0% = luckiest draw in {dataSize} samples
                                    </span>
                                )}
                            </div>
                        </div>
                    </div>

                    {/* New Graph for Cost to Pity and Gold from Selling Materials */}
                    {goldGraphData && (
                        <div style={{ marginTop: 20, display: "flex", justifyContent: "center" }}>
                            <div style={{ width: "100%", maxWidth: "800px" }}>
                                <Graph
                                    title={`Gold needed to achieve ${parseInt(desired_chance) === 100 ? "pity" : `${desired_chance}% chance to pass`} (0-${
                                        goldGraphData.costToPityData.length - 1
                                    } weeks from now)`}
                                    labels={["Cost to Pity", "Gold if sell mats", ...INPUT_LABELS.slice(0, 7)]}
                                    counts={[goldGraphData.costToPityData, goldGraphData.goldFromSellData, ...goldGraphData.individualPityCostsData]}
                                    mins={[0, 0, ...goldGraphData.individualPityCostsData.map(() => 0)]}
                                    maxs={[
                                        goldGraphData.costToPityData.length - 1,
                                        goldGraphData.goldFromSellData.length - 1,
                                        ...goldGraphData.individualPityCostsData.map((data) => data.length - 1),
                                    ]}
                                    width={GRAPH_WIDTH}
                                    height={GRAPH_HEIGHT}
                                    hasSelection={true}
                                    isLoading={longTermBusy}
                                    cumulative={false}
                                    graphType="Gold"
                                    xAxisLabel="Week number"
                                    yAxisLabel="Gold value"
                                    // yMaxOverride={newGraphData.costToPityData[0]}
                                    customColors={[
                                        "var(--series-brown)",
                                        "var(--series-gold)",
                                        "var(--series-red)",
                                        "var(--series-blue)",
                                        "var(--series-leaps)",
                                        "var(--series-shards)",
                                        "var(--series-oreha)",
                                        "var(--series-gold)",
                                        "var(--series-silver)",
                                    ]}
                                    weeklyBudgets={goldGraphData.weeklyBudgets}
                                    showWarning={totalWeeklyIncome.every((income) => income === 0)}
                                    warningMessage="You are earning nothing per week. Input your income (or click Fill Demo Income) to see your chances in the future."
                                />
                            </div>
                        </div>
                    )}
                </div>
            </div>
        </>
    )
}
