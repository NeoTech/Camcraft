//! Open DXF centerline endpoint picks preserve the source run (#1473).
//! Gate map: covers app::operation (contour_sketch, contour_sketch_pick, wcs_yaw).
//! Provenance: this suite's original ASCII DXF fixture and undoable app commands.

use super::*;
use camcraft_app::commands::SetWcsOrigin;
use camcraft_app::geo2d::Point2;
use camcraft_app::geometry::{Point3, quat_from_euler};
use camcraft_app::path_picks::PathPick;

#[test]
fn an_open_sketch_endpoint_pick_preserves_climb_and_chain_reverse() {
    for climb in [true, false] {
        for reverse in [false, true] {
            let (_dir, world, mut history) = sheet_world(&format!("start-{climb}-{reverse}"));
            let ids = plate_ids(&world);
            let source = SrcRef::Selection(vec![GeoSel {
                entity: "plate".into(), feature: FeatureSel::SketchChain(ids.open_line),
            }]);
            let mut params = stock_contour_params();
            if let OpParams::Contour { climb: c, side, stepdown, chain_overrides, .. } = &mut params {
                *c = climb;
                *side = Side::Auto;
                *stepdown = 1.0;
                *chain_overrides = vec![ChainOverride {
                    entity: "plate".into(), chain: ids.open_line, side: None, reverse,
                }];
            }
            run(&world, &mut history, CreateOperation::new(&world, "Automatic", TOOL, source.clone(), params.clone()), "create automatic");
            run(&world, &mut history, GenerateOperation::new(&world, "Automatic"), "generate automatic");
            let entity = model::find_by_name(&world, "Automatic").expect("automatic operation");
            let automatic = model::toolpath_of(&world, &entity).expect("automatic path").path;
            let start = automatic.moves.iter().find(|m| m.kind == MoveKind::Plunge).expect("plunge").to;
            let source_start = Point2::new(start.x, start.y);
            assert!(source_start == Point2::new(20.0, 10.0) || source_start == Point2::new(100.0, 10.0));
            assert_eq!(source_start.x, if climb ^ reverse { 20.0 } else { 100.0 });
            if let OpParams::Contour { picks, .. } = &mut params {
                picks.start = vec![PathPick::xy(source_start)];
            }
            run(&world, &mut history, CreateOperation::new(&world, "Picked", TOOL, source, params), "create picked");
            run(&world, &mut history, GenerateOperation::new(&world, "Picked"), "generate picked");
            let entity = model::find_by_name(&world, "Picked").expect("picked operation");
            let picked = model::toolpath_of(&world, &entity).expect("picked path").path;
            assert_eq!(picked.moves, automatic.moves, "pick must preserve every move and level");
        }
    }
}

#[test]
fn an_open_sketch_start_pick_requires_one_selected_chain() {
    let (_dir, world, mut history) = sheet_world("start-multiple-chains");
    let ids = plate_ids(&world);
    let source = SrcRef::Selection(vec![
        GeoSel { entity: "plate".into(), feature: FeatureSel::SketchChain(ids.open_line) },
        GeoSel { entity: "plate".into(), feature: FeatureSel::SketchChain(ids.outer) },
    ]);
    let mut params = stock_contour_params();
    if let OpParams::Contour { picks, .. } = &mut params {
        picks.start = vec![PathPick::xy(Point2::new(20.0, 10.0))];
    }
    run(&world, &mut history, CreateOperation::new(&world, "Multiple", TOOL, source, params), "create");
    let e = GenerateOperation::new(&world, "Multiple").err().expect("one pick must not apply to several chains");
    let msg = e.to_string();
    assert!(msg.contains("start point") && msg.contains("exactly one sketch chain"), "{msg}");
}

#[test]
fn an_open_sketch_start_pick_under_wcs_yaw_is_refused_by_name() {
    let (_dir, world, mut history) = sheet_world("start-yaw");
    let ids = plate_ids(&world);
    let location = Location {
        pos: Point3::new(20.0, 11.0, 0.0),
        rot: quat_from_euler(0.0, 0.0, 90f64.to_radians()),
    };
    run(&world, &mut history, SetWcsOrigin::new(&world, location), "set WCS yaw");
    let source = SrcRef::Selection(vec![GeoSel {
        entity: "plate".into(), feature: FeatureSel::SketchChain(ids.open_line),
    }]);
    let mut params = stock_contour_params();
    run(&world, &mut history, CreateOperation::new(&world, "Automatic", TOOL, source.clone(), params.clone()), "create automatic");
    run(&world, &mut history, GenerateOperation::new(&world, "Automatic"), "generate without a pick under yaw");
    if let OpParams::Contour { picks, .. } = &mut params {
        // This is the rotated start's XY, not the original source endpoint (20, 10).
        picks.start = vec![PathPick::xy(Point2::new(19.0, 11.0))];
    }
    run(&world, &mut history, CreateOperation::new(&world, "Picked", TOOL, source, params), "create picked");
    let e = GenerateOperation::new(&world, "Picked").err().expect("pick frame is not proved under yaw");
    let msg = e.to_string();
    assert!(msg.contains("start point") && msg.contains("WCS yaw") && msg.contains("pick frame mapping"), "{msg}");
}
