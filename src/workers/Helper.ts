export function ticks_to_counts(ticks: boolean[][]) {
    let out = Array.from({ length: 2 }, () => new Array(ticks[0].length).fill(0))
    for (let i = 0; i < ticks[0].length; i++) {
        out[0][i] = (ticks[0][i] ? 1 : 0) + (ticks[1][i] ? 1 : 0) + (ticks[2][i] ? 1 : 0) + (ticks[3][i] ? 1 : 0) + (ticks[4][i] ? 1 : 0)
        out[1][i] = ticks[5][i] ? 1 : 0
    }
    return out
}

export function shuffle(array: Uint32Array<ArrayBuffer>[]) {
    let currentIndex = array.length

    // While there remain elements to shuffle...
    while (currentIndex != 0) {
        // Pick a remaining element...
        let randomIndex = Math.floor(Math.random() * currentIndex)
        currentIndex--

        // And swap it with the current element.
        ;[array[currentIndex], array[randomIndex]] = [array[randomIndex], array[currentIndex]]
    }
}

export function assert(condition: boolean) {
    if (!condition) {
        throw "Assertion failed"
    }
}
export function add_cost(a: Uint32Array<ArrayBuffer>, b: number[][], index: number, multiplier = 1) {
    var n = Math.min(a.length, b.length)
    var i = n
    while (i--) {
        a[i] = Number(a[i]) + Number(b[i][index]) * multiplier
    }
    return a // returns the modified a
}

// export function weighted_random(items, weights) {
//     var i
//     let newArr = weights.map(
//         (
//             (sum) => (value) =>
//                 (sum += value)
//         )(0)
//     )
//     // for (i = 1; i < weights.length; i++) newArr[i] += newArr[i - 1]

//     var random = Math.random() //* newArr[newArr.length - 1]

//     // for (i = 0; i < newArr.length; i++) if (newArr[i] > random) break
//     i = newArr.findIndex((el) => random <= el)
//     return items[i]
// }
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

// export function pity(prob_dist_arr, costs, unlock, adv_hone_chances, adv_hone_costs) {
//     let pity = Array(adv_hone_costs[0].length).fill(0)
//     for (const [piece, chances] of prob_dist_arr.entries()) {
//         for (const [cost_type, cost] of costs.entries()) {
//             pity[cost_type] += chances.length * cost[piece]
//         }
//     }
//     for (const [piece, _chances] of adv_hone_chances.entries()) {
//         for (const [cost_type, _cost] of adv_hone_costs[piece].entries()) {
//               pity[cost_type] +=  adv_hone_costs[piece][cost_type][adv_hone_costs[piece][cost_type].length-1]
//         }
//     }
//     pity[3] += unlock[0]
//     pity[6] += unlock[1]
//     return pity
// }

// vibe coded
export function countFailuresGAS(cost_data: Uint32Array<ArrayBuffer>[], budget_data: Uint32Array<ArrayBuffer>[]): number[] {
    const N = cost_data.length
    const M = budget_data.length
    if (N === 0 || M === 0) return new Array(M).fill(0)

    const count = new Uint32Array(M) // faster numeric storage
    // assume n === 7
    for (let m = 0; m < N; m++) {
        const c = cost_data[m]
        // cache each cost value into local variables (avoids repeated array indexing)
        const c0 = c[0],
            c1 = c[1],
            c2 = c[2],
            c3 = c[3],
            c4 = c[4],
            c5 = c[5],
            c6 = c[6]

        for (let i = 0; i < M; i++) {
            const b = budget_data[i]
            // inline/unrolled comparison with a single boolean expression and no inner loop
            if (c0 > b[0] || c1 > b[1] || c2 > b[2] || c3 > b[3] || c4 > b[4] || c5 > b[5] || c6 > b[6]) {
                count[i]++
            }
        }
    }

    // return plain array (optional)
    return Array.from(count)
}

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

// // vibe coded
// export function ParseBottlenecks(inputRange: any[], targetSums: any[]) {
//   var raws  = inputRange .map(r => (typeof r[0]==='string') ? r[0] : '');
//   var sums  = targetSums.map(r => parseFloat(r[0]) || 0);

//   var categories = ['Red','Blue','Leaps','Shards','Oreha','Gold','Silver(WIP)'];

//   function parseOne(str) {
//     var row = categories.map( () =>0);
//     str.split(',').forEach(item => {
//       var m = item.match(/^\s*([^()]+)\((\d+)%\)/);
//       if (m) {
//         var name = m[1].trim(),
//             pct  = parseInt(m[2],10),
//             i    = categories.indexOf(name);
//         if (i>=0) row[i] = pct;
//       }
//     });
//     return row;
//   }

//   return raws.map(function(str, idx) {
//     var row   = parseOne(str),
//         total = row.reduce((a,b)=>a+b, 0),
//         target= sums[idx];
//     if (total>0 && target!==0) {
//       var factor = target/total;
//       return row.map(v=> v * factor);
//     } else {
//       // if no data or zero target, return zeros
//       return categories.map(()=>"");
//     }
//   });
// }
