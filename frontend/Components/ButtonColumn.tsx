import { taxRound } from "@/Utils/Helpers.ts"
import React, { useCallback } from "react"

type ButtonColumnProps = {
    headerName: string
    labels: string[]
    prices: Record<string, string>
    onValuesChange: (_next: Record<string, string>) => void
    hideRowsFrom?: number
    leftover: Record<string, string>
}

export default function ButtonColumn({ headerName, labels, prices, onValuesChange, hideRowsFrom, leftover }: ButtonColumnProps) {
    const handleBound = useCallback(
        (label: string) => {
            const next = { ...leftover }
            next[label] = "0"
            onValuesChange(next)
        },
        [onValuesChange, leftover],
    )

    const handleTradable = useCallback(
        (label: string) => {
            const next = { ...leftover }
            let this_price = parseFloat(prices[label])
            next[label] = String(this_price - taxRound(label, this_price))
            onValuesChange(next)
        },
        [onValuesChange, prices, leftover],
    )

    return (
        <div
            style={{
                display: "flex",
                padding: 6,
                outline: "none",
                minHeight: "200px",
                flexDirection: "column",
            }}
        >
            <div style={{ display: "grid", gap: 0, marginBottom: 4, height: 36 }}>
                <div
                    style={{
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "center",
                        color: "var(--text-primary)",
                        textWrap: "wrap",
                        height: 15,
                        textAlign: "center",
                        fontWeight: 500,
                        fontSize: "var(--font-size-xs)",
                        lineHeight: 0.8,
                        marginTop: 18,
                        width: 100,
                    }}
                >
                    {headerName}
                </div>
            </div>

            <div style={{ display: "grid", gridTemplateColumns: 100, gap: 0 }}>
                {labels.map((label, rowIndex) => {
                    const shouldHide = typeof hideRowsFrom === "number" && rowIndex >= hideRowsFrom
                    // console.log(leftover)
                    const isBound = leftover[label] === "0"

                    return (
                        <div
                            key={`${label}-buttons`}
                            style={{
                                height: 36,
                                display: "flex",
                                alignItems: "stretch",
                                border: "1px solid var(--border-accent)",
                                background: "transparent",
                                boxSizing: "border-box",
                                opacity: shouldHide ? 0 : 1,
                                pointerEvents: shouldHide ? "none" : "auto",
                                gap: 0,
                            }}
                        >
                            <button
                                onClick={() => handleBound(label)}
                                style={{
                                    flex: 1,
                                    background: isBound ? "var(--grid-cell-bg)" : "transparent",
                                    color: isBound ? "#000000" : "#ffffff",
                                    border: "none",
                                    cursor: "pointer",
                                    fontSize: "var(--font-size-xs)",
                                    fontWeight: 500,
                                    transition: "all 0.15s",
                                    borderRight: "1px solid var(--border-accent)",
                                }}
                            >
                                Bound
                            </button>
                            <button
                                onClick={() => handleTradable(label)}
                                style={{
                                    flex: 1,
                                    background: !isBound ? "var(--grid-cell-bg)" : "transparent",
                                    color: !isBound ? "#000000" : "#ffffff",
                                    border: "none",
                                    cursor: "pointer",
                                    fontSize: "var(--font-size-xs)",
                                    fontWeight: 500,
                                    transition: "all 0.15s",
                                }}
                            >
                                Tradable
                            </button>
                        </div>
                    )
                })}
            </div>
        </div>
    )
}
