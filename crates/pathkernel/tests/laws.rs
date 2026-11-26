//! #4055: the invariants of the path kernel's stage types, through the
//! public API only. A value of a public type is valid by construction: each
//! test builds the invalid cases and expects a named refusal. The K7 law
//! suite (#4027) joins this directory suite.
//!
//! Gate map: covers pathkernel::{frame, verdict, region, region_build, region_check, selection, guide, order, load, link, entry, entry::air, entry::column, entry::exit, entry::hole, entry::material, entry::proof, entry::room, entry::stage, plan, recipe}.
//! Provenance: our own design record docs/decisions/4024-path-kernel-design.md.

#[path = "laws/common.rs"]
mod common;
#[path = "laws/construction.rs"]
mod construction;
#[path = "laws/entry.rs"]
mod entry;
#[path = "laws/entry_stage.rs"]
mod entry_stage;
#[path = "laws/exit.rs"]
mod exit;
#[path = "laws/guide.rs"]
mod guide;
#[path = "laws/load.rs"]
mod load;
#[path = "laws/order.rs"]
mod order;
#[path = "laws/plan.rs"]
mod plan;
#[path = "laws/region.rs"]
mod region;
