import { CELL_H, CELL_W, JUICE_LABELS, PIECE_NAMES } from "@/Utils/Constants.ts"
import React, { useMemo } from "react"
import Icon from "./Icon.tsx"
import "./CheckboxGrid.css"

export type StatePair = [boolean, number]

interface RowBundleProps {
    bundleIndex: number
    progress: number
    unlock: boolean
    statePairs: StatePair[]
    onUpdateProgress: (_: number) => void
    onUpdateUnlock: (_: boolean) => void
    onUpdateStatePairs: (_: StatePair[]) => void
    allowUserChangeState: boolean
    upgrade: any
}

const RowBundle = ({ bundleIndex, progress, unlock, statePairs, onUpdateProgress, onUpdateUnlock, onUpdateStatePairs, allowUserChangeState, upgrade }: RowBundleProps) => {
    // 2.1 Find unique "book numbers" (excluding 0) and sort ascending
    const uniqueBookNumbers = useMemo(() => {
        const numbers = new Set<number>()
        statePairs.forEach((pair) => {
            if (pair[1] !== 0) numbers.add(pair[1])
        })
        return Array.from(numbers).sort((a, b) => a - b)
    }, [statePairs])

    // Determine Grid Dimensions
    // Rows: Unique Books + Juice (1) + Progress (1)
    const totalRows = uniqueBookNumbers.length + 2

    const pity_len = upgrade.prob_dist.length - 1;
    const max_len = upgrade.original_prob_dist_len - 1;
    const cols = max_len
    console.log(max_len, pity_len)
    // Helper to handle Book/Juice clicks
    const handleCellClick = (visualRowIndex: number, colIndex: number) => {
        // Determine what logic layer this row belongs to based on bottom-up logic
        // Bottom (totalRows - 1) = Progress
        // 2nd from Bottom (totalRows - 2) = Juice
        // Others = Books
        if (colIndex >= pity_len) {
            return
        }
        const isProgressRow = visualRowIndex === totalRows - 1
        const isJuiceRow = visualRowIndex === totalRows - 2

        // 1. Progress Row Logic
        if (isProgressRow) {
            // If clicking column 2 (index 2), progress becomes 3.
            // If allowUserChangeState is false, only this works.
            let new_progress = progress > colIndex ? colIndex : colIndex + 1;
            onUpdateProgress(new_progress)
            if (new_progress > 0) {
                onUpdateUnlock(true)
            }

            return
        }

        // 5. Block other changes if not allowed
        if (!allowUserChangeState || colIndex < progress) return

        // Create a copy of the state for this bundle
        const newPairs = [...statePairs]
        const currentPair = newPairs[colIndex] // [boolean, number]

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

    const handleUnlockClick = () => {
        if (unlock) {
            onUpdateProgress(0)
        }
        onUpdateUnlock(!unlock)

    }

    return (
        <div className="row-bundle-container" style={{ marginBottom: "5px" }}>
            <h4 style={{ margin: "0 0 0 0" }}>
                <Icon
                    iconName={PIECE_NAMES[upgrade.piece_type]}
                    display_text=""
                    display_text_right={PIECE_NAMES[upgrade.piece_type] + " +" + String(upgrade.upgrade_index + 1)}
                ></Icon>
            </h4>
            <div style={{ display: "flex", gap: 0, overflow: "auto", }}>
                <div style={{ position: "relative", width: CELL_W, height: totalRows * CELL_H, flex: "0 0 auto" }}>
                    <div
                        className="checkbox-grid-item"
                        style={{
                            width: CELL_W,
                            height: CELL_H,
                            position: "absolute",
                            bottom: 0,
                            left: 0,
                            display: "flex",
                            justifyContent: "center",
                            alignItems: "center",
                            cursor: "pointer",
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
                </div>
                <div style={{ display: "grid", gridTemplateColumns: `repeat(${cols}, ${CELL_W}px)`, gap: 0, marginBottom: "15px" }}>
                    {gridRows.flatMap((row, rIndex) =>
                        row.map((cell, cIndex) => {
                            const key = `${bundleIndex}-${rIndex}-${cIndex}`
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
                                        // border: "1px solid #eee",
                                        position: "relative",
                                        display: "flex",
                                        justifyContent: "center",
                                        alignItems: "center",
                                        // backgroundColor: "#fff",
                                        cursor: (!allowUserChangeState || cIndex < progress) && cell.type !== "progress" || cIndex >= pity_len ? "not-allowed" : "pointer",
                                        opacity: (!allowUserChangeState || cIndex < progress) && cell.type !== "progress" || cIndex >= pity_len ? 0.3 : 1,
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
                                        style={{
                                            "--checkbox-content": "um idk why an empty string doesn't over write the default",

                                            "cursor": (!allowUserChangeState || cIndex < progress || cIndex >= pity_len) && cell.type !== "progress" ? "not-allowed" : "pointer",
                                        } as React.CSSProperties}

                                    />

                                    {/* The Overlay Icon for True State */}
                                    {cell.active && (
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
                                                // backgroundColor: cell.type === "progress" ? "#e6f7ff" : "transparent", // Optional tint for progress
                                            }}
                                        >
                                            {cell.type === "progress" ? (

                                                <span>{cIndex + 1}</span>
                                            ) : (
                                                <div>
                                                    <Icon
                                                        iconName={cell.label}
                                                        size={Math.min(CELL_W, CELL_H) - 6}
                                                        // Hide text for the grid cells, only show image/symbol
                                                        display_text=""
                                                    />
                                                </div>
                                            )}
                                        </div>
                                    )}
                                </div>
                            )
                        }),
                    )}
                </div>
            </div>
        </div>
    )
}

// --- 3. Main Container Component ---
interface ComplexGridProps {
    flatProgressArr: number[]
    setFlatProgressArr: React.Dispatch<React.SetStateAction<number[]>>
    flatUnlockArr: boolean[]
    setFlatUnlockArr: React.Dispatch<React.SetStateAction<boolean[]>>
    flatStateBundle: StatePair[][]
    setFlatStateBundle: React.Dispatch<React.SetStateAction<StatePair[][]>>
    allowUserChangeState: boolean
    upgradeArr: any[]
}

export default function StateGridsManager({
    flatProgressArr,
    setFlatProgressArr,
    flatUnlockArr,
    setFlatUnlockArr,
    flatStateBundle,
    setFlatStateBundle,
    allowUserChangeState,
    upgradeArr,
}: ComplexGridProps) {
    // Requirement 7: overflow: auto to avoid going off edge
    // This container wraps all row bundles

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

    // Safe check to ensure lengths match
    if (flatProgressArr.length !== flatStateBundle.length || flatUnlockArr.length !== flatStateBundle.length) {
        return <div>Error: Input arrays have mismatched lengths.</div>
    }

    return (
        <div
            className="complex-grid-manager"
            style={{
                width: "100%",

                padding: "10px",
                boxSizing: "border-box",
            }}
        >
            {flatProgressArr.map((progressVal, index) => (
                <RowBundle
                    key={index}
                    bundleIndex={index}
                    progress={progressVal}
                    unlock={flatUnlockArr[index]}
                    statePairs={flatStateBundle[index]}
                    allowUserChangeState={allowUserChangeState}
                    onUpdateProgress={(val) => handleUpdateProgress(index, val)}
                    onUpdateUnlock={(val) => handleUpdateUnlock(index, val)}
                    onUpdateStatePairs={(pairs) => handleUpdateStateBundle(index, pairs)}
                    upgrade={upgradeArr[index]}
                />
            ))}
        </div>
    )
}
