import React, { useState, useEffect, useRef, useMemo, useCallback } from "react"
import SpreadsheetGrid from "../../components/SpreadsheetGrid.tsx"
import Graph from "../../components/Graph.tsx"
import { styles, createColumnDefs, GRAPH_WIDTH, GRAPH_HEIGHT, SMALL_GRAPH_WIDTH, SMALL_GRAPH_HEIGHT } from "./styles.ts"
import { BOTTOM_COLS, INPUT_LABELS, TOP_COLS, OUTPUT_LABELS } from "./constants.ts"
import { SpawnWorker } from "../../worker_setup.ts"
import { buildPayload } from "./Debounce.ts"
import { Upgrade, EQUIPMENT_TYPES } from "./utils.ts"
import GambaInfoBox from "./GambaInfoBox.tsx"
import GambaSelection from "./GambaSelection.tsx"
import { useGambaLogic } from "./GambaLogic.tsx"


type GambaSectionProps = {
    budget_inputs: any
    set_budget_inputs: React.Dispatch<React.SetStateAction<any>>
    userMatsValue: any
    setUserMatsValue: React.Dispatch<React.SetStateAction<any>>
    topGrid: boolean[][]
    bottomGrid: boolean[][]
    adv_hone_strategy: string
    express_event: boolean
    desired_chance: string
    bucketCount: string
    autoOptimization: boolean
    dataSize: string
    tooltipHandlers: {
        showUpgradeTooltip: (_upgrade: any, _costLabels: string[], _tapRecordCosts: number[], _x: number, _y: number) => void
        hideTooltip: () => void
        updateTooltipPosition: (_x: number, _y: number) => void
    }
    chance_result: any
    cachedChanceGraphData: { hist_counts?: any, hist_mins?: any, hist_maxs?: any } | null
    AnythingTicked: boolean
    CostToChanceBusy: boolean
    cumulativeGraph: boolean
    lockXAxis: boolean
    lockedMins: number[] | null
    lockedMaxs: number[] | null
    useGridInput: boolean
    normalCounts: number[][]
    advCounts: number[][]
    upgradeArr: any[]
    ParserBusy: boolean
}


