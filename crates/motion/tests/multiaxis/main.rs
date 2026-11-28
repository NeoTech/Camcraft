//! Domain suite `multiaxis` of camcraft-motion (#3189): multi-axis and rotary paths (the slow pivot5d and flow suites stay binaries of their own).
//! Provenance: Cargo's directory-suite layout; each module is a former flat suite and keeps its own
//! header and coverage paragraph, which the gate reads through this file.

mod multiaxis;
mod multiaxis_avoid;
mod multiaxis_rough;
mod rotary;
mod rotary_adaptive;
mod rotary_contour_x;
mod rotary_coverage;
mod rotary_dflat;
mod rotary_links;
mod rotary_offset;
mod rotary_zigzag_rings;
mod ruling;
mod swarf_pose_continuity;
mod swarf_reentry;
