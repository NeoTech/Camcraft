//! Domain suite `machining3d` of camcraft-motion (#3189): 3D roughing: pockets, ladders, shells, descents and the solid law.
//! Provenance: Cargo's directory-suite layout; each module is a former flat suite and keeps its own
//! header and coverage paragraph, which the gate reads through this file.

mod blind_floor;
mod engagement_3d;
mod entry_arc_3d;
mod islands_model_fault;
mod machining3d;
mod offset_shells;
mod one_descent;
mod pocket3d_hsm;
mod pocket3d_mask_edge;
mod pocket3d_pass_start;
mod pocket3d_stock_to_leave;
mod pocket3d_troch;
mod pocket_refusal;
mod posed_subtract;
mod solid_law;
mod stock_seam;
