//! Domain suite `finishing3d` of camcraft-motion (#3189): 3D finishing and Deburr (the slow finish3d suite stays a binary of its own).
//! Provenance: Cargo's directory-suite layout; each module is a former flat suite and keeps its own
//! header and coverage paragraph, which the gate reads through this file.

mod deburr;
mod deburr_filters;
mod deburr_sizing;
mod deburr_unreachable;
mod finish3d_gaplink;
mod finish3d_holes;
mod finish3d_hop_height;
mod finish3d_land;
mod finish3d_overhang_and_chord;
mod finish3d_pass_start;
mod finish3d_waterlines;
mod refusal_ghost_frames;
mod scallop_ceiling;
