use crate::{parser::PreparationOutput, state_bundle::StateBundle};

pub fn apply_prices(used: f64, prep_output: &PreparationOutput, index: usize) -> f64 {
    let bound_thresh = prep_output.bound_budgets[index];
    let trade_thresh = prep_output.trade_budgets[index] + bound_thresh;
    if used > trade_thresh {
        prep_output.market_price[index] * -(used - trade_thresh)
    } else if bound_thresh < used && used < trade_thresh {
        prep_output.tradable_price[index] * -(used - bound_thresh)
    } else {
        prep_output.leftover_price[index] * -(used - bound_thresh)
    }
}

pub fn encode_one_positions(v1: &[(bool, usize)]) -> String {
    v1.iter()
        .map(|(uppercase, num)| {
            let letter: char = if *num == 0 {
                'x'
            } else {
                (b'a' + (*num as u8 - 1)) as char
            };

            if *uppercase {
                letter.to_ascii_uppercase()
            } else {
                letter
            }
        })
        .collect()
}

impl StateBundle {
    pub fn encode_all(&self) -> String {
        let mut strings = Vec::new();
        strings.push(format!("{:?}", self.special_state));
        for (index, upgrade) in self.upgrade_arr.iter().enumerate() {
            strings.push(
                self.upgrade_arr[index].name_string.clone()
                    + ": "
                    + &encode_one_positions(&upgrade.state),
            );
        }
        strings.join("\n")
    }
}
