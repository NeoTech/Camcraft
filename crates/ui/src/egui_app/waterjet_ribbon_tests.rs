//! #1883: the Waterjet Profile appears on a waterjet's ribbon - its own
//! Waterjet section, with its own glyph - and on no other workspace's,
//! where its cell is dark and the tip names the waterjet to mount. The
//! op page shows the leads tab. The ribbon and the command door read
//! the one registry word (`KindInfo::admits`).
//! Provenance: our own tests over our own ribbon.

use super::op_icons::op_icon;
use super::op_tabs::{has_section, SEC_LEADS};
use super::panes::{self, ribbon_plan, Cmd};
use crate::state::{CommandKind, State};
use camcraft_app::machine::Kind;
use camcraft_app::operation::{OpType, Section};

fn sections(kind: Kind) -> Vec<Section> {
    ribbon_plan(kind, &[]).into_iter().filter_map(|g| g.section).collect()
}

#[test]
fn the_waterjet_profile_shows_on_a_waterjets_ribbon_only() {
    let jet = OpType::WaterjetProfile.ordinal();
    assert_eq!(sections(Kind::Waterjet), vec![Section::Waterjet], "a waterjet shows its own section alone");
    assert_eq!(Section::Waterjet.kinds_for(Kind::Waterjet), vec![jet]);
    assert!(ribbon_plan(Kind::Waterjet, &[]).iter().any(|g| g.cells.contains(&Cmd::Create(jet))), "the cell stands on a waterjet");
    for kind in [Kind::Mill, Kind::Lathe, Kind::MillTurn, Kind::Plasma, Kind::WireEdm, Kind::SinkerEdm] {
        assert!(!sections(kind).contains(&Section::Waterjet), "{kind:?} shows no waterjet section");
        assert!(ribbon_plan(kind, &[jet]).iter().all(|g| !g.cells.contains(&Cmd::Create(jet))), "{kind:?}: a waterjet pin stands nowhere");
    }

    let mut mill = State::new();
    mill.pending.push_back(CommandKind::SetStock {
        shape: camcraft_app::stock::StockShape::Box { dx: 40.0, dy: 40.0, dz: 6.0 },
        pos: [0.0, 0.0, 0.0],
        yaw_deg: 0.0,
    });
    mill.pending.push_back(CommandKind::LoadBuiltinMachine("VMC 3-axis".into()));
    mill.pending.push_back(CommandKind::AddTool { name: "Nozzle".into(), tool: camcraft_app::waterjet_tools::waterjet_tool(0.8) });
    mill.drain_actions();
    assert!(!panes::enabled(Cmd::Create(jet), &mill), "a mill carries no jet");
    let tip = panes::tip_of(Cmd::Create(jet), &mill);
    assert!(tip.contains("waterjet") && !tip.contains("mill-turn"), "{tip}");
}

#[test]
fn the_waterjet_profile_has_its_own_glyph_and_a_leads_tab() {
    let mine = op_icon(OpType::WaterjetProfile);
    assert_ne!(mine, op_icon(OpType::PlasmaProfile), "not the plasma's glyph");
    assert!(has_section(OpType::WaterjetProfile, SEC_LEADS), "the leads live on Lead in / Lead out");
}
