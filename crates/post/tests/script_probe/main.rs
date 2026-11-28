//! The `script_probe` suite of `camcraft-post` (#3188): probing cycles: dialects, guards, reports, rotated work offsets and the spindle.
//! Provenance: our own test layout (AGENTS.md "New tests join a directory suite", #919); each
//! module keeps its own provenance and Gate map.

#[macro_use]
#[path = "../recorded/mod.rs"]
mod recorded;

mod mazak_sequence_labels;
mod probe_dialects;
mod probe_guard;
mod probe_report;
mod probe_rotated_wcs;
mod probe_spindle;
mod probe_spindle_stop;
mod probe_tormach;
