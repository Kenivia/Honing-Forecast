import React, { useMemo } from "react"
import { ReactSortable } from "react-sortablejs"
import "./SpecialSortable.css" // Assuming you put the CSS below in this file
import { formatSig, piece_display_name } from "@/Utils/Helpers.ts"
import { PIECE_NAMES, MATS_LABELS } from "@/Utils/Constants.ts"
import Icon from "@/Components/Icon.tsx"
import SpreadsheetGrid from "@/Components/SpreadsheetGrid.tsx"
import type { InputsBundleWithSetters } from "@/Utils/InputBundles.ts"

interface Props {
    curIsBest: boolean
    evaluateAverageResult: any
    specialState: number[]
    setSpecialState: React.Dispatch<React.SetStateAction<number[]>>
    flatSucceedArr: boolean[]
    setFlatSucceedArr: React.Dispatch<React.SetStateAction<any>>
    flatUnlockArr: boolean[]
    setFlatUnlockArr: React.Dispatch<React.SetStateAction<any>>
    ranOutFreeTaps: boolean
    onRanOutFreeTaps: () => void
    inputsBundle: InputsBundleWithSetters
    allowUserChangeState: boolean
}

interface SortableItem {
    id: string
    u_index: number
}

export function SpecialSortable({
    curIsBest,
    evaluateAverageResult,
    specialState,
    setSpecialState,
    flatSucceedArr,
    setFlatSucceedArr,
    flatUnlockArr,
    setFlatUnlockArr,
    ranOutFreeTaps,
    onRanOutFreeTaps,
    inputsBundle,
    allowUserChangeState,
}: Props) {
    // 1. Map specialState (indices) to objects required by SortableJS
    // We use the value as the ID. *Assumes values in specialState are unique*
    // let truncated_special_state = useMemo(
    //     () => specialState.slice(0, evaluateAverageResult?.special_invalid_index),

    //     [specialState, evaluateAverageResult?.special_invalid_index],
    // )

    // let invalid_tail = useMemo(
    //     () => specialState.slice(evaluateAverageResult?.special_invalid_index, specialState.length),
    //     [specialState, evaluateAverageResult?.special_invalid_index],
    // )
    // console.log(evaluateAverageResult)
    let non_zeros_count = useMemo(
        () =>
            specialState
                .slice(0, evaluateAverageResult?.special_invalid_index)
                .filter(
                    (u_index, index) =>
                        evaluateAverageResult?.latest_special_probs[index] > 0 &&
                        !evaluateAverageResult.upgrade_arr[u_index].succeeded &&
                        evaluateAverageResult.upgrade_arr[u_index].is_normal_honing,
                ).length,
        [specialState, evaluateAverageResult?.latest_special_probs, evaluateAverageResult?.special_invalid_index, evaluateAverageResult?.upgrade_arr],
    )

    // Create column definition for Special Leap material
    const specialLeapColumnDef = useMemo(
        () => ({
            headerName: "",
            editable: true,
            flex: 1,
            width: "80px",
            background: "var(--grid-cell-bg)",
            backgroundSelected: "var(--grid-cell-selected)",
            color: "var(--grid-cell-text)",
        }),
        [],
    )

    const items: SortableItem[] = useMemo(
        () => specialState.filter((val) => evaluateAverageResult.upgrade_arr[val].is_normal_honing).map((val) => ({ id: String(val), u_index: val })),
        [specialState, evaluateAverageResult.upgrade_arr],
    )

    // Handle reordering
    const handleSetList = (newItems: SortableItem[]) => {
        // Extract the indices back into a number array
        const newIndices = newItems.filter((val) => evaluateAverageResult.upgrade_arr[val.u_index].is_normal_honing).map((item) => item.u_index)
        // console.log("new", newIndices)
        // Only update if the order actually changed to prevent infinite loops
        if (JSON.stringify(newIndices) !== JSON.stringify(specialState.filter((val) => evaluateAverageResult.upgrade_arr[val].is_normal_honing))) {
            setSpecialState(newIndices.concat(specialState.filter((val) => !evaluateAverageResult.upgrade_arr[val].is_normal_honing)))
        }
    }
    // console.log(items, handleSetList)
    // Toggle logic for the button column
    const toggleSuccess = (index: number) => {
        setFlatSucceedArr((prev: boolean[]) => {
            const copy = [...prev]
            copy[index] = !copy[index]
            return copy
        })
        if (!flatSucceedArr[index]) {
            if (!flatUnlockArr[index]) {
                setFlatUnlockArr((prev: boolean[]) => {
                    const copy = [...prev]
                    copy[index] = true
                    return copy
                })
            }
        }
    }

    return (
        <div style={{ justifyContent: "center", width: "100%", display: "flex", marginTop: 20 }}>
            {non_zeros_count > 0 && (
                <div>
                    <span>
                        {" "}
                        Keep attempting free taps until you run out, then move on to the next. <br></br>The instructions above takes into account the normal
                        taps you may need to do before or in-between free taps.
                    </span>
                    <div className="sequence-container">
                        {/* Headers */}
                        <div className="grid-header">Free tap order</div>
                        <div className="grid-header">Probability</div>
                        <div className="grid-header">{/* Empty for buttons */}</div>

                        <div className="grid-header" style={{ textWrap: "nowrap", width: 120 }}>
                            {allowUserChangeState ? "Update your special leaps here" : ""}
                        </div>

                        {/* Column 1: Sortable Names */}
                        <ReactSortable
                            list={items}
                            setList={handleSetList}
                            animation={10}
                            className="col-sortable"
                            ghostClass="sortable-ghost"
                            // style={
                            //     {
                            //         background: curIsBest ? "var(--btn-toggle-optimize-selected)" : "var(--sub-optimal)",
                            //     } as React.CSSProperties
                            // }
                        >
                            {items.map((item) => (
                                <div
                                    key={item.id}
                                    className="row-item name-cell"
                                    style={
                                        {
                                            background: curIsBest ? "var(--btn-toggle-optimize-selected)" : "var(--sub-optimal)",
                                        } as React.CSSProperties
                                    }
                                >
                                    {/* Access Name via u_index */}
                                    <Icon
                                        iconName={PIECE_NAMES[evaluateAverageResult.upgrade_arr[item.u_index].piece_type]}
                                        display_text=""
                                        display_text_right={piece_display_name(evaluateAverageResult.upgrade_arr[item.u_index])}
                                    ></Icon>
                                </div>
                            ))}
                        </ReactSortable>

                        {/* Column 2: Static Probabilities (Based on Index) */}
                        <div className="col-static">
                            {items.map((_, index) => (
                                <div key={`prob-${index}`} className="row-item prob-cell">
                                    {/* Access Prob via current Index */}
                                    {formatSig(evaluateAverageResult?.latest_special_probs?.[index] * 100.0, 3).concat("%") ?? "-"}
                                </div>
                            ))}
                        </div>

                        {/* Column 3: Static Buttons (Based on Index) */}
                        <div className="col-static">
                            {allowUserChangeState &&
                                items.map((val, index) => {
                                    const succeed = flatSucceedArr[val.u_index]
                                    const first_not_succeeded_index = specialState.findIndex((i) => !flatSucceedArr[i])
                                    const should_normal_tap = index >= evaluateAverageResult.special_invalid_index && !succeed
                                    const tap_previous_first = index > first_not_succeeded_index && !succeed
                                    const hasTransparentBg = should_normal_tap || tap_previous_first || (succeed && first_not_succeeded_index != index + 1)
                                    return (
                                        <div key={`btn-${val.u_index}`} className="row-item btn-cell">
                                            <button
                                                onClick={() => {
                                                    if (!hasTransparentBg) {
                                                        toggleSuccess(val.u_index)
                                                    }
                                                }}
                                                className="toggle-btn"
                                                disabled={hasTransparentBg}
                                                style={{
                                                    background:
                                                        succeed && first_not_succeeded_index == index + 1
                                                            ? "var(--btn-toggle-deselected)"
                                                            : hasTransparentBg
                                                              ? "transparent"
                                                              : "var(--btn-success)",
                                                    color: succeed || should_normal_tap || tap_previous_first ? "var(--muted-text)" : "var(--btn-success-text)",
                                                    pointerEvents: hasTransparentBg ? "none" : "auto",
                                                }}
                                            >
                                                {should_normal_tap
                                                    ? "Normal tap this "
                                                    : succeed && first_not_succeeded_index == index + 1
                                                      ? ""
                                                      : succeed && first_not_succeeded_index != index + 1
                                                        ? ""
                                                        : tap_previous_first
                                                          ? "Free tap above first"
                                                          : "Succeed free tap"}
                                            </button>
                                        </div>
                                    )
                                })}
                        </div>

                        {/* Column 4: Special Leap material grid */}
                        <div className="col-static" style={{ width: "100px" }}>
                            {allowUserChangeState &&
                                items.map((val, index) => {
                                    const succeed = flatSucceedArr[val.u_index]
                                    const first_not_succeeded_index = specialState.findIndex((i) => !flatSucceedArr[i])
                                    const should_normal_tap = index >= evaluateAverageResult.special_invalid_index
                                    const tap_previous_first = index > first_not_succeeded_index
                                    // const hasTransparentBg = should_normal_tap || tap_previous_first || (succeed && first_not_succeeded_index != index + 1)
                                    return (
                                        !should_normal_tap &&
                                        !succeed &&
                                        !tap_previous_first && (
                                            <div key={`spec-${val.u_index}`} className="row-item btn-cell" style={{ marginTop: 36 }}>
                                                <SpreadsheetGrid
                                                    columnDefs={[specialLeapColumnDef]}
                                                    labels={["Special Leap"]}
                                                    sheetValuesArr={[{ "Special Leap": inputsBundle.values.mats.owned["Special Leap"] ?? "0" }]}
                                                    setSheetValuesArr={[
                                                        (value) => {
                                                            let cleanValue = value["Special Leap"].replace(/[^0-9]/g, "")
                                                            cleanValue = cleanValue.replace(/^0+(?=\d)/, "")

                                                            cleanValue = String(Math.min(parseFloat(cleanValue), 33333))

                                                            const next = { ...inputsBundle.values.mats.owned }
                                                            next["Special Leap"] = cleanValue
                                                            inputsBundle.setters.mats.setOwned(next)
                                                        },
                                                    ]}
                                                    hideIcons={false}
                                                    fontSizeOverride={"var(--font-size-xs)"}
                                                />
                                            </div>
                                        )
                                    )
                                })}
                        </div>
                    </div>
                    {/* <div key={`out`} className="row-item btn-cell" style={{ marginTop: 20, marginBottom: 30 }}>
                        {(
                            <>
                                <button
                                    onClick={onRanOutFreeTaps}
                                    className="toggle-btn ran-out-btn"
                                    style={{ background: "var(--btn-demo)", color: "var(--text)" }}
                                >
                                    I've ran out of free taps (this just sets your special leaps owned to 0)
                                </button>
                             
                            </>
                        )}
                    </div> */}
                </div>
            )}
            {non_zeros_count == 0 && <div>You have no availiable free taps</div>}
        </div>
    )
}
