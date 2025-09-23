import React, { useState, useEffect, useRef, useMemo, useCallback } from "react"
import SpreadsheetGrid from "../../components/SpreadsheetGrid.tsx"
import Graph from "../../components/Graph.tsx"
import { styles, createColumnDefs } from "./styles.ts"
import { BOTTOM_COLS, INPUT_LABELS, TOP_COLS, OUTPUT_LABELS } from "./constants.ts"
import { SpawnWorker } from "../../worker_setup.ts"
import { buildPayload } from "./Debounce.ts"

function sortedUpgrades(upgradeArr: Upgrade[]) {
    let out = [...upgradeArr]
    out.sort((a, b) => {
        // Unfinished normal honing upgrades first
        if (a.is_finished < b.is_finished) {
            return -1
        }
        if (a.is_finished > b.is_finished) {
            return 1
        }
        if (a.is_normal_honing < b.is_normal_honing) {
            return 1
        }
        if (a.is_normal_honing > b.is_normal_honing) {
            return -1
        }

        // Then finished upgrades by completion order
        if (a.is_finished && b.is_finished) {
            return (a.completion_order || 0) - (b.completion_order || 0)
        }
        if (!a.is_finished && !b.is_finished) {
            if (a.upgrade_plus_num < b.upgrade_plus_num) {
                return -1
            }
            if (a.upgrade_plus_num > b.upgrade_plus_num) {
                return 1
            }
            return EQUIPMENT_TYPES.findIndex((value, _) => a.equipment_type == value)
                - EQUIPMENT_TYPES.findIndex((value, _) => b.equipment_type == value)
        }
        return 0
    })
    return out
}
// Helper function to calculate tap record costs
function calculateTapRecordCosts(upgrade: Upgrade) {
    const costs = new Array(10).fill(0)
    const taps = upgrade.taps_so_far ?? 0
    const juiceTaps = upgrade.juice_taps_so_far ?? 0
    const freeTaps = upgrade.free_taps_so_far ?? 0

    // Regular costs multiplied by taps
    for (let i = 0; i < 7; i++) {
        costs[i] = upgrade.costs[i] * taps
    }

    // Juice costs
    if (juiceTaps > 0) {
        const juiceCost = upgrade.one_juice_cost * juiceTaps
        if (upgrade.is_weapon) {
            costs[8] = juiceCost // Weapons add to 9th slot (index 8)
        } else {
            costs[7] = juiceCost // Armors add to 8th slot (index 7)
        }
    }

    // Free tap costs
    if (freeTaps > 0) {
        costs[9] = upgrade.special_cost * freeTaps
    }

    return costs
}

// Simple wrapper component for upgrade tooltips
type UpgradeTooltipProps = {
    upgrade: Upgrade
    children: React.ReactNode
    tooltipHandlers: GambaSectionProps['tooltipHandlers']
}

const UpgradeTooltip = React.memo(function UpgradeTooltip({ upgrade, children, tooltipHandlers }: UpgradeTooltipProps) {
    const costLabels = ['Red', 'Blue', 'Leaps', 'Shards', 'Oreha', "Gold", 'Silver', 'Red Juice', 'Blue Juice', 'Special Leaps']
    const tapRecordCosts = useMemo(() => calculateTapRecordCosts(upgrade),
        [upgrade])

    const handleMouseEnter = (e: React.MouseEvent) => {
        tooltipHandlers.showUpgradeTooltip(upgrade, costLabels, tapRecordCosts, e.clientX, e.clientY)
    }

    const handleMouseMove = (e: React.MouseEvent) => {
        tooltipHandlers.updateTooltipPosition(e.clientX, e.clientY)
    }

    const handleMouseLeave = () => {
        tooltipHandlers.hideTooltip()
    }

    return (
        <div
            onMouseEnter={handleMouseEnter}
            onMouseMove={handleMouseMove}
            onMouseLeave={handleMouseLeave}
        >
            {children}
        </div>
    )
})

