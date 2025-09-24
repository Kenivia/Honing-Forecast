/**
 * Helper functions for histogram data remapping when locking x-axis ranges
 *
 * Behavior:
 *  - If lockedMax === newMax -> return shallow copy (no remap)
 *  - Otherwise compute overlap between each incoming bucket interval [j*newBin, (j+1)*newBin)
 *    and each locked bucket interval [i*lockedBin, (i+1)*lockedBin) and distribute counts proportionally.
 *  - If lockedMax < newMax some incoming mass may fall beyond lockedMax; that mass is dropped.
 */

/**
 * Remap a single series counts from newMax -> lockedMax using bucket overlap fractions
 */
export function remapCountsSingleSeries(countsNew: number[], newMax: number, lockedMax: number): number[] {
    const n = countsNew.length
    if (n === 0) return []

    // defensive numeric guards
    if (!isFinite(newMax) || newMax <= 0) {
        // cannot reason about bin sizes; return shallow copy
        return countsNew.slice()
    }
    if (!isFinite(lockedMax) || lockedMax <= 0) {
        // locked range is non-positive: nothing mapped
        return new Array(n).fill(0)
    }

    // bin widths
    const newBin = newMax / n
    const lockedBin = lockedMax / n

    // identical ranges -> copy
    if (lockedMax === newMax) {
        return countsNew.slice()
    }

    const out = new Array(n).fill(0)

    // For each incoming bucket j, distribute countsNew[j] across locked buckets i that overlap.
    for (let j = 0; j < n; j++) {
        const c = countsNew[j] ?? 0
        if (c === 0) continue

        const bucketStartNew = j * newBin
        const bucketEndNew = (j + 1) * newBin

        // If bucket is entirely to the right of lockedMax, skip (truncated)
        if (bucketStartNew >= lockedMax) continue

        // Determine locked bucket indices that can overlap this incoming bucket.
        // compute conservative bounds so we loop only over plausible i's
        const iStart = Math.floor(bucketStartNew / lockedBin)
        const iEnd = Math.floor((Math.min(bucketEndNew, lockedMax) - 1e-12) / lockedBin) // inclusive bound
        const i0 = Math.max(0, iStart)
        const i1 = Math.min(n - 1, iEnd)

        // If i1 < i0 it's still possible they overlap partially within first/last locked bucket,
        // so clamp and fallback to checking a small neighborhood
        if (i1 < i0) {
            // fallback: check the locked bucket that contains bucketStartNew (if in range)
            const maybeI = Math.floor(bucketStartNew / lockedBin)
            const ii = Math.min(Math.max(0, maybeI), n - 1)
            const lockedStart = ii * lockedBin
            const lockedEnd = (ii + 1) * lockedBin
            const left = Math.max(bucketStartNew, lockedStart)
            const right = Math.min(bucketEndNew, lockedEnd, lockedMax)
            const overlap = Math.max(0, right - left)
            if (overlap > 0) {
                out[ii] += c * (overlap / newBin)
            }
            continue
        }

        for (let i = i0; i <= i1; i++) {
            const lockedStart = i * lockedBin
            const lockedEnd = (i + 1) * lockedBin
            // restrict right to lockedMax so we drop any portion past lockedMax
            const left = Math.max(bucketStartNew, lockedStart)
            const right = Math.min(bucketEndNew, lockedEnd, lockedMax)
            const overlap = Math.max(0, right - left)
            if (overlap <= 0) continue
            const fraction = overlap / newBin
            out[i] += c * fraction
        }
    }

    return out
}

/**
 * Remap all series given arrays
 */
export function remapCountsToLockedXAxis(countsArr: number[][], newMaxsArr: number[] | null | undefined, lockedMaxsArr: number[] | null): number[][] {
    if (!countsArr) return countsArr
    if (!lockedMaxsArr) return countsArr

    const nSeries = countsArr.length
    const out: number[][] = Array.from({ length: nSeries }, () => [])

    for (let i = 0; i < nSeries; i++) {
        const countsNew = countsArr[i] || []
        const newMax = newMaxsArr && typeof newMaxsArr[i] === "number" ? newMaxsArr[i] : 1
        const lockedMax = typeof lockedMaxsArr[i] === "number" ? lockedMaxsArr[i] : newMax

        // Use remapping routine for all cases; it will handle lockedMax > newMax (expansion),
        // lockedMax < newMax (compression + truncation) and equality (copy).
        out[i] = remapCountsSingleSeries(countsNew, newMax, lockedMax)
    }

    return out
}
