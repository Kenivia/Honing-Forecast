import { CELL_H, CELL_W, JUICE_LABELS, PIECE_NAMES } from "@/Utils/Constants.ts"
import { styles } from "@/Utils/Styles.ts"

import Icon from "@/Components/Icon.tsx"
import "@/Components/CheckboxGrid.css"
import "./StateGrid.css"
import { piece_display_name, toOrdinal } from "@/Utils/Helpers.ts"
import { useMemo } from "react"

export type StatePair = [boolean, number]
function sortUpgrades(list, upgradeArr, special_invalid_index) {
    let output = []
    let copy = upgradeArr.slice()
    // console.log(list.slice(), special_invalid_index)
    for (const [original_index, u_index] of list.entries()) {
        // console.log(original_index, u_index, output)
        if (original_index >= special_invalid_index) {
            // console.log(output.slice(), u_index, u_index in output)
            if (!output.includes(u_index)) {
                output.push(u_index)
            }
        } else {
            let this_upgrade = upgradeArr[u_index]
            for (const [index, element] of copy.entries()) {
                if (
                    element.upgrade_index <= this_upgrade.upgrade_index &&
                    element.is_normal_honing &&
                    element.piece_type == this_upgrade.piece_type &&
                    !element.succeeded &&
                    !output.includes(index)
                ) {
                    output.push(index)
                    // console.log(u_index, index, this_upgrade.upgrade_index)
                }
            }
        }
    }
    // console.log(output)
    return output
}
interface RowBundleProps {
    curIsBest: boolean
    bundleIndex: number
    progress: number
    unlock: boolean
    succeed: boolean
    statePairs: StatePair[]
    onUpdateProgress: (_: number) => void
    onUpdateUnlock: (_: boolean) => void
    onUpdateSucceed: (_: boolean) => void
    onUpdateStatePairs: (_: StatePair[]) => void
    allowUserChangeState: boolean
    upgrade: any
    uniqueBookNumbers: number[]
    freeTap: boolean
    freeTapOrder: number
}

