import { StyledSlider } from "@/Frontend/Utils/Styles.ts"

export interface SliderBundleProps {
    desiredChance: string
    uncleanedDesiredChance: string
    onDesiredChange: (_: string) => void
    onDesiredBlur: () => void
    lowThreshold?: number
    lowText?: string
    lowTextColor?: string
}

export function SliderBundle({
    desiredChance,
    uncleanedDesiredChance,
    onDesiredChange,
    onDesiredBlur,
    lowThreshold,
    lowText,
    lowTextColor = "var(--text-muted)",
}: SliderBundleProps) {
    const intDesired = parseInt(desiredChance) || 0
    const showLowText = typeof lowThreshold === "number" && lowText !== undefined && intDesired <= lowThreshold

    return (
        <div style={{ display: "flex", flexDirection: "column", height: 100, marginLeft: 100 }}>
            <div style={{ display: "flex", flexDirection: "row", gap: 100, marginLeft: -10 }}>
                <div style={{ width: 160, fontWeight: 700, fontSize: 20, textAlign: "right", paddingRight: 8, paddingTop: 4, color: "var(--input-bg)" }}>
                    Desired chance
                </div>
                <div style={{ position: "relative", display: "flex", alignItems: "center" }}>
                    <input
                        type="text"
                        value={uncleanedDesiredChance}
                        onChange={(e) => onDesiredChange(e.target.value)}
                        onBlur={onDesiredBlur}
                        placeholder="0"
                        style={{
                            width: 70,
                            fontSize: 16,
                            padding: "6px 8px",
                            borderRadius: 6,
                            background: "var(--input-bg)",
                            color: "var(--input-text)",
                            border: "1px solid var(--input-border)",
                        }}
                    />
                    <span style={{ position: "absolute", right: 10, pointerEvents: "none", color: "black" }}>%</span>
                </div>
            </div>
            <div style={{ display: "flex", flexDirection: "row", gap: 20 }}>
                <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
                    <StyledSlider
                        value={parseInt(desiredChance) || 0}
                        onChange={(_, value) => {
                            const intValue = Math.round(value as number)
                            onDesiredChange(intValue.toString())
                        }}
                        min={0}
                        max={100}
                        step={1}
                        valueLabelDisplay="off"
                    />
                </div>
            </div>

            {showLowText && (
                <span
                    style={{
                        color: lowTextColor,
                        fontSize: "var(--font-size-lg)",
                        // fontStyle: 'italic'
                    }}
                >
                    {lowText}
                </span>
            )}
        </div>
    )
}
