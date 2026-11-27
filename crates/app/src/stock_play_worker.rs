//! The traced playback's worker, re-exported (#2268): the worker is
//! camcraft-sim's (`camcraft_sim::play_worker`); it starts from a
//! `stock_play::PlaySeed` and times itself with this crate's clock seam
//! (`clock::monotonic`). Provenance: our own playback design; nothing
//! from vendor binaries.

pub use camcraft_sim::play_worker::{PlayFrame, PlayWorker, JUMP_MOVES, JUMP_PUBLISH, MIN_PUBLISH};
