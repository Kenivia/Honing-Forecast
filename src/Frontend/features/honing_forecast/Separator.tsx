import React, { useState, useEffect } from 'react'

type SeparatorProps = {
    activePage: 'chance-to-cost' | 'cost-to-chance' | 'gamba'
    onPageChange: (_page: 'chance-to-cost' | 'cost-to-chance' | 'gamba') => void
}

export default function Separator({ activePage, onPageChange }: SeparatorProps) {
    const [clickedButton, setClickedButton] = useState<string | null>(activePage)

    // Update clickedButton when activePage changes
    useEffect(() => {
        setClickedButton(activePage)
    }, [activePage])
    const buttonStyle = (isSelected: boolean) => ({
        backgroundColor: isSelected ? 'var(--btn-toggle-selected)' : 'var(--btn-toggle-deselected)',
        color: '#ffffff',
        border: '2px',
        padding: '6px 12px',
        borderRadius: 'var(--border-radius-small)',
        cursor: 'pointer',
        fontSize: '22px',
        fontWeight: "var(--font-weight-medium)",
        transition: 'all 0.2s ease',
        opacity: isSelected ? 1 : 0.6,
        paddingTop: "10px",
        textAlign: 'left' as const,
        display: 'flex',
        flexDirection: 'column' as const,
        alignItems: 'flex-start',
        height: '44px',
    })

    const helpTextStyle = {
        fontSize: '14px',
        fontWeight: 'normal',
        marginTop: '4px',
        whiteSpace: 'nowrap' as const,
        textAlign: 'left' as const,
    }

    const containerStyle = {
        height: '44px',
        display: 'flex',
        alignItems: 'flex-start',
        gap: 'var(--spacing-xl)',
        borderBottom: '4px solid var(--btn-toggle-selected)',
        marginTop: '0px',
        marginBottom: '0px',
        paddingTop: '8px',
        paddingBottom: '60px',
    }

    const handleButtonClick = (page: 'chance-to-cost' | 'cost-to-chance' | 'gamba') => {
        onPageChange(page)
        // Always show help text for the clicked button, don't toggle off
        setClickedButton(page)
    }

    return (
        <div style={containerStyle}>
            <button
                style={buttonStyle(activePage === 'chance-to-cost')}
                onClick={() => handleButtonClick('chance-to-cost')}
            >
                Chance mode
                {clickedButton === 'chance-to-cost' && (
                    <div style={helpTextStyle}>
                        I want to have x% chance to pass, how much mats will I need?
                    </div>
                )}
            </button>
            <button
                style={buttonStyle(activePage === 'cost-to-chance')}
                onClick={() => handleButtonClick('cost-to-chance')}
            >
                Budget mode
                {clickedButton === 'cost-to-chance' && (
                    <div style={helpTextStyle}>
                        I have this much mats, what are my odds of success?
                    </div>
                )}
            </button>
            <button
                style={buttonStyle(activePage === 'gamba')}
                onClick={() => handleButtonClick('gamba')}
            >
                Gamba simulator
                {clickedButton === 'gamba' && (
                    <div style={helpTextStyle}>
                        Try your luck
                    </div>
                )}
            </button>
        </div>
    )
}
