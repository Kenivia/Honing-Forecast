// CheckboxGrid.test.tsx
import React from "react"
import { render, fireEvent } from "@testing-library/react"
import { describe, test, expect, vi } from "vitest"
import CheckboxGrid from "./CheckboxGrid.tsx" // <-- adjust path if needed

type Marquee = {
    active: boolean
    grid: "top" | "bottom"
    startR: number
    endR: number
    startC: number
    endC: number
    initialState: boolean
}

function makeProps(overrides: Partial<any> = {}) {
    const grid = [
        [false, true, false],
        [true, false, true],
    ]
    const props = {
        grid,
        rows: grid.length,
        cols: grid[0].length,
        gridRef: React.createRef<HTMLDivElement>(),
        onGridMouseDown: vi.fn(),
        marquee: { active: false, grid: "top", startR: 0, endR: 0, startC: 0, endC: 0, initialState: false } as Marquee,
        CELL_W: 32,
        CELL_H: 32,
        gridName: "top" as "top" | "bottom",
        ...overrides,
    }
    return props
}

describe("CheckboxGrid", () => {
    test("renders the correct number of checkboxes", () => {
        const props = makeProps()
        const { container } = render(<CheckboxGrid {...props} />)
        // there should be rows * cols checkbox inputs
        const inputs = container.querySelectorAll('input[type="checkbox"]')
        expect(inputs.length).toBe(props.rows * props.cols)
    })

    test("calls onGridMouseDown with the correct gridName when outer div receives mouseDown", () => {
        const props = makeProps()
        const { container } = render(<CheckboxGrid {...props} />)
        const outerDiv = container.firstElementChild as Element
        expect(outerDiv).toBeTruthy()

        // fire mouseDown on the outer div
        fireEvent.mouseDown(outerDiv)

        // handler should be called once with (gridName, event)
        expect(props.onGridMouseDown).toHaveBeenCalledTimes(1)
        const callArgs = (props.onGridMouseDown as any).mock.calls[0]
        expect(callArgs[0]).toBe(props.gridName) // first arg is gridName
        // second arg should be an event-like object
        expect(callArgs[1]).toBeDefined()
        // basic sanity check on the event type
        expect(typeof callArgs[1]).toBe("object")
    })

    test("marquee inverts checkbox state only when marquee.active is true and marquee.grid matches gridName", () => {
        // sample grid from makeProps:
        // [ [F, T, F],
        //   [T, F, T] ]
        // we will set marquee to affect row 0, cols 1..2 (r=0, c=1..2)
        const marquee = {
            active: true,
            grid: "top",
            startR: 0,
            endR: 0,
            startC: 1,
            endC: 2,
            initialState: false,
        } as Marquee

        const props = makeProps({ marquee, gridName: "top" })
        const { container } = render(<CheckboxGrid {...props} />)
        const inputs = Array.from(container.querySelectorAll('input[type="checkbox"]')) as HTMLInputElement[]

        // compute expected checked for each cell based on original grid and marquee
        const original = props.grid.flat()
        const expected = original.map((origChecked: boolean, idx: number) => {
            const r = Math.floor(idx / props.cols)
            const c = idx % props.cols
            const inMarquee =
                marquee.active &&
                marquee.grid === props.gridName &&
                r >= Math.min(marquee.startR, marquee.endR) &&
                r <= Math.max(marquee.startR, marquee.endR) &&
                c >= Math.min(marquee.startC, marquee.endC) &&
                c <= Math.max(marquee.startC, marquee.endC)
            return inMarquee ? !marquee.initialState : origChecked
        })

        expect(inputs.length).toBe(expected.length)
        expected.forEach((exp, i) => {
            expect(inputs[i].checked).toBe(exp)
        })
    })

    test("title attribute uses +c (top) vs +(c*10) (bottom) formatting", () => {
        // Test both top and bottom quickly
        const propsTop = makeProps({ gridName: "top" })
        const { container: containerTop } = render(<CheckboxGrid {...propsTop} />)
        const titledDivsTop = Array.from(containerTop.querySelectorAll("div[title]")) as HTMLDivElement[]
        // since grid is 2x3, we expect 6 titled divs and titles repeating +1,+2,+3 twice
        expect(titledDivsTop.length).toBe(propsTop.rows * propsTop.cols)
        const titlesTop = titledDivsTop.map((d) => d.title)
        expect(titlesTop[0]).toBe("+1")
        expect(titlesTop[1]).toBe("+2")
        expect(titlesTop[2]).toBe("+3")
        // second row repeats +1..+3
        expect(titlesTop[3]).toBe("+1")
        expect(titlesTop[4]).toBe("+2")
        expect(titlesTop[5]).toBe("+3")

        const propsBottom = makeProps({ gridName: "bottom" })
        const { container: containerBottom } = render(<CheckboxGrid {...propsBottom} />)
        const titledDivsBottom = Array.from(containerBottom.querySelectorAll("div[title]")) as HTMLDivElement[]
        const titlesBottom = titledDivsBottom.map((d) => d.title)
        // bottom uses (c+1)*10 -> +10, +20, +30
        expect(titlesBottom[0]).toBe("+10")
        expect(titlesBottom[1]).toBe("+20")
        expect(titlesBottom[2]).toBe("+30")
    })
})
