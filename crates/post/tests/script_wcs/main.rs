//! The `script_wcs` suite of `camcraft-post` (#3188): work offsets, flipped and rotated work coordinate systems, inverted axes and setup changes.
//! Provenance: our own test layout (AGENTS.md "New tests join a directory suite", #919); each
//! module keeps its own provenance and Gate map.

#[macro_use]
#[path = "../recorded/mod.rs"]
mod recorded;
#[path = "../inverted_support/mod.rs"]
mod inverted_support;
#[path = "../macro_b/mod.rs"]
mod macro_b;
#[path = "../mirror_trace/mod.rs"]
mod mirror_trace;

mod axis_direction_cli;
mod inverted_equivalence;
mod inverted_post;
mod inverted_probe;
mod operator_side;
mod setup_change;
mod wcs_flip;
mod wcs_flip_dialects;
mod wcs_flip_rotated;
mod wcs_rotate;
mod work_offset_index;
