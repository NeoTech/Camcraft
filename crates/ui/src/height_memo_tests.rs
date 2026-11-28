//! Gate map: covers ui::height_memo, ui::height_display, ui::operations.
//! Display caches must retain still frames and follow actual geometry and draft inputs.
use camcraft_app::commands::{
    AddTool, AlignPartToSetup, CreateOperation, CreateSetup, DeleteEntity, ImportModel, MoveModel,
    SetStock, SetStockSourceOnly, SetWcsOrigin, TagFixture, UpdateModel,
};
use camcraft_app::geometry::{Mesh, Point3, Triangle, cube};
use camcraft_app::operation::{Height, HeightReference as R, OpParams, SrcRef, Tilt};
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;

use crate::state::{CommandKind, State};

fn tilt() -> Tilt {
    Tilt {
        a_deg: -90.0,
        b_deg: 0.0,
        origin: [0.0, 0.0, 3.0],
        basis_version: 1,
    }
}
fn placed(x: f64, y: f64, z: f64) -> Location {
    Location {
        pos: Point3::new(x, y, z),
        ..Location::default()
    }
}
fn stock(state: &mut State, dy: f64, y: f64) {
    state.history.execute(
        &state.world,
        SetStock::new(
            &state.world,
            StockShape::Box {
                dx: 40.0,
                dy,
                dz: 30.0,
            },
            placed(0.0, y, -20.0),
        )
        .expect("stock"),
    );
}
fn fixture() -> State {
    let mut state = State::new();
    stock(&mut state, 12.0, 5.0);
    state.history.execute(
        &state.world,
        ImportModel::from_mesh(
            "part",
            cube(Point3::new(0.0, -2.0, -3.0), Point3::new(2.0, 4.0, 9.0)),
        ),
    );
    let tool = catalog()
        .into_iter()
        .find(|(name, _)| *name == "EM-0400-2FL")
        .unwrap()
        .1;
    state.history.execute(
        &state.world,
        AddTool::new(&state.world, "tool", tool).unwrap(),
    );
    let mut params = crate::state::OpUi::default().params;
    if let OpParams::Face { tilt: t, .. } = &mut params {
        *t = Some(tilt());
    }
    state.history.execute(
        &state.world,
        CreateOperation::new(&state.world, "face", "tool", SrcRef::Stock, params).unwrap(),
    );
    state
        .pending
        .push_back(CommandKind::Select(Some("face".into())));
    state.drain_actions();
    state.sync_ops_ui();
    state
}
fn echo(state: &State, src: Option<&SrcRef>, h: Height) -> String {
    crate::operations::height_echo(
        &state.world,
        crate::interact::shown_setup(state).as_deref(),
        src,
        state.op_ui.params.tilt(),
        &h,
    )
}
fn model_top(state: &State) -> String {
    echo(state, Some(&SrcRef::Stock), Height::of(R::TopOfModel, 0.0))
}

#[test]
fn unchanged_viewport_and_height_rows_do_not_resolve_geometry_again() {
    let state = fixture();
    let draw = || {
        let labels = crate::interact::overlay_labels(&state, 4.0 / 3.0);
        let lines = crate::interact::height_plane_lines(&state);
        let rows = [R::TopOfStock, R::TopOfModel, R::BottomOfModel]
            .map(|r| echo(&state, Some(&SrcRef::Stock), Height::of(r, 0.0)));
        (labels, lines, rows)
    };
    let initial = draw();
    let builds = super::builds();
    for _ in 0..3 {
        assert_eq!(draw(), initial, "the same frame retains its display values");
        assert_eq!(
            super::builds(),
            builds,
            "a still frame must not project geometry again"
        );
    }
}

#[test]
fn stock_geometry_placement_and_draft_inputs_update_cached_display_values() {
    let mut state = fixture();
    let top = Height::of(R::TopOfStock, 0.0);
    let initial_bounds = crate::height_display::bounds(&state).unwrap();
    assert_eq!(echo(&state, None, top), "(= 11.000 along tool axis)");
    stock(&mut state, 16.0, 5.0);
    assert_eq!(echo(&state, None, top), "(= 13.000 along tool axis)");
    let larger = crate::height_display::bounds(&state).unwrap();
    assert!(larger.min.z < initial_bounds.min.z || larger.max.z > initial_bounds.max.z);
    stock(&mut state, 16.0, 8.0);
    assert_eq!(echo(&state, None, top), "(= 16.000 along tool axis)");
    assert_eq!(
        echo(&state, None, Height::of(R::TopOfStock, 2.0)),
        "(= 18.000 along tool axis)"
    );
    if let OpParams::Face { tilt: Some(t), .. } = &mut state.op_ui.params {
        t.origin[1] = 2.0;
    }
    assert_eq!(echo(&state, None, top), "(= 14.000 along tool axis)");
    if let OpParams::Face { tilt: Some(t), .. } = &mut state.op_ui.params {
        t.a_deg = 0.0;
        t.b_deg = 180.0;
    }
    assert_eq!(echo(&state, None, top), "(= 23.000 along tool axis)");
    let before = super::builds();
    let _ = crate::height_display::bounds(&state);
    assert!(
        super::builds() > before,
        "changed draft frame recomputes local extents"
    );
}

