// +11 to +18 double artisan, +15 to 18 mats cost reduced by 10%, unlock cost reduced by 20%
pub static EVENT_ARTISAN_MULTIPLIER: [f64; 25] = [
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
];

pub static EVENT_COST_REDUCTION: [[f64; 25]; 7] = [[
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.9, 0.9, 0.9, 0.9, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
]; 7];

pub static EVENT_UNLOCK_REDUCTION: [[f64; 25]; 2] = [[
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.8, 0.8, 0.8, 0.8, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
]; 2];

pub static NORMAL_JUICE_COST: [i64; 25] = [
    0, 0, 0, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 25, 25, 25, 25, 50, 50,
];

pub static SPECIAL_LEAPS_COST: [[i64; 25]; 2] = [
    [
        12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 16, 16, 16, 16, 20, 20, 20, 20, 20, 20, 20,
        20, 20,
    ],
    [
        30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 40, 40, 40, 40, 50, 50, 50, 50, 50, 50, 50,
        50, 50,
    ],
];

// Histogram bucket count
pub const BUCKET_COUNT: usize = 1000;

// RNG seed for deterministic testing
#[allow(dead_code)]
pub const RNG_SEED: u64 = 123456;

pub static NORMAL_HONE_CHANCES: [f64; 25] = [
    1.0, 1.0, 1.0, 0.45, 0.45, 0.45, 0.3, 0.3, 0.15, 0.15, 0.1, 0.1, 0.05, 0.05, 0.04, 0.04, 0.03,
    0.03, 0.03, 0.015, 0.015, 0.01, 0.01, 0.005, 0.005,
];

