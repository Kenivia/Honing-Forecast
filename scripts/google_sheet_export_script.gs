function exportAllGridsAsJson() {
    const s = SpreadsheetApp.getActiveSpreadsheet()
    const sheet = s.getActiveSheet()
    const name = sheet.getName()
    const values = sheet.getDataRange().getValues()

    const grids = {}

    const numRows = values.length
    const numCols = values[0].length

    for (let r = 0; r < numRows; r++) {
        for (let c = 0; c < numCols; c++) {
            const cell = values[r][c]

            // Detect signature: string starting with '#'
            if (typeof cell === "string" && cell.startsWith("#")) {
                const gridName = cell.substring(1).trim()

                const grid = []

                let dataRow = r + 1

                // Read rows until completely empty row
                while (dataRow < numRows) {
                    const row = values[dataRow]

                    // Check if row is completely empty
                    const isEmpty = row.filter((_, index) => index >= c + 1).every((v) => v === "" || v === null)

                    if (isEmpty) break

                    const rowData = []

                    // Read until first empty cell in this row
                    for (let cc = c + 1; cc < numCols; cc++) {
                        const val = values[dataRow][cc]

                        if (val === "" || val === null) break

                        rowData.push(val)
                    }

                    grid.push(rowData)
                    dataRow++
                }

                if (grid.length > 1) {
                    grids[gridName] = grid
                } else {
                    if (grid.length > 0) {
                        grids[gridName] = grid[0]
                    } else {
                        grids[gridName] = []
                    }
                }
            }
        }
    }

    const json = JSON.stringify(grids, null, 2)

    Logger.log(json)
    DriveApp.createFile(name + ".json", json)
}
