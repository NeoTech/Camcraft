//! Domain suite `adaptive2d` of camcraft-motion (#3189): 2D Adaptive clearing, avoid routes, trochoidal and slot paths, shape masks.
//! Provenance: Cargo's directory-suite layout; each module is a former flat suite and keeps its own
//! header and coverage paragraph, which the gate reads through this file.

#[path = "../common/mod.rs"]
mod common;
// The adaptive trochoid helpers, shared by the three adaptive_troch_* modules.
#[path = "../common/adaptive_troch.rs"]
mod fixtures;

mod adaptive_band;
mod adaptive_entry_pieces;
mod adaptive_hsm;
mod adaptive_orbit_seam;
mod adaptive_troch_island;
mod adaptive_troch_neck;
mod adaptive_troch_parallel;
mod avoid_adaptive;
mod avoid_contour;
mod avoid_engrave;
mod avoid_pocket;
mod optimal_load;
mod radial_step;
mod shape_masks;
mod slot_round_ends;
mod slot_start_exit;
mod trochoid_slot;
