import React from "react"
import { Upgrade, calculateCurrentChance, getTapCountRange } from "../../Utils/Helpers.ts"

interface GambaInfoBoxProps {
    upgradeArr: Upgrade[]
    selectedUpgradeIndex: number | null
    setUpgradeArr: React.Dispatch<React.SetStateAction<Upgrade[]>>
    attemptTap: () => void
    toggleAutoAttemptThisOne: () => void
    toggleAutoAttempt: () => void
    freeTap: () => void
    isAutoAttempting: boolean
    isAutoAttemptingThisOne: boolean
    unfinishedNormalUpgrades: Upgrade[]
    adv_hone_strategy: string
}

export default function GambaInfoBox({
    upgradeArr,
    selectedUpgradeIndex,
    setUpgradeArr,
    attemptTap,
    toggleAutoAttemptThisOne,
    toggleAutoAttempt,
    freeTap,
    isAutoAttempting,
    isAutoAttemptingThisOne,
    unfinishedNormalUpgrades,
    adv_hone_strategy
}: GambaInfoBoxProps) {
    const selectedUpgrade = selectedUpgradeIndex !== null ? upgradeArr[selectedUpgradeIndex] : null
    const isFinished = selectedUpgrade?.is_finished ?? false

    return (
        <div style={{
            width: 300,
            padding: 15,
            border: '1px solid var(--border-accent)',
            borderRadius: '8px',
            boxShadow: isFinished ? '0 0 10px var(--bright-green-glow), 0 0 20px var(--bright-green-glow)' : 'none'
        }}>
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
                            <div>For this upgrade, in a room of 100 people, you are luckier than {(100 - (upgradeArr[selectedUpgradeIndex].cumulative_chance || 0) * 100).toFixed(0)} of them.</div>
                        </>
                    ) : (
                        (() => {
                            const rangeInfo = getTapCountRange(upgradeArr[selectedUpgradeIndex])
                            return rangeInfo ? (
                                <>
                                    <div>
                                        Tap Count Range: {rangeInfo.range}
                                    </div>
                                    <div>{"For this upgrade, in a room of 100 people, you are luckier than " + (100 - upgradeArr[selectedUpgradeIndex].cumulative_chance * 100).toFixed(0) + " of them."}</div>


                                </>
                            ) : null
                        })()
                    )}

                    <div style={{ marginTop: 15, display: "flex", flexDirection: "column", gap: 8 }}>
                        <label style={{ display: "flex", alignItems: "center", gap: 5 }}>
                            <input
                                type="checkbox"
                                title="This mode does not automatically use your juice, and you may go into juice debt, nobody can stop you"
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
                        {/* <div style={{ marginTop: -10, paddingLeft: 20, fontSize: "var(--font-size-xs)" }}> {(Number(budgetRemainingData["Red juice"]) > 0 && upgradeArr[selectedUpgradeIndex]?.is_weapon) || (Number(budgetRemainingData["Blue juice"]) > 0 && !upgradeArr[selectedUpgradeIndex]?.is_weapon) ? "This mode does not automatically use your juice, and you may go into juice debt, nobody can stop you" : ""}</div> */}
                        <div> {!upgradeArr[selectedUpgradeIndex]?.is_normal_honing && (upgradeArr[selectedUpgradeIndex]?.use_juice != (adv_hone_strategy === "Juice on grace")) ? "(Graph below is not updated by this tick)" : ""}</div>
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
                            disabled={isAutoAttempting || upgradeArr[selectedUpgradeIndex]?.is_finished || !upgradeArr[selectedUpgradeIndex]?.is_normal_honing}
                            style={{
                                padding: '5px 10px',
                                backgroundColor: isAutoAttemptingThisOne ? 'var(--error-color)' : 'var(--btn-primary)',
                                color: isAutoAttemptingThisOne ? 'white' : 'var(--text-primary)',
                                border: isAutoAttemptingThisOne ? '2px solid var(--error-color)' : '1px solid var(--border-accent)',
                                fontWeight: isAutoAttemptingThisOne ? 'bold' : 'normal',
                                opacity: (isAutoAttempting || upgradeArr[selectedUpgradeIndex]?.is_finished || !upgradeArr[selectedUpgradeIndex]?.is_normal_honing) ? 0.5 : 1
                            }}
                        >
                            {isAutoAttemptingThisOne ? 'Auto Tapping This...' : 'Auto Tap This One'}
                        </button>

                        <button
                            onClick={toggleAutoAttempt}
                            disabled={unfinishedNormalUpgrades.length <= 1}
                            style={{
                                padding: '5px 10px',
                                backgroundColor: isAutoAttempting ? 'var(--error-color)' : 'var(--btn-primary)',
                                color: isAutoAttempting ? 'white' : 'var(--text-primary)',
                                border: isAutoAttempting ? '2px solid var(--error-color)' : '1px solid var(--border-accent)',
                                fontWeight: isAutoAttempting ? 'bold' : 'normal',
                                opacity: unfinishedNormalUpgrades.length <= 1 ? 0.5 : 1
                            }}
                        >
                            {isAutoAttempting ? 'Auto Tapping...' : 'Auto Tap All'}
                        </button>

                        <button
                            onClick={freeTap}
                            disabled={upgradeArr[selectedUpgradeIndex]?.is_normal_honing === false || upgradeArr[selectedUpgradeIndex]?.is_finished || isAutoAttempting || isAutoAttemptingThisOne}
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
    )
}
