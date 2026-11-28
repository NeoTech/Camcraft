//! The `script_rotary` suite of `camcraft-post` (#3188): rotary and trunnion output: inverse kinematics, path safety, poses and inverse-time feed.
//! Provenance: our own test layout (AGENTS.md "New tests join a directory suite", #919); each
//! module keeps its own provenance and Gate map.

#[path = "../ac_trunnion_support/mod.rs"]
mod ac_trunnion_support;

mod ac_trunnion_ik;
mod ac_trunnion_path_safety;
mod g93_feed;
mod mach4_trunnion;
mod opening_rotary_pose;
mod tormach_no_a_word;
mod tormach_rotary_travel;
mod trunnion_schema;
mod upright_after_tilt;
