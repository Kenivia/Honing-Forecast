use crate::upgrade::Upgrade;

pub const GRACE_FIRST_N: [usize; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 999];
pub const NON_GRACE_FIRST_N: [usize; 15] = [1, 2, 3, 4, 5, 8, 10, 12, 15, 20, 30, 40, 50, 60, 999];
// there's an overlap so (999,0) gets included in grace_first_n)

pub const MAX_NUM_STATE: usize = GRACE_FIRST_N.len() + NON_GRACE_FIRST_N.len();

/// theres only 31 possible options so this outpus 0 to 30
pub fn tuple_to_index(a: usize, b: usize) -> usize {
    if a < 999 {
        return a;
    } else if b == 0 {
        return 15;
    }
    {
        return 16 + NON_GRACE_FIRST_N.iter().position(|x| *x == b).unwrap();
    }
}
impl Upgrade {
    pub fn adv_state_to_index(&self) -> usize {
        self.state.payload[0].1 * MAX_NUM_STATE + self.state.payload[1].1
    }
}
// let mut grid = [[Vec3::zeroed(); NY]; NX];
// file.read_exact(bytemuck::bytes_of_mut(&mut grid))?;
