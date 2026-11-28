//! The `script_dialects` suite of `camcraft-post` (#3188): per-controller words: preambles, decimal points, traverses, spindle and coolant words, comments, units and lathe limits.
//! Provenance: our own test layout (AGENTS.md "New tests join a directory suite", #919); each
//! module keeps its own provenance and Gate map.

#[macro_use]
#[path = "../recorded/mod.rs"]
mod recorded;

#[path = "../../../app/tests/common/legacy_program.rs"]
#[allow(dead_code)]
mod legacy_program;

mod comment_escape;
mod control_feed_decimal_point;
mod decimal_point_fanuc_family;
mod doosan_dnm_post;
mod end_units;
mod fanuc_aicc;
mod fanuc_manual_guide_blank;
mod fanuc_mgi_blank;
mod fanuc_post;
mod first_traverse_clearance;
mod first_traverse_clearance_fanuc_family;
mod grbl_dialect;
mod grbl_family_same_tool;
mod haas_feed_decimal_point;
mod heidenhain_arc_centre;
mod heidenhain_block_form;
mod lathe_live_feed_cap;
mod lathe_post_refusals;
mod linuxcnc_post;
mod mach3_post;
mod mach4_feed_decimals;
mod mach4_post;
mod mazak_spindle_words;
mod sinumerik_810m_post;
mod tormach_coolant;
mod tormach_post;
mod tormach_same_tool;
mod tormach_stop_tool;
mod tormach_traverse_clearance;
