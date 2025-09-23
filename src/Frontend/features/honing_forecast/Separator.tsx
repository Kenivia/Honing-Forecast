import React, { useState, useEffect, useRef } from 'react'

type SeparatorProps = {
    activePage: 'chance-to-cost' | 'cost-to-chance' | 'gamba'
    onPageChange: (_page: 'chance-to-cost' | 'cost-to-chance' | 'gamba') => void
}

type TooltipProps = {
    text: string
    children: React.ReactNode
}

function CustomTooltip({ text, children }: TooltipProps) {
    const [isVisible, setIsVisible] = useState(false)
    const [position, setPosition] = useState({ x: 0, y: 0 })
    const elementRef = useRef<HTMLDivElement>(null)

    const handleMouseEnter = () => {
        setIsVisible(true)
    }

    const handleMouseMove = (e: MouseEvent) => {
        if (elementRef.current && elementRef.current.contains(e.target as Node)) {
            setPosition({
                x: e.clientX + 10, // Offset to the right of cursor
                y: e.clientY - 10  // Offset above cursor
            })
        }
    }

    const handleMouseLeave = () => {
        setIsVisible(false)
    }

    useEffect(() => {
        if (isVisible) {
            document.addEventListener('mousemove', handleMouseMove)
            return () => document.removeEventListener('mousemove', handleMouseMove)
        }
    }, [isVisible])

    const tooltipStyle: React.CSSProperties = {
        position: 'fixed',
        left: position.x,
        top: position.y,
        backgroundColor: 'var(--bg-tertiary)',
        color: 'var(--text-primary)',
        padding: '8px 12px',
        borderRadius: 'var(--border-radius-small)',
        fontSize: 'var(--font-size-sm)',
        fontWeight: 'var(--font-weight-normal)',
        border: '1px solid var(--border-secondary)',
        boxShadow: '0 4px 12px rgba(0, 0, 0, 0.3)',
        zIndex: 10000,
        pointerEvents: 'none',
        whiteSpace: 'nowrap',
        opacity: isVisible ? 1 : 0,
        transition: 'opacity 0.1s ease-in-out',
    }

    return (
        <>
            <div
                ref={elementRef}
                onMouseEnter={handleMouseEnter}
                onMouseLeave={handleMouseLeave}
            >
                {children}
            </div>
            {isVisible && (
                <div style={tooltipStyle}>
                    {text}
                </div>
            )}
        </>
    )
}

export default function Separator({ activePage, onPageChange }: SeparatorProps) {
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
            <CustomTooltip text="Calculate the cost required to achieve a desired success chance">
                <button
                    style={buttonStyle(activePage === 'chance-to-cost')}
                    onClick={() => onPageChange('chance-to-cost')}
                >
                    Chance mode
                </button>
            </CustomTooltip>
            <CustomTooltip text="Calculate the success chance for a given budget">
                <button
                    style={buttonStyle(activePage === 'cost-to-chance')}
                    onClick={() => onPageChange('cost-to-chance')}
                >
                    Budget mode
                </button>
            </CustomTooltip>
            <CustomTooltip text="Simulate gambling scenarios with different strategies">
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
