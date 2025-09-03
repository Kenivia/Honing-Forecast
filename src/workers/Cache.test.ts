import { describe, it, expect } from "vitest"
import { reconstruct1DTo2D, flatten2DUint32 } from "./Cache.js"

//im not sure how to test the indexedDB stuff so we will pray to jesus
describe("2 way", () => {
    it("2 way 1", () => {
        const flat = new Uint32Array([1, 2, 3, 4])
        const rows = reconstruct1DTo2D(flat, 1)
        const flat2 = flatten2DUint32(rows)
        expect(flat).toEqual(flat2)
        const rows2 = reconstruct1DTo2D(flat2, 1)
        expect(rows2).toEqual([new Uint32Array([1, 2, 3, 4])])
    })
    it("2 way 2", () => {
        const flat = new Uint32Array([1, 2, 3, 4])
        const rows = reconstruct1DTo2D(flat, 2)
        const flat2 = flatten2DUint32(rows)
        expect(flat).toEqual(flat2)
        const rows2 = reconstruct1DTo2D(flat2, 2)
        expect(rows2).toEqual([new Uint32Array([1, 2]), new Uint32Array([3, 4])])
    })
    it("2 way 4", () => {
        const flat = new Uint32Array([1, 2, 3, 4])
        const rows = reconstruct1DTo2D(flat, 4)
        const flat2 = flatten2DUint32(rows)
        expect(flat).toEqual(flat2)
        const rows2 = reconstruct1DTo2D(flat2, 4)
        expect(rows2).toEqual([new Uint32Array([1]), new Uint32Array([2]), new Uint32Array([3]), new Uint32Array([4])])
    })
})
describe("reconstruct1DTo2D", () => {
    it("returns empty array when n is 0", () => {
        const flat = new Uint32Array([1, 2, 3, 4])
        const rows = reconstruct1DTo2D(flat, 0)
        expect(rows).toEqual([])
    })

    it("splits a Uint32Array into rows when zeroCopy is true (default)", () => {
        const flat = new Uint32Array([10, 20, 30, 40])
        const rows = reconstruct1DTo2D(flat, 2) // n = 2 => m = 2
        expect(rows).toHaveLength(2)
        expect(Array.from(rows[0])).toEqual([10, 20])
        expect(Array.from(rows[1])).toEqual([30, 40])
    })

    it("uses zero-copy (subarray) when zeroCopy=true so modifying a row reflects on original buffer", () => {
        const flat = new Uint32Array([1, 2, 3, 4])
        const rows = reconstruct1DTo2D(flat, 2, { zeroCopy: true })
        // modify a value in the first row
        rows[0][0] = 999
        expect(flat[0]).toBe(999) // verifies shared underlying buffer
    })

    it("creates independent copies when zeroCopy=false so modifying rows does not change original", () => {
        const flat = new Uint32Array([7, 8, 9, 10])
        const rows = reconstruct1DTo2D(flat, 2, { zeroCopy: false })
        rows[0][0] = 555
        expect(flat[0]).toBe(7) // original unchanged
        expect(rows[0][0]).toBe(555)
    })

    it("accepts ArrayLike<number> (e.g., plain number array) as input", () => {
        const flat = [1, 2, 3, 4] // ArrayLike<number>
        const rows = reconstruct1DTo2D(flat, 2)
        expect(rows.map((r) => Array.from(r))).toEqual([
            [1, 2],
            [3, 4],
        ])
    })

    it("throws if n is not a non-negative integer", () => {
        const flat = new Uint32Array([1, 2, 3, 4])
        expect(() => reconstruct1DTo2D(flat, -1)).toThrow()
        expect(() => reconstruct1DTo2D(flat, 1.5)).toThrow()
    })

    it("throws if flat.length is not divisible by n", () => {
        const flat = new Uint32Array([1, 2, 3])
        expect(() => reconstruct1DTo2D(flat, 2)).toThrow(/divisible/)
    })
})
