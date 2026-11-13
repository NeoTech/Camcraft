//! #548 (part of #290): a sinker EDM machine and a burn-to-depth op set
//! up through the ui queue - the same commands the Machine window's
//! Apply and the op page's Apply push - and undone step by step. The
//! machine edit is validated at command creation (a blank orbit word is
//! refused by the loader's law, the machine standing as it was). The
//! pages themselves are pinned by the lib's `sinker_machine_tests`.
//! Everything here is a SYNTHETIC fixture; not exercised on a real EDM
//! control.
//! Provenance: our own tests, specced from the issue's "Done means".
//!
//! Gate map (#548): covers ui::state, ui::egui_app::machine_edm_editor, app::machine, app::operation (sinker_op), app::tools.

use camcraft_app::machine::{self, parse_mch};
use camcraft_app::model;
use camcraft_app::operation::{Height, OpParams, SinkerBurnOp, SinkerDatum, SinkerPass, SrcRef};
use camcraft_app::stock::StockShape;
use camcraft_app::tools::{Tool, ToolKind};
use camcraft_ui::state::{CommandKind, State};

const SINKER: &str = r#"{
  "general": { "model": "SYNTH SINKER", "vendor": "Camcraft", "kind": "sinker_edm" },
  "kinematics": { "default": {
    "units": { "length": "mm", "angle": "degrees" },
    "parts": [ { "id": "base", "name": "table", "type": "table", "parts": [
      { "id": "x", "name": "X", "type": "linear", "parts": [ { "id": "y", "name": "Y", "type": "linear" } ] },
      { "id": "z", "name": "Z", "type": "linear", "parts": [ { "id": "head", "name": "ram", "type": "head" } ] }
    ] } ] } },
  "edm": { "conditions": [ { "id": "S1", "name": "synthetic", "gap": 0.05, "undersize": 0.1, "role": "rough" } ] }
}"#;

fn electrode() -> Tool {
    Tool {
        kind: ToolKind::Electrode,
        diameter: 10.0,
        corner_r: 0.0,
        tip_angle: 0.0,
        pitch: 0.0,
        nose_radius: 0.0,
        width: 6.0,
        flute_len: 30.0,
        shank_d: 10.0,
        shank_len: 15.0,
        holder_d: 0.0,
        holder_len: 0.0,
        material_group: "graphite".into(),
        rpm: 0.0,
        feed: 50.0,
        tool_material: None,
        flutes: None,
        center_cutting: true,
        orientation: None,
        turning: None,
        pitch_min: 0.0,
        catalog: Default::default(),
        cutting: Vec::new(),
    }
}

fn burn() -> OpParams {
    OpParams::SinkerBurn(SinkerBurnOp {
        clearance_z: Height::fixed(25.0),
        top_z: Height::fixed(20.0),
        floor_z: Height::fixed(14.0),
        datum: SinkerDatum::WorkTop,
        depth: 6.0,
        x: 0.0,
        y: 0.0,
        feed: 40.0,
        condition: "S1".into(),
        pass: SinkerPass::Single,
        cavity_x: 0.0,
        cavity_y: 0.0,
        orbit: None,
        layer: None,
        wear_lost: None,
        wear_over: None,
    })
}

fn push(state: &mut State, c: CommandKind) {
    state.pending.push_back(c);
    state.drain_actions();
}

#[test]
fn a_sinker_machine_and_a_burn_to_depth_are_set_up_and_undone() {
    let mut state = State::new();
    push(&mut state, CommandKind::SetStock { shape: StockShape::Box { dx: 60.0, dy: 60.0, dz: 20.0 }, pos: [0.0; 3], yaw_deg: 0.0 });
    let m = parse_mch(SINKER).expect("synthetic sinker");
    push(&mut state, CommandKind::EditMachineApply { path: "synth.mch".into(), machine: Box::new(m.clone()) });
    assert_eq!(machine::of(&state.world).1, Some(m.clone()), "{:?}", state.status);
    push(&mut state, CommandKind::AddTool { name: "E-10x6".into(), tool: electrode() });
    push(&mut state, CommandKind::CreateOperation { name: "Burn1".into(), tool: "E-10x6".into(), src: SrcRef::Stock, params: burn() });
    assert_eq!(state.status, "operation Burn1 created", "a burn-to-depth enters the sinker");
    assert!(model::find_by_name(&state.world, "Burn1").is_some());

    // A machine edit the loader would refuse (an orbit with no word) is
    // refused by name; the mounted machine stands.
    let mut bad = m.clone();
    bad.edm.as_mut().expect("edm").orbits.push(machine::OrbitWord { kind: "circular".into(), word: String::new() });
    push(&mut state, CommandKind::EditMachineApply { path: "synth.mch".into(), machine: Box::new(bad) });
    assert!(state.status.contains("machine edit refused"), "{:?}", state.status);
    assert_eq!(machine::of(&state.world).1, Some(m.clone()));

    // Undo walks it back: the op, the tool, the machine.
    push(&mut state, CommandKind::Undo);
    assert!(model::find_by_name(&state.world, "Burn1").is_none(), "undo removes the burn");
    push(&mut state, CommandKind::Undo);
    assert!(!state.tool_names().iter().any(|t| t == "E-10x6"));
    push(&mut state, CommandKind::Undo);
    assert!(machine::of(&state.world).1.is_none_or(|m| m.edm.is_none()), "undo unmounts the sinker");
}