#[test]
fn model_geometry_pose_and_fixture_membership_update_cached_heights() {
    let mut state = fixture();
    assert_eq!(model_top(&state), "(= 4.000 along tool axis)");
    state.history.execute(
        &state.world,
        UpdateModel::new(
            &state.world,
            "part",
            cube(Point3::new(0.0, -2.0, -3.0), Point3::new(2.0, 7.0, 9.0)),
        )
        .unwrap(),
    );
    assert_eq!(model_top(&state), "(= 7.000 along tool axis)");
    state.history.execute(
        &state.world,
        MoveModel::new(&state.world, "part", placed(0.0, 6.0, 0.0)).unwrap(),
    );
    assert_eq!(model_top(&state), "(= 13.000 along tool axis)");
    state.history.execute(
        &state.world,
        ImportModel::from_mesh("other", cube(Point3::ZERO, Point3::new(1.0, 2.0, 1.0))),
    );
    assert_eq!(model_top(&state), "(= 13.000 along tool axis)");
    state.history.execute(
        &state.world,
        TagFixture::new(&state.world, "part", None).unwrap(),
    );
    assert_eq!(
        model_top(&state),
        "(= 2.000 along tool axis)",
        "fixture no longer sets model height"
    );
}

#[test]
fn setup_alignment_wcs_and_source_changes_update_cached_heights() {
    let mut state = fixture();
    state.history.execute(
        &state.world,
        CreateSetup::new(&state.world, Some("Setup1"), None).unwrap(),
    );
    assert_eq!(model_top(&state), "(= 4.000 along tool axis)");
    state.history.execute(
        &state.world,
        AlignPartToSetup::new(&state.world, "Setup1", Some(placed(0.0, 5.0, 0.0))).unwrap(),
    );
    assert_eq!(model_top(&state), "(= 9.000 along tool axis)");
    let wcs = Height::of(R::WorkOffset, 0.0);
    let _ = echo(&state, None, wcs);
    state.history.execute(
        &state.world,
        SetWcsOrigin::new(&state.world, placed(0.0, 8.0, 0.0)).unwrap(),
    );
    assert_eq!(echo(&state, None, wcs), "(= 8.000 along tool axis)");
    state.history.execute(
        &state.world,
        SetWcsOrigin::new(&state.world, placed(0.0, 10.0, 0.0)).unwrap(),
    );
    assert_eq!(echo(&state, None, wcs), "(= 10.000 along tool axis)");
    let selection = Height::of(R::TopOfSelection, 0.0);
    assert_eq!(
        echo(&state, Some(&SrcRef::Stock), selection),
        "(= 7.000 along tool axis)"
    );
    assert_eq!(
        echo(&state, Some(&SrcRef::Entity("part".into())), selection),
        "(= 6.000 along tool axis)"
    );
}

#[test]
fn missing_geometry_and_raw_stock_membership_update_cached_heights() {
    let mut state = fixture();
    let selection = Height::of(R::TopOfSelection, 0.0);
    let src = SrcRef::Entity("later".into());
    assert!(echo(&state, Some(&src), selection).contains("source entity is gone"));
    let tool = catalog()
        .into_iter()
        .find(|(name, _)| *name == "EM-0400-2FL")
        .unwrap()
        .1;
    state.history.execute(
        &state.world,
        AddTool::new(&state.world, "later", tool).unwrap(),
    );
    assert!(echo(&state, Some(&src), selection).contains("source entity carries no geometry"));
    state.history.execute(
        &state.world,
        DeleteEntity::new(&state.world, "later").unwrap(),
    );
    assert!(echo(&state, Some(&src), selection).contains("source entity is gone"));
    state.history.execute(
        &state.world,
        ImportModel::from_mesh("later", cube(Point3::ZERO, Point3::new(1.0, 20.0, 2.0))),
    );
    assert_eq!(
        echo(&state, Some(&src), selection),
        "(= -1.000 along tool axis)"
    );
    assert_eq!(model_top(&state), "(= 20.000 along tool axis)");
    state.history.execute(
        &state.world,
        SetStock::new(
            &state.world,
            StockShape::FromModel {
                source: "later".into(),
            },
            Location::default(),
        )
        .unwrap(),
    );
    assert_eq!(model_top(&state), "(= 20.000 along tool axis)");
    state.history.execute(
        &state.world,
        SetStockSourceOnly::new(&state.world, true).unwrap(),
    );
    assert_eq!(
        model_top(&state),
        "(= 4.000 along tool axis)",
        "raw stock source is not the model"
    );
}

