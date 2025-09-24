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

export default function CheckboxGrid({ grid, rows, cols, gridRef, onGridMouseDown, marquee, CELL_W, CELL_H, gridName }: Props) {
    const handleTouchStart = (e: React.TouchEvent) => {
        e.preventDefault();
        const mouseEvent = createMouseEventFromTouch(e, 'mousedown');
        onGridMouseDown(gridName, mouseEvent);
    };

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
