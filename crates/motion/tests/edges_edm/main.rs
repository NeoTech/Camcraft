//! Domain suite `edges_edm` of camcraft-motion (#3189): Chamfer, V-carve and the EDM kernels (wire fans, sinker outlines).
//! Provenance: Cargo's directory-suite layout; each module is a former flat suite and keeps its own
//! header and coverage paragraph, which the gate reads through this file.

mod chamfer_circular;
mod chamfer_minimum_radius;
mod chamfer_open_refusals;
mod sinker_outline;
mod v_carve;
mod wire_fans;
