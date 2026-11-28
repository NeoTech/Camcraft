//! Open-boundary start picks retain their source mapping refusal (#1473).
//! Gate map: covers app::operation (contour_open, generate).
//! Provenance: this suite's original block fixture and public CAM offset practice.

use super::*;
use camcraft_app::geo2d::Point2;
use camcraft_app::path_picks::PathPick;

#[test]
fn an_open_boundary_refuses_a_pick_at_its_offset_cutter_start() {
    let world = block_world();
    for extension in [0.0, 3.0] {
        for climb in [true, false] {
            let top = top_face(&world, "part", 10.0);
            let north = rim_edge(&world, |q| (q[1] - 20.0).abs() < 1e-9);
            let area = AreaControls {
                extension,
                open: Some(OpenBoundary { picks: pick("part", north), side: ChainSide::Outside }),
                ..Default::default()
            };
            let mut params = contour(10.0, 5.0, Side::Auto);
            if let OpParams::Contour { climb: c, .. } = &mut params {
                *c = climb;
            }
            let mut operation = op(sel("part", top), params, area);
            let (automatic, _) = run(&world, &operation).expect("automatic boundary");
            let cutter = automatic.moves.iter().find(|m| m.kind == MoveKind::Plunge).expect("plunge").to;
            if let OpParams::Contour { picks, .. } = &mut operation.params {
                picks.start = vec![PathPick::xy(Point2::new(cutter.x, cutter.y))];
            }
            let e = run(&world, &operation).expect_err("a cutter endpoint does not prove a source pick");
            assert!(e.contains("start point") && e.contains("source endpoint mapping"), "{e}");
        }
    }
}
