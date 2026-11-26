//! The evaluator: the metric (`average`, the expected gold over the free tap outcomes) and the
//! plain probabilities shown (`success_prob`). Both read sums off the exact or coarse lattice
//! (`lattice`) where they can, the plain probability the saddlepoint formula (`saddlepoint`) or
//! brute force (`brute`) where not.
pub mod average;
pub mod brute;
pub mod common;
pub mod lattice;
pub mod memo;
pub mod saddlepoint;
pub mod shape;
pub mod special;
pub mod success_prob;
