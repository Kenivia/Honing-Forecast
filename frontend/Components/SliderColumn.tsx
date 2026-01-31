import React, { useCallback } from "react"
import LeftoverSlider from "@/Components/LeftoverSlider.tsx"

type SliderColumnProps = {
    headerName: string
    // width: string
    labels: string[]
    values: Record<string, string>
    prices: Record<string, string>
    onValuesChange: (_next: Record<string, string>) => void
    hideRowsFrom?: number
    background?: string
}

export default function SliderColumn({
    headerName,
    // width,
    labels,
    values,
    prices,
    onValuesChange,
    hideRowsFrom,
    background = "transparent",
}: SliderColumnProps) {
    const handleRowChange = useCallback(
        (label: string, nextValue: string) => {
            const next = { ...values }
            next[label] = nextValue
            onValuesChange(next)
        },
        [onValuesChange, values],
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
                    return (
                        <div
                            key={`${label}-slider`}
                            style={{
                                height: 36,
                                display: "flex",
                                alignItems: "center",
                                border: "1px solid var(--border-accent)",
                                background,
                                boxSizing: "border-box",
                                opacity: shouldHide ? 0 : 1,
                                pointerEvents: shouldHide ? "none" : "auto",
                            }}
                        >
                            <LeftoverSlider
                                value={values[label] ?? ""}
                                maxValue={prices[label] ?? ""}
                                label={label}
                                onChange={(nextValue) => handleRowChange(label, nextValue)}
                            />
                        </div>
                    )
                })}
            </div>
        </div>
    )
}