// TypeScript interfaces
interface Upgrade {
    is_normal_honing: boolean
    prob_dist: number[]
    original_prob_dist: number[]
    base_chance: number
    costs: number[]
    one_juice_cost: number
    adv_juice_cost: number[]
    special_cost: number
    values: number[]
    prob_dist_len: number
    is_weapon: boolean
    artisan_rate: number
    tap_offset: number
    upgrade_plus_num: number
    special_value: number
    equipment_type?: string // Added for equipment type
    is_finished?: boolean // Track if upgrade is completed
    completion_order?: number // Track order of completion
    current_artisan?: number // Track current artisan for this upgrade
    taps_so_far?: number // Number of taps attempted so far
    juice_taps_so_far?: number // Number of taps with juice so far
    free_taps_so_far?: number // Number of free taps so far
    use_juice?: boolean // Whether juice is currently enabled for this upgrade
}


interface UpgradeState {
    artisan: number
    trialsSoFar: number
}

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
}

// Equipment types for armor pieces
const EQUIPMENT_TYPES = ['Helmet', 'Shoulder', 'Chest', 'Pants', 'Gloves', 'Weapon']

function calculateCurrentChance(upgrade: Upgrade) {
    if (!upgrade.is_normal_honing) return 0
    const baseChance = upgrade.base_chance
    const minCount = Math.min(upgrade.taps_so_far, 10)
    const currentChance = baseChance + (baseChance / 10) * minCount
    return Math.max(0, Math.min(1, upgrade.current_artisan >= 1 ? 1 : (upgrade.use_juice ? currentChance + upgrade.base_chance : currentChance)))
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
    desired_chance,
    bucketCount,
    autoOptimization,
    dataSize,
    tooltipHandlers,
    chance_result,
    cachedChanceGraphData,
    AnythingTicked,
    CostToChanceBusy,
    cumulativeGraph,
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
    const isAutoAttemptingRef = useRef(isAutoAttempting)
    const isAutoAttemptingThisOneRef = useRef(isAutoAttemptingThisOne)



    const intervalRef = useRef<number | null>(null)
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
    useEffect(() => { isAutoAttemptingRef.current = isAutoAttempting }, [isAutoAttempting])
    useEffect(() => { isAutoAttemptingThisOneRef.current = isAutoAttemptingThisOne }, [isAutoAttemptingThisOne])
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
            desired_chance,
            budget_inputs,
            adv_hone_strategy,
            express_event,
            bucketCount,
            autoOptimization,
            userMatsValue,
            dataSize,
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
                            const gridRow = topGrid[index] || []
                            seen_ind_normal[upgrade.upgrade_plus_num] += 1;
                            return gridRow[upgrade.upgrade_plus_num] || false
                        }
                        else {
                            if (index < seen_ind_adv[upgrade.upgrade_plus_num]) { return false }
                            const gridRow = bottomGrid[index] || []
                            seen_ind_adv[upgrade.upgrade_plus_num] += 1;
                            return gridRow[upgrade.upgrade_plus_num] || false
                        }
                    })
                    return { ...upgrade, equipment_type: assignedType || 'Armor', }
                }
            })
            upgradesWithTypes.sort((a, b) => { if (a.is_normal_honing) { return -999 } else { return a.upgrade_plus_num - b.upgrade_plus_num } })

            // Initialize upgrade completion tracking
            setUpgradeArr(upgradesWithTypes.map(upgrade => ({
                ...upgrade,
                is_finished: false,
                completion_order: 0,
                current_artisan: 0,
                taps_so_far: 0,
                juice_taps_so_far: 0,
                free_taps_so_far: 0,
                use_juice: false
            })))
            // Update refs
            currentUpgradeArrRef.current = upgradesWithTypes
        })
    }, [topGrid, bottomGrid, adv_hone_strategy, express_event, desired_chance, bucketCount, autoOptimization, userMatsValue, dataSize, budget_inputs])

    // Debounce effect for parser calls when grids change
    const topGridKey = useMemo(() => JSON.stringify(topGrid), [topGrid])
    const bottomGridKey = useMemo(() => JSON.stringify(bottomGrid), [bottomGrid])
    const advStrategyKey = useMemo(() => String(adv_hone_strategy), [adv_hone_strategy])
    const expressEventKey = useMemo(() => String(express_event), [express_event])
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
    }, [topGridKey, bottomGridKey, advStrategyKey, expressEventKey, refreshKeyMemo, callParser])

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
        return upgradeArr.filter(upgrade => !upgrade.is_finished && upgrade.is_normal_honing)
    }, [upgradeArr])

    // Handle upgrade selection
    const handleUpgradeSelection = useCallback((index: number) => {
        // Save current state before switching

        // Select new upgrade
        setSelectedUpgradeIndex(index)

        // Restore state for new upgrade

    }, [])

    const handleRefresh = useCallback(() => {
        setRefreshKey(prev => !prev)
    }, [])

    // Calculate current chance for normal honing


    const performAttempt = useCallback(() => {
        const selIdx = selectedUpgradeIndexRef.current

        const arr = upgradeArrRef.current
        // console.log(selIdx, arr[selIdx].taps_so_far)
        if (selIdx === null || !arr || !arr[selIdx]) {
            // nothing to attempt — stop auto attempting if running
            if (intervalRef.current != null) {
                clearTimeout(intervalRef.current)
                intervalRef.current = null
                setIsAutoAttempting(false)
                isAutoAttemptingRef.current = false
                setIsAutoAttemptingThisOne(false)
                isAutoAttemptingThisOneRef.current = false
            }
            return
        }

        const upgrade = arr[selIdx]

        // Skip if upgrade is already finished
        if (upgrade.is_finished) {
            return
        }

        let success = false
        let advTapCount = -1;
        if (upgrade.is_normal_honing) {
            const currentChance = calculateCurrentChance(upgrade)
            success = Math.random() < currentChance
        } else {
            // Advanced honing logic - simulate tap count based on probability distribution
            advTapCount = upgrade.tap_offset + Math.floor(Math.random() * upgrade.prob_dist_len)
            success = true
            upgrade.taps_so_far = advTapCount - 1
        }

        // Add costs to final_costs (functional update + keep ref in sync)
        setFinalCosts(prev => {
            const next = prev.slice()
            if (upgrade.is_normal_honing) {
                for (let i = 0; i < 7; i++) {
                    next[i] = (next[i] ?? 0) + (upgrade.costs?.[i] ?? 0)
                }
                if (upgrade.use_juice) {
                    const juiceCost = upgrade.one_juice_cost ?? 0
                    if (upgrade.is_weapon) next[8] = (next[8] ?? 0) + juiceCost
                    else next[7] = (next[7] ?? 0) + juiceCost
                }
            }
            else {
                for (let i = 0; i < 7; i++) {
                    next[i] = (next[i] ?? 0) + (upgrade.costs?.[i] * advTapCount)
                }
                if (upgrade.use_juice) {
                    const juiceCost = upgrade.adv_juice_cost[advTapCount - upgrade.tap_offset] ?? 0
                    if (upgrade.is_weapon) next[8] = (next[8] ?? 0) + juiceCost
                    else next[7] = (next[7] ?? 0) + juiceCost
                }
            }
            finalCostsRef.current = next
            return next
        })

        // Update upgrade counters
        upgrade.taps_so_far = (upgrade.taps_so_far || 0) + 1
        upgrade.juice_taps_so_far = (upgrade.juice_taps_so_far || 0) + (upgrade.use_juice ? 1 : 0)
        if (success) {
            upgrade.is_finished = true
            upgrade.completion_order = (upgrade.completion_order || 0) + 1
            setCompletionCounter(prev => prev + 1)
            // Handle auto-attempting logic
            if (isAutoAttemptingThisOneRef.current) {
                // Stop "this one" auto-attempting since the upgrade succeeded
                if (intervalRef.current != null) {
                    clearTimeout(intervalRef.current)
                    intervalRef.current = null
                }
                setIsAutoAttemptingThisOne(false)
                isAutoAttemptingThisOneRef.current = false
                setIsAutoAttempting(false)
                isAutoAttemptingRef.current = false

                // Move to next unfinished upgrade (top when sorted visually)

                const nextUnfinishedIndex = upgradeArrRef.current.findIndex(
                    (z) => z == sortedUpgrades(upgradeArrRef.current).find((upg, i) =>
                        !upg.is_finished && i !== selIdx))
                // console.log("thisone", nextUnfinishedIndex)
                if (nextUnfinishedIndex !== -1) {
                    setSelectedUpgradeIndex(nextUnfinishedIndex)
                    selectedUpgradeIndexRef.current = nextUnfinishedIndex
                }
            } else if (isAutoAttemptingRef.current) {

                // Move to next unfinished upgrade if auto-attempting all
                const nextUnfinishedIndex = upgradeArrRef.current.findIndex(
                    (z) => z == sortedUpgrades(upgradeArrRef.current).find((upg, i) =>
                        !upg.is_finished && i !== selIdx))
                // console.log("auto", nextUnfinishedIndex, upgradeArrRef.current[nextUnfinishedIndex])
                if (nextUnfinishedIndex !== -1) {
                    setSelectedUpgradeIndex(nextUnfinishedIndex)
                    selectedUpgradeIndexRef.current = nextUnfinishedIndex
                } else {
                    // No more unfinished normal honing upgrades, stop auto-attempting
                    if (intervalRef.current != null) {
                        clearTimeout(intervalRef.current)
                        intervalRef.current = null
                    }
                    setIsAutoAttempting(false)
                    isAutoAttemptingRef.current = false

                }
            } else {
                // Move to next unfinished upgrade (top when sorted visually)
                const nextUnfinishedIndex = upgradeArrRef.current.findIndex(
                    (z) => z == sortedUpgrades(upgradeArrRef.current).find((upg, i) =>
                        !upg.is_finished && i !== selIdx))
                if (nextUnfinishedIndex !== -1) {
                    setSelectedUpgradeIndex(nextUnfinishedIndex)
                    selectedUpgradeIndexRef.current = nextUnfinishedIndex

                }
            }

        } else {
            // failure: increment artisan
            const currentChance = calculateCurrentChance(upgrade)
            upgrade.current_artisan = Math.min(1, (upgrade.current_artisan || 0) + (46.51 / 100.0) * currentChance * (upgrade.artisan_rate ?? 0))
        }

        // Update the upgrade in the ref array
        upgradeArrRef.current[selIdx] = upgrade

        // If success (user-visible change) flush immediately, else schedule a coalesced update
        if (success) {
            flushUpgradeArrToState(true)
        } else {
            flushUpgradeArrToState(false)
        }
    }, [flushUpgradeArrToState]) // stable identity — uses refs internally
    // Attempt a tap
    const attemptTap = useCallback(() => performAttempt(), [performAttempt])


    // Toggle auto-attempt mode
    const startAuto = useCallback((ms: number) => {
        if (intervalRef.current != null) return
        isAutoAttemptingRef.current = true
        setIsAutoAttempting(true)

        const tick = () => {
            performAttempt()
            if (isAutoAttemptingRef.current) {
                intervalRef.current = window.setTimeout(tick, Math.max(5, ms)) // enforce min delay
            } else {
                intervalRef.current = null
            }
        }
        tick()
    }, [performAttempt])
    const stopAuto = useCallback(() => {
        if (intervalRef.current != null) {
            clearTimeout(intervalRef.current)
            intervalRef.current = null
        }
        setIsAutoAttempting(false)
        isAutoAttemptingRef.current = false
        setIsAutoAttemptingThisOne(false)
        isAutoAttemptingThisOneRef.current = false
    }, [])

    const toggleAutoAttempt = useCallback(() => {
        if (isAutoAttemptingRef.current) {
            stopAuto()
        } else {
            // Only start auto-attempt if there are unfinished normal honing upgrades
            if (unfinishedNormalUpgrades.length > 0) {
                if (!selectedUpgradeIndex ||
                    !upgradeArr[selectedUpgradeIndex] ||
                    upgradeArr[selectedUpgradeIndex].is_finished) {
                    const nextUnfinishedIndex = upgradeArrRef.current.findIndex(
                        (z) => z == sortedUpgrades(upgradeArrRef.current).find((upg, i) =>
                            !upg.is_finished && i !== selectedUpgradeIndex))
                    setSelectedUpgradeIndex(nextUnfinishedIndex)
                    selectedUpgradeIndexRef.current = nextUnfinishedIndex
                }
                startAuto(0) // 10ms per your request
            }
        }
    }, [startAuto, stopAuto, unfinishedNormalUpgrades, selectedUpgradeIndexRef, setSelectedUpgradeIndex, upgradeArr, selectedUpgradeIndex])

    const toggleAutoAttemptThisOne = useCallback(() => {
        if (isAutoAttemptingThisOneRef.current) {
            stopAuto()
        } else {
            // Only start "this one" auto-attempt if there's a selected upgrade and it's not finished
            if (selectedUpgradeIndex !== null && upgradeArr[selectedUpgradeIndex] && !upgradeArr[selectedUpgradeIndex].is_finished) {
                setIsAutoAttemptingThisOne(true)
                isAutoAttemptingThisOneRef.current = true
                startAuto(10) // 1ms for faster auto-attempting
            }
        }
    }, [startAuto, stopAuto, selectedUpgradeIndex, upgradeArr])

    // Free tap
    const freeTap = () => {
        if (selectedUpgradeIndex === null || !upgradeArr[selectedUpgradeIndex]) return

        const upgrade = upgradeArr[selectedUpgradeIndex]
        const success = Math.random() < upgrade.base_chance

        // Add special cost to 10th element
        setFinalCosts(prev => {
            const next = prev.slice()
            next[9] = (next[9] || 0) + upgrade.special_cost
            return next
        })

        if (success) {
            // Success - mark as finished and update tap counts
            setUpgradeArr(prev => {
                const next = prev.slice()
                next[selectedUpgradeIndex] = {
                    ...next[selectedUpgradeIndex],
                    is_finished: true,
                    completion_order: completionCounter + 1,
                    free_taps_so_far: (next[selectedUpgradeIndex].free_taps_so_far ?? 0) + 1,
                }
                return next
            })

            setCompletionCounter(prev => prev + 1)
        } else {
            // Failure - still track the free tap
            setUpgradeArr(prev => {
                const next = prev.slice()
                next[selectedUpgradeIndex] = {
                    ...next[selectedUpgradeIndex],
                    free_taps_so_far: (next[selectedUpgradeIndex].free_taps_so_far ?? 0) + 1
                }
                return next
            })
        }
    }

    // Calculate budget remaining
    const budgetRemaining = INPUT_LABELS.map((label, index) => {
        const budget = parseInt(budget_inputs[label] || '0')
        const finalCost = finalCosts[index] || 0
        return budget - finalCost
    })

    // Create budget data for SpreadsheetGrid
    const budgetTotalData = INPUT_LABELS.reduce((acc, label, index) => {
        acc[label] = (finalCosts[index] + (index === 3 ? unlockCosts[0] : index === 6 ? unlockCosts[1] : 0)).toString()
        return acc
    }, {} as Record<string, string>)

    const budgetRemainingData = INPUT_LABELS.reduce((acc, label, index) => {
        acc[label] = budgetRemaining[index].toString()
        return acc
    }, {} as Record<string, string>)

    // Column definitions for budget grid (2 columns: Total Cost, Remaining)
    const budgetColumnDefs = [
        {
            headerName: "Total Cost",
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

            <div style={{ ...styles.inputSection, flexDirection: "row", width: 1120 }}>
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
                    <div style={{ width: 200 }}>
                        <h4 style={{ margin: 0, fontSize: 'var(--font-size-sm)', marginBottom: 10 }}>
                            Upgrades: {(isAutoAttempting || isAutoAttemptingThisOne) && <span style={{ color: 'var(--error-color)', fontSize: 'var(--font-size-xs)' }}>AUTO ON</span>}
                        </h4>
                        <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
                            {sortedWithIndex.map(originalIndex => {
                                const upgrade = upgradeArr[originalIndex]
                                return (
                                    <UpgradeTooltip key={originalIndex} upgrade={upgrade} tooltipHandlers={tooltipHandlers}>
                                        <div
                                            onClick={() => handleUpgradeSelection(originalIndex)}
                                            style={{
                                                padding: '8px',
                                                border: selectedUpgradeIndex === originalIndex ? '2px solid var(--accent-color)' : '1px solid var(--border-accent)',
                                                borderRadius: '4px',
                                                cursor: 'pointer',
                                                backgroundColor: selectedUpgradeIndex === originalIndex ? 'var(--marquee-bg)' : 'transparent',
                                                fontSize: 'var(--font-size-sm)',
                                                position: 'relative',
                                                animation: (isAutoAttempting || isAutoAttemptingThisOne) && selectedUpgradeIndex === originalIndex ? 'pulse 1s infinite' : 'none',
                                                color: upgrade.is_finished ? 'var(--btn-success)' : 'var(--text-primary)'
                                            }}
                                        >
                                            {upgrade.is_normal_honing ? '+' : 'Adv +'}{upgrade.is_normal_honing ? upgrade.upgrade_plus_num + 1 : (upgrade.upgrade_plus_num + 1) * 10} {upgrade.equipment_type}
                                            {upgrade.is_normal_honing && (
                                                <span style={{ marginLeft: '4px', fontSize: 'var(--font-size-xs)' }}>
                                                    {((upgrade.current_artisan ?? 0) * 100).toFixed(0)}%
                                                </span>
                                            )}

                                        </div>
                                    </UpgradeTooltip>
                                )
                            })}
                        </div>
                    </div>

                    {/* Upgrade Info Box and Budget Remaining */}
                    <div style={{ display: "flex", flexDirection: "column", gap: 0 }}>
                        <div style={{ display: "flex", gap: 20 }}>
                            {/* Upgrade Info Box - Always shown */}
                            <div style={{ width: 250, padding: 15, border: '1px solid var(--border-accent)', borderRadius: '8px' }}>
                                <h4 style={{ margin: 0, marginBottom: 10 }}>
                                    {upgradeArr[selectedUpgradeIndex] === undefined ? "" :
                                        upgradeArr[selectedUpgradeIndex].is_normal_honing ? '+' : 'Adv +'}{upgradeArr[selectedUpgradeIndex] === undefined ? "" : upgradeArr[selectedUpgradeIndex].is_normal_honing ? upgradeArr[selectedUpgradeIndex].upgrade_plus_num + 1 : (upgradeArr[selectedUpgradeIndex].upgrade_plus_num + 1) * 10} {upgradeArr[selectedUpgradeIndex] === undefined ? "" : upgradeArr[selectedUpgradeIndex].equipment_type}</h4>
                                {selectedUpgradeIndex !== null && upgradeArr[selectedUpgradeIndex] ? (
                                    <>
                                        {upgradeArr[selectedUpgradeIndex].is_normal_honing ? (
                                            <>
                                                <div>Base Rate: {(upgradeArr[selectedUpgradeIndex].base_chance * 100).toFixed(2)}%</div>
                                                <div>Current Chance: {(calculateCurrentChance(upgradeArr[selectedUpgradeIndex]) * 100).toFixed(2)}%</div>
                                                <div>Artisan: {(upgradeArr[selectedUpgradeIndex].current_artisan * 100).toFixed(2)}%</div>
                                                <div>Trials: {upgradeArr[selectedUpgradeIndex].taps_so_far}</div>
                                                <div>Free Taps: {upgradeArr[selectedUpgradeIndex].free_taps_so_far}</div>
                                            </>
                                        ) : (
                                            <div>Tap Count Range: {upgradeArr[selectedUpgradeIndex].tap_offset} - {upgradeArr[selectedUpgradeIndex].tap_offset + upgradeArr[selectedUpgradeIndex].prob_dist_len}</div>
                                        )}

                                        <div style={{ marginTop: 15, display: "flex", flexDirection: "column", gap: 8 }}>
                                            <label style={{ display: "flex", alignItems: "center", gap: 5 }}>
                                                <input
                                                    type="checkbox"
                                                    checked={upgradeArr[selectedUpgradeIndex]?.use_juice ?? false}
                                                    onChange={(e) => {
                                                        if (selectedUpgradeIndex !== null) {
                                                            setUpgradeArr(prev => {
                                                                const next = prev.slice()
                                                                next[selectedUpgradeIndex] = {
                                                                    ...next[selectedUpgradeIndex],
                                                                    use_juice: e.target.checked
                                                                }
                                                                return next
                                                            })
                                                        }
                                                    }}
                                                />
                                                Use juice
                                            </label>

                                            <button
                                                onClick={attemptTap}
                                                disabled={upgradeArr[selectedUpgradeIndex]?.is_finished || isAutoAttempting || isAutoAttemptingThisOne}
                                                style={{
                                                    padding: '5px 10px',
                                                    opacity: upgradeArr[selectedUpgradeIndex]?.is_finished ? 0.5 : 1
                                                }}
                                            >
                                                {upgradeArr[selectedUpgradeIndex]?.is_finished ? "Success!" : "Tap"}
                                            </button>

                                            <button
                                                onClick={toggleAutoAttemptThisOne}
                                                disabled={upgradeArr[selectedUpgradeIndex]?.is_normal_honing === false || upgradeArr[selectedUpgradeIndex]?.is_finished}
                                                style={{
                                                    padding: '5px 10px',
                                                    backgroundColor: isAutoAttemptingThisOne ? 'var(--error-color)' : 'var(--btn-primary)',
                                                    color: isAutoAttemptingThisOne ? 'white' : 'var(--text-primary)',
                                                    border: isAutoAttemptingThisOne ? '2px solid var(--error-color)' : '1px solid var(--border-accent)',
                                                    fontWeight: isAutoAttemptingThisOne ? 'bold' : 'normal',
                                                    opacity: (upgradeArr[selectedUpgradeIndex]?.is_normal_honing === false || upgradeArr[selectedUpgradeIndex]?.is_finished) ? 0.5 : 1
                                                }}
                                            >
                                                {isAutoAttemptingThisOne ? 'Auto Tapping This...' : 'Auto Tap This One'}
                                            </button>

                                            <button
                                                onClick={toggleAutoAttempt}
                                                disabled={upgradeArr[selectedUpgradeIndex]?.is_normal_honing === false || unfinishedNormalUpgrades.length == 0}
                                                style={{
                                                    padding: '5px 10px',
                                                    backgroundColor: isAutoAttempting ? 'var(--error-color)' : 'var(--btn-primary)',
                                                    color: isAutoAttempting ? 'white' : 'var(--text-primary)',
                                                    border: isAutoAttempting ? '2px solid var(--error-color)' : '1px solid var(--border-accent)',
                                                    fontWeight: isAutoAttempting ? 'bold' : 'normal',
                                                    opacity: upgradeArr[selectedUpgradeIndex]?.is_normal_honing === false ? 0.5 : 1
                                                }}
                                            >
                                                {isAutoAttempting ? 'Auto Tapping...' : 'Auto Tap All'}
                                            </button>

                                            <button
                                                onClick={freeTap}
                                                disabled={upgradeArr[selectedUpgradeIndex]?.is_normal_honing === false || isAutoAttempting || isAutoAttemptingThisOne}
                                                style={{
                                                    padding: '5px 10px',
                                                    opacity: upgradeArr[selectedUpgradeIndex]?.is_normal_honing === false ? 0.5 : 1
                                                }}
                                            >
                                                Free Tap
                                            </button>
                                        </div>
                                    </>
                                ) : (
                                    <div style={{ color: 'var(--text-secondary)', fontStyle: 'italic' }}>
                                        Select an upgrade to begin
                                    </div>
                                )}
                            </div>

                            {/* Budget Grid (Total Cost + Remaining) */}
                            <div style={{ width: 200 }}>
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
                                    title="Budget vs Total Cost Distribution"
                                    labels={OUTPUT_LABELS}
                                    counts={AnythingTicked ? (chance_result?.hist_counts || cachedChanceGraphData?.hist_counts) : null}
                                    mins={chance_result?.hist_mins || cachedChanceGraphData?.hist_mins}
                                    maxs={chance_result?.hist_maxs || cachedChanceGraphData?.hist_maxs}
                                    width={640}
                                    height={320}
                                    budgets={OUTPUT_LABELS.map(label => Number(budget_inputs[label] || 0))}
                                    additionalBudgets={finalCosts.slice(0, 7).map((v, i) => v + (i === 3 ? unlockCosts[0] : i === 6 ? unlockCosts[1] : 0))} // First 7 elements for total costs
                                    hasSelection={AnythingTicked}
                                    isLoading={CostToChanceBusy}
                                    cumulative={cumulativeGraph}
                                />
                            </div>
                        </div>
                    </div>
                </div>
            </div>



        </>
    )
}