//  [  ( [ (upgrade_index, % amt, cost per juice(books = 1)  )] , gold_value of the juice/book wep version, armor version) ]
// add new entries from the bottom, order matters
pub static JUICE_BOOKS_AVAIL: &[&[(usize, f64, i64)]] = &[
    &[
        (3, 0.45, 20),
        (4, 0.45, 20),
        (5, 0.45, 20),
        (6, 0.3, 20),
        (7, 0.3, 20),
        (8, 0.15, 20),
        (9, 0.15, 20),
        (10, 0.1, 20),
        (11, 0.1, 20),
        (12, 0.05, 20),
        (13, 0.05, 20),
        (14, 0.04, 20),
        (15, 0.04, 20),
        (16, 0.03, 20),
        (17, 0.03, 20),
        (18, 0.03, 20),
        (19, 0.015, 25),
        (20, 0.015, 25),
        (21, 0.01, 25),
        (22, 0.01, 25),
        (23, 0.005, 50),
        (24, 0.005, 50),
    ],
    // Hellfire 11-14
    &[(10, 0.1, 1), (11, 0.1, 1), (12, 0.05, 1), (13, 0.05, 1)],
    // 15-18
    &[(14, 0.04, 1), (15, 0.04, 1), (16, 0.03, 1), (17, 0.03, 1)],
    //19-20
    &[(18, 0.03, 1), (19, 0.015, 1)],
];
#[derive(Debug)]
pub struct JuiceInfo {
    pub chances: Vec<Vec<f64>>,
    pub gold_costs: Vec<Vec<(f64, f64)>>,
    pub amt_used: Vec<Vec<i64>>,
    pub ids: Vec<Vec<usize>>,
    pub one_gold_cost: Vec<(f64, f64)>,
    pub leftover_values: Vec<Vec<(f64, f64)>>,
    pub one_leftover_value: Vec<(f64, f64)>,
    pub amt_used_id: Vec<Vec<i64>>,
}
pub fn get_avail_juice_combs(
    juice_prices: &[(f64, f64)],
    leftover_prices: &[(f64, f64)],
) -> JuiceInfo {
    let mut chances: Vec<Vec<f64>> = vec![vec![]; 25];
    let mut gold_costs: Vec<Vec<(f64, f64)>> = vec![vec![]; 25];
    let mut leftover_values: Vec<Vec<(f64, f64)>> = vec![vec![]; 25];
    let mut amt_used: Vec<Vec<i64>> = vec![vec![]; 25];
    let mut ids: Vec<Vec<usize>> = vec![vec![]; 25];

    let mut amt_used_id: Vec<Vec<i64>> = vec![vec![0; 25]; JUICE_BOOKS_AVAIL.len()];
    let mut one_gold_cost: Vec<(f64, f64)> = vec![(0.0, 0.0); JUICE_BOOKS_AVAIL.len()];
    let mut one_leftover_value: Vec<(f64, f64)> = vec![(0.0, 0.0); JUICE_BOOKS_AVAIL.len()];
    for (id, rows) in JUICE_BOOKS_AVAIL.iter().enumerate() {
        for &(upgrade_index, chance, amt) in rows.iter() {
            chances[upgrade_index].push(chance);

            gold_costs[upgrade_index].push((
                amt as f64 * juice_prices[id].0,
                amt as f64 * juice_prices[id].1,
            ));
            leftover_values[upgrade_index].push((
                amt as f64 * leftover_prices[id].0,
                amt as f64 * leftover_prices[id].1,
            ));
            amt_used[upgrade_index].push(amt);
            ids[upgrade_index].push(id);
            amt_used_id[id][upgrade_index] = amt;
        }
        one_gold_cost[id] = (juice_prices[id].0, juice_prices[id].1);
        one_leftover_value[id] = (leftover_prices[id].0, leftover_prices[id].1);
    }
    JuiceInfo {
        chances,
        gold_costs,
        amt_used,
        ids,
        one_gold_cost,
        leftover_values,
        one_leftover_value,
        amt_used_id,
    }
}
// these costs are manually copied from lost ark codex, dont bet on it being 100% correct
pub static DEFAULT_NORMAL_HONE_WEAPON_COST: [[i64; 25]; 7] = [
    [
        350, 450, 550, 650, 750, 800, 900, 1000, 1050, 1150, 1250, 1300, 1400, 1550, 1700, 1950,
        2200, 2450, 2700, 2950, 3200, 3700, 4000, 4200, 4500,
    ],
    [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        12, 13, 14, 15, 15, 15, 16, 16, 18, 18, 18, 21, 24, 27, 30, 33, 36, 39, 42, 45, 48, 52, 56,
        60, 65,
    ],
    [
        3000, 3100, 3200, 3300, 3500, 3700, 3900, 4200, 4400, 4700, 5000, 5300, 7600, 8200, 8800,
        9400, 12000, 12900, 13700, 16000, 17100, 18200, 19200, 20400, 21500,
    ],
    [
        5, 5, 5, 6, 6, 6, 8, 8, 10, 10, 12, 12, 15, 15, 18, 18, 25, 25, 25, 35, 35, 35, 35, 50, 50,
    ],
    [
        780, 790, 810, 860, 910, 990, 1080, 1190, 1310, 1460, 1620, 1790, 1990, 2200, 2430, 2670,
        2940, 3220, 3510, 3830, 4160, 4510, 4870, 5250, 5650,
    ],
    [
        50000, 50000, 50000, 50000, 50000, 50000, 55000, 55000, 55000, 55000, 55000, 55000, 55000,
        55000, 55000, 55000, 65000, 65000, 65000, 90000, 90000, 120000, 120000, 150000, 150000,
    ],
];

pub static DEFAULT_NORMAL_HONE_ARMOR_COST: [[i64; 25]; 7] = [
    [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        210, 270, 330, 390, 450, 480, 540, 600, 630, 690, 750, 780, 840, 930, 1020, 1170, 1320,
        1470, 1620, 1770, 1920, 2220, 2400, 2520, 2700,
    ],
    [
        7, 8, 8, 9, 9, 9, 10, 10, 11, 11, 11, 13, 14, 16, 18, 20, 22, 23, 25, 27, 29, 31, 34, 36,
        40,
    ],
    [
        1800, 1860, 1920, 1980, 2100, 2220, 2340, 2520, 2640, 2820, 3000, 3180, 4560, 4920, 5280,
        5640, 7200, 7740, 8220, 9600, 10260, 10920, 11520, 12240, 12900,
    ],
    [
        3, 3, 3, 4, 4, 4, 5, 5, 6, 6, 7, 7, 9, 9, 11, 11, 15, 15, 15, 21, 21, 21, 21, 30, 30,
    ],
    [
        470, 480, 490, 520, 550, 590, 650, 710, 790, 880, 970, 1070, 1190, 1320, 1460, 1600, 1760,
        1930, 2110, 2300, 2500, 2710, 2920, 3150, 3390,
    ],
    [
        30000, 30000, 30000, 30000, 30000, 30000, 33000, 33000, 33000, 33000, 33000, 33000, 33000,
        33000, 33000, 33000, 39000, 39000, 39000, 54000, 54000, 72000, 72000, 90000, 90000,
    ],
];

