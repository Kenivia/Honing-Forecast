/*eslint no-unused-vars: 0*/
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

export default function CheckboxGrid({ grid, rows, cols, gridRef, onGridMouseDown, marquee, CELL_W, CELL_H, gridName }: Props) {
    return (
        <div ref={gridRef as any} onMouseDown={(e) => onGridMouseDown(gridName, e)} style={{ display: 'grid', gridTemplateColumns: `repeat(${cols}, ${CELL_W}px)`, gap: 0 }}>
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
                        <div key={key} title={`+${(gridName) === "top" ? c + 1 : (c + 1) * 10}`} style={{ width: CELL_W, height: CELL_H, display: 'inline-flex', alignItems: 'center', justifyContent: 'center', margin: 0 }}>
                            <input
                                type="checkbox"
                                checked={inMarquee ? !marquee.initialState : checked}
                                readOnly
                                style={{ width: 26, height: 26, margin: 0, padding: 0, border: '1px solid #bbb', boxSizing: 'border-box' }}
                            />
                        </div>
                    )
                })
            )}
        </div>
    )
}
