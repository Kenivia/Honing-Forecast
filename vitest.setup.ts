import { expect } from "vitest"
import * as matchers from "@testing-library/jest-dom/matchers"
expect.extend(matchers)

// Mock scrollWidth in tests for layout calculations
Object.defineProperty(HTMLElement.prototype, "scrollWidth", {
    configurable: true,
    get() {
        return (this as HTMLElement).clientWidth || 1268
    },
})

// Stub getBoundingClientRect for elements used in layout
const origGetBoundingClientRect = HTMLElement.prototype.getBoundingClientRect
HTMLElement.prototype.getBoundingClientRect = function () {
    try {
        return origGetBoundingClientRect.apply(this)
    } catch {
        return {
            x: 0,
            y: 0,
            top: 0,
            left: 0,
            right: 800,
            bottom: 600,
            width: 800,
            height: 600,
            toJSON() {
                return ""
            },
        } as any
    }
}

// Provide window.innerWidth default
Object.defineProperty(window, "innerWidth", { configurable: true, value: 1280 })
