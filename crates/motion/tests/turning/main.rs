//! Domain suite `turning` of camcraft-motion (#3189): the turning kernels: face, profile, groove, part, bore and thread.
//! Provenance: Cargo's directory-suite layout; each module is a former flat suite and keeps its own
//! header and coverage paragraph, which the gate reads through this file.

mod turning_blade;
mod turning_bore;
mod turning_coarse;
mod turning_contour;
mod turning_contour_stock;
mod turning_face;
mod turning_form;
mod turning_g76_clearance;
mod turning_offset_spacing;
mod turning_offset_window;
mod turning_part_kept;
mod turning_part_kerf;
mod turning_prof;
mod turning_stock;
mod turning_thread;
mod turning_thread_cycle;
mod turning_thread_dips;
mod turning_thread_end_face;
mod turning_thread_helical;
mod turning_thread_infeed;
