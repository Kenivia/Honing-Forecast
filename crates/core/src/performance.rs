//! Performance / analytics for the purpose of arena, but i dont want to write two versions of every functions so just pass dummy for non-arena usage
//!
//! It tracks what method each P(X<B) evaluation actually uses, and how often the limiting case for SA occurs (answer: not very, like 1e-6)
//! It also produces a history of the best metric for the purpose of visualization
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Clone)]
pub struct Performance {
    pub states_evaluated: i64,
    pub sa_count: i64,
    pub ks_count: i64,
    pub edgeworth_count: i64,
    pub lugganani_count: i64,
    pub newton_iterations: i64,
    pub brute_count: i64,
    pub trivial_count: i64,
    pub householder_count: i64,
    pub bisection_count: i64,
    pub best_history: Vec<(f64, i64, f64)>, // time in seconds, states evaluated, metric
}

impl Default for Performance {
    fn default() -> Self {
        Self::new()
    }
}

impl Performance {
    pub fn new() -> Self {
        Performance {
            states_evaluated: 0,
            sa_count: 0,
            ks_count: 0,
            edgeworth_count: 0,
            lugganani_count: 0,
            newton_iterations: 0,
            brute_count: 0,
            trivial_count: 0,
            householder_count: 0,
            bisection_count: 0,
            best_history: vec![],
        }
    }
    pub fn aggregate_counts(&mut self, other: &Performance) {
        self.states_evaluated += other.states_evaluated;
        self.sa_count += other.sa_count;
        self.ks_count += other.ks_count;
        self.edgeworth_count += other.edgeworth_count;
        self.lugganani_count += other.lugganani_count;
        self.newton_iterations += other.newton_iterations;
        self.brute_count += other.brute_count;
        self.trivial_count += other.trivial_count;
        self.householder_count += other.householder_count;
        self.bisection_count += other.bisection_count;
    }
    pub fn to_write(&self) -> PerformanceToWrite {
        let ks_per_state = self.ks_count as f64 / self.states_evaluated as f64;

        let total = (self.sa_count + self.brute_count + self.trivial_count) as f64;
        let total_per_state = total / self.states_evaluated as f64;
        let sa_ratio: f64 = self.sa_count as f64 / total;
        let brute_ratio: f64 = self.brute_count as f64 / total;
        let trivial_ratio: f64 = self.trivial_count as f64 / total;

        let newton_per_sa = self.newton_iterations as f64 / self.sa_count as f64;
        let edgeworth_ratio = self.edgeworth_count as f64 / self.sa_count as f64;

        PerformanceToWrite {
            states_evaluated: self.states_evaluated,
            total_per_state,
            sa_ratio,
            brute_ratio,
            trivial_ratio,
            ks_per_state,
            newton_per_sa,
            edgeworth_ratio,
            bisection_ratio: (self.bisection_count as f64)
                / (self.bisection_count as f64 + self.householder_count as f64),
            best_history: self.best_history.clone(),
        }
    }
}

fn serialize_nan_as_neg<S>(x: &f64, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if x.is_nan() {
        s.serialize_f64(-0.0)
    } else {
        s.serialize_f64(*x)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PerformanceToWrite {
    pub states_evaluated: i64,

    #[serde(serialize_with = "serialize_nan_as_neg")]
    pub total_per_state: f64,

    #[serde(serialize_with = "serialize_nan_as_neg")]
    pub sa_ratio: f64,
    #[serde(serialize_with = "serialize_nan_as_neg")]
    pub brute_ratio: f64,
    #[serde(serialize_with = "serialize_nan_as_neg")]
    pub trivial_ratio: f64,

    #[serde(serialize_with = "serialize_nan_as_neg")]
    pub ks_per_state: f64,
    #[serde(serialize_with = "serialize_nan_as_neg")]
    pub newton_per_sa: f64,
    #[serde(serialize_with = "serialize_nan_as_neg")]
    pub edgeworth_ratio: f64, // edgeworth / (edge + lugganani)

    #[serde(serialize_with = "serialize_nan_as_neg")]
    pub bisection_ratio: f64, // edgeworth / (edge + lugganani)

    pub best_history: Vec<(f64, i64, f64)>,
}
