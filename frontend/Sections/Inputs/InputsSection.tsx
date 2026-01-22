import SpreadsheetGrid from "@/Components/SpreadsheetGrid.tsx"
import { JUICE_LABELS, MATS_LABELS } from "@/Utils/Constants.ts"
import { createColumnDefs, styles } from "@/Utils/Styles.ts"
import type { InputsBundleWithSetters } from "@/Utils/InputBundles.ts"
import React from "react"


type InputsSectionProps = {
    inputs: InputsBundleWithSetters
}

export default function InputsSection({ inputs }: InputsSectionProps) {
    const { matsColumnDef, juiceColumnDef } = createColumnDefs()
    const { values, setters } = inputs
    const { mats, juice } = values
    const { mats: matsSetters, juice: juiceSetters } = setters
    return (
        <div style={{ ...styles.inputSection, flexDirection: "row", maxWidth: "1200px", width: "100%" }}>

            <div style={{ display: "flex", flexDirection: "row", alignItems: "flex-start", gap: 0, minWidth: 200, flexShrink: 0, marginTop: -20 }}>
                <SpreadsheetGrid
                    columnDefs={matsColumnDef}
                    labels={MATS_LABELS}
                    sheetValuesArr={[mats.owned, mats.prices, mats.leftover]}
                    setSheetValuesArr={[matsSetters.setOwned, matsSetters.setPrices, matsSetters.setLeftover]}
                />

                <SpreadsheetGrid
                    columnDefs={juiceColumnDef}
                    labels={JUICE_LABELS.map((label_row) => label_row[0])}
                    sheetValuesArr={[juice.weapon.owned, juice.weapon.prices, juice.weapon.leftover]}
                    setSheetValuesArr={[juiceSetters.weapon.setOwned, juiceSetters.weapon.setPrices, juiceSetters.weapon.setLeftover]}
                />

                <SpreadsheetGrid
                    columnDefs={juiceColumnDef}
                    labels={JUICE_LABELS.map((label_row) => label_row[1])}
                    sheetValuesArr={[juice.armor.owned, juice.armor.prices, juice.armor.leftover]}
                    setSheetValuesArr={[juiceSetters.armor.setOwned, juiceSetters.armor.setPrices, juiceSetters.armor.setLeftover]}
                />
            </div>
        </div>
    )
}