// these are from Maxroll
pub static NORMAL_HONE_WEAPON_UNLOCK: [[i64; 25]; 2] = [
    [
        15000, 15000, 15000, 15000, 15000, 16000, 17000, 17000, 18000, 20000, 21000, 23000, 33000,
        38000, 43000, 49000, 66000, 75000, 85000, 106000, 120000, 135000, 152000, 170000, 190000,
    ],
    [
        240000, 240000, 240000, 270000, 270000, 320000, 340000, 374000, 396000, 520000, 525000,
        690000, 825000, 950000, 1075000, 1225000, 1650000, 1875000, 1955000, 2120000, 2400000,
        2700000, 3040000, 3400000, 4750000,
    ],
];

pub static NORMAL_HONE_ARMOR_UNLOCK: [[i64; 25]; 2] = [
    [
        9000, 9000, 9000, 9000, 9000, 9000, 10000, 10000, 10000, 12000, 12000, 13000, 19000, 22000,
        25000, 29000, 39000, 45000, 51000, 63000, 72000, 81000, 91000, 102000, 114000,
    ],
    [
        144000, 144000, 144000, 162000, 162000, 180000, 200000, 220000, 220000, 312000, 300000,
        390000, 475000, 550000, 625000, 725000, 975000, 1125000, 1173000, 1260000, 1440000,
        1620000, 1820000, 2040000, 2850000,
    ],
];

pub static ADV_HONE_COST: [[i64; 8]; 8] = [
    [300, 0, 550, 0, 1200, 0, 1400, 0],
    [0, 250, 0, 450, 0, 1000, 0, 1200],
    [8, 6, 11, 8, 25, 18, 32, 23],
    [4000, 2400, 8000, 4800, 11500, 7000, 13000, 8000],
    [12, 7, 13, 8, 28, 17, 30, 19],
    [900, 760, 2000, 1440, 3000, 2000, 4000, 2400],
    [25200, 22800, 40000, 32000, 55000, 44000, 70000, 56000],
    [6, 6, 9, 9, 20, 20, 24, 24],
];

pub static ADV_HONE_UNLOCK: [[i64; 8]; 2] = [
    [40000, 24000, 80000, 48000, 115000, 70000, 230000, 140000],
    [
        1440000, 1120000, 1600000, 1280000, 2300000, 1850000, 2530000, 2035000,
    ],
];

// columns:
// tap count, average juice used * 1000(so it's i64), frequency(out of 10 mil)
pub static ADV_DATA_10_20_JUICE: [[i64; 3]; 42] = [
    [22, 4000, 2],
    [23, 4795, 39],
    [24, 5643, 98],
    [25, 6076, 211],
    [26, 6231, 295],
    [27, 5291, 743],
    [28, 4494, 5309],
    [29, 5196, 14068],
    [30, 5848, 21224],
    [31, 6205, 24154],
    [32, 6158, 26455],
    [33, 5700, 41482],
    [34, 5518, 129055],
    [35, 5926, 271334],
    [36, 6518, 322652],
    [37, 6835, 284537],
    [38, 6697, 251493],
    [39, 6380, 328783],
    [40, 6451, 669210],
    [41, 6814, 1042097],
    [42, 7248, 985885],
    [43, 7469, 679618],
    [44, 7240, 506165],
    [45, 7111, 606035],
    [46, 7352, 905313],
    [47, 7671, 973818],
    [48, 7931, 636323],
    [49, 7937, 323301],
    [50, 7756, 224617],
    [51, 7911, 245260],
    [52, 8185, 236764],
    [53, 8394, 139800],
    [54, 8485, 52988],
    [55, 8402, 20811],
    [56, 8496, 14125],
    [57, 8762, 10046],
    [58, 8962, 4556],
    [59, 9072, 1062],
    [60, 9048, 188],
    [61, 9149, 47],
    [62, 9615, 26],
    [63, 9818, 11],
];
pub static ADV_DATA_30_40_JUICE: [[i64; 3]; 52] = [
    [21, 8500, 4],
    [22, 7222, 9],
    [23, 5193, 88],
    [24, 6030, 328],
    [25, 6793, 608],
    [26, 7428, 853],
    [27, 7657, 1018],
    [28, 6642, 1848],
    [29, 5886, 6525],
    [30, 6392, 15548],
    [31, 7132, 22614],
    [32, 7733, 25735],
    [33, 7962, 26110],
    [34, 7235, 34830],
    [35, 6644, 77913],
    [36, 6980, 154208],
    [37, 7621, 205642],
    [38, 8183, 203592],
    [39, 8367, 178877],
    [40, 7837, 198346],
    [41, 7406, 337776],
    [42, 7650, 554173],
    [43, 8175, 662355],
    [44, 8675, 588277],
    [45, 8823, 461672],
    [46, 8410, 439871],
    [47, 8122, 599895],
    [48, 8333, 817973],
    [49, 8748, 857608],
    [50, 9151, 668545],
    [51, 9256, 455879],
    [52, 8939, 371652],
    [53, 8786, 414863],
    [54, 8979, 459290],
    [55, 9295, 398486],
    [56, 9594, 260799],
    [57, 9680, 146625],
    [58, 9474, 95786],
    [59, 9397, 83428],
    [60, 9567, 72103],
    [61, 9806, 49853],
    [62, 10031, 25787],
    [63, 10144, 10949],
    [64, 10028, 5055],
    [65, 9953, 3077],
    [66, 10121, 1936],
    [67, 10305, 1022],
    [68, 10430, 419],
    [69, 10635, 115],
    [70, 10885, 26],
    [71, 10429, 7],
    [72, 10500, 2],
];

