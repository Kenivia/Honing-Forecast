use crate::state_bundle::StateBundle;

pub fn apply_prices(used: f64, thresh_price_pairs: &[(f64, f64)]) -> f64 {
    let mut out = 0.0;

    for (index, &(thresh, price)) in thresh_price_pairs.iter().enumerate() {
        if index + 1 < thresh_price_pairs.len() {
            let next_thresh = thresh_price_pairs[index + 1].0;
            if used <= thresh {
                out += price * (next_thresh - thresh);
            } else if used < next_thresh {
                out += price * (next_thresh - used);
            }
        } else {
            if used >= thresh {
                out += price * (thresh - used);
            }
        }
    }

    out
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
