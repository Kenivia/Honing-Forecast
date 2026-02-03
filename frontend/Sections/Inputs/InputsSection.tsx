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
    const [advancedMode, setAdvancedMode] = useState(false)
    const { matsColumnDef, juiceColumnDef } = createColumnDefs()
    const { values, setters } = inputs
    const { mats, juice } = values
    const { mats: matsSetters, juice: juiceSetters } = setters
    const matsBaseColumnDefs = matsColumnDef.slice(0, 2)
    const matsLeftoverDef = matsColumnDef[2]
    const juiceBaseColumnDefs = juiceColumnDef.slice(0, 2)
    const juiceLeftoverDef = juiceColumnDef[2]
    // console.log(mats, juice)
    matsLeftoverDef.headerName = advancedMode ? "Value of leftovers" : "Mat type*"
    juiceLeftoverDef.headerName = advancedMode ? "Value of leftovers" : "Mat type*"

    matsColumnDef[0].headerName = advancedMode ? "Owned mats" : "BOUND mats"
    juiceColumnDef[0].headerName = advancedMode ? "Owned mats" : "BOUND mats"
    const toggleAdvMode = (new_mode) => {
        setAdvancedMode(new_mode)

        if (!new_mode) {
            matsSetters.setLeftover(Array(mats.leftover.length).fill(0)) // its supposed to be an object here but cbb
            juiceSetters.weapon.setLeftover(Array(juice.weapon.leftover.length).fill(0))
            juiceSetters.armor.setLeftover(Array(juice.armor.leftover.length).fill(0))
        }
    }
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
                        // <SliderColumn
                        //     headerName={matsLeftoverDef.headerName}
                        //     labels={MATS_LABELS}
                        //     values={mats.leftover}
                        //     prices={mats.prices}
                        //     onValuesChange={matsSetters.setLeftover}
                        //     hideRowsFrom={7}
                        // />
                        <SpreadsheetGrid
                            columnDefs={[matsLeftoverDef]}
                            labels={MATS_LABELS.slice(0, 7)}
                            sheetValuesArr={[mats.leftover]}
                            setSheetValuesArr={[matsSetters.setLeftover]}
                            hideIcons={true}
                            fontSizeOverride={"var(--font-size-xs)"}
                        ></SpreadsheetGrid>
                    ) : (
                        // <ButtonColumn
                        //     headerName={matsLeftoverDef.headerName}
                        //     labels={MATS_LABELS}
                        //     prices={mats.prices}
                        //     onValuesChange={matsSetters.setLeftover}
                        //     hideRowsFrom={7}
                        //     leftover={mats.leftover}
                        // />
                        <div style={{ width: 72 }}></div>
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
                        <SpreadsheetGrid
                            columnDefs={[juiceLeftoverDef]}
                            labels={JUICE_LABELS.map((label_row) => label_row[0])}
                            sheetValuesArr={[juice.weapon.leftover]}
                            setSheetValuesArr={[juiceSetters.weapon.setLeftover]}
                            hideIcons={true}
                            fontSizeOverride={"var(--font-size-xs)"}
                        ></SpreadsheetGrid>
                    ) : (
                        // <SliderColumn
                        //     headerName={juiceLeftoverDef.headerName}
                        //     labels={JUICE_LABELS.map((label_row) => label_row[0])}
                        //     values={juice.weapon.leftover}
                        //     prices={juice.weapon.prices}
                        //     onValuesChange={juiceSetters.weapon.setLeftover}
                        // />
                        // <ButtonColumn
                        //     headerName={juiceLeftoverDef.headerName}
                        //     labels={JUICE_LABELS.map((label_row) => label_row[0])}
                        //     prices={juice.weapon.prices}
                        //     onValuesChange={juiceSetters.weapon.setLeftover}
                        //     leftover={juice.weapon.leftover}
                        // />
                        <div style={{ width: 72 }}></div>
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
                        // <SliderColumn
                        //     headerName={juiceLeftoverDef.headerName}
                        //     labels={JUICE_LABELS.map((label_row) => label_row[1])}
                        //     values={juice.armor.leftover}
                        //     prices={juice.armor.prices}
                        //     onValuesChange={juiceSetters.armor.setLeftover}
                        // />
                        <SpreadsheetGrid
                            columnDefs={[juiceLeftoverDef]}
                            labels={JUICE_LABELS.map((label_row) => label_row[0])}
                            sheetValuesArr={[juice.armor.leftover]}
                            setSheetValuesArr={[juiceSetters.armor.setLeftover]}
                            hideIcons={true}
                            fontSizeOverride={"var(--font-size-xs)"}
                        ></SpreadsheetGrid>
                    ) : (
                        // <ButtonColumn
                        //     headerName={juiceLeftoverDef.headerName}
                        //     labels={JUICE_LABELS.map((label_row) => label_row[1])}
                        //     prices={juice.armor.prices}
                        //     onValuesChange={juiceSetters.armor.setLeftover}
                        //     leftover={juice.armor.leftover}
                        // />
                        <div style={{ width: 72 }}></div>
                    )}
                </div>
            </div>
            <div style={{ position: "relative", marginLeft: 450, top: -130, display: "flex", alignItems: "flex-start", gap: 16, flexDirection: "column" }}>
                {
                    <span>
                        The optimizer tries to minimize the average gold spent. In some scenarios, you succeed everything with some bound mats to spare. By
                        default, we consider leftover mats as worthless (0 gold).
                    </span>
                }{" "}
                <LabeledCheckbox label="Custom leftover values" checked={advancedMode} setChecked={toggleAdvMode} />
                {advancedMode && (
                    <span>
                        Specify how much you think your leftover mats are worth.
                        <br></br> <br></br> For example, if you say leftover abidos are worth 100g each, then having 200 abidos leftover is considered the same
                        as having 20000 gold. This influences how the optimizer might try to save abidos, potentially by buying more juice. <br></br> <br></br>
                    </span>
                )}
            </div>
        </div>
    )
}