const RowBundle = ({
    curIsBest,
    bundleIndex,
    progress,
    unlock,
    succeed,
    statePairs,
    onUpdateProgress,
    onUpdateUnlock,
    onUpdateSucceed,
    onUpdateStatePairs,
    allowUserChangeState,
    upgrade,
    uniqueBookNumbers,
    freeTap,
    freeTapOrder,
}: RowBundleProps) => {
    // 2.1 Find unique "book numbers" (excluding 0) and sort ascending
    // const uniqueBookNumbers = useMemo(() => {
    //     const numbers = new Set<number>()
    //     statePairs.forEach((pair) => {
    //         if (pair[1] !== 0) numbers.add(pair[1])
    //     })
    //     return Array.from(numbers).sort((a, b) => a - b)
    // }, [statePairs])

    // Determine Grid Dimensions
    // Rows: Unique Books + Juice (1) + Progress (1)
    const totalRows = upgrade.is_normal_honing ? uniqueBookNumbers.length + 2 : 1
    // console.log(upgrade.prob_dist.length, upgrade.upgrade_index, upgrade.piece_type)
    const pity_len = Math.max(upgrade.prob_dist.length - 1, 1)
    const max_len = upgrade.original_prob_dist_len - 1
    const cols = max_len
    // console.log(max_len, pity_len)
    // Helper to handle Book/Juice clicks
    const handleCellClick = (visualRowIndex: number, colIndex: number) => {
        // Determine what logic layer this row belongs to based on bottom-up logic
        // Bottom (totalRows - 1) = Progress
        // 2nd from Bottom (totalRows - 2) = Juice
        // Others = Books

        // console.log("aaa", newPairs)
        // onUpdateStatePairs(newPairs)
        if (colIndex >= pity_len) {
            return
        }
        const isProgressRow = visualRowIndex === totalRows - 1
        const isJuiceRow = visualRowIndex === totalRows - 2

        // 1. Progress Row Logic
        if (isProgressRow) {
            if (allowUserChangeState) {
                let new_progress = progress > colIndex ? colIndex : colIndex + 1
                onUpdateProgress(new_progress)
                if (new_progress > 0) {
                    onUpdateUnlock(true)
                }
                if (new_progress >= pity_len) {
                    onUpdateSucceed(true)
                }
            }

            return
        }

        // 5. Block other changes if not allowed
        if (colIndex < progress || colIndex >= pity_len - 1) return

        // Create a copy of the state for this bundle
        const newPairs = [...statePairs]
        const currentPair = newPairs[colIndex] // [boolean, number]
        // for (let i = max_len - 1; i >= pity_len - 1; i--) {
        //     // console.log("index", i, pity_len, max_len)
        //     newPairs[i] = [false, 0]
        // }
        // 2. Juice Row Logic
        if (isJuiceRow) {
            // Toggle boolean juice

            newPairs[colIndex] = [!currentPair[0], currentPair[1]]
            onUpdateStatePairs(newPairs)
            return
        }

        // 3. Book Row Logic
        // We need to map visual row index to the specific Book Number
        // Visual Row 0 is the HIGHEST book number (rendered top)
        // Visual Row (totalRows - 3) is the LOWEST book number
        // Let's find which book number corresponds to this row.
        // Index in uniqueBookNumbers array:
        // If visualRow is 0, we want uniqueBookNumbers[length - 1]
        // If visualRow is totalRows - 3, we want uniqueBookNumbers[0]
        const bookArrayIndex = totalRows - 3 - visualRowIndex
        const targetBookNum = uniqueBookNumbers[bookArrayIndex]

        // Logic:
        // If current state has this book num, set to 0 (untoggle)
        // If current state has diff book num or 0, set to this book num (toggle on, overwriting others)
        if (currentPair[1] === targetBookNum) {
            newPairs[colIndex] = [currentPair[0], 0]
        } else {
            newPairs[colIndex] = [currentPair[0], targetBookNum]
        }

        onUpdateStatePairs(newPairs)
    }

    // Generate the renderable grid (2D array of cells)
    // We construct this Top-Down for rendering
    const gridRows = []
    if (upgrade.is_normal_honing) {
        // A. Book Rows (Top -> Down corresponds to High -> Low unique numbers)
        for (let i = uniqueBookNumbers.length - 1; i >= 0; i--) {
            const bookNum = uniqueBookNumbers[i]
            const row = statePairs.slice(0, max_len).map((pair) => ({
                active: pair[1] === bookNum,
                label: JUICE_LABELS[bookNum][upgrade.is_weapon ? 0 : 1],
                type: "book",
            }))
            gridRows.push(row)
        }

        // B. Juice Row (2nd from bottom)
        const juiceRow = statePairs.slice(0, max_len).map((pair) => ({
            active: pair[0] === true,
            label: JUICE_LABELS[0][upgrade.is_weapon ? 0 : 1],
            type: "juice",
        }))
        gridRows.push(juiceRow)

        // C. Progress Row (Bottom)
        const progressRow = Array.from({ length: cols }).map((_, cIndex) => ({
            active: cIndex < progress && cIndex < pity_len,
            label: "",
            type: "progress",
        }))
        gridRows.push(progressRow)
    }

    const handleUnlockClick = () => {
        if (unlock) {
            onUpdateProgress(0)
        }
        onUpdateUnlock(!unlock)
    }

    const handleSucceedClick = () => {
        // console.log(progress, pity_len, upgrade.prob_dist, upgrade.upgrade_index, upgrade.piece_type)
        if (!succeed) {
            if (!unlock) {
                onUpdateUnlock(true)
            }
            onUpdateProgress(Math.min(progress, pity_len))
        } else {
            onUpdateProgress(Math.min(progress, pity_len - 1))
        }

        onUpdateSucceed(!succeed)
    }

    // const nextProgressIndex = progress < pity_len ? progress : -1
    // console.log("grid rows", gridRows)
    return (
        <div className="row-bundle-container" style={{ marginBottom: "10px" }}>
            <div className="state-grid-row">
                <div className="state-grid-wrapper">
                    <div
                        style={{
                            display: "flex",
                            flexDirection: "row",
                            margin: "0 0 0 0",
                            textWrap: "nowrap",
                            gap: 30,
                            borderBottom: "2px solid var(--text-very-muted)",
                        }}
                    >
                        <Icon iconName={PIECE_NAMES[upgrade.piece_type]} display_text="" display_text_right={piece_display_name(upgrade)}></Icon>
                        <span style={freeTap && upgrade.is_normal_honing ? { color: "var(--free-tap)" } : undefined}>
                            {upgrade.is_normal_honing
                                ? freeTap
                                    ? "  Use special leaps on this until you run out"
                                    : " Normal tap this (no special) "
                                : "Use juice (and scroll) on ancestor's grace"}
                        </span>
                    </div>

                    <div className="state-grid-scroll">
                        <div style={{ position: "relative" }}>
                            {upgrade.is_normal_honing &&
                                gridRows.map((row, index) => (
                                    <div key={"row_bundle_label_" + String(index)} style={{}}>
                                        <Icon iconName={row[0].label} key={row[0].label + " left label"} display_text="" size={28}></Icon>
                                    </div>
                                ))}

                            {upgrade.is_normal_honing && allowUserChangeState && (
                                <div
                                    className="checkbox-grid-item"
                                    style={{
                                        width: CELL_W,
                                        height: CELL_H,
                                        position: upgrade.is_normal_honing ? "absolute" : "relative",
                                        bottom: 0,
                                        left: 0,
                                        top: upgrade.is_normal_honing ? CELL_H * 2 : 0,
                                        display: "flex",
                                        justifyContent: "center",
                                        alignItems: "center",
                                        cursor: "pointer",
                                        paddingTop: 0,
                                        marginRight: 14,
                                    }}
                                    onMouseDown={(e) => {
                                        e.preventDefault() // Prevent text selection
                                        handleUnlockClick()
                                    }}
                                >
                                    <input type="checkbox" readOnly checked={unlock} className="checkbox-grid-input" />
                                    {unlock && (
                                        <div
                                            style={{
                                                position: "absolute",
                                                top: 0,
                                                left: 0,
                                                width: "100%",
                                                height: "100%",
                                                display: "flex",
                                                alignItems: "center",
                                                justifyContent: "center",
                                                pointerEvents: "none",
                                            }}
                                        >
                                            <span>✓</span>
                                        </div>
                                    )}
                                </div>
                            )}
                        </div>
                        <div style={{ display: "grid", gridTemplateColumns: `repeat(${cols}, ${CELL_W}px)`, gap: 0, marginBottom: "15px" }}>
                            {gridRows.flatMap((row, rIndex) =>
                                row.map((cell, cIndex) => {
                                    const key = `${bundleIndex}-${rIndex}-${cIndex}`
                                    const showSucceedMarker = cell.type === "progress" && cIndex === progress - 1
                                    // console.log(cIndex)
                                    // if (cIndex > upgrade.prob_dist.length) {
                                    //     console.log(upgrade.prob_dist, cIndex)
                                    //     return
                                    // }
                                    return (
                                        <div
                                            key={key}
                                            className="checkbox-grid-item"
                                            style={{
                                                width: CELL_W,
                                                height: CELL_H,

                                                position: "relative",
                                                display: "flex",
                                                justifyContent: "center",
                                                alignItems: "center",

                                                // backgroundColor: "#fff",
                                                background:
                                                    cIndex >= pity_len - 1
                                                        ? "transparent"
                                                        : cell.type === "progress" || !cell.active
                                                          ? "inherit"
                                                          : curIsBest
                                                            ? "var(--btn-toggle-optimize-selected)"
                                                            : "var(--sub-optimal)",

                                                cursor:
                                                    ((cIndex < progress || cIndex >= pity_len - 1) && cell.type !== "progress") || cIndex >= pity_len
                                                        ? "not-allowed"
                                                        : "pointer",
                                                opacity:
                                                    ((cIndex < progress || cIndex >= pity_len - 1) && cell.type !== "progress") || cIndex >= pity_len ? 0.3 : 1,
                                            }}
                                            onMouseDown={(e) => {
                                                e.preventDefault() // Prevent text selection
                                                handleCellClick(rIndex, cIndex)
                                            }}
                                        >
                                            {/* The Base Checkbox (Visual only, state controlled by parent div click) */}
                                            <input
                                                type="checkbox"
                                                readOnly
                                                checked={cell.active}
                                                // style={{
                                                //     width: "24px",
                                                //     height: "24px",
                                                //     cursor: "inherit",
                                                //     // Hide the default checkbox checkmark if we have an icon,
                                                //     // or keep it if you want the icon to *cover* it.
                                                //     // Given the requirement "Icon goes over the checkbox visually",
                                                //     // we can keep the input for the box border, but the Icon sits on top.
                                                //     visibility: cell.active ? "hidden" : "visible",
                                                // }}
                                                className="checkbox-grid-input"
                                                style={
                                                    {
                                                        "--checkbox-content": "um idk why an empty string doesn't over write the default",
                                                        background: cell.type === "progress" && cell.active ? "var( --checkbox-checked-bg)" : "transparent",
                                                        border: cell.type === "progress" && !allowUserChangeState ? "none" : "1px solid var(--checkbox-border)",
                                                    } as React.CSSProperties
                                                }
                                            />

                                            {(cell.type === "progress" || cell.active || (succeed && showSucceedMarker)) && (
                                                <div
                                                    style={{
                                                        position: "absolute",
                                                        top: 0,
                                                        left: 0,
                                                        width: "100%",
                                                        height: "100%",
                                                        display: "flex",
                                                        alignItems: "center",
                                                        justifyContent: "center",
                                                        pointerEvents: "none", // Let clicks pass to the div
                                                        background: succeed && showSucceedMarker ? "var(--text-success)" : "inherit",
                                                        color: succeed && showSucceedMarker ? "var(--btn-success-text)" : "inherit",
                                                    }}
                                                >
                                                    {cell.type === "progress" ? (
                                                        <span>{succeed && showSucceedMarker ? "✓" : cIndex + 1}</span>
                                                    ) : cell.active && cIndex < pity_len - 1 ? (
                                                        <div>
                                                            <Icon
                                                                iconName={cell.label}
                                                                size={Math.min(CELL_W, CELL_H) - 6}
                                                                // Hide text for the grid cells, only show image/symbol
                                                                display_text=""
                                                            />
                                                        </div>
                                                    ) : null}
                                                </div>
                                            )}
                                        </div>
                                    )
                                }),
                            )}
                        </div>
                    </div>
                    {succeed && <div className="state-grid-overlay" />}
                </div>
                {(allowUserChangeState || succeed) && upgrade.is_normal_honing && (
                    <button
                        style={{
                            ...styles.demoButton,
                            background: succeed ? "var(--btn-toggle-deselected)" : "var(--btn-success)",
                            color: succeed ? "var(--muted-text)" : "var(--btn-success-text)",

                            height: 28,
                            padding: "0 10px",
                            alignSelf: "center",
                            marginTop: 4,
                            width: 90,
                        }}
                        onClick={handleSucceedClick}
                    >
                        {succeed ? "Undo" : "Succeed"}
                    </button>
                )}
            </div>
        </div>
    )
}

