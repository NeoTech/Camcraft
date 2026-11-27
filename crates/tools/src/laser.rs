//! Laser cutting head (#4081): the beam's kerf as the tool diameter.
//! Provenance: our own data model, on the waterjet nozzle (#1881); no
//! cutting value is inferred (the laser operation states power and feed).

use crate::tools::{Tool, ToolKind};

/// A laser head with a stated beam kerf. An invalid kerf stays invalid at
/// the tool door ([`refusal`]).
#[must_use]
pub fn laser_tool(kerf: f64) -> Tool {
    Tool { kind: ToolKind::Laser, material_group: String::new(), ..crate::tools::torch_tool(kerf) }
}

/// The head's refusal by name, used at the command and file doors.
/// `None` for every other kind and for a good laser head.
#[must_use]
pub fn refusal(tool: &Tool) -> Option<String> {
    if tool.kind != ToolKind::Laser { return None; }
    if tool.rpm != 0.0 {
        return Some("laser rpm must be 0; the head has no spindle".into());
    }
    if !tool.diameter.is_finite() || tool.diameter <= 0.0 {
        return Some("laser kerf (diameter) must be finite and above zero".into());
    }
    if tool.corner_r != 0.0 {
        return Some("laser corner_r must be 0; the diameter is the kerf".into());
    }
    crate::cutting_rows::refusal_for_kind(&tool.cutting, tool.kind)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cutting_rows::CuttingRow;

    #[test]
    fn a_laser_head_states_its_kerf_and_turns_nothing() {
        let t = laser_tool(0.15);
        assert_eq!((t.kind, t.diameter, t.rpm, t.corner_r), (ToolKind::Laser, 0.15, 0.0, 0.0));
        assert_eq!(refusal(&t), None);
        assert!(t.is_sane(), "{:?}", t.why_not_sane());
        assert_eq!(t.kind.to_string(), "laser head");
    }

    /// A refusal's words and the change that breaks the head.
    type Case = (&'static str, fn(&mut Tool));

    #[test]
    fn a_bad_laser_head_is_refused_by_name() {
        let cases: [Case; 5] = [
            ("rpm must be 0", |t| t.rpm = 1000.0),
            ("kerf (diameter) must be finite", |t| t.diameter = 0.0),
            ("kerf (diameter) must be finite", |t| t.diameter = f64::NAN),
            ("corner_r must be 0", |t| t.corner_r = 0.05),
            ("no cutting data rows", |t| t.cutting.push(CuttingRow::plain(None, 0.0, 1000.0))),
        ];
        for (want, broke) in cases {
            let mut t = laser_tool(0.15);
            broke(&mut t);
            let why = refusal(&t).unwrap_or_default();
            assert!(why.contains(want), "{want}: {why:?}");
            assert!(!t.is_sane(), "{want} reaches the sanity door");
        }
    }
}
