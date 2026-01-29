#[derive(Clone)]
pub struct AdaptiveScaler {
    /// The current scaling factor (starts at a reasonable guess, e.g., 1.0 or 100.0)
    pub current_scale: f64,
    /// How many uphill (bad) moves we've seen in this batch
    uphill_count: usize,
    /// How many of those uphill moves we actually accepted
    accepted_count: usize,
    /// How many moves to observe before updating the scale (e.g., 100)
    batch_size: usize,
    /// Damping factor (0.0 to 1.0). Lower = smoother, Higher = more responsive.
    /// Recommended: 0.1 to 0.2
    learning_rate: f64,
}

impl AdaptiveScaler {
    pub fn new(initial_guess: f64, batch_size: usize) -> Self {
        Self {
            current_scale: initial_guess,
            uphill_count: 0,
            accepted_count: 0,
            batch_size,
            learning_rate: 0.1,
        }
    }

    /// Call this AFTER you have decided whether to accept/reject.
    /// Returns: true if the scale was updated this step.
    pub fn update_stats(
        &mut self,
        is_uphill_move: bool,
        is_accepted: bool,
        progress: f64, // 0.0 (start) to 1.0 (end)
    ) {
        // We only care about "bad" (uphill) moves for tuning Boltzmann
        if !is_uphill_move {
            return;
            // return false;
        }

        self.uphill_count += 1;
        if is_accepted {
            self.accepted_count += 1;
        }

        // Only update scale once we have a full batch
        if self.uphill_count >= self.batch_size {
            self.recalibrate(progress);
            // Reset counters
            self.uphill_count = 0;
            self.accepted_count = 0;

            // return true;
        }
        // false
    }

    fn recalibrate(&mut self, target_rate: f64) {
        // 1. Calculate Measured Rate
        // Clamp to avoid log(0) or log(1) math errors
        let measured_rate =
            (self.accepted_count as f64 / self.uphill_count as f64).clamp(0.001, 0.999);

        // 2. Calculate Target Rate
        // Exponential decay: Start at 80% acceptance, end at 0.1%
        // let start_rate: f64 = 0.80;
        // let end_rate: f64 = 0.001;
        // Interpolate target based on progress
        // This is a simple log-linear decay for the target rate
        // let decay = (end_rate / start_rate).ln();
        // let target_rate = start_rate * (decay * progress).exp();

        // 3. The Robust Update Formula
        // S_new = S_old * (ln(measured) / ln(target))^learning_rate
        let ratio: f64 = measured_rate.ln() / target_rate.ln();

        // Safety clamp on the update ratio to prevent explosions
        // (e.g. don't let it grow/shrink by more than 2x in a single update)
        // let safe_ratio = ratio.clamp(0.5, 2.0);

        // Apply damped update
        let adjustment = ratio.powf(self.learning_rate);

        self.current_scale *= adjustment;
    }
}