/// A standing turned bar: radius `r`, z from 0 to 16, swept at `n` turns.
fn turned_bar(n: usize, r: f64) -> Vec<Triangle> {
    let v = |j: usize, rho: f64, z: f64| {
        let th = std::f64::consts::TAU * j as f64 / n as f64;
        Point3::new(rho * th.cos(), rho * th.sin(), z)
    };
    let mut t = Vec::new();
    for (seg, next) in [((0.0, 0.0), (r, 0.0)), ((r, 0.0), (r, 16.0)), ((r, 16.0), (0.0, 16.0))] {
        for j in 0..n {
            let k = (j + 1) % n;
            if seg.0 <= 1e-12 {
                t.push(Triangle::new(v(j, seg.0, seg.1), v(k, next.0, next.1), v(j, next.0, next.1)));
            } else if next.0 <= 1e-12 {
                t.push(Triangle::new(v(j, seg.0, seg.1), v(j + 1, seg.0, seg.1), v(k, next.0, next.1)));
            } else {
                t.push(Triangle::new(v(j, seg.0, seg.1), v(j + 1, seg.0, seg.1), v(k, next.0, next.1)));
                t.push(Triangle::new(v(j, seg.0, seg.1), v(k, next.0, next.1), v(j, next.0, next.1)));
            }
        }
    }
    t
}

#[test]
fn the_radial_axis_cache_keeps_still_frames_and_follows_the_bar() {
    // #2774 (frame-cost law): the radius rows and the viewport rings read
    // ONE cache. A still frame refits nothing; moved geometry refits once;
    // a bar replaced by a cube fits no axis, and that answer is cached too.
    let mut state = State::new();
    state.history.execute(
        &state.world,
        SetStock::new(
            &state.world,
            StockShape::Cylinder { diameter: 6.4, height: 16.0 },
            placed(0.0, 0.0, 0.0),
        )
        .expect("bar stock"),
    );
    state.history.execute(
        &state.world,
        ImportModel::from_mesh("bar", Mesh::from_facets(turned_bar(16, 3.0))),
    );
    let setup = crate::interact::shown_setup(&state);
    let src = SrcRef::Entity("bar".into());
    let read = || crate::height_memo::radial_axis(&state.world, setup.as_deref(), &src, None);
    let v = read().expect("a turned bar has an axis");
    assert!((v.part_rho - 3.0).abs() < 1e-9, "the part's rho: {}", v.part_rho);
    assert!((v.stock_rho.expect("round stock") - 3.2).abs() < 1e-9, "the stock's rho");
    let builds = super::radial_builds();
    for _ in 0..3 {
        assert!(read().is_some(), "the same frame keeps its axis");
        assert_eq!(super::radial_builds(), builds, "a still frame refits nothing");
    }
    state.history.execute(
        &state.world,
        MoveModel::new(&state.world, "bar", placed(4.0, 0.0, 0.0)).expect("move"),
    );
    let moved = read().expect("the moved bar is still turned");
    assert!(super::radial_builds() > builds, "moved geometry refits the axis");
    assert!((moved.part_rho - 3.0).abs() < 1e-9, "the rho is about the bar's OWN axis");
    state.history.execute(
        &state.world,
        UpdateModel::new(&state.world, "bar", cube(Point3::ZERO, Point3::new(2.0, 2.0, 2.0))).expect("cube"),
    );
    assert!(read().is_none(), "a cube fits no rotation axis");
    let after = super::radial_builds();
    assert!(after > builds, "the cube refits (and refuses) once");
    assert!(read().is_none(), "the refusal is cached");
    assert_eq!(super::radial_builds(), after, "a still frame keeps the refusal too");
}

