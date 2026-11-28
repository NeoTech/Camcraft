use camcraft_motion::geometry::{Point3, Triangle};
use camcraft_sim::stock_sim::{Dexel, SimOp, Stock, credit_breaks};

use super::*;

#[test]
fn an_open_chain_offsets_on_its_named_air_side_and_keeps_its_break() {
    let run = vec![Point2::new(-20.0, 20.0), Point2::new(20.0, 20.0)];
    let section = |_| Ok(Some(rect(-20.0, -20.0, 20.0, 20.0)));
    let p = params(2.0, Side::Outside);
    let path =
        chamfer_open_runs(std::slice::from_ref(&run), &p, &section).expect("open air offset");
    let cut: Vec<_> = path
        .moves
        .iter()
        .filter(|m| m.kind == MoveKind::Feed && (m.to.z - 17.0).abs() < 1e-12)
        .collect();
    assert_eq!(
        cut.len(),
        1,
        "one constant-depth feed runs the interior offset band: {cut:?}"
    );
    assert!(
        cut.iter().any(|m| (m.to.y - 22.0).abs() < 1e-6),
        "the selected run stands 2 mm into air: {cut:?}"
    );
    assert!(
        path.moves
            .iter()
            .filter(|m| m.kind == MoveKind::Feed)
            .all(|m| (m.to.z - m.from.z).abs() < 1e-12),
        "tip-plane changes are rapids, never feeds that descend into the part: {:?}",
        path.moves
    );
    for end in [Point2::new(-20.0, 20.0), Point2::new(20.0, 20.0)] {
        assert!(
            path.moves.iter().any(|m| {
                m.kind == MoveKind::Plunge
                    && (m.to.x - end.x).abs() < 1e-6
                    && (m.to.y - end.y).abs() < 1e-6
                    && (m.to.z - p.corner_z()).abs() < 1e-12
            }),
            "the cone plunges on the zero-radius end cap at {end:?}: {:?}",
            path.moves
        );
    }
    assert!(
        path.moves
            .iter()
            .filter(|m| m.kind == MoveKind::Feed)
            .all(|m| (m.to.z - p.tip_z()).abs() < 1e-12),
        "all Chamfer feeds stay on the offset band"
    );
    assert_eq!(
        path.breaks[0].edge,
        run.iter()
            .map(|q| camcraft_motion::geometry::Point3::new(q.x, q.y, 20.0))
            .collect::<Vec<_>>(),
        "the offset does not grant credit beyond the picked chain"
    );
    let wrong = refusal(chamfer_open_runs(
        std::slice::from_ref(&run),
        &params(2.0, Side::Inside),
        &section,
    ));
    assert!(
        wrong.contains("wrong class") && wrong.contains("Side Inside"),
        "{wrong}"
    );

    let blocked = Region::new(vec![
        rect(-20.0, -20.0, 20.0, 20.0).loops()[0].clone(),
        rect(-10.0, 21.0, 10.0, 25.0).loops()[0].clone(),
    ])
    .expect("wall and air-side material");
    let blocked = refusal(chamfer_open_runs(std::slice::from_ref(&run), &p, &|_| {
        Ok(Some(blocked.clone()))
    }));
    assert!(
        blocked.contains("air side") && blocked.contains("smaller offset"),
        "{blocked}"
    );

    let hole = Region::new(vec![
        Loop::new(vec![
            Point2::new(-30.0, -30.0),
            Point2::new(30.0, -30.0),
            Point2::new(30.0, 30.0),
            Point2::new(-30.0, 30.0),
        ])
        .expect("outer"),
        Loop::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(0.0, 20.0),
            Point2::new(20.0, 20.0),
            Point2::new(20.0, 0.0),
        ])
        .expect("hole"),
    ])
    .expect("holed section");
    let hole_run = vec![Point2::new(5.0, 20.0), Point2::new(15.0, 20.0)];
    let hole_path = chamfer_open_runs(
        std::slice::from_ref(&hole_run),
        &params(2.0, Side::Inside),
        &|_| Ok(Some(hole.clone())),
    )
    .expect("an inside wall");
    assert!(
        feed_xy(&hole_path)
            .iter()
            .any(|q| (q.y - 18.0).abs() < 1e-6),
        "Inside offsets a clear interior span toward the bounded air of the hole: {:?}",
        feed_xy(&hole_path)
    );
}

#[test]
fn a_small_tip_offset_keeps_the_open_run_end_caps_safe_on_a_fine_grid() {
    let run = vec![Point2::new(-20.0, 20.0), Point2::new(20.0, 20.0)];
    let section = |_| Ok(Some(rect(-20.0, -20.0, 20.0, 20.0)));
    let mut p = params(0.1, Side::Outside);
    p.width = 0.5;
    let path =
        chamfer_open_runs(std::slice::from_ref(&run), &p, &section).expect("offset end caps");
    let facets = [
        Triangle::new(
            Point3::new(-20.0, -20.0, 20.0),
            Point3::new(20.0, -20.0, 20.0),
            Point3::new(20.0, 20.0, 20.0),
        ),
        Triangle::new(
            Point3::new(-20.0, -20.0, 20.0),
            Point3::new(20.0, 20.0, 20.0),
            Point3::new(-20.0, 20.0, 20.0),
        ),
    ];
    let mut stock =
        Stock::Column(Dexel::block(-20.0, -20.0, 20.0, 20.0, 20.0, 0.05).expect("fine stock"));
    stock
        .simulate(&[SimOp {
            moves: &path.moves,
            radius: 5.0,
            ball: false,
            cone: Some(1.0),
            corner: None,
        }])
        .expect("fine cone sweep");
    let mut deviation = stock.deviate(&facets, 0.01).expect("model deviation");
    credit_breaks(
        &mut deviation,
        stock.column().expect("column stock"),
        &facets,
        &path.breaks,
    );
    assert_eq!(
        deviation.gouge_cells,
        0,
        "the translated cone stays on the declared end cap: {:?}",
        (deviation.max_gouge, deviation.max_gouge_at)
    );
}
