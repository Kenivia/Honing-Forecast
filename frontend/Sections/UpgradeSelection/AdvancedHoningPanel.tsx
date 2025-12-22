import React from "react"
import { styles } from "@/Utils/Styles.ts"
import CheckboxGrid from "@/Components/CheckboxGrid.tsx"
import Icon from "@/Components/Icon.tsx"
import { BOTTOM_ROWS, BOTTOM_COLS, CELL_W, CELL_H } from "@/Utils/Constants.ts"

type AdvancedHoningPanelProps = {
    bottomGrid: boolean[][]
    setBottomGrid: React.Dispatch<React.SetStateAction<any>>
    prev_checked_arr_bottom: boolean[]
    set_prev_checked_arr_bottom: React.Dispatch<React.SetStateAction<boolean[]>>
    bottomGridRef: React.RefObject<HTMLDivElement | null>
    marquee: any
    onGridMouseDown: (_grid: "top" | "bottom", _e: React.MouseEvent) => void
    adv_hone_strategy: string
    adv_hone_strategy_change: (_v: string) => void
    useGridInput?: boolean
    advCounts?: number[][]
    onNumericInputChange?: (_grid: "top" | "bottom", _row: number, _col: number, _value: number) => void
}

export default function AdvancedHoningPanel({
    bottomGrid,
    setBottomGrid,
    prev_checked_arr_bottom,
    set_prev_checked_arr_bottom,
    bottomGridRef,
    marquee,
    onGridMouseDown,
    adv_hone_strategy,
    adv_hone_strategy_change,
    useGridInput = true,
    advCounts,
    onNumericInputChange,
}: AdvancedHoningPanelProps) {
    return (
        <div>
            <h2 style={{ ...styles.sectionTitle, marginTop: "-8px" }}>Advanced Honing</h2>
            <div style={styles.gridSection}>
                <div style={{ display: "flex", gap: 8 }}>
                    <div style={{ display: "flex", flexDirection: "column", justifyContent: "flex-start", textWrap: "nowrap", gap: 0 }}>
                        {["", "Helmet", "Shoulder", "Chest", "Pants", "Glove", "Weapon"].map((lab) => (
                            <div
                                key={"Adv Honing label" + lab}
                                style={{
                                    height: 28,
                                    color: "var(--text-secondary)",
                                    display: "flex",
                                    alignItems: "center",
                                    justifyContent: "flex-end",
                                    paddingRight: 8,
                                }}
                            >
                                {lab ? <Icon iconName={lab} display_text="" size={28} style={{ fontSize: "var(--font-size-sm)" }} /> : ""}
                            </div>
                        ))}
                    </div>
                    <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
                        <div style={{ display: "grid", gridTemplateColumns: `repeat(${BOTTOM_COLS}, ${CELL_W}px)`, gap: 0, paddingLeft: 1 }}>
                            {[10, 20, 30, 40].map((n, i) => {
                                const label = `+${n}`
                                return (
                                    <div key={label + " adv"} className="checkbox-item">
                                        <input
                                            id={label + " adv"}
                                            type="checkbox"
                                            className="visually-hidden"
                                            checked={prev_checked_arr_bottom[i]}
                                            onChange={() =>
                                                setBottomGrid((prev: any) => {
                                                    const copy = prev.map((row: any) => row.slice())
                                                    const newState = !prev_checked_arr_bottom[i]
                                                    for (let rr = 0; rr < BOTTOM_ROWS; rr++) {
                                                        copy[rr][i] = newState
                                                    }
                                                    set_prev_checked_arr_bottom((prev) => {
                                                        const newArr = [...prev]
                                                        newArr[i] = newState
                                                        return newArr
                                                    })
                                                    return copy
                                                })
                                            }
                                        />
                                        <label htmlFor={label + " adv"} className="box">
                                            <span className="box-text">{label}</span>
                                        </label>
                                    </div>
                                )
                            })}
                        </div>
                        <CheckboxGrid
                            grid={bottomGrid}
                            rows={BOTTOM_ROWS}
                            cols={BOTTOM_COLS}
                            gridRef={bottomGridRef}
                            onGridMouseDown={onGridMouseDown}
                            marquee={marquee}
                            CELL_W={CELL_W}
                            CELL_H={CELL_H}
                            gridName="bottom"
                            useGridInput={useGridInput}
                            numericInputs={advCounts}
                            onNumericInputChange={onNumericInputChange}
                        />
                    </div>
                </div>

                <div style={{ marginTop: 16, display: "flex", justifyContent: "flex-end" }}>
                    <select
                        value={adv_hone_strategy}
                        onChange={(e) => adv_hone_strategy_change(e.target.value)}
                        style={{
                            padding: "8px 12px",
                            borderRadius: "var(--border-radius-small)",
                            background: "var(--bg-tertiary)",
                            color: "var(--text-primary)",
                            border: "1px solid var(--border-secondary)",
                            fontSize: "var(--font-size-sm)",
                        }}
                    >
                        <option value="No juice">No juice</option>
                        <option value="Juice on grace">Juice on grace</option>
                    </select>
                </div>
            </div>
        </div>
    )
}
