import React, { useEffect, useMemo } from "react"
import { StyledSlider } from "@/Utils/Styles.ts"

type LeftoverSliderProps = {
    value: string
    maxValue: string
    label: string
    onChange: (_next: string) => void
}

const clamp = (value: number, max: number) => Math.min(Math.max(value, 0), max)

export default function LeftoverSlider({ value, maxValue, label, onChange }: LeftoverSliderProps) {
    const numericMax = useMemo(() => {
        const parsed = parseFloat(maxValue)
        return Number.isFinite(parsed) ? Math.max(0, parsed) : 0
    }, [maxValue])

    const numericValue = useMemo(() => {
        const parsed = parseFloat(value)
        return Number.isFinite(parsed) ? parsed : 0
    }, [value])

    const clampedValue = clamp(numericValue, numericMax)

    useEffect(() => {
        if (!Number.isFinite(numericValue) || numericValue < 0 || numericValue > numericMax) {
            onChange(clampedValue.toString())
        }
    }, [numericValue, numericMax, clampedValue, onChange])

    const factor = label == "Silver" ? 0.0001 : label == "Shards" ? 0.001 : label == "Red" || label == "Blue" ? 0.01 : 1
    return (
        <div style={{ width: "100%", padding: "0 6px", boxSizing: "border-box", marginTop: "4px" }}>
            <StyledSlider
                style={{ width: "100%" }}
                value={clampedValue}
                onChange={(_, nextValue) => {
                    const valueNumber = Array.isArray(nextValue) ? nextValue[0] : nextValue
                    // const rounded = Math.round((valueNumber as number) * 100) / 100
                    onChange(valueNumber.toString())
                }}
                min={0}
                max={numericMax}
                step={Math.max(Math.floor((numericMax * 0.05) / factor) * factor, factor)}
                valueLabelDisplay="auto"
                valueLabelFormat={() => `Every leftover ${label} is considered ${clampedValue} gold`}
            />
        </div>
    )
}
