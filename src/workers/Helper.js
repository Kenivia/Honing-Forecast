export function ticks_to_counts(ticks){
    let out = Array.from({ length: 2 }, () => new Array(ticks[0].length).fill(0));
    for (let i = 0; i < ticks[0].length; i++){
        out[0][i] = (ticks[0][i] ? 1:0)+(ticks[1][i] ? 1:0)+(ticks[2][i] ? 1:0)+(ticks[3][i] ? 1:0)+(ticks[4][i] ? 1:0)
        out[1][i] = (ticks[5][i] ? 1:0)
    }
    return out
}






export function assert(condition) {
    if (!condition) {
        throw "Assertion failed"  
    }
}
export function add_cost(a, b, index, multiplier=1) {
  if (!Array.isArray(a) || !Array.isArray(b)) throw new Error('Both inputs must be arrays');
  var n = Math.min(a.length,b.length);
  var i = n;
  while (i--) {
    a[i] = Number(a[i]) + Number(b[i][index]) * multiplier;
  }
  return a; // returns the modified a
}

export function weighted_random(items, weights) {
    var i
    let newArr = weights.map(
        (
            (sum) => (value) =>
                (sum += value)
        )(0)
    )
    // for (i = 1; i < weights.length; i++) newArr[i] += newArr[i - 1]

    var random = Math.random() //* newArr[newArr.length - 1]

    // for (i = 0; i < newArr.length; i++) if (newArr[i] > random) break
    i = newArr.findIndex((el) => random <= el)
    return items[i]
}
export function myformat(f) {
    f *= 100
    let place = 0
    while (true){
      if (f>= 1/(10**place)){
        return f.toFixed(place)
      }
      if (place >= 4){
        return "0"
      }
      place ++
    }
    
}
export function average(ind_chances, costs, unlock, adv_hone_chances, adv_hone_costs) {  
    let average = Array(adv_hone_costs[0].length).fill(0)
    for (const [piece, chances] of ind_chances.entries()) {
        for (const [cost_type, cost] of costs.entries()) {
            for (const [tap_1, prob] of chances.entries()) {
                average[cost_type] += (tap_1 + 1) * prob * cost[piece]
            }
        }
    }
    for (const [piece, chances] of adv_hone_chances.entries()) {
        for (const [cost_type, _cost] of adv_hone_costs[piece].entries()) {
            for (const [tap_1, prob] of chances.entries()) {
                average[cost_type] += prob * adv_hone_costs[piece][cost_type][tap_1]
            }
        }
    }
    average[3] += unlock[0]
    average[6] += unlock[1]
    return average
}

export function pity(ind_chances, costs, unlock, adv_hone_chances, adv_hone_costs) {
    let pity = Array(adv_hone_costs[0].length).fill(0)
    for (const [piece, chances] of ind_chances.entries()) {
        for (const [cost_type, cost] of costs.entries()) {
            pity[cost_type] += chances.length * cost[piece]
        }
    }
    for (const [piece, _chances] of adv_hone_chances.entries()) {
        for (const [cost_type, _cost] of adv_hone_costs[piece].entries()) {
              pity[cost_type] +=  adv_hone_costs[piece][cost_type][adv_hone_costs[piece][cost_type].length-1]
        }
    }
    pity[3] += unlock[0]
    pity[6] += unlock[1]
    return pity
}
// vibe coded
export function countFailuresGAS(cost_data, budget_data) {
  var N = cost_data.length;
  var M = budget_data.length;
  var n = (N>0 ? cost_data[0].length : 0);
  if (N === 0 || M === 0 || n === 0) return new Array(M).fill(0);

  // choose strategy: if many more budgets than cost rows, transpose budgets
  // Threshold is heuristic: if M >= 3 * N, transpose; tweak for your data.
  if (M >= 3 * N) {
    return _countFailuresTranspose(cost_data, budget_data);
  } else {
    return _countFailuresSimple(cost_data, budget_data);
  }
}
export function _countFailuresSimple(cost_data, budget_data) {
  var N = cost_data.length;
  var M = budget_data.length;
  var count = new Array(M);
  for (var i = 0; i < M; i++) count[i] = 0;

  for (var m = 0; m < N; m++) {
    var c = cost_data[m];
    // cache cost values into locals
    var c0 = c[0], c1 = c[1], c2 = c[2], c3 = c[3], c4 = c[4], c5 = c[5], c6 = c[6];

    // inner loop over budgets
    for (var j = 0; j < M; j++) {
      var b = budget_data[j];
      // cache budget values into locals (reduces repeated b[...] lookups)
      var b0 = b[0], b1 = b[1], b2 = b[2], b3 = b[3], b4 = b[4], b5 = b[5], b6 = b[6];
      // short-circuit comparisons (left-to-right)
      if (c0 > b0 || c1 > b1 || c2 > b2 || c3 > b3 || c4 > b4 || c5 > b5 || c6 > b6) {
        count[j]++;
      }
    }
  }
  return count;
}
export function _countFailuresTranspose(cost_data, budget_data) {
  var N = cost_data.length;
  var M = budget_data.length;

  // build column arrays budgetsCols[j][i] = budget_data[i][j]
  var budgetsCols = [
    new Array(M), new Array(M), new Array(M),
    new Array(M), new Array(M), new Array(M), new Array(M)
  ];
  for (var i = 0; i < M; i++) {
    var b = budget_data[i];
    budgetsCols[0][i] = b[0];
    budgetsCols[1][i] = b[1];
    budgetsCols[2][i] = b[2];
    budgetsCols[3][i] = b[3];
    budgetsCols[4][i] = b[4];
    budgetsCols[5][i] = b[5];
    budgetsCols[6][i] = b[6];
  }

  var count = new Array(M);
  for (let i = 0; i < M; i++) count[i] = 0;

  for (var m = 0; m < N; m++) {
    var c = cost_data[m];
    var c0 = c[0], c1 = c[1], c2 = c[2], c3 = c[3], c4 = c[4], c5 = c[5], c6 = c[6];

    // iterate budgets by index but read from contiguous column arrays
    for (let i = 0; i < M; i++) {
      // read each column's element for index i
      if (
        c0 > budgetsCols[0][i] ||
        c1 > budgetsCols[1][i] ||
        c2 > budgetsCols[2][i] ||
        c3 > budgetsCols[3][i] ||
        c4 > budgetsCols[4][i] ||
        c5 > budgetsCols[5][i] ||
        c6 > budgetsCols[6][i]
      ) {
        count[i]++;
      }
    }
  }

  return count;
}





