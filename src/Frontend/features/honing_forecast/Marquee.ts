import { TOP_ROWS, TOP_COLS, BOTTOM_ROWS, BOTTOM_COLS } from "./constants.ts"

export function coordsToCell(ref: HTMLDivElement | null, clientX: number, clientY: number, rows: number, cols: number) {
    if (!ref) return { r: 0, c: 0 }
    const rect = ref.getBoundingClientRect()
    const x = Math.max(0, Math.min(clientX - rect.left, rect.width - 1))
    const y = Math.max(0, Math.min(clientY - rect.top, rect.height - 1))
    let cellW = rect.width / cols
    let cellH = rect.height / rows
    const c = Math.floor(x / cellW)
    const r = Math.floor(y / cellH)
    return { r: Math.max(0, Math.min(rows - 1, r)), c: Math.max(0, Math.min(cols - 1, c)) }
}

export function GridMouseDownLogic({
    topGridRef,
    bottomGridRef,
    marqueeRef,
    topGrid,
    bottomGrid,
    setMarquee,
}: {
    topGridRef: React.RefObject<HTMLDivElement | null>
    bottomGridRef: React.RefObject<HTMLDivElement | null>
    marqueeRef: React.MutableRefObject<any>
    topGrid: boolean[][]
    bottomGrid: boolean[][]
    setMarquee: React.Dispatch<React.SetStateAction<any>>
}) {
    return (grid: "top" | "bottom", e: React.MouseEvent) => {
        e.preventDefault()
        const ref = grid === "top" ? topGridRef.current : bottomGridRef.current
        const rows = grid === "top" ? TOP_ROWS : BOTTOM_ROWS
        const cols = grid === "top" ? TOP_COLS : BOTTOM_COLS
        const { r, c } = coordsToCell(ref, e.clientX, e.clientY, rows, cols)
        const initialState = grid === "top" ? topGrid[r][c] : bottomGrid[r][c]
        const next = {
            active: true,
            grid,
            startR: r,
            startC: c,
            endR: r,
            endC: c,
            startClientX: e.clientX,
            startClientY: e.clientY,
            endClientX: e.clientX,
            endClientY: e.clientY,
            initialState,
        }
        setMarquee(next)
        marqueeRef.current = next
    }
}

export function mouseMoveLogic(
    ev: MouseEvent,
    marqueeRef: React.MutableRefObject<any>,
    topGridRef: React.RefObject<HTMLDivElement | null>,
    bottomGridRef: React.RefObject<HTMLDivElement | null>,
    setMarquee: React.Dispatch<React.SetStateAction<any>>
) {
    const m = marqueeRef.current
    if (!m || !m.active || !m.grid) return
    const grid = m.grid
    const ref = grid === "top" ? topGridRef.current : bottomGridRef.current
    const rows = grid === "top" ? TOP_ROWS : BOTTOM_ROWS
    const cols = grid === "top" ? TOP_COLS : BOTTOM_COLS
    const { r, c } = coordsToCell(ref, ev.clientX, ev.clientY, rows, cols)
    setMarquee((prev: any) => ({ ...prev, endR: r, endC: c, endClientX: ev.clientX, endClientY: ev.clientY }))
}

export function createMouseUpHandler({
    marqueeRef,
    topGridRef,
    bottomGridRef,
    setTopGrid,
    setBottomGrid,
    set_prev_checked_arr,
    set_prev_checked_arr_bottom,
    setMarquee,
}: {
    marqueeRef: React.MutableRefObject<any>
    topGridRef: React.RefObject<HTMLDivElement | null>
    bottomGridRef: React.RefObject<HTMLDivElement | null>
    setTopGrid: React.Dispatch<React.SetStateAction<any>>
    setBottomGrid: React.Dispatch<React.SetStateAction<any>>
    set_prev_checked_arr: React.Dispatch<React.SetStateAction<boolean[]>>
    set_prev_checked_arr_bottom: React.Dispatch<React.SetStateAction<boolean[]>>
    setMarquee: React.Dispatch<React.SetStateAction<any>>
}) {
    return (ev: MouseEvent) => {
        const m = marqueeRef.current
        if (!m || !m.active || !m.grid) return
        const { grid, startClientX, startClientY, endClientX, endClientY, initialState } = m
        const ref = grid === "top" ? topGridRef.current : bottomGridRef.current
        const rows = grid === "top" ? TOP_ROWS : BOTTOM_ROWS
        const cols = grid === "top" ? TOP_COLS : BOTTOM_COLS
        const startCell = coordsToCell(ref, startClientX, startClientY, rows, cols)
        const endCell = coordsToCell(ref, endClientX || ev.clientX, endClientY || ev.clientY, rows, cols)
        const r1 = Math.min(startCell.r, endCell.r)
        const r2 = Math.max(startCell.r, endCell.r)
        const c1 = Math.min(startCell.c, endCell.c)
        const c2 = Math.max(startCell.c, endCell.c)
        const setter = grid === "top" ? setTopGrid : setBottomGrid
        setter((prev: any) => {
            const copy = prev.map((row: any) => row.slice())
            const newState = !initialState
            for (let rr = r1; rr <= r2; rr++) {
                for (let cc = c1; cc <= c2; cc++) {
                    if (rr < copy.length && cc < copy[rr].length) copy[rr][cc] = newState
                }
            }

            // Update column headers based on new state
            if (grid === "top") {
                set_prev_checked_arr((prev) => {
                    const newArr = [...prev]
                    for (let cc = c1; cc <= c2; cc++) {
                        let allChecked = true
                        for (let rr = 0; rr < rows; rr++) {
                            if (!copy[rr][cc]) {
                                allChecked = false
                                break
                            }
                        }
                        newArr[cc] = allChecked
                    }
                    return newArr
                })
            } else {
                set_prev_checked_arr_bottom((prev) => {
                    const newArr = [...prev]
                    for (let cc = c1; cc <= c2; cc++) {
                        let allChecked = true
                        for (let rr = 0; rr < rows; rr++) {
                            if (!copy[rr][cc]) {
                                allChecked = false
                                break
                            }
                        }
                        newArr[cc] = allChecked
                    }
                    return newArr
                })
            }

            return copy
        })
        setMarquee({
            active: false,
            grid: null,
            startR: 0,
            startC: 0,
            endR: 0,
            endC: 0,
            startClientX: 0,
            startClientY: 0,
            endClientX: 0,
            endClientY: 0,
            initialState: false,
        })
    }
}
