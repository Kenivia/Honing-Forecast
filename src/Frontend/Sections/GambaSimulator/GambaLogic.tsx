import { useCallback, useRef } from "react"
import { Upgrade, getNextUnfinishedIndex, calculateCurrentChance, updateCumulativeChance } from "../../Utils/Helpers.ts"

interface GambaLogicProps {
    upgradeArr: Upgrade[]
    setUpgradeArr: React.Dispatch<React.SetStateAction<Upgrade[]>>
    selectedUpgradeIndex: number | null
    setSelectedUpgradeIndex: React.Dispatch<React.SetStateAction<number | null>>
    finalCosts: number[]
    setFinalCosts: React.Dispatch<React.SetStateAction<number[]>>
    completionCounter: number
    setCompletionCounter: React.Dispatch<React.SetStateAction<number>>
    isAutoAttempting: boolean
    setIsAutoAttempting: React.Dispatch<React.SetStateAction<boolean>>
    isAutoAttemptingThisOne: boolean
    setIsAutoAttemptingThisOne: React.Dispatch<React.SetStateAction<boolean>>
    unfinishedNormalUpgrades: Upgrade[]
    flushUpgradeArrToState: (_immediate?: boolean) => void
}

export function useGambaLogic({
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
}: GambaLogicProps) {
    // Refs for stable references
    const upgradeArrRef = useRef(upgradeArr)
    const selectedUpgradeIndexRef = useRef(selectedUpgradeIndex)
    const finalCostsRef = useRef(finalCosts)
    const isAutoAttemptingRef = useRef(isAutoAttempting)
    const isAutoAttemptingThisOneRef = useRef(isAutoAttemptingThisOne)
    const intervalRef = useRef<number | null>(null)

    // Update refs when state changes
    upgradeArrRef.current = upgradeArr
    selectedUpgradeIndexRef.current = selectedUpgradeIndex
    finalCostsRef.current = finalCosts
    isAutoAttemptingRef.current = isAutoAttempting
    isAutoAttemptingThisOneRef.current = isAutoAttemptingThisOne

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

        // Helper function to perform a single attempt
        const performSingleAttempt = () => {
            let success = false
            let advTapCount = -1;

            if (upgrade.is_normal_honing) {
                const currentChance = calculateCurrentChance(upgrade)
                success = Math.random() < currentChance
            } else {
                // Advanced honing logic - simulate tap count based on probability distribution
                // Use other strategy's probability distribution if juice is ticked
                let probDistToUse = upgrade.prob_dist
                if (upgrade.use_juice && upgrade.other_prob_dist) {
                    probDistToUse = upgrade.other_prob_dist
                }

                // Sample from the probability distribution
                const random = Math.random()
                let cumulativeProb = 0
                let tapIndex = 0
                for (let i = 0; i < probDistToUse.length; i++) {
                    cumulativeProb += probDistToUse[i]
                    if (random <= cumulativeProb) {
                        tapIndex = i
                        break
                    }
                }
                upgrade.cumulative_chance = cumulativeProb

                advTapCount = upgrade.tap_offset + tapIndex
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
                        if (upgrade.is_weapon) next[7] = (next[7] ?? 0) + juiceCost
                        else next[8] = (next[8] ?? 0) + juiceCost
                    }
                }
                else {
                    for (let i = 0; i < 7; i++) {
                        next[i] = (next[i] ?? 0) + (upgrade.costs?.[i] * advTapCount)
                    }
                    if (upgrade.use_juice) {
                        const juiceCost = upgrade.adv_juice_cost[advTapCount - upgrade.tap_offset] ?? 0
                        if (upgrade.is_weapon) next[7] = (next[7] ?? 0) + juiceCost
                        else next[8] = (next[8] ?? 0) + juiceCost
                    }
                }
                finalCostsRef.current = next
                return next
            })

            // Update upgrade counters
            upgrade.taps_so_far = (upgrade.taps_so_far || 0) + 1
            upgrade.juice_taps_so_far = (upgrade.juice_taps_so_far || 0) + (upgrade.use_juice ? 1 : 0)

            // Update cumulative chance for normal honing
            if (upgrade.is_normal_honing) {
                const currentChance = calculateCurrentChance(upgrade)
                updateCumulativeChance(upgrade, currentChance)

                // Increment artisan on failure for normal honing
                if (!success) {
                    upgrade.current_artisan = Math.min(1, (upgrade.current_artisan || 0) + (46.51 / 100.0) * currentChance * (upgrade.artisan_rate ?? 0))
                }
            }

            return { success, advTapCount }
        }

        // Perform attempts until success when isAutoAttempting is true
        let success = false

        if (isAutoAttemptingRef.current && !isAutoAttemptingThisOneRef.current) {
            // Loop until success for auto-attempting all
            do {
                const result = performSingleAttempt()
                success = result.success
            } while (!success)
        } else {
            // Single attempt for manual or "this one" auto-attempting
            const result = performSingleAttempt()
            success = result.success
        }

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

                // const nextUnfinishedIndex = upgradeArrRef.current.findIndex(
                //     (z) => z == sortedUpgrades(upgradeArrRef.current).find((upg, i) =>
                //         !upg.is_finished && i !== selIdx))
                // // console.log("thisone", nextUnfinishedIndex)
                // if (nextUnfinishedIndex !== -1) {
                //     setSelectedUpgradeIndex(nextUnfinishedIndex)
                //     selectedUpgradeIndexRef.current = nextUnfinishedIndex
                // }
            } else if (isAutoAttemptingRef.current) {

                // Move to next unfinished upgrade if auto-attempting all
                const nextUnfinishedIndex = getNextUnfinishedIndex(upgradeArrRef.current, selIdx)
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
            }
            // Note: Manual taps do not automatically move to next upgrade

        }

        // Update the upgrade in the ref array
        upgradeArrRef.current[selIdx] = upgrade

        // If success (user-visible change) flush immediately, else schedule a coalesced update
        if (success) {
            flushUpgradeArrToState(true)
        } else {
            flushUpgradeArrToState(false)
        }
    }, [flushUpgradeArrToState, setFinalCosts, setCompletionCounter, setSelectedUpgradeIndex, setIsAutoAttempting, setIsAutoAttemptingThisOne])

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
    }, [performAttempt, setIsAutoAttempting])

    const stopAuto = useCallback(() => {
        if (intervalRef.current != null) {
            clearTimeout(intervalRef.current)
            intervalRef.current = null
        }
        setIsAutoAttempting(false)
        isAutoAttemptingRef.current = false
        setIsAutoAttemptingThisOne(false)
        isAutoAttemptingThisOneRef.current = false
    }, [setIsAutoAttempting, setIsAutoAttemptingThisOne])

    const toggleAutoAttempt = useCallback(() => {
        if (isAutoAttemptingRef.current) {
            stopAuto()
        } else {
            // Only start auto-attempt if there are unfinished normal honing upgrades
            if (unfinishedNormalUpgrades.length > 0) {
                if (!selectedUpgradeIndex ||
                    !upgradeArr[selectedUpgradeIndex] ||
                    upgradeArr[selectedUpgradeIndex].is_finished) {
                    const nextUnfinishedIndex = getNextUnfinishedIndex(upgradeArrRef.current, selectedUpgradeIndex)
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
    }, [startAuto, stopAuto, selectedUpgradeIndex, upgradeArr, setIsAutoAttemptingThisOne])

    // Free tap
    const freeTap = useCallback(() => {
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
                const updatedUpgrade = {
                    ...next[selectedUpgradeIndex],
                    is_finished: true,
                    completion_order: completionCounter + 1,
                    free_taps_so_far: (next[selectedUpgradeIndex].free_taps_so_far ?? 0) + 1,
                }
                // Update cumulative chance for normal honing (free tap uses base chance)
                if (updatedUpgrade.is_normal_honing) {
                    updateCumulativeChance(updatedUpgrade, updatedUpgrade.base_chance)
                }
                next[selectedUpgradeIndex] = updatedUpgrade
                return next
            })

            setCompletionCounter(prev => prev + 1)
        } else {
            // Failure - still track the free tap
            setUpgradeArr(prev => {
                const next = prev.slice()
                const updatedUpgrade = {
                    ...next[selectedUpgradeIndex],
                    free_taps_so_far: (next[selectedUpgradeIndex].free_taps_so_far ?? 0) + 1
                }
                // Update cumulative chance for normal honing (free tap uses base chance)
                if (updatedUpgrade.is_normal_honing) {
                    updateCumulativeChance(updatedUpgrade, updatedUpgrade.base_chance)
                }
                next[selectedUpgradeIndex] = updatedUpgrade
                return next
            })
        }
    }, [selectedUpgradeIndex, upgradeArr, setFinalCosts, setUpgradeArr, completionCounter, setCompletionCounter])

    return {
        attemptTap,
        toggleAutoAttempt,
        toggleAutoAttemptThisOne,
        freeTap
    }
}