#[test]
fn the_radial_axis_cache_follows_a_new_axis_statement() {
    // #2774 review (item 4): the judged axis reads the op's own stated
    // axis and the machine's rotary lines, so those inputs are part of
    // the display key - a new statement refits the axis once and then a
    // still frame keeps it.
    let mut state = State::new();
    state.history.execute(
        &state.world,
        SetStock::new(
            &state.world,
            StockShape::Cylinder { diameter: 6.4, height: 16.0 },
            placed(0.0, 0.0, 0.0),
        )
        .expect("bar stock"),
    );
    state.history.execute(
        &state.world,
        ImportModel::from_mesh("bar", Mesh::from_facets(turned_bar(16, 3.0))),
    );
    let setup = crate::interact::shown_setup(&state);
    let src = SrcRef::Entity("bar".into());
    let unstated = crate::height_memo::radial_axis(&state.world, setup.as_deref(), &src, None).expect("the fit");
    let builds = super::radial_builds();
    let stated = crate::height_memo::radial_axis(&state.world, setup.as_deref(), &src, Some([0.0, 0.0, 1.0]))
        .expect("the stated axis is judged");
    assert!(super::radial_builds() > builds, "a new axis statement is a new key: it refits");
    assert_eq!(stated.dir, unstated.dir, "the statement and the fit answer the same line here");
    let again = crate::height_memo::radial_axis(&state.world, setup.as_deref(), &src, Some([0.0, 0.0, 1.0]));
    assert_eq!(super::radial_builds(), builds + 1, "a still frame keeps the stated axis too");
    assert_eq!(again.map(|v| v.dir), Some(stated.dir), "the stated-axis answer is cached");
}

#[test]
fn local_extents_follow_model_changes_when_stock_is_missing() {
    let mut state = fixture();
    let stock_name = camcraft_app::setup::stock_entity_name(&state.world, None);
    state.history.execute(
        &state.world,
        DeleteEntity::new(&state.world, &stock_name).unwrap(),
    );
    let initial = crate::height_display::bounds(&state).unwrap();
    assert!((initial.max.z - 4.0).abs() < 1e-9);
    let builds = super::builds();
    assert_eq!(crate::height_display::bounds(&state), Some(initial));
    assert_eq!(super::builds(), builds);
    state.history.execute(
        &state.world,
        UpdateModel::new(
            &state.world,
            "part",
            cube(Point3::new(0.0, -2.0, -3.0), Point3::new(2.0, 7.0, 9.0)),
        )
        .unwrap(),
    );
    assert!((crate::height_display::bounds(&state).unwrap().max.z - 7.0).abs() < 1e-9);
    state.history.execute(
        &state.world,
        MoveModel::new(&state.world, "part", placed(0.0, 6.0, 0.0)).unwrap(),
    );
    assert!((crate::height_display::bounds(&state).unwrap().max.z - 13.0).abs() < 1e-9);
}

#[test]
fn display_callers_fill_the_cache_then_reuse_their_geometry_reads() {
    let state = fixture();
    let h = Height::of(R::TopOfModel, 7.25);
    let before_echo = super::builds();
    assert_eq!(echo(&state, None, h), "(= 11.250 along tool axis)");
    assert_eq!(
        super::builds(),
        before_echo + 1,
        "the Heights echo must fill the display cache"
    );
    let _ = echo(&state, None, h);
    assert_eq!(
        super::builds(),
        before_echo + 1,
        "the unchanged Heights echo must hit the cache"
    );

    let before_planes = super::builds();
    let planes = crate::interact::height_planes(&state);
    assert!(!planes.is_empty());
    let after_planes = super::builds();
    assert!(
        after_planes > before_planes,
        "height planes must fill the display cache"
    );
    let _ = crate::interact::height_planes(&state);
    assert_eq!(
        super::builds(),
        after_planes,
        "unchanged height planes must hit the cache"
    );

    let before_bounds = super::builds();
    assert!(crate::height_display::bounds(&state).is_some());
    assert_eq!(
        super::builds(),
        before_bounds + 1,
        "local extents must fill the display cache"
    );
    let _ = crate::height_display::bounds(&state);
    assert_eq!(
        super::builds(),
        before_bounds + 1,
        "unchanged local extents must hit the cache"
    );
}


#[test]
fn display_cache_has_no_production_panic_path() {
    let production = include_str!("height_memo.rs").split("#[cfg(test)]").next().unwrap();
    for forbidden in [".expect(", ".unwrap(", "panic!("] {
        assert!(!production.contains(forbidden), "display cache must not use {forbidden}");
    }
}
