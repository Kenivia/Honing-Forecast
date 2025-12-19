import { describe, it, expect, vi, beforeEach } from "vitest"
import { render, fireEvent, cleanup } from "@testing-library/react"
import SpreadsheetGrid from "./SpreadsheetGrid.tsx"

describe("SpreadsheetGrid", () => {
    const columnDefs = [
        { headerName: "Budget", editable: true, flex: 1, background: "", backgroundSelected: "", color: "" },
        { headerName: "Gold", editable: true, flex: 1, background: "", backgroundSelected: "", color: "" },
    ]
    const labels = ["Item1", "Item2", "Item3"]

    beforeEach(() => {
        cleanup()
    })

    it("renders cells and headers", () => {
        const set_sheet_values = vi.fn()
        const { getAllByPlaceholderText, getByText } = render(
            <SpreadsheetGrid columnDefs={columnDefs} labels={labels} sheetValuesArr={[{}]} setSheetValuesArr={[set_sheet_values]} />
        )

        // Check headers
        expect(getByText("Budget")).toBeDefined()
        expect(getByText("Gold")).toBeDefined()

        // Check inputs exist - there should be 6 inputs total (3 rows × 2 columns)
        const inputs = getAllByPlaceholderText("0")
        expect(inputs).toHaveLength(6)
    })

    it("updates budget values on input change", async () => {
        const set_sheet_values = vi.fn()
        const { getAllByPlaceholderText } = render(
            <SpreadsheetGrid columnDefs={columnDefs} labels={labels} sheetValuesArr={[{}]} setSheetValuesArr={[set_sheet_values]} />
        )

        const inputs = getAllByPlaceholderText("0")
        const firstBudgetInput = inputs[0] // First row, first column (Budget)
        fireEvent.change(firstBudgetInput, { target: { value: "123" } })

        expect(set_sheet_values).toHaveBeenCalledWith({ Item1: "123" })
    })

    it("normalizes secondary values on blur", () => {
        const set_sheet_values = vi.fn()
        const setSecondaryValues = vi.fn()
        const secondaryValues = { Item1: "012.3400" }

        const { getAllByPlaceholderText } = render(
            <SpreadsheetGrid
                columnDefs={columnDefs}
                labels={labels}
                sheetValuesArr={[{}, secondaryValues]}
                setSheetValuesArr={[set_sheet_values, setSecondaryValues]}
            />
        )

        const inputs = getAllByPlaceholderText("0")
        const secondColInput = inputs[1] // first row, second column (Gold)
        fireEvent.blur(secondColInput)

        expect(setSecondaryValues).toHaveBeenCalledWith({ Item1: "12.34" })
    })

    it("does not allow editing readOnly cells", () => {
        const set_sheet_values = vi.fn()

        const { getAllByPlaceholderText } = render(
            <SpreadsheetGrid
                columnDefs={columnDefs}
                labels={labels}
                sheetValuesArr={[{}]}
                setSheetValuesArr={[undefined]} // readOnly
            />
        )

        const inputs = getAllByPlaceholderText("0")
        fireEvent.change(inputs[0], { target: { value: "999" } })
        expect(set_sheet_values).not.toHaveBeenCalled()
    })
})
