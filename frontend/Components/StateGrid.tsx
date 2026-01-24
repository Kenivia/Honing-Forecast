import { CELL_H, CELL_W, JUICE_LABELS } from "@/Utils/Constants.ts";
import React, { useMemo } from "react";
import Icon from "./Icon.tsx";


export type StatePair = [boolean, number];



interface RowBundleProps {
    bundleIndex: number;
    progress: number;
    statePairs: StatePair[];
    onUpdateProgress: (_: number) => void;
    onUpdateStatePairs: (_: StatePair[]) => void;
    allowUserChangeState: boolean;
    upgrade: any
}

const RowBundle = ({
    bundleIndex,
    progress,
    statePairs,
    onUpdateProgress,
    onUpdateStatePairs,
    allowUserChangeState,
    upgrade,

}: RowBundleProps) => {
    // 2.1 Find unique "book numbers" (excluding 0) and sort ascending
    const uniqueBookNumbers = useMemo(() => {
        const numbers = new Set<number>();
        statePairs.forEach((pair) => {
            if (pair[1] !== 0) numbers.add(pair[1]);
        });
        return Array.from(numbers).sort((a, b) => a - b);
    }, [statePairs]);

    // Determine Grid Dimensions
    // Rows: Unique Books + Juice (1) + Progress (1)
    const totalRows = uniqueBookNumbers.length + 2;
    const cols = statePairs.length;

    // Helper to handle Book/Juice clicks
    const handleCellClick = (visualRowIndex: number, colIndex: number) => {
        // Determine what logic layer this row belongs to based on bottom-up logic
        // Bottom (totalRows - 1) = Progress
        // 2nd from Bottom (totalRows - 2) = Juice
        // Others = Books

        const isProgressRow = visualRowIndex === totalRows - 1;
        const isJuiceRow = visualRowIndex === totalRows - 2;

        // 1. Progress Row Logic
        if (isProgressRow) {
            // If clicking column 2 (index 2), progress becomes 3.
            // If allowUserChangeState is false, only this works.
            onUpdateProgress(colIndex + 1);
            return;
        }

        // 5. Block other changes if not allowed
        if (!allowUserChangeState) return;

        // Create a copy of the state for this bundle
        const newPairs = [...statePairs];
        const currentPair = newPairs[colIndex]; // [boolean, number]

        // 2. Juice Row Logic
        if (isJuiceRow) {
            // Toggle boolean juice
            newPairs[colIndex] = [!currentPair[0], currentPair[1]];
            onUpdateStatePairs(newPairs);
            return;
        }

        // 3. Book Row Logic
        // We need to map visual row index to the specific Book Number
        // Visual Row 0 is the HIGHEST book number (rendered top)
        // Visual Row (totalRows - 3) is the LOWEST book number
        // Let's find which book number corresponds to this row.
        // Index in uniqueBookNumbers array:
        // If visualRow is 0, we want uniqueBookNumbers[length - 1]
        // If visualRow is totalRows - 3, we want uniqueBookNumbers[0]
        const bookArrayIndex = (totalRows - 3) - visualRowIndex;
        const targetBookNum = uniqueBookNumbers[bookArrayIndex];

        // Logic:
        // If current state has this book num, set to 0 (untoggle)
        // If current state has diff book num or 0, set to this book num (toggle on, overwriting others)
        if (currentPair[1] === targetBookNum) {
            newPairs[colIndex] = [currentPair[0], 0];
        } else {
            newPairs[colIndex] = [currentPair[0], targetBookNum];
        }

        onUpdateStatePairs(newPairs);
    };

    // Generate the renderable grid (2D array of cells)
    // We construct this Top-Down for rendering
    const gridRows = [];

    // A. Book Rows (Top -> Down corresponds to High -> Low unique numbers)
    for (let i = uniqueBookNumbers.length - 1; i >= 0; i--) {
        const bookNum = uniqueBookNumbers[i];
        const row = statePairs.map((pair) => ({
            active: pair[1] === bookNum,
            label: JUICE_LABELS[bookNum][upgrade.is_weapon ? 0 : 1],
            type: "book",
        }));
        gridRows.push(row);
    }

    // B. Juice Row (2nd from bottom)
    const juiceRow = statePairs.map((pair) => ({
        active: pair[0] === true,
        label: JUICE_LABELS[0][upgrade.is_weapon ? 0 : 1],
        type: "juice",
    }));
    gridRows.push(juiceRow);

    // C. Progress Row (Bottom)
    const progressRow = Array.from({ length: cols }).map((_, cIndex) => ({
        active: cIndex < progress,
        label: "",
        type: "progress",
    }));
    gridRows.push(progressRow);

    return (
        <div className="row-bundle-container" style={{ marginBottom: "20px" }}>
            <h4 style={{ margin: "0 0 0 0" }}>Row Bundle {bundleIndex + 1}</h4>
            <div
                style={{ display: "grid", gridTemplateColumns: `repeat(${cols}, ${CELL_W}px)`, gap: 0 }}
            >
                {gridRows.flatMap((row, rIndex) =>
                    row.map((cell, cIndex) => {
                        const key = `${bundleIndex}-${rIndex}-${cIndex}`;

                        return (
                            <div
                                key={key}
                                className="checkbox-grid-input"
                                style={{
                                    width: CELL_W,
                                    height: CELL_H,
                                    // border: "1px solid #eee",
                                    position: "relative",
                                    display: "flex",
                                    justifyContent: "center",
                                    alignItems: "center",
                                    // backgroundColor: "#fff",
                                    cursor: (!allowUserChangeState && cell.type !== 'progress') ? 'not-allowed' : 'pointer',
                                    opacity: (!allowUserChangeState && cell.type !== 'progress') ? 0.8 : 1
                                }}
                                onMouseDown={(e) => {
                                    e.preventDefault(); // Prevent text selection
                                    handleCellClick(rIndex, cIndex);
                                }}
                            >
                                {/* Requirements: 
                  - False = Empty Checkbox 
                  - True = Icon overlaying checkbox
                */}

                                {/* The Base Checkbox (Visual only, state controlled by parent div click) */}
                                <input
                                    type="checkbox"
                                    readOnly
                                    checked={cell.active}
                                    style={{
                                        width: "24px",
                                        height: "24px",
                                        cursor: "inherit",
                                        // Hide the default checkbox checkmark if we have an icon, 
                                        // or keep it if you want the icon to *cover* it.
                                        // Given the requirement "Icon goes over the checkbox visually",
                                        // we can keep the input for the box border, but the Icon sits on top.
                                        visibility: cell.active ? "hidden" : "visible"
                                    }}
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
                                            backgroundColor: cell.type === 'progress' ? '#e6f7ff' : 'transparent' // Optional tint for progress
                                        }}
                                    >
                                        {cell.type === "progress" ? (
                                            // Progress just shows a simple tick or fill, usually. 
                                            // But prompt implies specific icons only for state/juice. 
                                            // Prompt says: "The bottom row should represent progress... sequence of trues".
                                            // Prompt 6 says: "true values represented by an icon".
                                            // It doesn't specify an icon for progress, so we use a generic tick or fill.
                                            <span style={{ fontSize: "18px", color: "blue" }}>✓</span>
                                        ) : (
                                            <div style={{ marginLeft: -6 }}> <Icon
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
                        );
                    })
                )}
            </div>
        </div>
    );
};

