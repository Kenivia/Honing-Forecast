// pub const INIT_TEMP: f64 = 0.5;
// pub const RESOLUTION_CUTOFF_TEMP: f64 = 33.3;
pub const MAX_BEST_SIZE: usize = 10;
// pub const ITERS_PER_TEMP: i64 = 7;
pub const MAX_ITERS: i64 = 24000;
// pub const BATCH_SIZE: i64 = 500;
// pub const ALPHA: f64 = 0.99;
pub const NON_IMPACT_WEIGHT: f64 = 3.0;
// pub const SPECIAL_START_CHANCE: f64 = 1.0;
// pub const MIN_SPECIAL_CHANCE: f64 = 0.05;
pub const CONCURRENT_COUNT: i64 = 1;

// pub const JUICE_TEMP_FACTOR: f64 = 3.0;
// pub const SPECIAL_TEMP_FACTOR: f64 = 1.0;

// pub const BLOCK_SIZE_MULTIPLIER: f64 = 1.0;
// pub const SPECIAL_CROSSOVER_MULTIPLIER: f64 = 0.5;
// pub const JUICE_CROSSOVER_MULTIPLIER: f64 = 1.0;

// ANNEALING SCHEDULE (more specifically the expected acceptance rate schedule)
pub const MAGIC_NUMBER: f64 = 0.44;
pub const WARM_UP_PHASE_END: f64 = 0.15;

// pub const WARM_UP_LEARNING_FACTOR: f64 = 200.0;
// pub const USUAL_LEARNING_FACTOR: f64 = 500.0;

pub const COOLING_PHASE_START: f64 = 0.65;

// pub const DEFAULT_RESOLUTION: usize = 10;
pub const SPECIAL_AFFINITY_DECAY: f64 = 0.999;
pub const SPECIAL_AFFINITY_GROWTH: f64 = 1.02;