pub static ADV_DATA_10_20: [[i64; 3]; 51] = [
    [24, 0, 1],
    [25, 0, 1],
    [28, 0, 6],
    [29, 0, 9],
    [30, 0, 26],
    [31, 0, 32],
    [32, 0, 52],
    [33, 0, 85],
    [34, 0, 267],
    [35, 0, 621],
    [36, 0, 1256],
    [37, 0, 1672],
    [38, 0, 2349],
    [39, 0, 3385],
    [40, 0, 6297],
    [41, 0, 12826],
    [42, 0, 23046],
    [43, 0, 32724],
    [44, 0, 40714],
    [45, 0, 50937],
    [46, 0, 74146],
    [47, 0, 125135],
    [48, 0, 195974],
    [49, 0, 258368],
    [50, 0, 296130],
    [51, 0, 326203],
    [52, 0, 395992],
    [53, 0, 549695],
    [54, 0, 747081],
    [55, 0, 862872],
    [56, 0, 833489],
    [57, 0, 745729],
    [58, 0, 727344],
    [59, 0, 786459],
    [60, 0, 810029],
    [61, 0, 689339],
    [62, 0, 479708],
    [63, 0, 311591],
    [64, 0, 218914],
    [65, 0, 166035],
    [66, 0, 114395],
    [67, 0, 63448],
    [68, 0, 27788],
    [69, 0, 10646],
    [70, 0, 4322],
    [71, 0, 1884],
    [72, 0, 705],
    [73, 0, 221],
    [74, 0, 45],
    [75, 0, 6],
    [76, 0, 1],
];

pub static ADV_DATA_30_40: [[i64; 3]; 53] = [
    [23, 0, 4],
    [24, 0, 9],
    [25, 0, 19],
    [26, 0, 51],
    [27, 0, 52],
    [28, 0, 80],
    [29, 0, 219],
    [30, 0, 886],
    [31, 0, 1746],
    [32, 0, 2369],
    [33, 0, 2943],
    [34, 0, 3650],
    [35, 0, 6970],
    [36, 0, 16758],
    [37, 0, 30605],
    [38, 0, 40920],
    [39, 0, 44762],
    [40, 0, 48334],
    [41, 0, 70495],
    [42, 0, 130738],
    [43, 0, 212965],
    [44, 0, 262089],
    [45, 0, 262241],
    [46, 0, 252262],
    [47, 0, 306453],
    [48, 0, 466362],
    [49, 0, 654423],
    [50, 0, 725110],
    [51, 0, 644656],
    [52, 0, 542435],
    [53, 0, 560308],
    [54, 0, 705902],
    [55, 0, 824623],
    [56, 0, 766965],
    [57, 0, 565420],
    [58, 0, 392767],
    [59, 0, 332250],
    [60, 0, 330994],
    [61, 0, 300193],
    [62, 0, 214095],
    [63, 0, 119646],
    [64, 0, 60469],
    [65, 0, 37048],
    [66, 0, 26980],
    [67, 0, 17817],
    [68, 0, 9189],
    [69, 0, 3399],
    [70, 0, 986],
    [71, 0, 253],
    [72, 0, 74],
    [73, 0, 11],
    [74, 0, 4],
    [75, 0, 1],
];

