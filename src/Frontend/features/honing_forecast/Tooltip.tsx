import React from 'react'

export type TooltipType = 'separator' | 'upgrade'

export interface TooltipState {
    visible: boolean
    type: TooltipType | null
    x: number
    y: number
    content: string | null
    upgradeData?: {
        upgrade: any
        costLabels: string[]
        tapRecordCosts: number[]
    } | null
}

export function createTooltipHandlers(
    setTooltip: React.Dispatch<React.SetStateAction<TooltipState>>
) {
    const showSeparatorTooltip = (text: string, x: number, y: number) => {
        setTooltip({
            visible: true,
            type: 'separator',
            x: x + 10,
            y: y - 10,
            content: text,
            upgradeData: null
        })
    }

    const showUpgradeTooltip = (
        upgrade: any,
        costLabels: string[],
        tapRecordCosts: number[],
        x: number,
        y: number
    ) => {
        setTooltip({
            visible: true,
            type: 'upgrade',
            x: x + 10,
            y: y,
            content: null,
            upgradeData: {
                upgrade,
                costLabels,
                tapRecordCosts
            }
        })
    }

    const hideTooltip = () => {
        setTooltip(prev => ({ ...prev, visible: false }))
    }

    const updateTooltipPosition = (x: number, y: number) => {
        setTooltip(prev => ({
            ...prev,
            x: x + 10,
            y: y - 10
        }))
    }

    return {
        showSeparatorTooltip,
        showUpgradeTooltip,
        hideTooltip,
        updateTooltipPosition
    }
}

export function renderTooltip(tooltip: TooltipState, mainScale: number = 1, zoomCompensation: number = 1) {
    if (!tooltip.visible) return null

    // Combine main scale and zoom compensation for final tooltip scale
    const finalScale = mainScale * zoomCompensation

    const tooltipStyle: React.CSSProperties = {
        position: 'fixed',
        left: tooltip.x,
        top: tooltip.y,
        backgroundColor: 'var(--bg-tertiary)',
        color: 'var(--text-primary)',
        padding: tooltip.type === 'upgrade' ? '12px 16px' : '8px 12px',
        borderRadius: 'var(--border-radius-small)',
        fontSize: 'var(--font-size-sm)',
        fontWeight: 'var(--font-weight-normal)',
        border: '1px solid var(--border-secondary)',
        boxShadow: '0 4px 12px rgba(0, 0, 0, 0.3)',
        zIndex: 10000,
        pointerEvents: 'none',
        opacity: tooltip.visible ? 1 : 0,
        transition: 'opacity 0.1s ease-in-out',
        minWidth: tooltip.type === 'upgrade' ? '300px' : 'auto',
        maxWidth: tooltip.type === 'upgrade' ? '400px' : 'auto',
        whiteSpace: tooltip.type === 'separator' ? 'nowrap' : 'normal',
        transform: `scale(${finalScale})`,
        transformOrigin: 'top left'
    }

    if (tooltip.type === 'separator') {
        return (
            <div style={tooltipStyle}>
                {tooltip.content}
            </div>
        )
    }

    if (tooltip.type === 'upgrade' && tooltip.upgradeData) {
        const { upgrade, costLabels, tapRecordCosts } = tooltip.upgradeData

        const leftColumnStyle: React.CSSProperties = {
            display: 'flex',
            flexDirection: 'column',
            gap: '4px',
            marginRight: '16px',
            width: '35%',
            minWidth: '120px'
        }

        const rightColumnStyle: React.CSSProperties = {
            display: 'flex',
            flexDirection: 'column',
            gap: '2px',
            width: '65%',
            minWidth: '180px'
        }

        const containerStyle: React.CSSProperties = {
            display: 'flex',
            flexDirection: 'row'
        }

        return (
            <div style={tooltipStyle}>
                <div style={containerStyle}>
                    <div style={leftColumnStyle}>
                        <div><strong>{upgrade.is_normal_honing ? '+' : 'Adv +'}{upgrade.upgrade_plus_num + (upgrade.is_normal_honing ? 1 : 0)} {upgrade.equipment_type}</strong></div>

                        {upgrade.is_finished && upgrade.current_artisan && (
                            <div>Artisan: {(upgrade.current_artisan * 100).toFixed(2)}%</div>
                        )}
                        <div>Taps: {upgrade.taps_so_far ?? 0}</div>
                        <div>Juice Taps: {upgrade.juice_taps_so_far ?? 0}</div>
                        <div>Free Taps: {upgrade.free_taps_so_far ?? 0}</div>
                        {upgrade.is_normal_honing && upgrade.is_finished && (
                            <div>In a room of 100 people, you are less lucky than {((upgrade.cumulative_chance || 0) * 100).toFixed(0)} of them.</div>
                        )}
                    </div>
                    <div style={rightColumnStyle}>
                        <div><strong>Costs so far(excluding unlock):</strong></div>
                        {costLabels.map((label, index) => (
                            <div key={index}>
                                {label}: {tapRecordCosts[index].toLocaleString()}
                            </div>
                        ))}
                    </div>
                </div>
            </div>
        )
    }

    return null
}
