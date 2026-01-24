import { StatePair } from "@/Components/StateGrid.tsx"


export function applyFlatToGrid(
    evaluateAverageResult:any, 

    flatProgressArr: number[],
    progressGrid:number[][],
    setProgressGrid: React.Dispatch<React.SetStateAction<any>>,

    flatUnlockArr: boolean[],
    unlockGrid:boolean[][],
    setUnlockGrid: React.Dispatch<React.SetStateAction<any>>,

    flatStateBundle: StatePair[][],
    stateBundleGrid: StatePair[][][],
    setStateBundleGrid: React.Dispatch<React.SetStateAction<any>>,

){
    let newProgressGrid = [...progressGrid]
    let newUnlockGrid = [...unlockGrid]
    let newStateBundleGrid = [...stateBundleGrid]

    for (let index = 0;  index < flatProgressArr.length; index ++){
        let  row  :number  = evaluateAverageResult.upgrade_arr[index].piece_type
        let col : number   = evaluateAverageResult.upgrade_arr[index].upgrade_index
        // console.log(row, col, newProgressGrid)
        // console.log(evaluateAverageResult)
        newProgressGrid[row][col] = flatProgressArr[index]
        newUnlockGrid[row][col] = flatUnlockArr[index]

        newStateBundleGrid[row][col] = flatStateBundle[index]
    }
    setProgressGrid(newProgressGrid)
    setUnlockGrid(newUnlockGrid)
    setStateBundleGrid(newStateBundleGrid)
}