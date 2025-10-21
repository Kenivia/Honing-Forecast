import React, { useState, useRef, useEffect, useCallback } from "react"
import Icon from "./Icon.tsx"
import { ColumnDef } from "../Utils/Styles.ts"

interface SpreadsheetGridProps {
    columnDefs: ColumnDef[]
    labels: string[]
    // sheet values per-column. index matches columnDefs index
    sheetValuesArr: Record<string, string>[]
    // setters per-column. Optional for read-only columns; index matches columnDefs
    setSheetValuesArr: Array<((_next: Record<string, string>) => void) | undefined>
    hideIcons?: boolean
}

interface Selection {
    startRow: number
    startCol: number
    endRow: number
    endCol: number
}

export default function SpreadsheetGrid({ columnDefs, labels, sheetValuesArr, setSheetValuesArr, hideIcons = false }: SpreadsheetGridProps) {
    const [selection, setSelection] = useState<Selection | null>(null)
    const [isSelecting, setIsSelecting] = useState(false)
    const [_copiedData, setCopiedData] = useState<string[][] | null>(null)
    const gridRef = useRef<HTMLDivElement | null>(null)

    // pointer-down tracking so we can detect small drags vs clicks when starting in an input
    const pointerDownRef = useRef<{
        startX: number
        startY: number
        startRow: number
        startCol: number
        startedOnInput: boolean
        moved: boolean
    } | null>(null)

    // keep old body user-select so we can restore it after drag
    const prevUserSelectRef = useRef<string | undefined>(undefined)

    // const labels = useMemo(() => labels, [])

    // ---------- helpers ----------
    const clamp = (v: number, max: number) => Math.min(Math.max(v, 0), max)
    const clampCol = useCallback((v: number) => Math.min(Math.max(v, 0), Math.max(0, columnDefs.length - 1)), [columnDefs.length])

    const isCellSelected = (rowIndex: number, colIndex: number) => {
        if (!selection) return false
        const { startRow, startCol, endRow, endCol } = selection
        const minRow = Math.min(startRow, endRow)
        const maxRow = Math.max(startRow, endRow)
        const minCol = Math.min(startCol, endCol)
        const maxCol = Math.max(startCol, endCol)

        return rowIndex >= minRow && rowIndex <= maxRow && colIndex >= minCol && colIndex <= maxCol
    }

    const handleCellChange = (rowIndex: number, colIndex: number, value: string) => {
        if (!columnDefs[colIndex].editable) return

        const label = labels[rowIndex]
        if (!label) return

        // Get current column values and setter
        const colValues = sheetValuesArr[colIndex] ?? {}
        const setter = setSheetValuesArr[colIndex]

        if (!setter) return // read-only column

        if (colIndex === 0) {
            // Budget column - only allow non-negative integers, strip leading zeros
            let cleanValue = value.replace(/[^0-9]/g, "")
            cleanValue = cleanValue.replace(/^0+(?=\d)/, "")
            if (cleanValue.length > 10) {
                cleanValue = "999999999"
            }
            const next = { ...colValues }
            next[label] = cleanValue
            setter(next)
        } else if (colIndex === 1) {
            // Gold value-like column - allow non-negative decimals while typing
            let clean = value.replace(/[^0-9.]/g, "")
            const firstDot = clean.indexOf(".")
            if (firstDot !== -1) {
                clean = clean.slice(0, firstDot + 1) + clean.slice(firstDot + 1).replace(/\./g, "")
            }
            if (clean.length > 20) clean = clean.slice(0, 20)
            const next = { ...colValues }
            next[label] = clean
            setter(next)
        } else {
            // Generic editable column: store raw string
            const next = { ...colValues }
            next[label] = value
            setter(next)
        }
    }

    const handleCellBlur = (rowIndex: number, colIndex: number) => {
        const label = labels[rowIndex]
        if (!label) return
        // Only normalize decimal-like second column
        if (colIndex === 1) {
            const colValues = sheetValuesArr[1] ?? {}
            const setter = setSheetValuesArr[1]
            if (!setter) return
            let val = colValues[label] ?? ""
            // normalize on blur: strip leading zeros in int part, trailing zeros in frac; drop dot if needed
            let clean = String(val).replace(/[^0-9.]/g, "")
            const firstDot = clean.indexOf(".")
            let hadDot = firstDot !== -1
            if (firstDot !== -1) {
                clean = clean.slice(0, firstDot + 1) + clean.slice(firstDot + 1).replace(/\./g, "")
            }
            let intPart = hadDot ? clean.slice(0, clean.indexOf(".")) : clean
            let fracPart = hadDot ? clean.slice(clean.indexOf(".") + 1) : ""
            intPart = intPart.replace(/^0+(?=\d)/, "")
            if (intPart === "" && (fracPart !== "" || hadDot)) intPart = "0"
            fracPart = fracPart.replace(/0+$/g, "")
            let normalized = intPart
            if (fracPart.length > 0) normalized += "." + fracPart
            if (normalized.length > 20) normalized = normalized.slice(0, 20)
            if (normalized !== (colValues[label] ?? "")) {
                const next = { ...colValues }
                next[label] = normalized
                setter(next)
            }
        }
    }

    // ---------- capture mousedown (so drags that begin inside inputs are detected) ----------
    useEffect(() => {
        const grid = gridRef.current
        if (!grid) return

        const onMouseDownCapture = (e: MouseEvent) => {
            // only handle left button drags
            if (e.button !== 0) return

            // find an ancestor cell that contains data-row
            const target = e.target as HTMLElement | null
            const cell = target?.closest("[data-row]") as HTMLElement | null
            if (!cell) return

            const rowIndex = clamp(Number(cell.dataset.row), labels.length - 1)
            const colIndex = clampCol(Number(cell.dataset.col ?? "0"))

            pointerDownRef.current = {
                startX: e.clientX,
                startY: e.clientY,
                startRow: rowIndex,
                startCol: colIndex,
                startedOnInput: !!target?.closest("input,textarea"),
                moved: false,
            }

            setIsSelecting(true)
            if (selection) {
                setSelection(null)
            } else {
                setSelection({
                    startRow: rowIndex,
                    startCol: colIndex,
                    endRow: rowIndex,
                    endCol: colIndex,
                })
            }
            // DO NOT call e.preventDefault() here — we want a plain click (no drag) to still focus the input for editing.
        }

        grid.addEventListener("mousedown", onMouseDownCapture, true) // capture phase
        return () => grid.removeEventListener("mousedown", onMouseDownCapture, true)
    }, [labels.length, clampCol, selection]) // reattach if ref changes

    // ---------- mousemove + mouseup to update selection when dragging ----------
    useEffect(() => {
        const onMouseMove = (e: MouseEvent) => {
            const pd = pointerDownRef.current
            if (!pd) return

            const dx = Math.abs(e.clientX - pd.startX)
            const dy = Math.abs(e.clientY - pd.startY)
            const movedNow = dx > 5 || dy > 5 // small threshold = click vs drag

            if (movedNow && !pd.moved) {
                pd.moved = true
                // if user started the interaction inside an input, blur + disable user-select to stop text-select
                if (pd.startedOnInput) {
                    prevUserSelectRef.current = document.body.style.userSelect
                    try {
                        document.body.style.userSelect = "none"
                    } catch {
                        // Ignore errors
                    }
                    // blur active element to avoid text caret/selection interfering with drag
                    if (document.activeElement instanceof HTMLElement) {
                        ;(document.activeElement as HTMLElement).blur()
                    }
                }
            }

            // determine cell under pointer
            const el = document.elementFromPoint(e.clientX, e.clientY) as HTMLElement | null
            const cell = el?.closest("[data-row]") as HTMLElement | null
            if (cell) {
                const row = clamp(Number(cell.dataset.row), labels.length - 1)
                const col = clampCol(Number(cell.dataset.col ?? "0"))
                setSelection((prev) =>
                    prev
                        ? { ...prev, endRow: row, endCol: col }
                        : {
                              startRow: row,
                              startCol: col,
                              endRow: row,
                              endCol: col,
                          }
                )
            }
        }

        const onMouseUp = () => {
            // restore `user-select` if we changed it
            if (pointerDownRef.current?.moved && prevUserSelectRef.current !== undefined) {
                try {
                    document.body.style.userSelect = prevUserSelectRef.current
                } catch {
                    // Ignore errors
                }
                prevUserSelectRef.current = undefined
            }
            pointerDownRef.current = null
            setIsSelecting(false)
        }

        document.addEventListener("mousemove", onMouseMove)
        document.addEventListener("mouseup", onMouseUp)
        return () => {
            document.removeEventListener("mousemove", onMouseMove)
            document.removeEventListener("mouseup", onMouseUp)
        }
    }, [labels.length, clampCol])

    // ---------- native copy / paste handlers using system clipboard ----------
    useEffect(() => {
        const onCopy = (e: ClipboardEvent) => {
            // if editing an input/textarea, let native copy happen (don't override)
            const active = document.activeElement as HTMLElement | null
            if (active && (active.tagName === "INPUT" || active.tagName === "TEXTAREA")) return

            if (!selection) return

            const { startRow, startCol, endRow, endCol } = selection
            const minRow = Math.min(startRow, endRow)
            const maxRow = Math.max(startRow, endRow)
            const minCol = Math.min(startCol, endCol)
            const maxCol = Math.max(startCol, endCol)

            const rowsOut: string[] = []
            for (let r = minRow; r <= maxRow; r++) {
                const cols: string[] = []
                for (let c = minCol; c <= maxCol; c++) {
                    const label = labels[r]
                    cols.push((sheetValuesArr[c]?.[label] ?? "") === "" ? "0" : sheetValuesArr[c]?.[label] ?? "")
                }
                rowsOut.push(cols.join("\t")) // tab separated per row
            }
            const text = rowsOut.join("\n")
            if (e.clipboardData) {
                e.clipboardData.setData("text/plain", text)
                e.preventDefault()
            } else if ((window as any).clipboardData) {
                // IE fallback (unlikely needed)
                ;(window as any).clipboardData.setData("Text", text)
                e.preventDefault()
            }
            // store for internal paste if needed
            setCopiedData(rowsOut.map((r) => r.split("\t")))
        }

        const onPaste = (e: ClipboardEvent) => {
            // if editing an input/textarea, let native paste happen
            // const active = document.activeElement as HTMLElement | null
            // if (active && (active.tagName === 'INPUT' || active.tagName === 'TEXTAREA')) return

            const text = e.clipboardData?.getData("text/plain") ?? ""
            if (!text) return

            // parse text into tokens separated by newline or tabs/commas/spaces
            const parsedTokens = text
                .split(/\n/) // split rows first
                .map((r) =>
                    r
                        .split(/\t|,|\s+/)
                        .map((c) => c.trim())
                        .filter(Boolean)
                )

            // determine paste start: use selection start if present, otherwise use focused cell
            let startRow = selection?.startRow ?? 0
            let startCol = selection?.startCol ?? 0
            if (!selection) {
                const focused = document.activeElement as HTMLElement | null
                const cell = focused?.closest("[data-row]") as HTMLElement | null
                if (cell) {
                    startRow = clamp(Number(cell.dataset.row), 0)
                    startCol = clampCol(Number(cell.dataset.col ?? "0"))
                }
            }

            // For simplicity, paste tokens column-wise into the startCol only (common UX)
            if (!columnDefs[startCol].editable) return

            // copy existing column object, or create new
            const colValues = { ...(sheetValuesArr[startCol] ?? {}) }
            for (let r = 0; r < parsedTokens.length; r++) {
                const tokenRow = parsedTokens[r]
                if (tokenRow.length === 0) continue
                const cellToken = tokenRow[0] // take first cell of each pasted row for single-column paste
                const targetRow = startRow + r
                if (targetRow < labels.length) {
                    let cleanValue = cellToken.trim().replace(/[^0-9]/g, "")
                    cleanValue = cleanValue.replace(/^0+(?=\d)/, "")
                    if (cleanValue.length > 10) cleanValue = "999999999"
                    colValues[labels[targetRow]] = cleanValue
                }
            }
            const setter = setSheetValuesArr[startCol]
            if (setter) setter(colValues)
            e.preventDefault()
        }

        document.addEventListener("copy", onCopy)
        document.addEventListener("paste", onPaste)
        return () => {
            document.removeEventListener("copy", onCopy)
            document.removeEventListener("paste", onPaste)
        }
    }, [selection, sheetValuesArr, labels, setSheetValuesArr, columnDefs, clampCol]) // re-register when selection or inputs change

    // ---------- optional grid-level keyboard handler (keeps existing behavior when grid has focus) ----------
    const handleKeyDown = (_e: React.KeyboardEvent) => {
        // keep earlier copy/paste handling as a fallback if you want it, but
        // we primarily rely on native clipboard events above.
        // if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'a') {
        //     // Example: select all
        //     e.preventDefault()
        //     setSelection({
        //         startRow: 0, startCol: 0, endRow: labels.length - 1, endCol: clampCol(columnDefs.length - 1)
        //     })
        //     return
        // }
        // Clear selected editable cells to 0 on Backspace/Delete for editable grids
        //
    }

    // ---------- render ----------
    return (
        <div
            ref={gridRef}
            onKeyDown={handleKeyDown}
            tabIndex={0}
            style={{
                display: "flex",
                padding: 6,
                outline: "none",
                minHeight: "200px",
            }}
        >
            {!hideIcons && (
                <div style={{ flex: 1, width: 50 }}>
                    {[""].concat(labels).map((lab) => (
                        <div
                            key={lab}
                            style={{
                                height: 36,
                                color: "var(--text-secondary)",
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "flex-end",
                                paddingRight: 8,
                                whiteSpace: "nowrap",
                                fontSize: "var(--font-size-sm)",
                                paddingTop: 8,
                            }}
                        >
                            <Icon iconName={lab} size={28} />
                        </div>
                    ))}
                </div>
            )}

            <div style={{ flex: 1 }}>
                {/* Column headers (plain text, aligned) */}
                <div style={{ display: "grid", gridTemplateColumns: columnDefs.map(() => "1fr").join(" "), gap: 0, marginBottom: 4, height: 36 }}>
                    {columnDefs.map((colDef, colIndex) => (
                        <div
                            key={`hdr-${colIndex}`}
                            style={{
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "center",
                                color: "var(--text-primary)",
                                textWrap: "wrap",
                                height: colDef.headerName.length > 12 ? 30 : 15,
                                textAlign: "center",
                                fontWeight: 500,
                                fontSize: "var(--font-size-md)",
                                marginLeft: colDef.headerName.length > 12 ? -8 : -12,
                                marginTop: colDef.headerName.length > 12 ? 0 : 18,
                            }}
                        >
                            {colDef.headerName}
                        </div>
                    ))}
                </div>

                <div style={{ display: "grid", gridTemplateColumns: columnDefs.map(() => "1fr").join(" "), gap: 0 }}>
                    {labels.map((label, rowIndex) =>
                        columnDefs.map((colDef, colIndex) => (
                            <div
                                key={`${label}-${colIndex}`}
                                data-row={rowIndex}
                                data-col={colIndex}
                                style={{ height: 36, display: "flex", alignItems: "center" }}
                                onMouseEnter={() => {
                                    if (isSelecting && pointerDownRef.current) {
                                        setSelection((prev) =>
                                            prev
                                                ? { ...prev, endRow: rowIndex, endCol: colIndex }
                                                : {
                                                      startRow: rowIndex,
                                                      startCol: colIndex,
                                                      endRow: rowIndex,
                                                      endCol: colIndex,
                                                  }
                                        )
                                    }
                                }}
                            >
                                <input
                                    type="text"
                                    readOnly={!columnDefs[colIndex].editable || (colIndex === 1 && rowIndex >= 7)}
                                    value={sheetValuesArr[colIndex]?.[label] ?? ""}
                                    onChange={(e) => handleCellChange(rowIndex, colIndex, e.target.value)}
                                    onKeyDown={(e) => {
                                        e.stopPropagation()
                                    }}
                                    onBlur={() => handleCellBlur(rowIndex, colIndex)}
                                    onFocus={() => {
                                        setSelection({
                                            startRow: rowIndex,
                                            startCol: colIndex,
                                            endRow: rowIndex,
                                            endCol: colIndex,
                                        })
                                    }}
                                    style={{
                                        width: "100px",
                                        height: "100%",
                                        padding: "6px 8px",
                                        border: "1px solid var(--border-accent)",
                                        background:
                                            columnDefs[colIndex].backgroundRanOut && parseInt(sheetValuesArr[colIndex][label]) < 0
                                                ? columnDefs[colIndex].backgroundRanOut
                                                : isCellSelected(rowIndex, colIndex)
                                                ? columnDefs[colIndex].backgroundSelected
                                                : columnDefs[colIndex].background,
                                        color: columnDefs[colIndex].color,
                                        fontSize: "var(--font-size-sm)",
                                        outline: "none",
                                        boxSizing: "border-box",
                                        cursor: !columnDefs[colIndex].editable || (colIndex === 1 && rowIndex >= 7) ? "default" : "text",
                                        opacity: colIndex >= 1 && rowIndex >= 7 ? 0.5 : 1,
                                    }}
                                    placeholder="0"
                                />
                            </div>
                        ))
                    )}
                </div>
            </div>
        </div>
    )
}
