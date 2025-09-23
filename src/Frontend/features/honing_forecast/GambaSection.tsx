import React, { useState, useEffect, useRef, useMemo, useCallback } from "react"
import SpreadsheetGrid from "../../components/SpreadsheetGrid.tsx"
import { styles, createColumnDefs } from "./styles.ts"
import { BOTTOM_COLS, INPUT_LABELS, TOP_COLS } from "./constants.ts"
import { SpawnWorker } from "../../worker_setup.ts"
import { buildPayload } from "./Debounce.ts"

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
    artisan: number
}

interface TapRecord {
    name: string
    taps: number
    costs: number[]
}

interface UpgradeState {
    artisan: number
    trialsSoFar: number
    useJuice: boolean
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
}

// Equipment types for armor pieces
const EQUIPMENT_TYPES = ['Helmet', 'Shoulder', 'Chest', 'Pants', 'Gloves', 'Weapon']

function calculateCurrentChance(upgrade: Upgrade, _artisan: number, trialsSoFar, useJuice) {
    if (!upgrade.is_normal_honing) return 0
    const baseChance = upgrade.base_chance
    const minCount = Math.min(trialsSoFar, 10)
    const currentChance = baseChance + (baseChance / 10) * minCount
    return _artisan >= 1 ? 1 : (useJuice ? currentChance + upgrade.base_chance : currentChance)
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
}: GambaSectionProps) {
    const { costToChanceColumnDefs } = createColumnDefs(true)

    // State management
    const [upgradeArr, setUpgradeArr] = useState<Upgrade[]>([])
    const [selectedUpgradeIndex, setSelectedUpgradeIndex] = useState<number | null>(null)
    const [finalCosts, setFinalCosts] = useState<number[]>(new Array(10).fill(0))
    const [artisan, setArtisan] = useState<number>(0)
    const [trialsSoFar, setTrialsSoFar] = useState<number>(0)
    const [useJuice, setUseJuice] = useState<boolean>(false)
    const [tapRecords, setTapRecords] = useState<TapRecord[]>([])
    const [upgradeStates, setUpgradeStates] = useState<UpgradeState[]>([])
    const [isAutoAttempting, setIsAutoAttempting] = useState<boolean>(false)
    const [freeTaps, setFreeTaps] = useState<number>(0)

    // Worker refs and debounce
    const parserWorkerRef = useRef<Worker | null>(null)
    const debounceTimerRef = useRef<number | null>(null)
    const autoAttemptIntervalRef = useRef<number | null>(null)
    const currentUpgradeArrRef = useRef<Upgrade[]>([])
    const currentSelectedIndexRef = useRef<number | null>(null)
    const selectedUpgradeIndexRef = useRef(selectedUpgradeIndex)
    const upgradeArrRef = useRef(upgradeArr)
    const upgradeStatesRef = useRef(upgradeStates)
    const finalCostsRef = useRef(finalCosts)
    const useJuiceRef = useRef(useJuice)
    const artisanRef = useRef(artisan)
    const trialsSoFarRef = useRef(trialsSoFar)
    const tapRecordsRef = useRef(tapRecords)
    const isAutoAttemptingRef = useRef(isAutoAttempting)



    const intervalRef = useRef<number | null>(null)
    // sync refs whenever state changes
    useEffect(() => { selectedUpgradeIndexRef.current = selectedUpgradeIndex }, [selectedUpgradeIndex])
    useEffect(() => { upgradeArrRef.current = upgradeArr }, [upgradeArr])
    useEffect(() => { upgradeStatesRef.current = upgradeStates }, [upgradeStates])
    useEffect(() => { finalCostsRef.current = finalCosts }, [finalCosts])
    useEffect(() => { useJuiceRef.current = useJuice }, [useJuice])
    useEffect(() => { artisanRef.current = artisan }, [artisan])
    useEffect(() => { trialsSoFarRef.current = trialsSoFar }, [trialsSoFar])
    useEffect(() => { tapRecordsRef.current = tapRecords }, [tapRecords])
    useEffect(() => { isAutoAttemptingRef.current = isAutoAttempting }, [isAutoAttempting])
    // Initialize parser worker
    useEffect(() => {
        // Create a persistent worker for parser calls
        parserWorkerRef.current = new Worker(new URL("../../js_to_wasm.ts", import.meta.url), { type: "module" })
        return () => {
            if (parserWorkerRef.current) {
                parserWorkerRef.current.terminate()
            }
            // Clean up auto-attempt interval
            if (autoAttemptIntervalRef.current) {
                clearInterval(autoAttemptIntervalRef.current)
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

        parserWorkerRef.current.postMessage({
            id,
            payload,
            which_one: "ParserUnified"
        })

        parserWorkerRef.current.onmessage = (e) => {
            if (e.data.type === "result" && e.data.id === id) {
                const upgrades = e.data.result.upgrades as Upgrade[]

                // Add equipment types to upgrades based on ticked equipment in grids
                let seen_ind_normal = Array.from({ length: TOP_COLS }, () => 0);
                let seen_ind_adv = Array.from({ length: BOTTOM_COLS }, () => 0);
                let upgradesWithTypes = upgrades.map(upgrade => {
                    if (!upgrade.is_normal_honing) {
                        upgrade = { ...upgrade, upgrade_plus_num: upgrade.upgrade_plus_num * 10, }
                    }

                    if (upgrade.is_weapon) {
                        return { ...upgrade, equipment_type: 'Weapon' }
                    } else {
                        // For armor, find which equipment types are ticked in the grid
                        const equipmentTypes = EQUIPMENT_TYPES.slice(0, 5) // Exclude Weapon
                        // const tickedTypes = equipmentTypes.filter((_, index) => {
                        //     // Check if this equipment type is ticked in the grid at the upgrade level
                        //     const gridRow = topGrid[index] || []
                        //     return gridRow[upgrade.upgrade_plus_num] || false
                        // })

                        // // Assign equipment type based on what's ticked
                        // if (tickedTypes.length > 0) {
                        //     // Find the first ticked equipment type for this upgrade level
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

                setUpgradeArr(upgradesWithTypes)
                // Initialize upgrade states array
                setUpgradeStates(upgradesWithTypes.map(() => ({
                    artisan: 0,
                    trialsSoFar: 0,
                    useJuice: false
                })))
                // Update refs
                currentUpgradeArrRef.current = upgradesWithTypes
            }
        }
    }, [topGrid, bottomGrid, adv_hone_strategy, express_event, desired_chance, bucketCount, autoOptimization, userMatsValue, dataSize, budget_inputs])

    // Debounce effect for parser calls when grids change
    const topGridKey = useMemo(() => JSON.stringify(topGrid), [topGrid])
    const bottomGridKey = useMemo(() => JSON.stringify(bottomGrid), [bottomGrid])
    const advStrategyKey = useMemo(() => String(adv_hone_strategy), [adv_hone_strategy])
    const expressEventKey = useMemo(() => String(express_event), [express_event])

    useEffect(() => {
        // Clear existing timer
        if (debounceTimerRef.current) {
            window.clearTimeout(debounceTimerRef.current)
            debounceTimerRef.current = null
        }

        // Clear tap records and final costs when grids or strategy change
        setTapRecords([])
        setFinalCosts(new Array(10).fill(0))
        setArtisan(0)
        setTrialsSoFar(0)
        setSelectedUpgradeIndex(null)
        setFreeTaps(0)

        // Stop auto-attempting when grids change
        if (autoAttemptIntervalRef.current) {
            clearInterval(autoAttemptIntervalRef.current)
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
    }, [topGridKey, bottomGridKey, advStrategyKey, expressEventKey, callParser])

    // Keep refs updated
    useEffect(() => {
        currentUpgradeArrRef.current = upgradeArr
    }, [upgradeArr])

    useEffect(() => {
        currentSelectedIndexRef.current = selectedUpgradeIndex
    }, [selectedUpgradeIndex])

    // Save current upgrade state
    const saveCurrentUpgradeState = useCallback(() => {
        if (selectedUpgradeIndex !== null && upgradeStates[selectedUpgradeIndex]) {
            const newStates = [...upgradeStates]
            newStates[selectedUpgradeIndex] = {
                artisan,
                trialsSoFar,
                useJuice
            }
            setUpgradeStates(newStates)
        }
    }, [selectedUpgradeIndex, upgradeStates, artisan, trialsSoFar, useJuice])

    // Restore upgrade state when selecting a different upgrade
    const restoreUpgradeState = useCallback((index: number) => {
        if (upgradeStates[index]) {
            const state = upgradeStates[index]
            setArtisan(state.artisan)
            setTrialsSoFar(state.trialsSoFar)
            setUseJuice(state.useJuice)
        } else {
            setArtisan(0)
            setTrialsSoFar(0)
            setUseJuice(false)
        }
    }, [upgradeStates])

    // Handle upgrade selection
    const handleUpgradeSelection = useCallback((index: number) => {
        // Save current state before switching
        saveCurrentUpgradeState()

        // Select new upgrade
        setSelectedUpgradeIndex(index)

        // Restore state for new upgrade
        restoreUpgradeState(index)
    }, [saveCurrentUpgradeState, restoreUpgradeState])

    // Calculate current chance for normal honing


    // Calculate costs for tap record
    const calculateTapRecordCosts = (upgrade: Upgrade, taps: number, juiceTaps: number, freeTaps: number) => {
        const costs = new Array(10).fill(0)

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
    const performAttempt = useCallback(() => {
        const selIdx = selectedUpgradeIndexRef.current
        const arr = upgradeArrRef.current
        if (selIdx === null || !arr || !arr[selIdx]) {
            // nothing to attempt — stop auto attempting if running
            if (intervalRef.current != null) {
                clearInterval(intervalRef.current)
                intervalRef.current = null
                setIsAutoAttempting(false)
                isAutoAttemptingRef.current = false
            }
            return
        }

        const upgrade = arr[selIdx]
        const currentChance = calculateCurrentChance(upgrade, artisanRef.current ?? 0, trialsSoFarRef.current, useJuiceRef.current)
        const success = Math.random() < currentChance

        // Add costs to final_costs (functional update + keep ref in sync)
        setFinalCosts(prev => {
            const next = prev.slice()
            for (let i = 0; i < 7; i++) {
                next[i] = (next[i] ?? 0) + (upgrade.costs?.[i] ?? 0)
            }
            if (useJuiceRef.current) {
                const juiceCost = upgrade.one_juice_cost ?? 0
                if (upgrade.is_weapon) next[8] = (next[8] ?? 0) + juiceCost
                else next[7] = (next[7] ?? 0) + juiceCost
            }
            finalCostsRef.current = next
            return next
        })

        if (success) {
            // success: build tap record
            const totalTaps = trialsSoFarRef.current + 1
            const juiceTaps = useJuiceRef.current ? totalTaps : 0
            const tapRecordCosts = calculateTapRecordCosts(upgrade, totalTaps, juiceTaps, 0)

            const tapRecord: TapRecord = {
                name: `${upgrade.is_normal_honing ? '+' : 'Adv +'}${upgrade.upgrade_plus_num + (upgrade.is_normal_honing ? 1 : 0)} ${upgrade.equipment_type}`,
                taps: totalTaps,
                costs: tapRecordCosts
            }

            setTapRecords(prev => {
                const next = [...prev, tapRecord]
                tapRecordsRef.current = next
                return next
            })

            // remove this upgrade from arrays (use functional updates, update refs)
            setUpgradeArr(prev => {
                const next = prev.filter((_, i) => i !== selIdx)
                upgradeArrRef.current = next
                return next
            })
            setUpgradeStates(prev => {
                const next = prev.filter((_, i) => i !== selIdx)
                upgradeStatesRef.current = next
                return next
            })

            // reset per-your-original logic
            setTrialsSoFar(0); trialsSoFarRef.current = 0
            setArtisan(0); artisanRef.current = 0
            setUseJuice(false); useJuiceRef.current = false

            // clear selection (you kept info box visible in your original; set to null or decide otherwise)
            setSelectedUpgradeIndex(null); selectedUpgradeIndexRef.current = null

        } else {
            // failure: increment trials and artisan (functional + update refs immediately)
            setTrialsSoFar(prev => {
                const next = prev + 1
                trialsSoFarRef.current = next
                return next
            })

            setArtisan(prev => {
                let newArtisan = prev + (46.51 / 100.0) * currentChance * (upgrade.artisan_rate ?? 0)
                if (newArtisan >= 1) newArtisan = 1
                artisanRef.current = newArtisan
                return newArtisan
            })
        }
    }, []) // stable identity — uses refs internally
    // Attempt a tap
    const attemptTap = useCallback(() => performAttempt(), [performAttempt])


    // Toggle auto-attempt mode
    const startAuto = useCallback((ms = 10) => {
        if (intervalRef.current != null) return
        // use window.setInterval for TS int typing
        intervalRef.current = window.setInterval(() => {
            performAttempt()
        }, ms)
        setIsAutoAttempting(true)
        isAutoAttemptingRef.current = true
    }, [performAttempt])
    const stopAuto = useCallback(() => {
        if (intervalRef.current != null) {
            clearInterval(intervalRef.current)
            intervalRef.current = null
        }
        setIsAutoAttempting(false)
        isAutoAttemptingRef.current = false
    }, [])

    const toggleAutoAttempt = useCallback(() => {
        if (isAutoAttemptingRef.current) stopAuto()
        else startAuto(10) // 10ms per your request
    }, [startAuto, stopAuto])

    // Free tap
    const freeTap = () => {
        if (selectedUpgradeIndex === null || !upgradeArr[selectedUpgradeIndex]) return

        const upgrade = upgradeArr[selectedUpgradeIndex]
        const success = Math.random() < upgrade.base_chance

        // Increment free tap counter
        setFreeTaps(prevFreeTaps => prevFreeTaps + 1)

        // Add special cost to 10th element
        const newFinalCosts = [...finalCosts]
        newFinalCosts[9] += upgrade.special_cost
        setFinalCosts(newFinalCosts)

        if (success) {
            // Success - add to tap records and remove from upgrade array
            const totalTaps = trialsSoFar
            const juiceTaps = useJuice ? totalTaps : 0
            const currentFreeTaps = freeTaps + 1
            const tapRecordCosts = calculateTapRecordCosts(upgrade, totalTaps, juiceTaps, currentFreeTaps)

            const tapRecord: TapRecord = {
                name: `Free ${upgrade.is_normal_honing ? '+' : 'Adv +'}${upgrade.upgrade_plus_num + (upgrade.is_normal_honing ? 1 : 0)} ${upgrade.equipment_type}`,
                taps: totalTaps,
                costs: tapRecordCosts
            }
            setTapRecords([...tapRecords, tapRecord])

            // Remove upgrade from array and its state
            const newUpgradeArr = upgradeArr.filter((_, index) => index !== selectedUpgradeIndex)
            const newUpgradeStates = upgradeStates.filter((_, index) => index !== selectedUpgradeIndex)
            setUpgradeArr(newUpgradeArr)
            setUpgradeStates(newUpgradeStates)

            // Keep the info box visible but reset state
            setTrialsSoFar(0)
            setArtisan(0)
            setUseJuice(false)
            setFreeTaps(0)
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
        acc[label] = finalCosts[index].toString()
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
                    backgroundColor: value < 0 ? 'var(--error-color)' : 'transparent',
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


                    {/* Upgrade Selection Grid */}
                    <div style={{ width: 200 }}>
                        <h4 style={{ margin: 0, fontSize: 'var(--font-size-sm)', marginBottom: 10 }}>
                            Upgrades: {isAutoAttempting && <span style={{ color: 'var(--error-color)', fontSize: 'var(--font-size-xs)' }}>🔄 AUTO</span>}
                        </h4>
                        <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
                            {upgradeArr.map((upgrade, index) => (
                                <div
                                    key={index}
                                    onClick={() => handleUpgradeSelection(index)}
                                    style={{
                                        padding: '8px',
                                        border: selectedUpgradeIndex === index ? '2px solid var(--accent-color)' : '1px solid var(--border-accent)',
                                        borderRadius: '4px',
                                        cursor: 'pointer',
                                        backgroundColor: selectedUpgradeIndex === index ? 'var(--marquee-bg)' : 'transparent',
                                        fontSize: 'var(--font-size-sm)',
                                        position: 'relative',
                                        animation: isAutoAttempting && selectedUpgradeIndex === index ? 'pulse 1s infinite' : 'none'
                                    }}
                                >
                                    {upgrade.is_normal_honing ? '+' : 'Adv +'}{upgrade.upgrade_plus_num + (upgrade.is_normal_honing ? 1 : 0)} {upgrade.equipment_type}
                                    {isAutoAttempting && selectedUpgradeIndex === index && (
                                        <span style={{
                                            position: 'absolute',
                                            right: '4px',
                                            top: '4px',
                                            fontSize: '10px',
                                            color: 'var(--error-color)'
                                        }}>
                                            🔄
                                        </span>
                                    )}
                                </div>
                            ))}
                        </div>
                    </div>

                    {/* Upgrade Info Box and Budget Remaining */}
                    <div style={{ display: "flex", gap: 20 }}>
                        {/* Upgrade Info Box - Always shown */}
                        <div style={{ width: 250, padding: 15, border: '1px solid var(--border-accent)', borderRadius: '8px' }}>
                            <h4 style={{ margin: 0, marginBottom: 10 }}>Upgrade Info</h4>
                            {selectedUpgradeIndex !== null && upgradeArr[selectedUpgradeIndex] ? (
                                <>
                                    {upgradeArr[selectedUpgradeIndex].is_normal_honing ? (
                                        <>
                                            <div>Base Rate: {(upgradeArr[selectedUpgradeIndex].base_chance * 100).toFixed(2)}%</div>
                                            <div>Current Chance: {(calculateCurrentChance(upgradeArr[selectedUpgradeIndex], artisan, trialsSoFar, useJuice) * 100).toFixed(2)}%</div>
                                            <div>Artisan: {(artisan * 100).toFixed(2)}%</div>
                                            <div>Trials: {trialsSoFar}</div>
                                            <div>Free Taps: {freeTaps}</div>
                                        </>
                                    ) : (
                                        <div>Tap Count Range: {upgradeArr[selectedUpgradeIndex].tap_offset} - {upgradeArr[selectedUpgradeIndex].tap_offset + upgradeArr[selectedUpgradeIndex].prob_dist_len}</div>
                                    )}

                                    <div style={{ marginTop: 15, display: "flex", flexDirection: "column", gap: 8 }}>
                                        <label style={{ display: "flex", alignItems: "center", gap: 5 }}>
                                            <input
                                                type="checkbox"
                                                checked={useJuice}
                                                onChange={(e) => setUseJuice(e.target.checked)}
                                            />
                                            Use juice
                                        </label>

                                        <button
                                            onClick={attemptTap}
                                            disabled={!upgradeArr[selectedUpgradeIndex]?.is_normal_honing}
                                            style={{ padding: '5px 10px', opacity: upgradeArr[selectedUpgradeIndex]?.is_normal_honing ? 1 : 0.5 }}
                                        >
                                            Tap
                                        </button>

                                        <button
                                            onClick={toggleAutoAttempt}
                                            style={{
                                                padding: '5px 10px',
                                                backgroundColor: isAutoAttempting ? 'var(--error-color)' : 'var(--btn-primary)',
                                                color: isAutoAttempting ? 'white' : 'var(--text-primary)',
                                                border: isAutoAttempting ? '2px solid var(--error-color)' : '1px solid var(--border-accent)',
                                                fontWeight: isAutoAttempting ? 'bold' : 'normal'
                                            }}
                                        >
                                            {isAutoAttempting ? '🔄 Auto-Attempting...' : '▶️ Auto-Attempt All'}
                                        </button>

                                        <button onClick={freeTap} style={{ padding: '5px 10px' }}>
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
                </div>
            </div>


            {/* Tap Records */}
            <div style={{ marginTop: 20 }}>
                <h4 style={{ margin: 0, marginBottom: 10 }}>Tap Records:</h4>
                <div style={{ display: "flex", flexDirection: "column", gap: 5 }}>
                    {tapRecords.map((record, index) => (
                        <div key={index} style={{ fontSize: 'var(--font-size-sm)' }}>
                            {record.name}: {record.taps} taps, Costs: [{record.costs.join(', ')}]
                        </div>
                    ))}
                </div>
            </div>
        </>
    )
}
