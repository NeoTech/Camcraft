//! #3237 UI pins for the 2D Chamfer Lead tab and its undoable Apply.

use camcraft_app::machining2d::Lead;

use super::*;

fn opened_chamfer(params: OpParams) -> State {
    let mut state = State::new();
    state.pending.push_back(CommandKind::SetStock {
        shape: StockShape::Box { dx: 60.0, dy: 60.0, dz: 20.0 },
        pos: [0.0, 0.0, 0.0],
        yaw_deg: 0.0,
    });
    state.drain_actions();
    let tool = catalog()
        .into_iter()
        .find(|(name, _)| *name == "CM-0600-90")
        .map(|(_, tool)| tool)
        .expect("catalog chamfer tool");
    state.history.execute(
        &state.world,
        AddTool::new(&state.world, "CM-0600-90", tool).expect("tool"),
    );
    state.history.execute(
        &state.world,
        ImportModel::from_mesh(
            "part",
            cube(Point3::new(-20.0, -20.0, 0.0), Point3::new(20.0, 20.0, 10.0)),
        ),
    );
    state.history.execute(
        &state.world,
        CreateOperation::new(
            &state.world,
            "Op",
            "CM-0600-90",
            SrcRef::Stock,
            params,
        )
        .expect("operation"),
    );
    state.pending.push_back(CommandKind::Select(Some("Op".into())));
    state.drain_actions();
    state.ui_target = UiTarget::Operation("Op".into());
    state.sync_ops_ui();
    state
}

#[test]
fn opening_and_applying_the_chamfer_lead_tab_keeps_a_legacy_operation_legacy() {
    let kind = camcraft_app::operation::KindInfo::ALL
        .iter()
        .position(|row| row.label == "2D Chamfer")
        .expect("Chamfer kind");
    let mut params = crate::operations::retype(&pocket(Entry3D::Plunge), kind);
    if let OpParams::Chamfer2D { lead, lead_out, .. } = &mut params {
        *lead = None;
        *lead_out = None;
    }
    let mut state = opened_chamfer(params);
    state.op_tab = super::super::op_tabs::SEC_LEADS;
    let mut harness = page(state);
    harness.run();
    assert!(matches!(
        &harness.state().op_ui.params,
        OpParams::Chamfer2D {
            lead: None,
            ..
        }
    ));
    assert!(matches!(
        live(harness.state()),
        OpParams::Chamfer2D { lead: None, .. }
    ));
    harness.get_by_label("Apply").click();
    harness.run();
    harness.state_mut().drain_actions();
    assert!(matches!(
        live(harness.state()),
        OpParams::Chamfer2D {
            lead: None,
            ..
        }
    ));
}

#[test]
fn chamfer_lead_shapes_apply_and_undo_without_entry_controls() {
    let kind = camcraft_app::operation::KindInfo::ALL
        .iter()
        .position(|row| row.label == "2D Chamfer")
        .expect("Chamfer kind");
    let params = crate::operations::retype(&pocket(Entry3D::Plunge), kind);
    let mut state = opened_chamfer(params);
    state.op_tab = super::super::op_tabs::SEC_LEADS;
    let mut harness = page(state);
    harness.run();
    assert!(harness.query_by_label("Lead in").is_some());
    assert!(harness.query_by_label("Lead out").is_some());
    for entry_control in [
        "Entry",
        "Level to level",
        "Between passes",
        "Vertical lead-in height",
        "Compensation",
        "Tabs",
    ] {
        assert!(
            harness.query_by_label(entry_control).is_none(),
            "{entry_control} does not belong to Chamfer"
        );
    }

    if let OpParams::Chamfer2D { lead, lead_out, .. } =
        &mut harness.state_mut().op_ui.params
    {
        *lead = Some(Lead::Arc { radius: 3.25 });
        *lead_out = Some(Lead::Tangential { radius: 2.5 });
    } else {
        panic!("Chamfer draft");
    }
    harness.run();
    assert!(harness.query_by_label("Lead arc radius").is_some());
    assert!(harness.query_by_label("Lead-out arc radius").is_some());
    assert!(
        matches!(
            live(harness.state()),
            OpParams::Chamfer2D {
                lead: Some(Lead::Perpendicular { length: 3.0 }),
                lead_out: None,
                ..
            }
        ),
        "the draft waits for Apply"
    );

    harness.get_by_label("Apply").click();
    harness.run();
    harness.state_mut().drain_actions();
    assert!(matches!(
        live(harness.state()),
        OpParams::Chamfer2D {
            lead: Some(Lead::Arc { radius }),
            lead_out: Some(Lead::Tangential { radius: out }),
            ..
        } if (radius - 3.25).abs() < 1e-9 && (out - 2.5).abs() < 1e-9
    ));
    let state = harness.state_mut();
    state.history.undo(&state.world);
    assert!(matches!(
        live(state),
        OpParams::Chamfer2D {
            lead: Some(Lead::Perpendicular { length: 3.0 }),
            lead_out: None,
            ..
        }
    ));
}

#[test]
fn typing_a_chamfer_lead_size_stays_in_the_draft() {
    let kind = camcraft_app::operation::KindInfo::ALL
        .iter()
        .position(|row| row.label == "2D Chamfer")
        .expect("Chamfer kind");
    let mut params = crate::operations::retype(&pocket(Entry3D::Plunge), kind);
    if let OpParams::Chamfer2D { lead, lead_out, .. } = &mut params {
        *lead = Some(Lead::Arc { radius: 3.0 });
        *lead_out = None;
    }
    let mut state = opened_chamfer(params);
    state.op_tab = super::super::op_tabs::SEC_LEADS;
    let mut harness = page(state);
    harness.run();

    let label = harness.get_by_label("Lead arc radius").rect();
    let spin = harness
        .query_all_by(|node| node.role() == Role::SpinButton)
        .find(|node| (node.rect().center().y - label.center().y).abs() < 2.0)
        .map(|node| node.rect())
        .expect("lead radius on its label row");
    harness
        .query_all_by(|node| node.role() == Role::SpinButton)
        .find(|node| node.rect() == spin)
        .expect("lead radius")
        .click();
    harness.run();
    harness
        .query_all_by(|node| node.role() == Role::SpinButton)
        .find(|node| (node.rect().top() - spin.top()).abs() < 1.0)
        .expect("editing lead radius")
        .type_text("7.25");
    harness.run();
    harness.run();

    assert!(matches!(
        &harness.state().op_ui.params,
        OpParams::Chamfer2D {
            lead: Some(Lead::Arc { radius }),
            ..
        } if (*radius - 7.25).abs() < 1e-9
    ));
    assert!(matches!(
        live(harness.state()),
        OpParams::Chamfer2D {
            lead: Some(Lead::Arc { radius: 3.0 }),
            ..
        }
    ), "the number stays staged until Apply");
}
