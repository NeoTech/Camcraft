//! Domain suite `machining2d` of camcraft-motion (#3189): 2D Pocket, Face, rest machining, corner blends and the headless binary.
//! Provenance: Cargo's directory-suite layout; each module is a former flat suite and keeps its own
//! header and coverage paragraph, which the gate reads through this file.

mod corner_arc_radius;
mod corner_blend;
mod corner_blend_arc;
mod corner_blend_arc_contour;
mod corner_relief;
mod face_blend;
mod face_concave_holes;
mod face_holes;
mod face_keep;
mod face_start_exit;
mod golden_undercut;
mod headless_bin;
mod machining2d;
mod overhang_2d;
mod pocket_wall_finish;
mod pocket_wall_finish_levels;
mod rest2d;
mod rest_corner_relief;
mod rest_previous;
mod rest_relief;
mod rest_zigzag;
mod tiny_step;