// --- 3. Main Container Component ---
interface ComplexGridProps {
    flatProgressArr: number[];
    setFlatProgressArr: React.Dispatch<React.SetStateAction<number[]>>;
    flatStateBundle: StatePair[][];
    setFlatStateBundle: React.Dispatch<React.SetStateAction<StatePair[][]>>;
    allowUserChangeState: boolean;
    upgradeArr: any[],
}


export default function StateGridsManager({
    flatProgressArr,
    setFlatProgressArr,
    flatStateBundle,
    setFlatStateBundle,
    allowUserChangeState,
    upgradeArr
}: ComplexGridProps) {

    // Requirement 7: overflow: auto to avoid going off edge
    // This container wraps all row bundles

    const handleUpdateProgress = (index: number, newValue: number) => {
        const newArr = [...flatProgressArr];
        newArr[index] = newValue;
        setFlatProgressArr(newArr);
    };

    const handleUpdateStateBundle = (index: number, newPairs: StatePair[]) => {
        // 4. Changes reflect in flatStateBundle
        // We need to deep copy the outer array or map it to trigger React state change
        const newBundle = [...flatStateBundle];
        newBundle[index] = newPairs;
        setFlatStateBundle(newBundle);
    };

    // Safe check to ensure lengths match
    if (flatProgressArr.length !== flatStateBundle.length) {
        return <div>Error: Input arrays have mismatched lengths.</div>;
    }

    return (
        <div
            className="complex-grid-manager"
            style={{
                width: "100%",
                overflowX: "auto", // Requirement 7
                padding: "10px",
                boxSizing: "border-box",
            }}
        >
            {flatProgressArr.map((progressVal, index) => (

                <RowBundle
                    key={index}
                    bundleIndex={index}
                    progress={progressVal}
                    statePairs={flatStateBundle[index]}
                    allowUserChangeState={allowUserChangeState}
                    onUpdateProgress={(val) => handleUpdateProgress(index, val)}
                    onUpdateStatePairs={(pairs) => handleUpdateStateBundle(index, pairs)}
                    upgrade={
                        upgradeArr[index]}
                />
            ))}
        </div>
    );
}