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

    matsColumnDef[0].headerName = advancedMode ? "Owned mats" : "BOUND owned"
    juiceColumnDef[0].headerName = advancedMode ? "Owned mats" : "BOUND owned"

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
                {!advancedMode && (
                    <span>
                        The optimizer considers all possible outcomes. In some scenarios, you succeed everything without using up your bound mats. In
                        non-advanced mode, we consider these leftover mats as worthless (0 gold).
                    </span>
                )}{" "}
                <LabeledCheckbox label="Advanced Mode" checked={advancedMode} setChecked={setAdvancedMode} />
                {advancedMode && (
                    <span>
                        Here, you can specify how the leftover mats should be valued. If you think you will use your leftover bound mats later to save you gold,
                        then you can put them at a higher value.
                        <br></br> <br></br>
                        Furthermore, if one of your mat type is almost all tradable and you want to consider the leftover as slightly less than market price(due
                        to 5% tax), you can do so by changing the leftover values. Unfortunately we are unable to consider more than 1 price threshold, so if
                        you have a mix of tradable and bound mats, you should just put your bound mats.
                        <br></br> <br></br>
                        These values influence how the optimizer behaves, most notably your juice & books. If you have bound juice (so holding onto leftover is
                        worthless), the optimizer will try and use them up.
                    </span>
                )}
            </div>
        </div>
    )
}
