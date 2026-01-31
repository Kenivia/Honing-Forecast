import SpreadsheetGrid from "@/Components/SpreadsheetGrid.tsx"
import SliderColumn from "@/Components/SliderColumn.tsx"
import { JUICE_LABELS, MATS_LABELS } from "@/Utils/Constants.ts"
import { createColumnDefs, styles } from "@/Utils/Styles.ts"
import type { InputsBundleWithSetters } from "@/Utils/InputBundles.ts"
import React from "react"

type InputsSectionProps = {
    inputsBundle: InputsBundleWithSetters
}

export default function InputsSection({ inputsBundle: inputs }: InputsSectionProps) {
    const { matsColumnDef, juiceColumnDef } = createColumnDefs()
    const { values, setters } = inputs
    const { mats, juice } = values
    const { mats: matsSetters, juice: juiceSetters } = setters
    const matsBaseColumnDefs = matsColumnDef.slice(0, 2)
    const matsLeftoverDef = matsColumnDef[2]
    const juiceBaseColumnDefs = juiceColumnDef.slice(0, 2)
    const juiceLeftoverDef = juiceColumnDef[2]
    return (
        <div style={{ ...styles.inputSection, flexDirection: "row", maxWidth: "1200px" }}>
            <div
                style={{
                    display: "flex",
                    flexDirection: "row",
                    gap: 40,
                    marginLeft: "auto",
                    marginRight: "auto",
                    minWidth: 200,
                    flexShrink: 0,
                    marginTop: -20,
                    width: "90%",
                }}
            >
                <div style={{ display: "flex", flexDirection: "row", gap: 0 }}>
                    <SpreadsheetGrid
                        columnDefs={matsBaseColumnDefs}
                        labels={MATS_LABELS}
                        sheetValuesArr={[mats.owned, mats.prices]}
                        setSheetValuesArr={[matsSetters.setOwned, matsSetters.setPrices]}
                    />
                    <SliderColumn
                        headerName={matsLeftoverDef.headerName}
                        // width={matsLeftoverDef.width}
                        labels={MATS_LABELS}
                        values={mats.leftover}
                        prices={mats.prices}
                        onValuesChange={matsSetters.setLeftover}
                        hideRowsFrom={7}
                    // background={matsLeftoverDef.background}
                    />
                </div>

                <div style={{ display: "flex", flexDirection: "row", gap: 0 }}>
                    <SpreadsheetGrid
                        columnDefs={juiceBaseColumnDefs}
                        labels={JUICE_LABELS.map((label_row) => label_row[0])}
                        sheetValuesArr={[juice.weapon.owned, juice.weapon.prices]}
                        setSheetValuesArr={[juiceSetters.weapon.setOwned, juiceSetters.weapon.setPrices]}
                    />
                    <SliderColumn
                        headerName={juiceLeftoverDef.headerName}
                        // width={juiceLeftoverDef.width}
                        labels={JUICE_LABELS.map((label_row) => label_row[0])}
                        values={juice.weapon.leftover}
                        prices={juice.weapon.prices}
                        onValuesChange={juiceSetters.weapon.setLeftover}
                    // background={juiceLeftoverDef.background}
                    />
                </div>

                <div style={{ display: "flex", flexDirection: "row", gap: 0 }}>
                    <SpreadsheetGrid
                        columnDefs={juiceBaseColumnDefs}
                        labels={JUICE_LABELS.map((label_row) => label_row[1])}
                        sheetValuesArr={[juice.armor.owned, juice.armor.prices]}
                        setSheetValuesArr={[juiceSetters.armor.setOwned, juiceSetters.armor.setPrices]}
                    />
                    <SliderColumn
                        headerName={juiceLeftoverDef.headerName}
                        // width={juiceLeftoverDef.width}
                        labels={JUICE_LABELS.map((label_row) => label_row[1])}
                        values={juice.armor.leftover}
                        prices={juice.armor.prices}
                        onValuesChange={juiceSetters.armor.setLeftover}
                    // background={ zjuiceLeftoverDef.background}
                    />
                </div>
            </div>
        </div>
    )
}
