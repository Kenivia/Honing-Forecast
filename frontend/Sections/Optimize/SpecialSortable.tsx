import React, { useMemo } from "react"
import { ReactSortable } from "react-sortablejs"
import "./SpecialSortable.css" // Assuming you put the CSS below in this file
import { formatSig } from "@/Utils/Helpers.ts"
import { PIECE_NAMES } from "@/Utils/Constants.ts"
import Icon from "@/Components/Icon.tsx"

interface Props {
    evaluateAverageResult: {
        upgrade_arr: { name_string: string }[]
        latest_special_probs: (number | string)[]
    }
    specialState: number[]
    setSpecialState: React.Dispatch<React.SetStateAction<number[]>>
    flatSucceedArr: boolean[]
    setFlatSucceedArr: React.Dispatch<React.SetStateAction<any>>
    flatUnlockArr: boolean[]
    setFlatUnlockArr: React.Dispatch<React.SetStateAction<any>>
}

interface SortableItem {
    id: string
    u_index: number
}

export function SpecialSortable({
    evaluateAverageResult,
    specialState,
    setSpecialState,
    flatSucceedArr,
    setFlatSucceedArr,
    flatUnlockArr,
    setFlatUnlockArr,
}: Props) {
    // 1. Map specialState (indices) to objects required by SortableJS
    // We use the value as the ID. *Assumes values in specialState are unique*
    const items: SortableItem[] = useMemo(() => specialState.map((val) => ({ id: String(val), u_index: val })), [specialState])

    // Handle reordering
    const handleSetList = (newItems: SortableItem[]) => {
        // Extract the indices back into a number array
        const newIndices = newItems.map((item) => item.u_index)

        // Only update if the order actually changed to prevent infinite loops
        if (JSON.stringify(newIndices) !== JSON.stringify(specialState)) {
            setSpecialState(newIndices)
        }
    }

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
        <div className="sequence-container">
            {/* Headers */}
            <div className="grid-header">Free tap order</div>
            <div className="grid-header">Probability</div>
            <div className="grid-header">{/* Empty for buttons */}</div>

            {/* Column 1: Sortable Names */}
            <ReactSortable list={items} setList={handleSetList} animation={150} className="col-sortable" ghostClass="sortable-ghost">
                {items.map((item) => (
                    <div key={item.id} className="row-item name-cell">
                        {/* Access Name via u_index */}
                        <Icon
                            iconName={PIECE_NAMES[evaluateAverageResult.upgrade_arr[item.u_index].piece_type]}
                            display_text=""
                            display_text_right={
                                PIECE_NAMES[evaluateAverageResult.upgrade_arr[item.u_index].piece_type] +
                                " +" +
                                String(evaluateAverageResult.upgrade_arr[item.u_index].upgrade_index + 1)
                            }
                        ></Icon>
                    </div>
                ))}
            </ReactSortable>

            {/* Column 2: Static Probabilities (Based on Index) */}
            <div className="col-static">
                {specialState.map((_, index) => (
                    <div key={`prob-${index}`} className="row-item prob-cell">
                        {/* Access Prob via current Index */}
                        {formatSig(evaluateAverageResult?.latest_special_probs?.[index] * 100.0, 3).concat("%") ?? "-"}
                    </div>
                ))}
            </div>

            {/* Column 3: Static Buttons (Based on Index) */}
            <div className="col-static">
                {specialState.map((u_index, index) => {
                    const succeed = flatSucceedArr[u_index]
                    const first_not_succeeded_index = specialState.findIndex((i) => !flatSucceedArr[i])
                    const should_normal_tap = index >= evaluateAverageResult.special_invalid_index
                    const tap_previous_first = index > first_not_succeeded_index
                    const hasTransparentBg =
                        should_normal_tap || tap_previous_first || (succeed && first_not_succeeded_index != index + 1)
                    return (
                        <div key={`btn-${u_index}`} className="row-item btn-cell">
                            <button
                                onClick={() => {
                                    if (!hasTransparentBg) {
                                        toggleSuccess(u_index)
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
                                        ? "Undo"
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
        </div>
    )
}
