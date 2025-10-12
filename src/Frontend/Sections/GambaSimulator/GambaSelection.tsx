import React from "react"
import { Upgrade } from "@/Frontend/Utils/Helpers.ts"

interface GambaSelectionProps {
    upgradeArr: Upgrade[]
    sortedWithIndex: number[]
    selectedUpgradeIndex: number | null
    handleUpgradeSelection: (_index: number) => void
    isAutoAttempting: boolean
    isAutoAttemptingThisOne: boolean
    tooltipHandlers: {
        showUpgradeTooltip: (_upgrade: any, _costLabels: string[], _tapRecordCosts: number[], _x: number, _y: number) => void
        hideTooltip: () => void
        updateTooltipPosition: (_x: number, _y: number) => void
    }
}

// Simple wrapper component for upgrade tooltips
type UpgradeTooltipProps = {
    upgrade: Upgrade
    children: React.ReactNode
    tooltipHandlers: GambaSelectionProps['tooltipHandlers']
}

const UpgradeTooltip = React.memo(function UpgradeTooltip({ upgrade, children, tooltipHandlers }: UpgradeTooltipProps) {
    const costLabels = ['Red', 'Blue', 'Leaps', 'Shards', 'Oreha', "Gold", 'Silver', 'Red Juice', 'Blue Juice', 'Special Leaps']
    const tapRecordCosts = React.useMemo(() => {
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
        costs[9] = upgrade.special_cost * freeTaps

        return costs
    }, [upgrade.taps_so_far, upgrade.juice_taps_so_far, upgrade.free_taps_so_far, upgrade.costs, upgrade.one_juice_cost, upgrade.special_cost, upgrade.is_weapon])

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

export default function GambaSelection({
    upgradeArr,
    sortedWithIndex,
    selectedUpgradeIndex,
    handleUpgradeSelection,
    isAutoAttempting,
    isAutoAttemptingThisOne,
    tooltipHandlers
}: GambaSelectionProps) {
    return (
        <div style={{ width: 170 }}>
            <style>
                {`
                    .upgrade-scroll-container::-webkit-scrollbar {
                        width: 6px;
                    }
                    .upgrade-scroll-container::-webkit-scrollbar-track {
                        background: var(--background-secondary);
                        border-radius: 3px;
                    }
                    .upgrade-scroll-container::-webkit-scrollbar-thumb {
                        background: var(--border-accent);
                        border-radius: 3px;
                    }
                    .upgrade-scroll-container::-webkit-scrollbar-thumb:hover {
                        background: var(--accent-color);
                    }
                `}
            </style>
            <h4 style={{ margin: 0, fontSize: 'var(--font-size-sm)', marginBottom: 10 }}>
                Upgrades: {(isAutoAttempting || isAutoAttemptingThisOne) && <span style={{ color: 'var(--error-color)', fontSize: 'var(--font-size-xs)' }}>AUTO ON</span>}
            </h4>
            <div
                className="upgrade-scroll-container"
                style={{
                    display: "flex",
                    flexDirection: "column",
                    gap: 2,
                    width: 170,
                    maxHeight: "800px", // Cap at approximately 10 items (8px padding + 2px gap per item)
                    overflowY: "auto",
                    paddingRight: "4px" // Add some padding for the scrollbar
                }}>
                {sortedWithIndex.map(originalIndex => {
                    const upgrade = upgradeArr[originalIndex]
                    return (
                        <UpgradeTooltip key={originalIndex} upgrade={upgrade} tooltipHandlers={tooltipHandlers}>
                            <div
                                onClick={() => handleUpgradeSelection(originalIndex)}
                                style={{
                                    padding: '8px',
                                    border: selectedUpgradeIndex === originalIndex ? '2px solid var(--selected-blue)' : '1px solid var(--border-accent)',
                                    borderRadius: '4px',
                                    cursor: 'pointer',
                                    backgroundColor: 'transparent',
                                    fontSize: 'var(--font-size-sm)',
                                    position: 'relative',
                                    animation: (isAutoAttempting || isAutoAttemptingThisOne) && selectedUpgradeIndex === originalIndex ? 'pulse 1s infinite' : 'none',
                                    color: upgrade.is_finished ? 'var(--btn-success)' : 'var(--text-primary)'
                                }}
                            >
                                {upgrade.is_normal_honing ? '+' : 'Adv +'}{upgrade.is_normal_honing ? upgrade.upgrade_plus_num + 1 : (upgrade.upgrade_plus_num + 1) * 10} {upgrade.equipment_type}
                                {upgrade.is_normal_honing && (
                                    <span style={{ marginLeft: '4px', fontSize: 'var(--font-size-xs)' }}>
                                        {((upgrade.current_artisan ?? 0) * 100).toFixed(0)}% Artisan
                                    </span>
                                )}

                            </div>
                        </UpgradeTooltip>
                    )
                })}
            </div>
        </div>
    )
}
