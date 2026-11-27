//! The planner switch value: the old planners or the path kernel (record
//! "The one switch"). This module only parses the value; the one place
//! that reads the process setting is in `camcraft-app` (the generate
//! dispatch), so that no other code reads it.
//! Provenance: our own design record (`RECOMMENDED.md`, `## #4024`);
//! the same switch form as `CAMCRAFT_CARRY_ENGINE` (#1168) and
//! `CAMCRAFT_GPU_COMPUTE`.

/// The process setting that selects the planner engine.
pub const PLANNER_ENV: &str = "CAMCRAFT_PLANNER";

/// Which planner generates the phase-1 kinds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PlannerEngine {
    /// The planners of `camcraft-motion` as they are (the default until the
    /// switch-over PR).
    #[default]
    Old,
    /// The path kernel. A kind that it does not plan yet is refused by name.
    New,
}

impl PlannerEngine {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Old => "old",
            Self::New => "new",
        }
    }
}

/// The engine for the setting `value` (`None` = not set), and a diagnostic
/// when the value is not known. A bad value gives the default: never a
/// panic, and never a silent change of the planner.
#[must_use]
pub fn engine_from_switch(value: Option<&str>) -> (PlannerEngine, Option<String>) {
    let Some(raw) = value else { return (PlannerEngine::default(), None) };
    match raw.trim().to_ascii_lowercase().as_str() {
        "" | "old" => (PlannerEngine::Old, None),
        "new" | "kernel" => (PlannerEngine::New, None),
        _ => (
            PlannerEngine::default(),
            Some(format!(
                "{PLANNER_ENV}={raw:?} is not known (use \"old\" or \"new\"): the {} planner runs",
                PlannerEngine::default().name()
            )),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_is_the_old_planner() {
        assert_eq!(PlannerEngine::default(), PlannerEngine::Old);
        assert_eq!(engine_from_switch(None), (PlannerEngine::Old, None));
        assert_eq!(engine_from_switch(Some("")), (PlannerEngine::Old, None));
        assert_eq!(engine_from_switch(Some("OLD")), (PlannerEngine::Old, None));
    }

    #[test]
    fn new_selects_the_kernel() {
        assert_eq!(engine_from_switch(Some("new")), (PlannerEngine::New, None));
        assert_eq!(engine_from_switch(Some(" New ")), (PlannerEngine::New, None));
        assert_eq!(engine_from_switch(Some("kernel")), (PlannerEngine::New, None));
    }

    #[test]
    fn a_bad_value_gives_the_default_and_a_diagnostic() {
        let (e, diag) = engine_from_switch(Some("fast"));
        assert_eq!(e, PlannerEngine::Old);
        let diag = diag.unwrap();
        assert!(diag.contains("CAMCRAFT_PLANNER") && diag.contains("\"fast\"") && diag.contains("old planner"), "{diag}");
    }
}
