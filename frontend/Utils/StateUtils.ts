import { StatePair } from "@/Sections/Optimize/StateGrid.tsx"

export function applyFlatToGrid(
    evaluateAverageResult: any,

    flatProgressArr: number[],
    progressGrid: number[][],
    setProgressGrid: React.Dispatch<React.SetStateAction<any>>,

    flatUnlockArr: boolean[],
    unlockGrid: boolean[][],
    setUnlockGrid: React.Dispatch<React.SetStateAction<any>>,

    flatSucceedArr: boolean[],
    succeededGrid: boolean[][],
    setSucceededGrid: React.Dispatch<React.SetStateAction<any>>,

    flatStateBundle: StatePair[][],
    stateBundleGrid: StatePair[][][],
    setStateBundleGrid: React.Dispatch<React.SetStateAction<any>>,
) {
    if (!evaluateAverageResult) {
        return
    }
    let newSucceededGrid = succeededGrid.map((row) => [...row])
    let newProgressGrid = progressGrid.map((row) => [...row])
    let newUnlockGrid = unlockGrid.map((row) => [...row])
    let newStateBundleGrid = stateBundleGrid.map((row) => [...row])

    // console.log("before")
    // console.log(newSucceededGrid)
    // console.log(flatSucceedArr)
    for (let index = 0; index < flatProgressArr.length; index++) {
        let row: number = evaluateAverageResult.upgrade_arr[index].piece_type
        let col: number = evaluateAverageResult.upgrade_arr[index].upgrade_index
        // console.log(row, col)
        // console.log(evaluateAverageResult)
        newProgressGrid[row][col] = flatProgressArr[index]
        newUnlockGrid[row][col] = flatUnlockArr[index]
        newSucceededGrid[row][col] = false // Always false, never set to true
        newStateBundleGrid[row][col] = flatStateBundle[index]
    }

    // console.log("after")
    // console.log(newSucceededGrid)

    setProgressGrid(newProgressGrid)
    setUnlockGrid(newUnlockGrid)
    setSucceededGrid(newSucceededGrid)
    setStateBundleGrid(newStateBundleGrid)
}
