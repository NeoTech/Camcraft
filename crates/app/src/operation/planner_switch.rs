//! The one planner switch (#4055): the generate dispatch asks it which
//! planner runs the 16 phase-1 kinds. It is the only code that reads the
//! process setting `CAMCRAFT_PLANNER`. The default is the old planners.
//! A phase-1 kind that the path kernel does not plan yet is refused by
//! name under "new": never a silent fallback to the old planner (law 4).
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`
//! "The one switch".

use std::sync::OnceLock;

use camcraft_motion::pathkernel::{PLANNER_ENV, PlannerEngine, engine_from_switch};

use super::{GenError, OpType};

/// The phase-1 kinds of the record: 2D milling and 3D Pocket. Only these
/// kinds read the switch; every other kind keeps its own planner.
pub const PHASE1_KINDS: [OpType; 16] = [
    OpType::Face,
    OpType::Contour,
    OpType::Pocket,
    OpType::Adaptive2D,
    OpType::Offset2D,
    OpType::Engrave,
    OpType::VCarve,
    OpType::Chamfer2D,
    OpType::Circular,
    OpType::Slot,
    OpType::TrochoidalSlot,
    OpType::Trace,
    OpType::SlotUndercut,
    OpType::Thread,
    OpType::Bore,
    OpType::Pocket3D,
];

/// The planner engine of this process, read from `CAMCRAFT_PLANNER` one
/// time. A bad value gives the old planner and one diagnostic in the log.
pub fn planner_engine() -> PlannerEngine {
    static ENGINE: OnceLock<PlannerEngine> = OnceLock::new();
    *ENGINE.get_or_init(|| {
        let raw = std::env::var_os(PLANNER_ENV).map(|v| v.to_string_lossy().into_owned());
        let (engine, diag) = engine_from_switch(raw.as_deref());
        if let Some(d) = diag {
            tracing::warn!("{d}");
        }
        if engine != PlannerEngine::default() {
            tracing::info!("{PLANNER_ENV}: the {} planner generates the phase-1 kinds", engine.name());
        }
        engine
    })
}

/// Lets the dispatch run the old planner of `kind` under `engine`, or
/// refuses it by name. Under "new", each phase-1 kind is refused until its
/// kernel planner is added to the dispatch.
pub fn check_engine(kind: OpType, engine: PlannerEngine) -> Result<(), GenError> {
    if engine == PlannerEngine::Old || !PHASE1_KINDS.contains(&kind) {
        return Ok(());
    }
    Err(GenError::Invalid(format!(
        "the new path planner does not plan {kind} yet ({PLANNER_ENV}=new) - unset {PLANNER_ENV} or set it to \"old\""
    )))
}
