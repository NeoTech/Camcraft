//! #3195: hole patterns from numbers - a point list, a line, a grid and a
//! full or partial bolt-hole circle - give hole centres by plain
//! trigonometry (x = cx + r cos a, y = cy + r sin a), and degenerate or
//! non-finite input is refused by the field's name.

use camcraft_geometry::geo2d::Point2;
use camcraft_geometry::hole_pattern::{CircleSpacing, HolePattern, MAX_HOLES};

fn near(got: &[Point2], want: &[(f64, f64)]) {
    assert_eq!(got.len(), want.len(), "{got:?}");
    for (g, w) in got.iter().zip(want) {
        assert!((g.x - w.0).abs() < 1e-9 && (g.y - w.1).abs() < 1e-9, "{got:?} vs {want:?}");
    }
}

fn refusal(p: &HolePattern) -> String {
    match p.points() {
        Ok(pts) => panic!("{p:?} should be refused, got {pts:?}"),
        Err(e) => e,
    }
}

#[test]
fn a_point_list_keeps_its_points_in_order() {
    let p = HolePattern::Points(vec![Point2::new(1.0, 2.0), Point2::new(-3.0, 4.5)]);
    near(&p.points().expect("points"), &[(1.0, 2.0), (-3.0, 4.5)]);
}

#[test]
fn a_line_steps_its_pitch_along_its_angle() {
    let p = HolePattern::Line { start: Point2::new(10.0, 5.0), pitch: 20.0, count: 3, angle_deg: 90.0 };
    near(&p.points().expect("line"), &[(10.0, 5.0), (10.0, 25.0), (10.0, 45.0)]);
}

#[test]
fn a_grid_runs_x_first_then_y_and_turns_about_its_start() {
    let p = HolePattern::Grid {
        start: Point2::new(0.0, 0.0),
        pitch_x: 10.0,
        pitch_y: 5.0,
        count_x: 2,
        count_y: 2,
        angle_deg: 0.0,
    };
    near(&p.points().expect("grid"), &[(0.0, 0.0), (10.0, 0.0), (0.0, 5.0), (10.0, 5.0)]);
    let turned = HolePattern::Grid {
        start: Point2::new(1.0, 1.0),
        pitch_x: 10.0,
        pitch_y: 5.0,
        count_x: 2,
        count_y: 2,
        angle_deg: 90.0,
    };
    near(&turned.points().expect("grid"), &[(1.0, 1.0), (1.0, 11.0), (-4.0, 1.0), (-4.0, 11.0)]);
}

#[test]
fn a_full_bolt_circle_spaces_its_holes_evenly_from_the_start_angle() {
    let p = HolePattern::BoltCircle {
        center: Point2::new(50.0, 40.0),
        diameter: 60.0,
        count: 4,
        start_deg: 0.0,
        spacing: CircleSpacing::Full,
    };
    near(&p.points().expect("circle"), &[(80.0, 40.0), (50.0, 70.0), (20.0, 40.0), (50.0, 10.0)]);
    // Smid's six-hole circle at 30 degrees: r = 50, the first hole at
    // (r cos 30, r sin 30).
    let six = HolePattern::BoltCircle {
        center: Point2::new(0.0, 0.0),
        diameter: 100.0,
        count: 6,
        start_deg: 30.0,
        spacing: CircleSpacing::Full,
    };
    let pts = six.points().expect("six");
    near(&pts[..2], &[(50.0 * 3f64.sqrt() / 2.0, 25.0), (0.0, 50.0)]);
}

#[test]
fn a_partial_bolt_circle_takes_a_pitch_or_a_span() {
    let by_pitch = HolePattern::BoltCircle {
        center: Point2::new(0.0, 0.0),
        diameter: 20.0,
        count: 3,
        start_deg: 0.0,
        spacing: CircleSpacing::Pitch(45.0),
    };
    let h = 10.0 / 2f64.sqrt();
    near(&by_pitch.points().expect("pitch"), &[(10.0, 0.0), (h, h), (0.0, 10.0)]);
    let by_span = HolePattern::BoltCircle {
        center: Point2::new(0.0, 0.0),
        diameter: 20.0,
        count: 3,
        start_deg: 0.0,
        spacing: CircleSpacing::Span(-180.0),
    };
    near(&by_span.points().expect("span"), &[(10.0, 0.0), (0.0, -10.0), (-10.0, 0.0)]);
}

#[test]
fn degenerate_and_non_finite_patterns_are_refused_by_name() {
    let o = Point2::new(0.0, 0.0);
    let cases = [
        (HolePattern::Points(vec![]), "no points"),
        (HolePattern::Points(vec![Point2::new(f64::NAN, 0.0)]), "point 1"),
        (HolePattern::Points(vec![o, Point2::new(0.0, 0.0)]), "points 1 and 2"),
        (HolePattern::Line { start: o, pitch: 0.0, count: 2, angle_deg: 0.0 }, "pitch"),
        (HolePattern::Line { start: o, pitch: 5.0, count: 0, angle_deg: 0.0 }, "count"),
        (HolePattern::Line { start: o, pitch: 5.0, count: 2, angle_deg: f64::INFINITY }, "angle"),
        (HolePattern::Line { start: Point2::new(f64::NAN, 0.0), pitch: 5.0, count: 2, angle_deg: 0.0 }, "start"),
        (
            HolePattern::Grid { start: o, pitch_x: 5.0, pitch_y: -1.0, count_x: 2, count_y: 2, angle_deg: 0.0 },
            "pitch_y",
        ),
        (
            HolePattern::Grid { start: o, pitch_x: 5.0, pitch_y: 5.0, count_x: 200, count_y: 200, angle_deg: 0.0 },
            "more than",
        ),
        (
            HolePattern::BoltCircle { center: o, diameter: 0.0, count: 4, start_deg: 0.0, spacing: CircleSpacing::Full },
            "diameter",
        ),
        (
            HolePattern::BoltCircle { center: o, diameter: 10.0, count: 4, start_deg: 0.0, spacing: CircleSpacing::Pitch(120.0) },
            "full turn",
        ),
        (
            HolePattern::BoltCircle { center: o, diameter: 10.0, count: 3, start_deg: 0.0, spacing: CircleSpacing::Span(360.0) },
            "span",
        ),
        (
            HolePattern::BoltCircle { center: o, diameter: 10.0, count: 1, start_deg: 0.0, spacing: CircleSpacing::Span(90.0) },
            "count",
        ),
    ];
    for (p, word) in cases {
        let why = refusal(&p);
        assert!(why.contains(word), "{p:?}: '{why}' should name '{word}'");
    }
    const { assert!(MAX_HOLES < 200 * 200, "the grid case above must break the cap") };
}