pub fn get_adv_data_juice(segment: i64) -> f64 {
    let mut out: f64 = 0.0_f64;
    let data: &[[i64; 3]] = if segment <= 1 {
        &ADV_DATA_10_20_JUICE
    } else {
        &ADV_DATA_30_40_JUICE
    };
    let sum_taps: i64 = data
        .iter()
        .map(|row| row.get(2).copied().unwrap_or(0))
        .sum();
    for i in 0..data.len() {
        out += data[i][1] as f64 * data[i][2] as f64 / sum_taps as f64;
    }
    out /= 1000.0_f64;
    out
}

pub fn get_event_modified_armor_unlock_cost(express_event: bool) -> [[i64; 25]; 2] {
    if !express_event {
        return NORMAL_HONE_ARMOR_UNLOCK;
    }

    let mut result = [[0i64; 25]; 2];
    for cost_type in 0..2 {
        for level in 0..25 {
            let base_cost = NORMAL_HONE_ARMOR_UNLOCK[cost_type][level] as f64;
            let reduction = EVENT_UNLOCK_REDUCTION[cost_type][level];
            result[cost_type][level] = (base_cost * reduction).ceil() as i64;
        }
    }
    result
}

pub fn get_event_modified_adv_unlock_cost(express_event: bool) -> [[i64; 8]; 2] {
    if !express_event {
        return ADV_HONE_UNLOCK;
    }

    let mut result = [[0i64; 8]; 2];
    for cost_type in 0..2 {
        for level in 0..8 {
            let base_cost = ADV_HONE_UNLOCK[cost_type][level] as f64;
            let reduction = EVENT_UNLOCK_REDUCTION[cost_type][level];
            result[cost_type][level] = (base_cost * reduction).ceil() as i64;
        }
    }
    result
}

pub fn get_event_modified_weapon_costs(express_event: bool) -> [[i64; 25]; 7] {
    if !express_event {
        return DEFAULT_NORMAL_HONE_WEAPON_COST;
    }

    let mut result = [[0i64; 25]; 7];
    for cost_type in 0..7 {
        for level in 0..25 {
            let base_cost = DEFAULT_NORMAL_HONE_WEAPON_COST[cost_type][level] as f64;
            let reduction = EVENT_COST_REDUCTION[cost_type][level];
            result[cost_type][level] = (base_cost * reduction).ceil() as i64;
        }
    }
    result
}

pub fn get_event_modified_armor_costs(express_event: bool) -> [[i64; 25]; 7] {
    if !express_event {
        return DEFAULT_NORMAL_HONE_ARMOR_COST;
    }

    let mut result = [[0i64; 25]; 7];
    for cost_type in 0..7 {
        for level in 0..25 {
            let base_cost = DEFAULT_NORMAL_HONE_ARMOR_COST[cost_type][level] as f64;
            let reduction = EVENT_COST_REDUCTION[cost_type][level];
            result[cost_type][level] = (base_cost * reduction).ceil() as i64;
        }
    }
    result
}

pub fn get_event_modified_weapon_unlock_cost(express_event: bool) -> [[i64; 25]; 2] {
    if !express_event {
        return NORMAL_HONE_WEAPON_UNLOCK;
    }

    let mut result = [[0i64; 25]; 2];
    for cost_type in 0..2 {
        for level in 0..25 {
            let base_cost = NORMAL_HONE_WEAPON_UNLOCK[cost_type][level] as f64;
            let reduction = EVENT_UNLOCK_REDUCTION[cost_type][level];
            result[cost_type][level] = (base_cost * reduction).ceil() as i64;
        }
    }
    result
}

pub fn get_event_modified_artisan(express_event: bool) -> [f64; 25] {
    if !express_event {
        return [1.0; 25];
    }

    EVENT_ARTISAN_MULTIPLIER
}
