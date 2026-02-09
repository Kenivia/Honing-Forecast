import SpreadsheetGrid from "@/Components/SpreadsheetGrid.tsx"
import LabeledCheckbox from "@/Components/LabeledCheckbox.tsx"
import { JUICE_LABELS, MATS_LABELS } from "@/Utils/Constants.ts"
import { createColumnDefs, styles } from "@/Utils/Styles.ts"
import type { InputsBundleWithSetters } from "@/Utils/InputBundles.ts"
import React, { useState } from "react"

type InputsSectionProps = {
    inputsBundle: InputsBundleWithSetters
    inputToggles: { mats: boolean[]; weapon: boolean[]; juice: boolean[] }
    setInputToggles: (_toggles: { mats: boolean[]; weapon: boolean[]; juice: boolean[] }) => void
}

export default function InputsSection({ inputsBundle: inputs, inputToggles, setInputToggles }: InputsSectionProps) {
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

    matsColumnDef[0].headerName = advancedMode ? "Owned mats" : "Untradable mats"
    juiceColumnDef[0].headerName = advancedMode ? "Owned mats" : "Untradable mats"
    const toggleAdvMode = (new_mode) => {
        setAdvancedMode(new_mode)

        if (!new_mode) {
            matsSetters.setLeftover(Object.fromEntries(Array.from(MATS_LABELS.map((label) => [label, "0"])))) // its supposed to be an object here but cbb
            juiceSetters.weapon.setLeftover(Object.fromEntries(Array.from(JUICE_LABELS.map((row) => [row[0], "0"]))))
            juiceSetters.armor.setLeftover(Object.fromEntries(Array.from(JUICE_LABELS.map((row) => [row[1], "0"]))))
        }
    }
    return (
        <div style={{ ...styles.inputSection, flexDirection: "row", maxWidth: "1200px" }}>
            <div style={{ marginTop: -10, marginBottom: 10 }}>
                Our goal is to minimize the Average gold used, both in tapping, buying extra mats and equivalently consuming any tradable mats you may have.
            </div>
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
                        columnDefs={matsBaseColumnDefs.concat(advancedMode ? [matsLeftoverDef] : [])}
                        labels={MATS_LABELS}
                        sheetValuesArr={[mats.owned, mats.prices, mats.leftover]}
                        setSheetValuesArr={[matsSetters.setOwned, matsSetters.setPrices, matsSetters.setLeftover]}
                        rowCheckboxes={{
                            value: inputToggles.mats,
                            onChange: (newValues) => setInputToggles({ ...inputToggles, mats: newValues }),
                        }}
                    />
                    {/* {advancedMode ? (
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
                        <div style={{ width: 60 }}></div>
                    )} */}
                    {!advancedMode && <div style={{ width: 60 }}></div>}
                </div>

                <div style={{ display: "flex", flexDirection: "row", gap: 0 }}>
                    <SpreadsheetGrid
                        columnDefs={juiceBaseColumnDefs.concat(advancedMode ? [juiceLeftoverDef] : [])}
                        labels={JUICE_LABELS.map((label_row) => label_row[0])}
                        sheetValuesArr={[juice.weapon.owned, juice.weapon.prices, juice.weapon.leftover]}
                        setSheetValuesArr={[juiceSetters.weapon.setOwned, juiceSetters.weapon.setPrices, juiceSetters.weapon.setLeftover]}
                        // rowCheckboxes={{
                        //     value: inputToggles.juice,
                        //     onChange: (newValues) => setInputToggles({ ...inputToggles, juice: newValues }),
                        // }}
                    />
                    {/* {advancedMode ? (
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
                        <div style={{ width: 60 }}></div>
                    )} */}
                    {!advancedMode && <div style={{ width: 60 }}></div>}
                </div>

                <div style={{ display: "flex", flexDirection: "row", gap: 0 }}>
                    <SpreadsheetGrid
                        columnDefs={juiceBaseColumnDefs.concat(advancedMode ? [juiceLeftoverDef] : [])}
                        labels={JUICE_LABELS.map((label_row) => label_row[1])}
                        sheetValuesArr={[juice.armor.owned, juice.armor.prices, juice.armor.leftover]}
                        setSheetValuesArr={[juiceSetters.armor.setOwned, juiceSetters.armor.setPrices, juiceSetters.armor.setLeftover]}
                        // rowCheckboxes={{
                        //     value: inputToggles.juice,
                        //     onChange: (newValues) => setInputToggles({ ...inputToggles, juice: newValues }),
                        // }}
                    />
                    {!advancedMode && (
                        // <ButtonColumn
                        //     headerName={juiceLeftoverDef.headerName}
                        //     labels={JUICE_LABELS.map((label_row) => label_row[1])}
                        //     prices={juice.armor.prices}
                        //     onValuesChange={juiceSetters.armor.setLeftover}
                        //     leftover={juice.armor.leftover}
                        // />
                        <div style={{ width: 60 }}></div>
                    )}
                </div>
            </div>{" "}
            <div style={{ position: "relative", marginLeft: 450, top: -130, display: "flex", alignItems: "flex-start", gap: 16, flexDirection: "column" }}>
                {<span></span>} <LabeledCheckbox label="Custom leftover values" checked={advancedMode} setChecked={toggleAdvMode} />
                {advancedMode && (
                    <span>
                        Specify how much your leftover mats are worth. By default, this is 0 for all mats.
                        <br></br> <br></br> For example, if you say leftover abidos are worth 100g each, then having 200 abidos leftover is considered the same
                        as having 20000 gold. This influences how the optimizer might try to save abidos, potentially by buying more juice. <br></br> <br></br>
                    </span>
                )}
            </div>
        </div>
    )
}
