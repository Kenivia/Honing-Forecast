#[derive(Clone, Debug)]
pub struct StateBundle {
    pub state: Vec<Vec<(bool, usize)>>,
    pub names: Vec<String>,

    // the above entries are tied to each upgrade, so arr[upgrade_index] correspond to the appropriate info for a particular upgrade
    // probably should make another struct for this at some point
    pub special_state: Vec<usize>, // arbitrary length
    pub metric: f64,
    pub state_index: Vec<Vec<Vec<i64>>>, // i pre-added this for caching but havnt implemented anything
}
pub fn encode_one_positions(v1: &[(bool, usize)]) -> String {
    v1.iter()
        .map(|(uppercase, num)| {
            let letter: char = if *num == 0 {
                'x' as char
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
        for (index, i) in self.state.iter().enumerate() {
            strings.push(self.names[index].clone() + ": " + &encode_one_positions(i));
        }
        strings.join("\n")
    }
}
