//! Domain suite `contour2d` of camcraft-motion (#3189): 2D Contour: sides, starts, tabs, whole-model outlines, arcs and compensation.
//! Provenance: Cargo's directory-suite layout; each module is a former flat suite and keeps its own
//! header and coverage paragraph, which the gate reads through this file.

#[path = "../common/mod.rs"]
mod common;

mod arc_turn_kinds;
mod arcs;
mod climb_direction;
mod contour_auto_side;
mod contour_inside_ramp;
mod contour_outside_floor;
mod contour_start;
mod contour_start_comp;
mod contour_tab_bridge;
mod contour_tab_entry;
mod contour_tab_height;
mod contour_tab_lead;
mod contour_tab_necks;
mod contour_tabs;
mod contour_whole;
mod contour_whole_new_outline;
mod contour_whole_no_floor;
mod control_comp;
mod lead_room;
mod wear_comp;
