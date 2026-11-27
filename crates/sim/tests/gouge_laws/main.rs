//! The `gouge_laws` suite of `camcraft-sim` (#3188): the gouge, crossing, boundary, undercut and avoid laws of the verifier.
//! Provenance: our own test layout (AGENTS.md "New tests join a directory suite", #919); each
//! module keeps its own provenance and Gate map.

mod boundary_arc_planes;
mod boundary_arc_walls;
mod boundary_walls;
mod coarse_collision;
mod corner_relief;
mod crossing_buckets;
mod crossing_retrace;
mod crossing_touch;
mod model_excess;
mod undercut_laws;
mod verify_avoid;
mod verify_avoid_undercut;
mod wall_slack;