export function Unlock(hone_counts, weap_unlock, armor_unlock, adv_counts=null, adv_unlock=null) {
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
    if (Array.isArray(adv_unlock)){
      for (const [cost_type, element] of adv_unlock.entries()) {
          for (const [index, cost] of element.entries()) {
            if (index % 2 == 1){
              if (cost_type == 0) {
                  shard_unlock += adv_counts[0][ Math.floor((index -1) / 2)] * cost
              }
              if (cost_type == 1) {
                  silver_unlock += adv_counts[0][Math.floor((index -1) / 2)] * cost
              }
            }
            if (index % 2 == 0){
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



// vibe coded
export function ParseBottlenecks(inputRange, targetSums) {
  var raws  = inputRange .map(r => (typeof r[0]==='string') ? r[0] : '');
  var sums  = targetSums.map(r => parseFloat(r[0]) || 0);
  
  var categories = ['Red','Blue','Leaps','Shards','Oreha','Gold','Silver(WIP)'];
  
  function parseOne(str) {
    var row = categories.map( () =>0);
    str.split(',').forEach(item => {
      var m = item.match(/^\s*([^()]+)\((\d+)%\)/);
      if (m) {
        var name = m[1].trim(),
            pct  = parseInt(m[2],10),
            i    = categories.indexOf(name);
        if (i>=0) row[i] = pct;
      }
    });
    return row;
  }
  
  return raws.map(function(str, idx) {
    var row   = parseOne(str),
        total = row.reduce((a,b)=>a+b, 0),
        target= sums[idx];
    if (total>0 && target!==0) {
      var factor = target/total;
      return row.map(v=> v * factor);
    } else {
      // if no data or zero target, return zeros
      return categories.map(()=>"");
    }
  });
}


// // test function 
// function GenerateData(counts, chances, weap_costs, armor_costs, weap_unlock, armor_unlock,  actual_budgets, labels, time_limit, adv_counts, adv_costs, adv_unlock, adv_data_10_20, adv_data_30_40 , rigged=false){
//   let [ind_chances, hone_costs, adv_hone_chances, adv_hone_costs] = parser(hone_counts, chances, weap_costs, armor_costs, adv_counts, adv_costs,adv_data_10_20_juice, adv_data_30_40_juice,adv_data_10_20, adv_data_30_40, adv_hone_strategy)
    
//   // let adv_hone_costs = JSON.parse(_json_adv_cost)
//   // return adv_hone_costs[0]
//   const cost_size = 10;
//   return MC_data(ind_chances, hone_costs, time_limit, cost_size, counts, weap_unlock, armor_unlock,adv_counts,adv_hone_chances, adv_hone_costs, adv_unlock,)
// }