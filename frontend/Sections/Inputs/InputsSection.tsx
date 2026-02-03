import SpreadsheetGrid from "@/Components/SpreadsheetGrid.tsx"
import SliderColumn from "@/Components/SliderColumn.tsx"
import ButtonColumn from "@/Components/ButtonColumn.tsx"
import LabeledCheckbox from "@/Components/LabeledCheckbox.tsx"
import { JUICE_LABELS, MATS_LABELS } from "@/Utils/Constants.ts"
import { createColumnDefs, styles } from "@/Utils/Styles.ts"
import type { InputsBundleWithSetters } from "@/Utils/InputBundles.ts"
import React, { useState } from "react"

type InputsSectionProps = {
    inputsBundle: InputsBundleWithSetters
}

export default function InputsSection({ inputsBundle: inputs }: InputsSectionProps) {
    const [advancedMode, setAdvancedMode] = useState(true)
    const { matsColumnDef, juiceColumnDef } = createColumnDefs()
    const { values, setters } = inputs
    const { mats, juice } = values
    const { mats: matsSetters, juice: juiceSetters } = setters
    const matsBaseColumnDefs = matsColumnDef.slice(0, 2)
    const matsLeftoverDef = matsColumnDef[2]
    const juiceBaseColumnDefs = juiceColumnDef.slice(0, 2)
    const juiceLeftoverDef = juiceColumnDef[2]
    console.log(mats, juice)
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
                    {advancedMode ? (
                        <SliderColumn
                            headerName={matsLeftoverDef.headerName}
                            labels={MATS_LABELS}
                            values={mats.leftover}
                            prices={mats.prices}
                            onValuesChange={matsSetters.setLeftover}
                            hideRowsFrom={7}
                        />
                    ) : (
                        <ButtonColumn
                            headerName={matsLeftoverDef.headerName}
                            labels={MATS_LABELS}
                            prices={mats.prices}
                            onValuesChange={matsSetters.setLeftover}
                            hideRowsFrom={7}
                            leftover={mats.leftover}
                        />
                    )}
                </div>

                <div style={{ display: "flex", flexDirection: "row", gap: 0 }}>
                    <SpreadsheetGrid
                        columnDefs={juiceBaseColumnDefs}
                        labels={JUICE_LABELS.map((label_row) => label_row[0])}
                        sheetValuesArr={[juice.weapon.owned, juice.weapon.prices]}
                        setSheetValuesArr={[juiceSetters.weapon.setOwned, juiceSetters.weapon.setPrices]}
                    />
                    {advancedMode ? (
                        <SliderColumn
                            headerName={juiceLeftoverDef.headerName}
                            labels={JUICE_LABELS.map((label_row) => label_row[0])}
                            values={juice.weapon.leftover}
                            prices={juice.weapon.prices}
                            onValuesChange={juiceSetters.weapon.setLeftover}
                        />
                    ) : (
                        <ButtonColumn
                            headerName={juiceLeftoverDef.headerName}
                            labels={JUICE_LABELS.map((label_row) => label_row[0])}
                            prices={juice.weapon.prices}
                            onValuesChange={juiceSetters.weapon.setLeftover}
                            leftover={juice.weapon.leftover}
                        />
                    )}
                </div>

                <div style={{ display: "flex", flexDirection: "row", gap: 0 }}>
                    <SpreadsheetGrid
                        columnDefs={juiceBaseColumnDefs}
                        labels={JUICE_LABELS.map((label_row) => label_row[1])}
                        sheetValuesArr={[juice.armor.owned, juice.armor.prices]}
                        setSheetValuesArr={[juiceSetters.armor.setOwned, juiceSetters.armor.setPrices]}
                    />
                    {advancedMode ? (
                        <SliderColumn
                            headerName={juiceLeftoverDef.headerName}
                            labels={JUICE_LABELS.map((label_row) => label_row[1])}
                            values={juice.armor.leftover}
                            prices={juice.armor.prices}
                            onValuesChange={juiceSetters.armor.setLeftover}
                        />
                    ) : (
                        <ButtonColumn
                            headerName={juiceLeftoverDef.headerName}
                            labels={JUICE_LABELS.map((label_row) => label_row[1])}
                            prices={juice.armor.prices}
                            onValuesChange={juiceSetters.armor.setLeftover}
                            leftover={juice.armor.leftover}
                        />
                    )}
                </div>
            </div>
            <div style={{ position: "relative", marginLeft: 500, top: -100, display: "flex", alignItems: "center", gap: 16 }}>
                <span>*dummy help text</span>
                <LabeledCheckbox label="Advanced Mode" checked={advancedMode} setChecked={setAdvancedMode} />
            </div>
        </div>
    )
}
