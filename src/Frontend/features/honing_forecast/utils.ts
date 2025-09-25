/**
 * Convert ticks (boolean grid) to counts (integer array)
 * Mirrors the Rust implementation in helpers.rs
 *
 * @param ticks - 6xN boolean array where:
 *   - rows 0-4 represent armor pieces (Helmet, Shoulder, Chest, Pants, Glove)
 *   - row 5 represents weapon
 * @returns 2xN array where:
 *   - [0][i] = sum of ticks[0..4][i] (armor count for level i)
 *   - [1][i] = ticks[5][i] ? 1 : 0 (weapon count for level i)
 */
export function ticksToCounts(ticks: boolean[][]): number[][] {
    if (ticks.length === 0 || ticks[0].length === 0) {
        return [[], []]
    }

    const cols = ticks[0].length
    const out: number[][] = [[], []]

    for (let i = 0; i < cols; i++) {
        // Sum ticks[0..4][i] for armor count
        let armorCount = 0
        for (let row = 0; row < 5; row++) {
            if (ticks[row] && ticks[row][i]) {
                armorCount++
            }
        }
        out[0][i] = armorCount

        // ticks[5][i] as 0/1 for weapon count
        out[1][i] = ticks[5] && ticks[5][i] ? 1 : 0
    }

    return out
}

/**
 * Convert counts (integer array) to ticks (boolean grid)
 * Inverse of ticksToCounts - fills checkboxes from top
 *
 * @param counts - 2xN array where:
 *   - [0][i] = armor count for level i (max 5)
 *   - [1][i] = weapon count for level i (max 1)
 * @returns 6xN boolean array
 */
export function countsToTicks(counts: number[][]): boolean[][] {
    if (counts.length < 2 || counts[0].length === 0) {
        return Array.from({ length: 6 }, () => [])
    }

    const cols = counts[0].length
    const ticks: boolean[][] = Array.from({ length: 6 }, () => Array(cols).fill(false))

    for (let i = 0; i < cols; i++) {
        const armorCount = Math.min(5, Math.max(0, counts[0][i] || 0))
        const weaponCount = Math.min(1, Math.max(0, counts[1][i] || 0))

        // Fill armor checkboxes from top
        for (let row = 0; row < armorCount; row++) {
            ticks[row][i] = true
        }

        // Fill weapon checkbox if needed
        if (weaponCount > 0) {
            ticks[5][i] = true
        }
    }

    return ticks
}
