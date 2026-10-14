// The setting is a display gamma that grows in a straight line with it, so two settings differ
// by a power law and nothing else. Only this ratio of the line's offset to its slope can be told
// from captures; 62.6 fits those in scripts/brightness/inputs storage to 0.15 levels on average.

pub const GAMMA_RATIO: f64 = 62.6;
pub const TARGET_BRIGHTNESS: f64 = 50.0;
