//! The `sim_timeline` suite of `camcraft-sim` (#3188): the simulation timeline, playback, operation steps, plasma programs and the headless binary.
//! Provenance: our own test layout (AGENTS.md "New tests join a directory suite", #919); each
//! module keeps its own provenance and Gate map.

mod headless_bin;
mod plasma_laws;
mod plasma_program;
mod play_keys;
mod play_precalc;
mod sim_op_steps;
