/*eslint no-unused-vars: 0*/
import './CheckboxGrid.css';

type Props = {
    grid: boolean[][]
    rows: number
    cols: number
    gridRef: React.RefObject<HTMLDivElement | null>
    onGridMouseDown: (grid: 'top' | 'bottom', e: React.MouseEvent) => void
    marquee: any
    CELL_W: number
    CELL_H: number
    gridName: 'top' | 'bottom'   // NEW required prop
    useGridInput?: boolean
    numericInputs?: number[][]
    onNumericInputChange?: (grid: 'top' | 'bottom', row: number, col: number, value: number) => void
}

// Helper function to convert touch event to mouse event
function createMouseEventFromTouch(touchEvent: React.TouchEvent, type: 'mousedown' | 'mousemove' | 'mouseup'): React.MouseEvent {
    const touch = touchEvent.touches[0] || touchEvent.changedTouches[0];
    return {
        ...touchEvent,
        type,
        clientX: touch.clientX,
        clientY: touch.clientY,
        button: 0, // Left mouse button
        buttons: type === 'mousedown' ? 1 : (type === 'mouseup' ? 0 : 1),
        preventDefault: touchEvent.preventDefault.bind(touchEvent),
        stopPropagation: touchEvent.stopPropagation.bind(touchEvent),
    } as unknown as React.MouseEvent;
}

export default function CheckboxGrid({ grid, rows, cols, gridRef, onGridMouseDown, marquee, CELL_W, CELL_H, gridName, useGridInput = true, numericInputs, onNumericInputChange }: Props) {
    const handleTouchStart = (e: React.TouchEvent) => {
        // Only prevent default behavior for grid interactions
        e.preventDefault();
        const mouseEvent = createMouseEventFromTouch(e, 'mousedown');
        onGridMouseDown(gridName, mouseEvent);
    };

    if (!useGridInput && numericInputs && onNumericInputChange) {
        // Render numeric input mode - only show inputs for "Gloves" row (index 4) and "Weapon" row (index 5)
        return (
            <div
                ref={gridRef as any}
                style={{ display: 'grid', gridTemplateColumns: `repeat(${cols}, ${CELL_W}px)`, gap: 0 }}
            >
                {grid.flatMap((row, r) =>
                    row.map((checked, c) => {
                        const key = `${r}-${c}`

                        // Only show inputs for Gloves row (4) and Weapon row (5)
                        if (r === 4 || r === 5) {
                            const inputValue = numericInputs[r === 4 ? 0 : 1][c] || 0
                            return (
                                <div key={key} className="checkbox-grid-item" style={{ width: CELL_W, height: CELL_H }}>
                                    <input
                                        type="number"
                                        min="0"
                                        value={inputValue === 0 ? '' : inputValue}
                                        onChange={(e) => {
                                            const inputVal = e.target.value
                                            // Allow empty string, interpret as 0
                                            const value = inputVal === '' ? 0 : Math.max(0, parseInt(inputVal) || 0)
                                            onNumericInputChange(gridName, r, c, value)
                                        }}
                                        className="numeric-grid-input"
                                        placeholder='0'
                                        style={{
                                            width: '100%',
                                            height: '100%',
                                            border: '1px solid var(--input-border)',
                                            borderRadius: '2px',
                                            background: 'var(--input-bg)',
                                            color: 'var(--input-text)',
                                            textAlign: 'center',
                                            fontSize: '16px',
                                            padding: '0',
                                            margin: '0',
                                            boxSizing: 'border-box',
                                            overflow: 'hidden',
                                            textOverflow: 'ellipsis'
                                        }}
                                    />
                                </div>
                            )
                        } else {
                            // Empty cells for other rows
                            return (
                                <div key={key} className="checkbox-grid-item" style={{ width: CELL_W, height: CELL_H }}>
                                    {/* Empty cell */}
                                </div>
                            )
                        }
                    })
                )}
            </div>
        )
    }

    // Render checkbox mode (default)
    return (
        <div
            ref={gridRef as any}
            onMouseDown={(e) => onGridMouseDown(gridName, e)}
            onTouchStart={handleTouchStart}
            style={{ display: 'grid', gridTemplateColumns: `repeat(${cols}, ${CELL_W}px)`, gap: 0 }}
        >
            {grid.flatMap((row, r) =>
                row.map((checked, c) => {
                    const key = `${r}-${c}`

                    // show marquee inversion only for the owning grid
                    let inMarquee = false
                    if (marquee.active && marquee.grid === gridName) {
                        const r1 = Math.min(marquee.startR, marquee.endR)
                        const r2 = Math.max(marquee.startR, marquee.endR)
                        const c1 = Math.min(marquee.startC, marquee.endC)
                        const c2 = Math.max(marquee.startC, marquee.endC)
                        inMarquee = r >= r1 && r <= r2 && c >= c1 && c <= c2
                    }

                    return (
                        <div key={key} title={`+${(gridName) === "top" ? c + 1 : (c + 1) * 10}`} className="checkbox-grid-item" style={{ width: CELL_W, height: CELL_H }}>
                            <input
                                type="checkbox"
                                readOnly
                                checked={inMarquee ? !marquee.initialState : checked}
                                className="checkbox-grid-input"
                            />
                        </div>
                    )
                })
            )}
        </div>
    )
}
