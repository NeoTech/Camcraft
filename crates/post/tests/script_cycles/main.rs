//! The `script_cycles` suite of `camcraft-post` (#3188): canned drilling, boring, reaming and tapping cycles and their dwells on each dialect.
//! Provenance: our own test layout (AGENTS.md "New tests join a directory suite", #919); each
//! module keeps its own provenance and Gate map.

#[macro_use]
#[path = "../recorded/mod.rs"]
mod recorded;

mod bore_cycle;
mod cycle_dwell;
mod cycle_hop;
mod dwell_cycles_modal_p;
mod expanded_bore_refusal;
mod fanuc_rigid_tap;
mod fine_bore_cycle;
mod g73_g85_dwell_linuxcnc;
mod g83_dwell;
mod g83_dwell_linuxcnc;
mod haas_dwell_cycles;
mod heidenhain_cycles;
mod ream_cycle;
mod ream_lathe;
mod sinumerik_dwell;
mod sinumerik_g83_dwell;
mod sinumerik_g89_dwell;
mod tap_feed_mill_posts;
