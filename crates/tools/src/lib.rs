//! camcraft-tools - the tool-catalog seam of the M4.1 crate split
//! (R2, docs/M4.1-CRATE-SPLIT.md). Just the plain types: the R0 audit
//! proved tools.rs imports nothing; the DXF profile converter uses geometry.
//! The entity wrappers live in the app's schema, the picker
//! and sheet rendering in commands/tool and tool_list.
//!
//! Provenance: end mill / ball nose / bull nose shapes and the
//! coatings/carbide taxonomy are general tool-catalog practice from
//! public machine-tool handbooks and ISO 13399 concepts; nothing from
//! any vendor catalog file. The catalog is user data, not code -
//! this crate only says what a tool record IS.

pub mod catalog_asset;
#[cfg(test)]
mod catalog_asset_tests;
pub mod cutting;
// #3068: the router chip-load curve by cutter diameter.
pub mod cutting_router;
// #541: the sinker EDM electrode as a tool record.
pub mod electrode;
// #1152: the body electrode's outline law.
pub mod electrode_outline;
// #368: a modelled tool body as the stack laws read it.
pub mod holder_profile;
// #793: the built-in BT/CAT taper holder library, drawn as spun profiles.
pub mod holder_library;
// #3367: the tool meridian (lines and arcs) and its convex atoms.
pub mod meridian;
// #93: a drawn revolved (spun) profile for milling tools and holders.
pub mod spun;
// #93 slice 2: a milling tool's own drawn cutter, checked against its numbers.
pub mod cutter_profile;
// #93 slice 3: the drawn cutter edited by its parameters (refit, neck).
pub mod cutter_fit;
// #852: a necked tool's drawn shank, as the stack laws judge it.
pub mod neck;
// #369: a lathe-held body as the revolved-stock law reads it.
pub mod lathe_holder;
// #396: the insert plate a lathe body's judge leaves out.
pub mod lathe_insert_zone;
// #630: the lathe tool orientation code and its control point.
pub mod orientation;
// #894: the thread mills - form constants, sanity and the pitch door.
pub mod thread_mill;
pub mod thread_hand;
// #4031: a rotating cutter's hand (right M3, left M4), separate from the tap's thread hand.
pub mod cutter_hand;
// #1444: the undercut cutters - T-slot, keyseat, dovetail, lollipop.
pub mod undercut;
pub mod catalog_fields;
// #890: a tool's cutting data per workpiece material.
pub mod cutting_rows;
pub mod waterjet;
// #1521: a torch's cut-data recipe per material and plate thickness.
pub mod plasma_cut;
pub mod wire;
pub mod tools;
// #3368: a tapered mill's tip form (sharp, flat, ball) and its geometry.
pub mod taper;
// #3369: a form mill's outline, pilot bearing and router bit presets.
pub mod form_mill;
// #1186: the tool sanity law, said by name.
pub mod tool_sanity;
// #2566: machine controller tool tables (Tormach PathPilot CSV, Hurco WinMax TCM).
pub mod tool_table;
mod tool_table_hurco;
mod tool_table_hurco_export;
mod tool_table_pathpilot;
mod tool_table_pathpilot_desc;
mod tool_table_xml;
// #891: turning surface speed, feed per rev and the max-rpm clamp.
pub mod turning_data;

// #3642: reusable tool components (ISO 13399 kinds), the ISO 1832
// insert code, the ISO 5608 holder code, connection codes and the fit
// checks between them.
pub mod component;
pub mod component_fit;
#[cfg(test)]
mod component_fit_tests;
pub mod connection_code;
pub mod insert_codes;
#[cfg(test)]
mod insert_designation_tests;
pub mod insert_iso1832;
pub mod insert_seat;
// #3643: spindle tool assemblies - component frames, forward kinematics,
// gauge length, cutting parameters and the judged holder body.
pub mod component_geometry;
pub mod frame;
pub mod tool_assembly;
#[cfg(test)]
mod tool_assembly_tests;
// #3643: lathe tool assemblies - the nose centre as the tip, the
// orientation, the turning values and the clearance bands.
pub mod lathe_assembly;
#[cfg(test)]
mod lathe_assembly_tests;
// #3646: multi-seat indexable face mills and end mills resolved to the
// milling tool record, never smaller than the real cutter.
pub mod milling_cutter;
#[cfg(test)]
mod milling_cutter_tests;

// #743: DXF half-profile parsing and projection into validated holder profiles.
pub mod tool_profile_dxf;
