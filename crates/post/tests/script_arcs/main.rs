//! The `script_arcs` suite of `camcraft-post` (#3188): arcs and cutter compensation: planes, centres, coincident and same-angle arcs, radius words.
//! Provenance: our own test layout (AGENTS.md "New tests join a directory suite", #919); each
//! module keeps its own provenance and Gate map.

#[macro_use]
#[path = "../recorded/mod.rs"]
mod recorded;

#[path = "../coincident_arcs/reader.rs"]
#[allow(dead_code)]
mod reader;

mod arc_planes;
mod cnc_arc_lattice;
mod coincident_arcs;
mod comp;
mod comp_plane;
mod radius_arcs;
mod same_angle_arcs;
mod same_angle_stream;
mod vertical_arc_entry;