export default function GambaSection({
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
    tooltipHandlers,
    chance_result,
    cachedChanceGraphData,
    AnythingTicked,
    CostToChanceBusy,
    cumulativeGraph,
    lockXAxis,
    lockedMins,
    lockedMaxs,
    useGridInput,
    normalCounts,
    advCounts,
}: GambaSectionProps) {
    const { costToChanceColumnDefs } = createColumnDefs(true)

    // State management
    const [upgradeArr, setUpgradeArr] = useState<Upgrade[]>([])
    const [selectedUpgradeIndex, setSelectedUpgradeIndex] = useState<number | null>(0)
    const [finalCosts, setFinalCosts] = useState<number[]>(new Array(10).fill(0))
    const [unlockCosts, setUnlockCosts] = useState<number[]>(new Array(2).fill(0))
    const [isAutoAttempting, setIsAutoAttempting] = useState<boolean>(false)
    const [isAutoAttemptingThisOne, setIsAutoAttemptingThisOne] = useState<boolean>(false)
    const [completionCounter, setCompletionCounter] = useState<number>(0)
    const [refreshKey, setRefreshKey] = useState<boolean>(false)

    // Worker refs and debounce
    const parserWorkerRef = useRef<Worker | null>(null)
    const debounceTimerRef = useRef<number | null>(null)
    const pendingRequests = useRef(new Map<string, (_data: any) => void>())
    const autoAttemptIntervalRef = useRef<number | null>(null)
    const currentUpgradeArrRef = useRef<Upgrade[]>([])
    const currentSelectedIndexRef = useRef<number | null>(null)
    const selectedUpgradeIndexRef = useRef(selectedUpgradeIndex)
    const upgradeArrRef = useRef(upgradeArr)
    const finalCostsRef = useRef(finalCosts)
    const unlockCostsRef = useRef(unlockCosts)



    const flushTimeoutRef = useRef<number | null>(null)

    const flushUpgradeArrToState = useCallback((immediate = false) => {
        if (immediate) {
            if (flushTimeoutRef.current) {
                clearTimeout(flushTimeoutRef.current)
                flushTimeoutRef.current = null
            }
            // shallow copy the array to trigger React update
            setUpgradeArr(_prev => upgradeArrRef.current.slice())
            return
        }
        if (flushTimeoutRef.current != null) return
        flushTimeoutRef.current = window.setTimeout(() => {
            setUpgradeArr(_prev => upgradeArrRef.current.slice())
            flushTimeoutRef.current = null
        }, 100) // coalesce UI updates every 100ms
    }, [])

    // sync refs whenever state changes
    useEffect(() => { selectedUpgradeIndexRef.current = selectedUpgradeIndex }, [selectedUpgradeIndex])
    useEffect(() => { upgradeArrRef.current = upgradeArr }, [upgradeArr])
    useEffect(() => { finalCostsRef.current = finalCosts }, [finalCosts])
    useEffect(() => { unlockCostsRef.current = unlockCosts }, [unlockCosts])
    // Initialize parser worker
    useEffect(() => {
        // Create a persistent worker for parser calls
        parserWorkerRef.current = new Worker(new URL("../../js_to_wasm.ts", import.meta.url), { type: "module" })

        // Set up message handler once
        parserWorkerRef.current.onmessage = (e) => {
            const { id, type, result } = e.data
            if (type === 'result' && pendingRequests.current.has(id)) {
                pendingRequests.current.get(id)!(result)
                pendingRequests.current.delete(id)
            }
        }

        return () => {
            if (parserWorkerRef.current) {
                parserWorkerRef.current.terminate()
            }
            // Clean up auto-attempt interval
            if (autoAttemptIntervalRef.current) {
                clearTimeout(autoAttemptIntervalRef.current)
            }
        }
    }, [])

    // Function to call parser and get upgrade array using unified payload format
    const callParser = useCallback(async () => {
        if (!parserWorkerRef.current) return

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

        const id = Math.random().toString(36).substr(2, 9)

        // Create promise for this request
        const p = new Promise(_resolve => pendingRequests.current.set(id, _resolve))

        parserWorkerRef.current.postMessage({
            id,
            payload,
            which_one: "ParserUnified"
        })

        p.then((result: any) => {
            const upgrades = result.upgrades as Upgrade[]
            const unlocks = result.unlocks as number[]
            const other_strategy_prob_dists = result.other_strategy_prob_dists as number[][]
            setUnlockCosts(unlocks)
            unlockCostsRef.current = unlocks

            // Add equipment types to upgrades based on ticked equipment in grids
            let seen_ind_normal = Array.from({ length: TOP_COLS }, () => 0);
            let seen_ind_adv = Array.from({ length: BOTTOM_COLS }, () => 0);
            let upgradesWithTypes = upgrades.map(upgrade => {
                if (upgrade.is_weapon) {
                    return { ...upgrade, equipment_type: 'Weapon' }
                } else {
                    // For armor, find which equipment types are ticked in the grid
                    const equipmentTypes = EQUIPMENT_TYPES.slice(0, 5) // Exclude Weapon
                    const assignedType = equipmentTypes.find((_, index) => {
                        if (upgrade.is_normal_honing) {
                            if (index < seen_ind_normal[upgrade.upgrade_plus_num]) { return false }
                            if (useGridInput) {
                                const gridRow = topGrid[index] || []
                                seen_ind_normal[upgrade.upgrade_plus_num] += 1;
                                return gridRow[upgrade.upgrade_plus_num] || false
                            } else {
                                // Use numeric input data - check if this equipment type has a count > 0
                                const armorCount = normalCounts[0][upgrade.upgrade_plus_num] || 0
                                seen_ind_normal[upgrade.upgrade_plus_num] += 1;
                                return index < armorCount
                            }
                        }
                        else {
                            if (index < seen_ind_adv[upgrade.upgrade_plus_num]) { return false }
                            if (useGridInput) {
                                const gridRow = bottomGrid[index] || []
                                seen_ind_adv[upgrade.upgrade_plus_num] += 1;
                                return gridRow[upgrade.upgrade_plus_num] || false
                            } else {
                                // Use numeric input data - check if this equipment type has a count > 0
                                const armorCount = advCounts[0][upgrade.upgrade_plus_num] || 0
                                seen_ind_adv[upgrade.upgrade_plus_num] += 1;
                                return index < armorCount
                            }
                        }
                    })
                    return { ...upgrade, equipment_type: assignedType || 'Armor', }
                }
            })
            upgradesWithTypes.sort((a, b) => { if (a.is_normal_honing) { return -999 } else { return a.upgrade_plus_num - b.upgrade_plus_num } })

            // Assign other_prob_dist to advanced honing upgrades
            let advUpgradeIndex = 0
            upgradesWithTypes.forEach(upgrade => {
                if (!upgrade.is_normal_honing && advUpgradeIndex < other_strategy_prob_dists.length) {
                    upgrade.other_prob_dist = other_strategy_prob_dists[advUpgradeIndex]
                    advUpgradeIndex++
                }
            })

            // Initialize upgrade completion tracking
            setUpgradeArr(upgradesWithTypes.map(upgrade => ({
                ...upgrade,
                is_finished: false,
                completion_order: 0,
                current_artisan: 0,
                taps_so_far: 0,
                juice_taps_so_far: 0,
                free_taps_so_far: 0,
                use_juice: adv_hone_strategy === "Juice on grace" && !upgrade.is_normal_honing,
                cumulative_chance: 0
            })))
            // Update refs
            currentUpgradeArrRef.current = upgradesWithTypes
        })
    }, [topGrid, bottomGrid, adv_hone_strategy, express_event, bucketCount, autoOptimization, userMatsValue, dataSize, budget_inputs, useGridInput, normalCounts, advCounts])

    // Debounce effect for parser calls when grids change
    const advStrategyKey = useMemo(() => String(adv_hone_strategy), [adv_hone_strategy])
    const expressEventKey = useMemo(() => String(express_event), [express_event])
    const useGridInputKey = useMemo(() => String(useGridInput), [useGridInput])
    const normalCountsKey = useMemo(() => JSON.stringify(normalCounts), [normalCounts])
    const advCountsKey = useMemo(() => JSON.stringify(advCounts), [advCounts])
    const refreshKeyMemo = useMemo(() => refreshKey, [refreshKey])

    useEffect(() => {
        // Clear existing timer
        if (debounceTimerRef.current) {
            window.clearTimeout(debounceTimerRef.current)
            debounceTimerRef.current = null
        }

        // Clear final costs when grids or strategy change
        setFinalCosts(new Array(10).fill(0))
        setSelectedUpgradeIndex(null)

        // Stop auto-attempting when grids change
        if (autoAttemptIntervalRef.current) {
            clearTimeout(autoAttemptIntervalRef.current)
            autoAttemptIntervalRef.current = null
        }
        setIsAutoAttempting(false)

        // Start new delayed work
        debounceTimerRef.current = window.setTimeout(() => {
            callParser()
            debounceTimerRef.current = null
        }, 100) // 100ms debounce

        return () => {
            if (debounceTimerRef.current) {
                window.clearTimeout(debounceTimerRef.current)
                debounceTimerRef.current = null
            }
        }
    }, [advStrategyKey, expressEventKey, refreshKeyMemo, callParser, useGridInputKey, normalCountsKey, advCountsKey])

    // Keep refs updated
    useEffect(() => {
        currentUpgradeArrRef.current = upgradeArr
    }, [upgradeArr])

    useEffect(() => {
        currentSelectedIndexRef.current = selectedUpgradeIndex
    }, [selectedUpgradeIndex])




    // Memoize sorted indices to avoid O(n²) findIndex operations in render
    const sortedWithIndex = useMemo(() => {
        // create array of indices and sort indices only (no object copies)
        const idxs = upgradeArr.map((_, i) => i)
        idxs.sort((ia, ib) => {
            const a = upgradeArr[ia], b = upgradeArr[ib]
            // same comparison code as sortedUpgrades but using a & b
            if (a.is_finished < b.is_finished) return -1
            if (a.is_finished > b.is_finished) return 1
            if (a.is_normal_honing < b.is_normal_honing) return 1
            if (a.is_normal_honing > b.is_normal_honing) return -1
            if (a.is_finished && b.is_finished) {
                return (a.completion_order || 0) - (b.completion_order || 0)
            }
            if (!a.is_finished && !b.is_finished) {
                if (a.upgrade_plus_num < b.upgrade_plus_num) return -1
                if (a.upgrade_plus_num > b.upgrade_plus_num) return 1
                return EQUIPMENT_TYPES.findIndex(v => a.equipment_type == v)
                    - EQUIPMENT_TYPES.findIndex(v => b.equipment_type == v)
            }
            return 0
        })
        return idxs // array of original indices in sorted order
    }, [upgradeArr])

    // Get unfinished normal honing upgrades for auto-attempt logic
    const unfinishedNormalUpgrades = useMemo(() => {
        return upgradeArr.filter(upgrade => !upgrade.is_finished)
    }, [upgradeArr])

    // Use the GambaLogic hook for tapping functionality
    const { attemptTap, toggleAutoAttempt, toggleAutoAttemptThisOne, freeTap } = useGambaLogic({
        upgradeArr,
        setUpgradeArr,
        selectedUpgradeIndex,
        setSelectedUpgradeIndex,
        finalCosts,
        setFinalCosts,
        completionCounter,
        setCompletionCounter,
        isAutoAttempting,
        setIsAutoAttempting,
        isAutoAttemptingThisOne,
        setIsAutoAttemptingThisOne,
        unfinishedNormalUpgrades,
        flushUpgradeArrToState
    })

    // Handle upgrade selection
    const handleUpgradeSelection = useCallback((index: number) => {
        setSelectedUpgradeIndex(index)
    }, [])

    const handleRefresh = useCallback(() => {
        setRefreshKey(prev => !prev)
    }, [])


    // Calculate budget remaining
    const budgetRemaining = INPUT_LABELS.map((label, index) => {
        const budget = parseInt(budget_inputs[label] || '0')
        const finalCost = finalCosts[index] || 0
        return budget - finalCost
    })

    // Create budget data for SpreadsheetGrid
    const budgetTotalData = INPUT_LABELS.reduce((acc, label, index) => {
        acc[label] = (finalCosts[index] + (index === 3 ? unlockCosts[0] : index === 6 ? unlockCosts[1] : 0)).toFixed(0).toString()
        return acc
    }, {} as Record<string, string>)

    const budgetRemainingData = INPUT_LABELS.reduce((acc, label, index) => {
        acc[label] = budgetRemaining[index].toFixed(0).toString()
        return acc
    }, {} as Record<string, string>)

    // Column definitions for budget grid (2 columns: Total Cost, Remaining)
    const budgetColumnDefs = [
        {
            headerName: "Cost so far",
            field: "total",
            editable: false,
            flex: 1,
            cellStyle: {
                backgroundColor: 'var(--background-secondary)',
                color: 'var(--text-primary)'
            }
        },
        {
            headerName: "Remaining",
            field: "remaining",
            editable: false,
            flex: 1,
            cellStyle: (params: any) => {
                const value = parseInt(params.value || '0')
                return {
                    backgroundColor: value < 0 ? 'var(--ran-out)' : 'transparent',
                    color: value < 0 ? 'white' : 'var(--text-primary)'
                }
            }
        }
    ]

    return (
        <>
            <style>
                {`
                    @keyframes pulse {
                        0% { opacity: 1; }
                        50% { opacity: 0.7; }
                        100% { opacity: 1; }
                    }
                `}
            </style>
            {/* <h3 style={{ color: 'var(--text-primary)', fontSize: 'var(--font-size-base)', fontWeight: 'var(--font-weight-semibold)', margin: '16px 0 0px 0' }}>Gamba Section</h3> */}

            <div style={{ ...styles.inputSection, flexDirection: "row", maxWidth: "1200px", width: "100%" }}>
                <div style={{ display: "flex", gap: 20, alignItems: "flex-start" }}>
                    {/* Budget Input Grid */}
                    <div style={{ display: 'flex', flexDirection: "column", gap: 0, alignItems: 'flex-start', justifyContent: 'start', width: autoOptimization ? 210 : 300 }}>
                        <div style={{ width: 210 }}>
                            <SpreadsheetGrid
                                columnDefs={costToChanceColumnDefs}
                                labels={INPUT_LABELS}
                                sheet_values={budget_inputs}
                                set_sheet_values={set_budget_inputs}
                                secondaryValues={userMatsValue}
                                setSecondaryValues={setUserMatsValue}
                            />
                        </div>
                        <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 8 }}>
                            <div style={{ ...styles.inputLabelCell, whiteSpace: 'nowrap', backgroundColor: 'var(--bg-tertiary)' }}>Chance of Success</div>
                            <div style={{ ...styles.inputCell, border: 'none', background: "transparent", color: 'var(--text-success)', fontSize: 'var(--font-size-xl)' }}>{chance_result ? (String(chance_result.chance) + '%') : '-'}</div>
                        </div>
                        <button
                            onClick={handleRefresh}
                            style={{
                                padding: '8px 16px',
                                backgroundColor: 'var(--btn-primary)',
                                color: 'var(--text-primary)',
                                border: '1px solid var(--border-accent)',
                                borderRadius: '4px',
                                cursor: 'pointer',
                                fontSize: 'var(--font-size-sm)',
                                fontWeight: 'var(--font-weight-bold)'
                            }}
                        >
                            Restart gamba
                        </button>
                    </div>

                    {/* Upgrade Selection Grid */}
                    <GambaSelection
                        upgradeArr={upgradeArr}
                        sortedWithIndex={sortedWithIndex}
                        selectedUpgradeIndex={selectedUpgradeIndex}
                        handleUpgradeSelection={handleUpgradeSelection}
                        isAutoAttempting={isAutoAttempting}
                        isAutoAttemptingThisOne={isAutoAttemptingThisOne}
                        tooltipHandlers={tooltipHandlers}
                    />

                    {/* Upgrade Info Box and Budget Remaining */}
                    <div style={{ display: "flex", flexDirection: "column", gap: 0 }}>
                        <div style={{ display: "flex", gap: 20 }}>
                            {/* Upgrade Info Box - Always shown */}
                            <GambaInfoBox
                                upgradeArr={upgradeArr}
                                selectedUpgradeIndex={selectedUpgradeIndex}
                                setUpgradeArr={setUpgradeArr}
                                attemptTap={attemptTap}
                                toggleAutoAttemptThisOne={toggleAutoAttemptThisOne}
                                toggleAutoAttempt={toggleAutoAttempt}
                                freeTap={freeTap}
                                isAutoAttempting={isAutoAttempting}
                                isAutoAttemptingThisOne={isAutoAttemptingThisOne}
                                unfinishedNormalUpgrades={unfinishedNormalUpgrades}
                                adv_hone_strategy={adv_hone_strategy}
                            />

                            {/* Budget Grid (Total Cost + Remaining) */}
                            <div style={{ width: 200, marginLeft: 70 }}>
                                <SpreadsheetGrid
                                    columnDefs={budgetColumnDefs}
                                    labels={INPUT_LABELS}
                                    sheet_values={budgetTotalData}
                                    set_sheet_values={() => { }} // Read-only
                                    secondaryValues={budgetRemainingData}
                                    setSecondaryValues={() => { }} // Read-only
                                    readOnly={true}
                                />
                            </div>
                        </div>
                        {/* Graph Section */}
                        <div style={{ marginTop: 20, display: 'flex', gap: 20 }}>
                            <div style={{ flex: 1 }}>
                                <Graph
                                    title="Cost distribution?"
                                    labels={OUTPUT_LABELS}
                                    counts={AnythingTicked ? (chance_result?.hist_counts || cachedChanceGraphData?.hist_counts) : null}
                                    mins={chance_result?.hist_mins || cachedChanceGraphData?.hist_mins}
                                    maxs={chance_result?.hist_maxs || cachedChanceGraphData?.hist_maxs}
                                    width={SMALL_GRAPH_WIDTH}
                                    height={SMALL_GRAPH_HEIGHT}
                                    budgets={OUTPUT_LABELS.map(label => Number(budget_inputs[label] || 0))}
                                    additionalBudgets={finalCosts.slice(0, 7).map((v, i) => v + (i === 3 ? unlockCosts[0] : i === 6 ? unlockCosts[1] : 0))} // First 7 elements for total costs
                                    hasSelection={AnythingTicked}
                                    isLoading={CostToChanceBusy}
                                    cumulative={cumulativeGraph}
                                    lockXAxis={lockXAxis}
                                    lockedMins={lockedMins}
                                    lockedMaxs={lockedMaxs}
                                    graphType={"Histogram"}
                                />
                            </div>
                        </div>
                    </div>
                </div>
            </div >



        </>
    )
}