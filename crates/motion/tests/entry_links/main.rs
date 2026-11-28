//! Domain suite `entry_links` of camcraft-motion (#3189): entries, ramps, level links, cut levels and pass starts.
//! Provenance: Cargo's directory-suite layout; each module is a former flat suite and keeps its own
//! header and coverage paragraph, which the gate reads through this file.

#[path = "../common/mod.rs"]
mod common;

mod entry_approach;
mod entry_arc;
mod entry_ramp_angle;
mod equal_steps;
mod level_links;
mod pass_start_engagement;
mod pass_start_levels;
mod refusal_ghost;
mod top_z_level;
mod top_z_stock_over;
mod vertical_arc_entry;
mod xy_arc_entry;
