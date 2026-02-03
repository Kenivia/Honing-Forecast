import React, { useMemo } from "react"
import { ReactSortable } from "react-sortablejs"
import "./SpecialSortable.css" // Assuming you put the CSS below in this file
import { formatSig, piece_display_name } from "@/Utils/Helpers.ts"
import { PIECE_NAMES } from "@/Utils/Constants.ts"
import Icon from "@/Components/Icon.tsx"

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
        () => specialState.slice(0, evaluateAverageResult?.special_invalid_index).filter((x, index) => evaluateAverageResult?.latest_special_probs[index] > 0).length,
        [specialState, evaluateAverageResult?.latest_special_probs, evaluateAverageResult?.special_invalid_index,],
    )
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
        <div style={{ justifyContent: "center", width: "100%", display: "flex", marginTop: 20 }}>
            {non_zeros_count > 0 && (
                <div>
                    <div className="sequence-container">
                        {/* Headers */}
                        <div className="grid-header">Free tap order</div>
                        <div className="grid-header">Probability</div>
                        <div className="grid-header">{/* Empty for buttons */}</div>
                        <div className="grid-header">{ }</div>

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
                                const hasTransparentBg = should_normal_tap || tap_previous_first || (succeed && first_not_succeeded_index != index + 1)
                                return (
                                    <div key={`btn-${u_index}`} className="row-item btn-cell">
                                        <span
                                        >
                                            {should_normal_tap
                                                ? "Normal tap this "
                                                : succeed && first_not_succeeded_index == index + 1
                                                    ? "Undo"
                                                    : succeed && first_not_succeeded_index != index + 1
                                                        ? ""
                                                        : tap_previous_first
                                                            ? "Free tap this"
                                                            : "Free tap this"}
                                        </span>
                                    </div>
                                )
                            })}
                        </div>

                        {/* Column 4: Ran out button (next to active succeed) */}
                        {/* <div className="col-static">
                        {specialState.map((u_index, index) => {
                            const succeed = flatSucceedArr[u_index]
                            const first_not_succeeded_index = specialState.findIndex((i) => !flatSucceedArr[i])
                            const should_normal_tap = index >= evaluateAverageResult.special_invalid_index
                            const tap_previous_first = index > first_not_succeeded_index
                            const hasTransparentBg = should_normal_tap || tap_previous_first || (succeed && first_not_succeeded_index != index + 1)
                            const showRanOutButton = !hasTransparentBg && !succeed
                            return (
                                
                            )
                        })}
                    </div> */}
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

            )}{non_zeros_count == 0 && (<div>You have no availiable free taps</div>
            )}
        </div>
    )
}
