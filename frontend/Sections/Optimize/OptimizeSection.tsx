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


    flatStateBundle: StatePair[][]
    setFlatStateBundle: React.Dispatch<React.SetStateAction<any>>
    allowUserChangeState: boolean,

    evaluateAverageResult: any,
    // gridRefs: React.RefObject<HTMLDivElement>[]
    // marquee: any
}

export default function OptimizeSection({
    inputsBundle,
    optimizeAvgBusy,
    optimizeAvgResult,
    setOptimizeButtonPress,
    flatProgressArr,
    setFlatProgressArr,
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

    return (
        <div style={{ ...styles.inputSection, flexDirection: "row", maxWidth: "1200px", width: "100%" }}>
            {flatStateBundle && flatProgressArr && evaluateAverageResult && (<StateGridsManager
                flatProgressArr={flatProgressArr}
                setFlatProgressArr={setFlatProgressArr}
                flatStateBundle={flatStateBundle}
                setFlatStateBundle={setFlatStateBundle}
                allowUserChangeState={allowUserChangeState}
                upgradeArr={evaluateAverageResult.upgrade_arr}

            />)}
            <div>
                {evaluateAverageResult?.metric}
            </div>
            <div style={{ display: "flex", flexDirection: "row", alignItems: "flex-start", gap: 0, minWidth: 200, flexShrink: 0, marginTop: -20 }}>
                <SpreadsheetGrid
                    columnDefs={wideMatsColumnDefs}
                    labels={MATS_LABELS}
                    sheetValuesArr={[mats.owned, mats.prices, mats.leftover]}
                    setSheetValuesArr={[matsSetters.setOwned, matsSetters.setPrices, matsSetters.setLeftover]}
                />

                <SpreadsheetGrid
                    columnDefs={wideMatsColumnDefs}
                    labels={JUICE_LABELS.map((label_row) => label_row[0])}
                    sheetValuesArr={[juice.weapon.owned, juice.weapon.prices, juice.weapon.leftover]}
                    setSheetValuesArr={[juiceSetters.weapon.setOwned, juiceSetters.weapon.setPrices, juiceSetters.weapon.setLeftover]}
                />

                <SpreadsheetGrid
                    columnDefs={wideMatsColumnDefs}
                    labels={JUICE_LABELS.map((label_row) => label_row[1])}
                    sheetValuesArr={[juice.armor.owned, juice.armor.prices, juice.armor.leftover]}
                    setSheetValuesArr={[juiceSetters.armor.setOwned, juiceSetters.armor.setPrices, juiceSetters.armor.setLeftover]}
                />
            </div>
        </div>
    )
}
