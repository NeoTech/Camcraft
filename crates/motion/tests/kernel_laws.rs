//! #4154 (K7a of the path kernel, #4027): the law measures of
//! `common/law_measure/`, their unit tests on hand-made toolpaths, and the
//! readings of the OLD planners of the phase-1 kinds on the disc of #3934
//! and the dog bone of #3741, pinned as they are today. A reading that
//! breaks a law is not a failure here: the table marks it, and the kernel
//! must improve it.
//!
//! Gate map: covers the test helpers `common/law_measure` and the old
//! planners of camcraft_motion::machining2d and machining3d (read only),
//! and the order and link stages of the kernel (camcraft_pathkernel::order,
//! camcraft_pathkernel::link; #4177) on the same fixtures, and the converter
//! from a kernel plan to a toolpath (camcraft_motion::kernel_emit; #4269).
//! Provenance: our own design record (#4024, "the laws as checks" and the
//! switch-over proof); no vendor material consulted.

mod common;
#[path = "kernel_laws/measure_units.rs"]
mod measure_units;
#[path = "kernel_laws/fixture.rs"]
mod fixture;
#[path = "kernel_laws/laws_pockets.rs"]
mod laws_pockets;
#[path = "kernel_laws/laws_profiles.rs"]
mod laws_profiles;
#[path = "kernel_laws/laws_holes.rs"]
mod laws_holes;
#[path = "kernel_laws/laws_slots.rs"]
mod laws_slots;
#[path = "kernel_laws/k4_order_link.rs"]
mod k4_order_link;
#[path = "kernel_laws/k6a_plan_emit.rs"]
mod k6a_plan_emit;
