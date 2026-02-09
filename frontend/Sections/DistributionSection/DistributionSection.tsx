import React from "react"

import LabeledCheckbox from "@/Components/LabeledCheckbox.tsx"
import { styles } from "@/Utils/Styles.ts"
import { GRAPH_COLORS, JUICE_LABELS, MATS_LABELS, OUTPUT_LABELS } from "@/Utils/Constants.ts"
import { MaterialGraph } from "@/Components/MaterialGraph.tsx"
type DistributionSectionProps = {
    cumulativeGraph: boolean
    histogramResult: any
    setCumulativeGraph: (_next: boolean) => void
}

export default function DistributionSection({ cumulativeGraph, histogramResult, setCumulativeGraph }: DistributionSectionProps) {
    return (
        <div style={{ ...styles.inputSection, flexDirection: "row", maxWidth: "1200px", width: "100%" }}>
            <div style={{ marginBottom: 30 }}>Distribution of costs, these graphs reflect the free taps & juice usage as specified in the next tab. </div>
            <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                <LabeledCheckbox label="Cumulative Graph" checked={cumulativeGraph} setChecked={setCumulativeGraph} />
            </div>

            {MATS_LABELS.slice(0, 7)
                .concat(JUICE_LABELS.map((l) => l[0]))
                .concat(JUICE_LABELS.map((l) => l[1]))
                .map((label, index) => (
                    <MaterialGraph
                        key={index}
                        data={histogramResult?.cum_percentiles[index]}
                        average={histogramResult?.average[index]}
                        secondaryAnnotation={histogramResult?.budgets[index]}
                        color={GRAPH_COLORS[index]}
                        cumulative={cumulativeGraph}
                        height={120}
                        label={label}
                    />
                ))}
        </div>
    )
}