// --- 3. Main Container Component ---
interface ComplexGridProps {
    curIsBest: boolean
    flatProgressArr: number[]
    setFlatProgressArr: React.Dispatch<React.SetStateAction<number[]>>
    flatUnlockArr: boolean[]
    setFlatUnlockArr: React.Dispatch<React.SetStateAction<boolean[]>>
    flatSucceedArr: boolean[]
    setFlatSucceedArr: React.Dispatch<React.SetStateAction<boolean[]>>
    flatStateBundle: StatePair[][]
    setFlatStateBundle: React.Dispatch<React.SetStateAction<StatePair[][]>>
    allowUserChangeState: boolean
    evaluateAverageResult: any
    // upgradeArr: any[]
    // specialState: number[]
    // juiceInfo: any
    // special_invalid_index: number
    // special_probs: number[]
    // setSpecialState: React.Dispatch<React.SetStateAction<number[]>>
}

export default function StateGridsManager({
    curIsBest,
    flatProgressArr,
    setFlatProgressArr,
    flatUnlockArr,
    setFlatUnlockArr,
    flatSucceedArr,
    setFlatSucceedArr,
    flatStateBundle,
    setFlatStateBundle,
    allowUserChangeState,
    evaluateAverageResult,
    // setSpecialState,
}: ComplexGridProps) {
    // Requirement 7: overflow: auto to avoid going off edge
    // This container wraps all row bundles
    const upgradeArr = evaluateAverageResult.upgrade_arr
    const specialState = evaluateAverageResult.special_state
    const juiceInfo = evaluateAverageResult.prep_output.juice_info
    const special_invalid_index = evaluateAverageResult.special_invalid_index
    const special_probs = evaluateAverageResult.latest_special_probs
    const handleUpdateProgress = (index: number, newValue: number) => {
        const newArr = [...flatProgressArr]
        newArr[index] = newValue
        setFlatProgressArr(newArr)
    }

    const handleUpdateStateBundle = (index: number, newPairs: StatePair[]) => {
        // 4. Changes reflect in flatStateBundle
        // We need to deep copy the outer array or map it to trigger React state change
        const newBundle = [...flatStateBundle]
        newBundle[index] = newPairs
        setFlatStateBundle(newBundle)
    }

    const handleUpdateUnlock = (index: number, newValue: boolean) => {
        const newArr = [...flatUnlockArr]
        newArr[index] = newValue
        setFlatUnlockArr(newArr)
    }

    const handleUpdateSucceed = (index: number, newValue: boolean) => {
        const newArr = [...flatSucceedArr]
        newArr[index] = newValue
        setFlatSucceedArr(newArr)
    }

    // Safe check to ensure lengths match

    let truncated_special_state = useMemo(
        () => specialState.slice(0, special_invalid_index),

        [specialState, special_invalid_index],
    )

    let invalid_tail = useMemo(() => specialState.slice(special_invalid_index, specialState.length), [specialState, special_invalid_index])
    if (
        flatProgressArr.length !== flatStateBundle.length ||
        flatUnlockArr.length !== flatStateBundle.length ||
        flatSucceedArr.length !== flatStateBundle.length
    ) {
        return <div>Error: Input arrays have mismatched lengths.</div>
    }
    // console.log(juiceInfo)
    // console.log(truncated_special_state, invalid_tail)
    // let temp = specialState.slice()
    let temp = sortUpgrades(specialState, upgradeArr, special_invalid_index)

    return (
        <div
            className="complex-grid-manager"
            style={{
                width: "100%",

                padding: "10px",
                boxSizing: "border-box",
            }}
        >
            {temp.map((u_index, index) => (
                <RowBundle
                    key={u_index}
                    curIsBest={curIsBest}
                    bundleIndex={u_index}
                    progress={flatProgressArr[u_index]}
                    unlock={flatUnlockArr[u_index]}
                    succeed={flatSucceedArr[u_index]}
                    statePairs={flatStateBundle[u_index]}
                    allowUserChangeState={allowUserChangeState && index == 0}
                    onUpdateProgress={(val) => handleUpdateProgress(u_index, val)}
                    onUpdateUnlock={(val) => handleUpdateUnlock(u_index, val)}
                    onUpdateSucceed={(val) => handleUpdateSucceed(u_index, val)}
                    onUpdateStatePairs={(pairs) => handleUpdateStateBundle(u_index, pairs)}
                    upgrade={upgradeArr[u_index]}
                    uniqueBookNumbers={juiceInfo.ids[upgradeArr[u_index].upgrade_index].slice(1)}
                    freeTap={
                        specialState.findIndex((x) => x == u_index) < truncated_special_state.length &&
                        special_probs[specialState.findIndex((x) => x == u_index)] > 0
                    }
                    freeTapOrder={index + 1}
                />
            ))}
        </div>
    )
}
