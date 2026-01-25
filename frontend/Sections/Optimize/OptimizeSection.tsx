import SpreadsheetGrid from "@/Components/SpreadsheetGrid.tsx"
import { CELL_H, CELL_W, JUICE_LABELS, MATS_LABELS } from "@/Utils/Constants.ts"
import { createColumnDefs, styles } from "@/Utils/Styles.ts"
import type { InputsBundleWithSetters } from "@/Utils/InputBundles.ts"
import React from "react"
import StateGrid, { StatePair } from "@/Components/StateGrid.tsx"
import StateGridsManager from "@/Components/StateGrid.tsx"

type OptimizeSectionProps = {
    inputsBundle: InputsBundleWithSetters

    optimizeAvgBusy: boolean
    optimizeAvgResult: any
    setOptimizeButtonPress: React.Dispatch<React.SetStateAction<any>>
    // onGridMouseDown: (_grid: "top" | "bottom", _e: React.MouseEvent) => void



    flatProgressArr: number[]
    setFlatProgressArr: React.Dispatch<React.SetStateAction<any>>

    flatUnlockArr: boolean[]
    setFlatUnlockArr: React.Dispatch<React.SetStateAction<any>>

    flatSucceedArr: boolean[]
    setFlatSucceedArr: React.Dispatch<React.SetStateAction<any>>

    flatStateBundle: StatePair[][]
    setFlatStateBundle: React.Dispatch<React.SetStateAction<any>>
    allowUserChangeState: boolean,

    evaluateAverageResult: any,
    // gridRefs: React.RefObject<HTMLDivElement>[]
    // marquee: any
}

function my_alr_spent_map(already_spent: any, labels: string[], index: number) {
    return already_spent
        ?
        Object.fromEntries(
            labels.map((label, lab_index) => [
                label,
                String(already_spent[index][lab_index]),
            ]),
        )

        : Object.fromEntries(labels.map((label) => [label, "Calculating..."]))
}
export default function OptimizeSection({
    inputsBundle,
    optimizeAvgBusy,
    optimizeAvgResult,
    setOptimizeButtonPress,
    flatProgressArr,
    setFlatProgressArr,
    flatUnlockArr,
    setFlatUnlockArr,
    flatSucceedArr,
    setFlatSucceedArr,
    flatStateBundle,
    setFlatStateBundle,
    allowUserChangeState,
    evaluateAverageResult
    // gridRefs,
    // onGridMouseDown,
    // marquee,
}: OptimizeSectionProps) {
    const { wideMatsColumnDefs } = createColumnDefs()
    const { values, setters } = inputsBundle
    const { mats, juice } = values
    const { mats: matsSetters, juice: juiceSetters } = setters
    const already_spent = evaluateAverageResult?.prep_output.already_spent

    return (
        <div style={{ ...styles.inputSection, flexDirection: "row", maxWidth: "1200px", width: "100%" }}>
            {flatStateBundle && flatProgressArr && evaluateAverageResult && (
                <StateGridsManager
                    flatProgressArr={flatProgressArr}
                    setFlatProgressArr={setFlatProgressArr}
                    flatUnlockArr={flatUnlockArr}
                    setFlatUnlockArr={setFlatUnlockArr}
                    flatSucceedArr={flatSucceedArr}
                    setFlatSucceedArr={setFlatSucceedArr}
                    flatStateBundle={flatStateBundle}
                    setFlatStateBundle={setFlatStateBundle}
                    allowUserChangeState={allowUserChangeState}
                    upgradeArr={evaluateAverageResult.upgrade_arr}
                />
            )}
            <div>

                Already spent: {evaluateAverageResult?.prep_output.already_spent[3]}
                <br />
                Average cost from now on: {evaluateAverageResult?.metric - evaluateAverageResult?.prep_output.already_spent[3]}
                <br />
                Already spent + more to come: {evaluateAverageResult?.metric}

            </div>

            {(

                <div style={{ display: "flex", flexDirection: "row", alignItems: "flex-start", gap: 0, minWidth: 200, flexShrink: 0, marginTop: 0 }}>
                    <div>Already spent:</div>
                    <SpreadsheetGrid
                        columnDefs={wideMatsColumnDefs}
                        labels={MATS_LABELS.slice(0, 7)}
                        sheetValuesArr={[my_alr_spent_map(already_spent, MATS_LABELS.slice(0, 7), 0)]}
                        setSheetValuesArr={[null]}
                    />

                    <SpreadsheetGrid
                        columnDefs={wideMatsColumnDefs}
                        labels={JUICE_LABELS.map((label_row) => label_row[0])}
                        sheetValuesArr={[my_alr_spent_map(already_spent, JUICE_LABELS.map((label_row) => label_row[0]), 1)]}
                        setSheetValuesArr={[null]}
                    />

                    <SpreadsheetGrid
                        columnDefs={wideMatsColumnDefs}
                        labels={JUICE_LABELS.map((label_row) => label_row[1])}
                        sheetValuesArr={[my_alr_spent_map(already_spent, JUICE_LABELS.map((label_row) => label_row[1]), 1)]}
                        setSheetValuesArr={[null]}
                    />
                </div>)}
        </div>
    )
}
