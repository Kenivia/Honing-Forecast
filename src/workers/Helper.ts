export const LABELS = ["Red", "Blue", "Leaps", "Shards", "Oreha", "Gold", "Silver(WIP)"]

// Counts how many of each piece type is ticked
export function ticks_to_counts(ticks: boolean[][]) {
    let out = Array.from({ length: 2 }, () => new Array(ticks[0].length).fill(0))
    for (let i = 0; i < ticks[0].length; i++) {
        out[0][i] = (ticks[0][i] ? 1 : 0) + (ticks[1][i] ? 1 : 0) + (ticks[2][i] ? 1 : 0) + (ticks[3][i] ? 1 : 0) + (ticks[4][i] ? 1 : 0)
        out[1][i] = ticks[5][i] ? 1 : 0
    }
    return out
}

// Supposed the top of the line shuffle algorithm from stack overflow
export function shuffle(array: Uint32Array<ArrayBuffer>[] | number[]) {
    let currentIndex = array.length
    while (currentIndex != 0) {
        let randomIndex = Math.floor(Math.random() * currentIndex)
        currentIndex--
        ;[array[currentIndex], array[randomIndex]] = [array[randomIndex], array[currentIndex]]
    }
}

// I didnt know throw existed and now i can't be bothered to change
export function assert(condition: boolean) {
    if (!condition) {
        throw "Assertion failed"
    }
}

// vibe coded, used for adjusting best_budget
export function add_cost(a: Uint32Array<ArrayBuffer>, b: number[][], index: number, multiplier = 1) {
    var n = Math.min(a.length, b.length)
    var i = n
    while (i--) {
        a[i] = Number(a[i]) + Number(b[i][index]) * multiplier
    }
    return a
}

export function myformat(f: number): string {
    f *= 100
    let place = 0
    while (true) {
        if (f >= 1 / 10 ** place) {
            return f.toFixed(place)
        }
        if (place >= 4) {
            return "0"
        }
        place++
    }
}

// Average and Pity give the same results as the maxroll calculator until advanced honing is involved,
// i think they genuinely forced the RNG to go to the BEST BEST and the WORST WORST for those
// scenarios, which didn't happen for me in 10 million trials
// can't be bothered to match them, so I no longer show this(if people wanted to compare they can go to maxroll)
// Although I'll need average for calculating free tap & juice values, so leaving this here
// export function average(prob_dist_arr, costs, unlock, adv_hone_chances, adv_hone_costs) {
//     let average = Array(adv_hone_costs[0].length).fill(0)
//     for (const [piece, chances] of prob_dist_arr.entries()) {
//         for (const [cost_type, cost] of costs.entries()) {
//             for (const [tap_1, prob] of chances.entries()) {
//                 average[cost_type] += (tap_1 + 1) * prob * cost[piece]
//             }
//         }
//     }
//     for (const [piece, chances] of adv_hone_chances.entries()) {
//         for (const [cost_type, _cost] of adv_hone_costs[piece].entries()) {
//             for (const [tap_1, prob] of chances.entries()) {
//                 average[cost_type] += prob * adv_hone_costs[piece][cost_type][tap_1]
//             }
//         }
//     }
//     average[3] += unlock[0]
//     average[6] += unlock[1]
//     return average
// }

// vibe coded, at least the first parts, seems like a lot of code for adding on shard & silver unlock costs...
export function Unlock(hone_counts: number[][], weap_unlock: number[][], armor_unlock: number[][], adv_counts = null, adv_unlock = null): [number, number] {
    let shard_unlock = 0
    let silver_unlock = 0
    for (const [cost_type, element] of weap_unlock.entries()) {
        for (const [index, cost] of element.entries()) {
            if (cost_type == 0) {
                shard_unlock += hone_counts[1][index] * cost
            }
            if (cost_type == 1) {
                silver_unlock += hone_counts[1][index] * cost
            }
        }
    }
    for (const [cost_type, element] of armor_unlock.entries()) {
        for (const [index, cost] of element.entries()) {
            if (cost_type == 0) {
                shard_unlock += hone_counts[0][index] * cost
            }
            if (cost_type == 1) {
                silver_unlock += hone_counts[0][index] * cost
            }
        }
    }
    if (Array.isArray(adv_unlock)) {
        for (const [cost_type, element] of adv_unlock.entries()) {
            for (const [index, cost] of element.entries()) {
                if (index % 2 == 1) {
                    if (cost_type == 0) {
                        shard_unlock += adv_counts[0][Math.floor((index - 1) / 2)] * cost
                    }
                    if (cost_type == 1) {
                        silver_unlock += adv_counts[0][Math.floor((index - 1) / 2)] * cost
                    }
                }
                if (index % 2 == 0) {
                    if (cost_type == 0) {
                        shard_unlock += adv_counts[1][Math.floor(index / 2)] * cost
                    }
                    if (cost_type == 1) {
                        silver_unlock += adv_counts[1][Math.floor(index / 2)] * cost
                    }
                }
            }
        }
    }

    return [shard_unlock, silver_unlock]
}
