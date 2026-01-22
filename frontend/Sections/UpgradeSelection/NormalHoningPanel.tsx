import React from "react"
import { styles } from "@/Utils/Styles.ts"
import CheckboxGrid from "@/Components/CheckboxGrid.tsx"
import Icon from "@/Components/Icon.tsx"
import { TOP_ROWS, TOP_COLS, CELL_W, CELL_H } from "@/Utils/Constants.ts"

type NormalHoningPanelProps = {
    topGrid: boolean[][]
    setTopGrid: React.Dispatch<React.SetStateAction<any>>
    prev_checked_arr: boolean[]
    set_prev_checked_arr: React.Dispatch<React.SetStateAction<boolean[]>>
    topGridRef: React.RefObject<HTMLDivElement | null>
    marquee: any
    onGridMouseDown: (_grid: "top" | "bottom", _e: React.MouseEvent) => void
}

export default function NormalHoningPanel({
    topGrid,
    setTopGrid,
    prev_checked_arr,
    set_prev_checked_arr,
    topGridRef,
    marquee,
    onGridMouseDown,
}: NormalHoningPanelProps) {
    return (
        <div>
            <h2 style={{ ...styles.sectionTitle, marginTop: "-8px" }}>Normal Honing</h2>
            <div style={{ ...styles.gridSection, maxWidth: "851px" }}>
                <div style={{ display: "flex", gap: 8 }}>
                    <div style={{ width: 100, display: "flex", flexDirection: "column", justifyContent: "flex-start", textWrap: "nowrap", gap: 0 }}>
                        {["", "Helmet", "Shoulder", "Chest", "Pants", "Glove", "Weapon"].map((lab) => (
                            <div
                                key={"Normal Honing label" + lab}
                                style={{
                                    height: 28,
                                    color: "var(--text-secondary)",
                                    display: "flex",
                                    alignItems: "center",
                                    justifyContent: "flex-end",
                                    paddingRight: 8,
                                }}
                            >
                                {lab ? <Icon iconName={lab} size={28} style={{ fontSize: "var(--font-size-sm)" }} /> : ""}
                            </div>
                        ))}
                    </div>
                    <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
                        <div style={{ display: "grid", gridTemplateColumns: `repeat(${TOP_COLS}, ${CELL_W}px)`, gap: 0, paddingLeft: 1 }}>
                            {topGrid[0].map((_col, col_num) => {
                                const label = `+${col_num + 1}`
                                return (
                                    <div key={label} className="checkbox-item">
                                        <input
                                            id={label}
                                            type="checkbox"
                                            className="visually-hidden"
                                            checked={prev_checked_arr[col_num]}
                                            onChange={() =>
                                                setTopGrid((prev: any) => {
                                                    const copy = prev.map((row: any) => row.slice())
                                                    const newState = !prev_checked_arr[col_num]
                                                    for (let rr = 0; rr < TOP_ROWS; rr++) {
                                                        copy[rr][col_num] = newState
                                                    }
                                                    set_prev_checked_arr((prev) => {
                                                        const newArr = [...prev]
                                                        newArr[col_num] = newState
                                                        return newArr
                                                    })
                                                    return copy
                                                })
                                            }
                                        />
                                        <label htmlFor={label} className="box">
                                            <span className="box-text">{label}</span>
                                        </label>
                                    </div>
                                )
                            })}
                        </div>
                        <CheckboxGrid
                            grid={topGrid}
                            rows={TOP_ROWS}
                            cols={TOP_COLS}
                            gridRef={topGridRef}
                            onGridMouseDown={onGridMouseDown}
                            marquee={marquee}
                            CELL_W={CELL_W}
                            CELL_H={CELL_H}
                            gridName="top"
                        />
                    </div>
                </div>
            </div>
        </div>
    )
}
