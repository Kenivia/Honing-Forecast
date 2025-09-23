import React from 'react'

type SeparatorProps = {
    activePage: 'chance-to-cost' | 'cost-to-chance' | 'gamba'
    onPageChange: (_page: 'chance-to-cost' | 'cost-to-chance' | 'gamba') => void
    tooltipHandlers: {
        showSeparatorTooltip: (_text: string, _x: number, _y: number) => void
        hideTooltip: () => void
        updateTooltipPosition: (_x: number, _y: number) => void
    }
}

type TooltipProps = {
    text: string
    children: React.ReactNode
    tooltipHandlers: SeparatorProps['tooltipHandlers']
}

function CustomTooltip({ text, children, tooltipHandlers }: TooltipProps) {
    const handleMouseEnter = (e: React.MouseEvent) => {
        tooltipHandlers.showSeparatorTooltip(text, e.clientX, e.clientY)
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
}

export default function Separator({ activePage, onPageChange, tooltipHandlers }: SeparatorProps) {
    const buttonStyle = (isSelected: boolean) => ({
        backgroundColor: isSelected ? 'var(--btn-toggle-selected)' : 'var(--btn-toggle-deselected)',
        color: isSelected ? '#ffffff' : '#ffffff',
        border: '2px',
        padding: '6px 12px',
        borderRadius: 'var(--border-radius-small)',
        cursor: 'pointer',
        fontSize: '22px',
        fontWeight: "var(--font-weight-medium)",
        transition: 'all 0.2s ease',
        opacity: isSelected ? 1 : 0.6,
        paddingTop: "10px",
    })

    const containerStyle = {
        height: '44px',
        display: 'flex',
        alignItems: 'center',
        gap: 'var(--spacing-xl)',
        borderBottom: '4px solid var(--btn-toggle-selected)',
        marginTop: '0px',
        marginBottom: '0px',
    }

    return (
        <div style={containerStyle}>
            <CustomTooltip text="Calculate the cost required to achieve a desired success chance" tooltipHandlers={tooltipHandlers}>
                <button
                    style={buttonStyle(activePage === 'chance-to-cost')}
                    onClick={() => onPageChange('chance-to-cost')}
                >
                    Chance mode
                </button>
            </CustomTooltip>
            <CustomTooltip text="Calculate the success chance for a given budget" tooltipHandlers={tooltipHandlers}>
                <button
                    style={buttonStyle(activePage === 'cost-to-chance')}
                    onClick={() => onPageChange('cost-to-chance')}
                >
                    Budget mode
                </button>
            </CustomTooltip>
            <CustomTooltip text="Simulate gambling scenarios with different strategies" tooltipHandlers={tooltipHandlers}>
                <button
                    style={buttonStyle(activePage === 'gamba')}
                    onClick={() => onPageChange('gamba')}
                >
                    Gamba simulator
                </button>
            </CustomTooltip>
        </div>
    )
}
