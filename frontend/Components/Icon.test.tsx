import { render, screen, cleanup } from "@testing-library/react"
import { describe, test, expect, beforeEach } from "vitest"
import Icon from "./Icon.tsx"

describe("Icon component", () => {
    beforeEach(() => {
        cleanup()
    })

    test("renders the correct image for a known icon", () => {
        render(<Icon iconName="Helmet" />)
        const img = screen.getByRole("img", { name: /Helmet/i })
        expect(img).toBeInTheDocument()
        expect(img).toHaveAttribute("src", "/Honing-Forecast/Icons/Equipments/Helmet.webp")
    })

    test("renders the display_text if provided", () => {
        render(<Icon iconName="Helmet" display_text="My Helmet" />)
        expect(screen.getByText("My Helmet")).toBeInTheDocument()
    })

    test("falls back to name text if iconName not in IconMap", () => {
        render(<Icon iconName="UnknownIcon" />)
        expect(screen.getByText("UnknownIcon")).toBeInTheDocument()
        // There should be no <img> element
        expect(screen.queryByRole("img")).not.toBeInTheDocument()
    })

    test("applies custom style", () => {
        render(<Icon iconName="Helmet" style={{ color: "red" }} />)
        const container = screen.getByText("Helmet").parentElement
        expect(container).toHaveStyle("color: rgb(255, 0, 0)")
    })
})
