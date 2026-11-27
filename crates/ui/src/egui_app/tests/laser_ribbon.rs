//! #4081: the Laser Profile appears on a laser cutter's ribbon - its own
//! Laser section, with its own glyph - and on no other workspace's,
//! where its cell is dark and the tip names the laser cutter to mount.
//! The op page shows the leads tab and the power on Feeds.
//! Provenance: our own tests over our own ribbon.

use super::super::op_icons::op_icon;
use super::super::op_tabs::{has_section, SEC_FEEDS, SEC_LEADS};
use super::super::panes::{self, ribbon_plan, Cmd};
use crate::state::{CommandKind, State};
use camcraft_app::machine::Kind;
use camcraft_app::operation::{OpType, Section};
use egui::Vec2;
use egui_kittest::{Harness, kittest::Queryable};

fn sections(kind: Kind) -> Vec<Section> {
    ribbon_plan(kind, &[]).into_iter().filter_map(|g| g.section).collect()
}

#[test]
fn the_laser_profile_shows_on_a_laser_cutters_ribbon_only() {
    let laser = OpType::LaserProfile.ordinal();
    assert_eq!(sections(Kind::Laser), vec![Section::Laser], "a laser cutter shows its own section alone");
    assert_eq!(Section::Laser.kinds_for(Kind::Laser), vec![laser]);
    assert!(ribbon_plan(Kind::Laser, &[]).iter().any(|g| g.cells.contains(&Cmd::Create(laser))), "the cell stands on a laser cutter");
    for kind in [Kind::Mill, Kind::Lathe, Kind::MillTurn, Kind::Plasma, Kind::WireEdm, Kind::SinkerEdm, Kind::Waterjet] {
        assert!(!sections(kind).contains(&Section::Laser), "{kind:?} shows no laser section");
        assert!(ribbon_plan(kind, &[laser]).iter().all(|g| !g.cells.contains(&Cmd::Create(laser))), "{kind:?}: a laser pin stands nowhere");
    }

    let mut mill = State::new();
    mill.pending.push_back(CommandKind::SetStock {
        shape: camcraft_app::stock::StockShape::Box { dx: 40.0, dy: 40.0, dz: 3.0 },
        pos: [0.0, 0.0, 0.0],
        yaw_deg: 0.0,
    });
    mill.pending.push_back(CommandKind::LoadBuiltinMachine("VMC 3-axis".into()));
    mill.pending.push_back(CommandKind::AddTool { name: "Beam".into(), tool: camcraft_app::laser_tools::laser_tool(0.2) });
    mill.drain_actions();
    assert!(!panes::enabled(Cmd::Create(laser), &mill), "a mill carries no beam");
    let tip = panes::tip_of(Cmd::Create(laser), &mill);
    assert!(tip.contains("laser") && !tip.contains("mill-turn"), "{tip}");
}

#[test]
fn the_laser_profile_has_its_own_glyph_and_its_tabs() {
    let mine = op_icon(OpType::LaserProfile);
    assert_ne!(mine, op_icon(OpType::WaterjetProfile), "not the waterjet's glyph");
    assert_ne!(mine, op_icon(OpType::PlasmaProfile), "not the plasma's glyph");
    assert!(has_section(OpType::LaserProfile, SEC_LEADS), "the leads live on Lead in / Lead out");
}

fn laser_fields(tab: usize, power: f64) -> Harness<'static, State> {
    let mut state = State::new();
    state.op_ui.params = crate::operations::quick_params(OpType::LaserProfile.ordinal());
    let camcraft_app::operation::OpParams::LaserProfile(params) = &mut state.op_ui.params else {
        panic!("the laser draft must have laser parameters");
    };
    params.power = power;
    Harness::builder().with_size(Vec2::new(900.0, 900.0))
        .build_ui_state(move |ui, state: &mut State| super::super::params_page::params_fields(ui, state, tab), state)
}

#[test]
fn the_feeds_tab_states_the_power_and_names_a_bad_one() {
    let mut h = laser_fields(SEC_FEEDS, 50.0);
    h.run();
    assert!(h.query_by_label("Power").is_some(), "the power row");
    assert!(h.query_by_label("Pierce time").is_some(), "the pierce row");
    assert!(h.query_by_label("Abrasive delay").is_none(), "a laser has no abrasive");
    assert!(h.query_by_label_contains("at most 100 %").is_none());
    let mut h = laser_fields(SEC_FEEDS, 120.0);
    h.run();
    assert!(h.query_by_label_contains("at most 100 %").is_some(), "a bad power is named on the page");
}
