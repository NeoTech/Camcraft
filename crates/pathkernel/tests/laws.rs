//! #4055: the invariants of the path kernel's stage types, through the
//! public API only. A value of a public type is valid by construction: each
//! test builds the invalid cases and expects a named refusal. The K7 law
//! suite (#4027) joins this directory suite: one generic check for each law
//! (`l1_guide` .. `l7_move`, `link_rule`) over the stage traits, the kit
//! that runs a kind's planners (`kit`, `grid`), the stand-in planners that
//! prove each check (`standins`), and the registry of the 16 phase-1 kinds
//! (`registry`).
//!
//! Gate map: covers pathkernel::{frame, verdict, region, region_build, region_check, selection, guide, order, load, link, entry, plan, recipe}.
//! Provenance: our own design record docs/decisions/4024-path-kernel-design.md.

#[path = "laws/common.rs"]
mod common;
#[path = "laws/construction.rs"]
mod construction;
#[path = "laws/entry.rs"]
mod entry;
#[path = "laws/grid.rs"]
mod grid;
#[path = "laws/guide.rs"]
mod guide;
#[path = "laws/kit.rs"]
mod kit;
#[path = "laws/l1_guide.rs"]
mod l1_guide;
#[path = "laws/l2_sides.rs"]
mod l2_sides;
#[path = "laws/l3_order.rs"]
mod l3_order;
#[path = "laws/l4_fallback.rs"]
mod l4_fallback;
#[path = "laws/l5_entry.rs"]
mod l5_entry;
#[path = "laws/l6_load.rs"]
mod l6_load;
#[path = "laws/l7_move.rs"]
mod l7_move;
#[path = "laws/link_rule.rs"]
mod link_rule;
#[path = "laws/load.rs"]
mod load;
#[path = "laws/order.rs"]
mod order;
#[path = "laws/plan.rs"]
mod plan;
#[path = "laws/region.rs"]
mod region;
#[path = "laws/registry.rs"]
mod registry;
#[path = "laws/standins.rs"]
mod standins;
