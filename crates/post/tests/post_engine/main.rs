//! The `post_engine` suite of `camcraft-post` (#3188): the post engine, its machine files, the post binary's command line and the explain output.
//! Provenance: our own test layout (AGENTS.md "New tests join a directory suite", #919); each
//! module keeps its own provenance and Gate map.

#[macro_use]
#[path = "../recorded/mod.rs"]
mod recorded;

mod declared_options;
mod explain;
mod explain_fanuc_family;
mod explain_lathe;
mod explain_remaining;
mod explain_tormach;
mod folder_scan;
mod machine_check;
mod machine_controller;
mod machine_tree;
mod mch_head_search;
mod post_engine;
mod post_output_budget;
mod post_preview;
mod session_files;
mod tool_change_macros;
mod tool_reservations;
mod tool_reservations_cli;
mod vendor_mch_cli;
