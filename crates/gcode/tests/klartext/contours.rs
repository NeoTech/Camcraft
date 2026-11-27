//! Transition elements: RND rounding arcs and CHF chamfers between straight lines give the published corner geometry (cut-back points, arc centre, tangency), or a refusal that names the block.

use camcraft_cl::cl::{Move, MoveKind};
use camcraft_gcode::GcodeError;

use super::{assert_to, moves, near, ok, pgm, refusal};

/// Assert an arc move about (cx, cy) in the given direction.
fn assert_arc(m: &Move, cx: f64, cy: f64, ccw: bool) {
    let a = m.arc.expect("an arc move");
    assert!(
        near(a.center.x, cx) && near(a.center.y, cy) && a.ccw == ccw,
        "arc {a:?}, expected centre ({cx}, {cy}) ccw={ccw}"
    );
}

#[test]
fn klartext_rnd_rounds_a_right_angle_with_its_own_feed() {
    let p = ok(&pgm(&[
        "L X+0 Y+0 Z-1 R0 F100",
        "L X+20 Y+0",
        "RND R5 F50",
        "L X+20 Y+20",
    ]));
    let m = moves(&p);
    assert_eq!(m.len(), 4, "line, cut-back line, rounding arc, line");
    // The corner (20, 0) is not machined: the line stops R tan(45 deg) = 5 before it.
    assert_to(m[1], 15.0, 0.0, -1.0);
    assert_to(m[2], 20.0, 5.0, -1.0);
    assert_arc(m[2], 15.0, 5.0, true);
    assert_to(m[3], 20.0, 20.0, -1.0);
    // F50 acts only on the rounding arc; the modal F100 stays.
    let feeds: Vec<_> = m.iter().map(|m| m.feed).collect();
    assert_eq!(
        feeds,
        vec![Some(100.0), Some(100.0), Some(50.0), Some(100.0)]
    );
    // Each move maps to the block that programmed it.
    let lines: Vec<_> = (0..4).map(|i| p.line_for_move(0, i)).collect();
    assert_eq!(lines, vec![Some(2), Some(3), Some(4), Some(5)]);
}

#[test]
fn klartext_chf_and_rnd_chain_on_right_turns() {
    let p = ok(&pgm(&[
        "L X+0 Y+0 Z-1 R0 F200",
        "L X+30 Y+0",
        "CHF 3",
        "L X+30 Y-10",
        "RND R2",
        "L X+0 Y-10",
    ]));
    let m = moves(&p);
    assert_eq!(m.len(), 6);
    // CHF 3 cuts 3 mm from each line; the chamfer is a feed line at the modal F.
    assert_to(m[1], 27.0, 0.0, -1.0);
    assert_to(m[2], 30.0, -3.0, -1.0);
    assert!(m[2].arc.is_none() && m[2].kind == MoveKind::Feed);
    assert_eq!(m[2].feed, Some(200.0));
    // The middle line starts at the chamfer end and stops 2 mm before (30, -10).
    assert_to(m[3], 30.0, -8.0, -1.0);
    assert_to(m[4], 28.0, -10.0, -1.0);
    assert_arc(m[4], 28.0, -8.0, false);
    assert_to(m[5], 0.0, -10.0, -1.0);
}

#[test]
fn klartext_rnd_on_an_obtuse_corner_is_tangent_to_both_lines() {
    let p = ok(&pgm(&[
        "L X+0 Y+0 Z-1 R0 F100",
        "L X+10 Y+0",
        "RND R4",
        "L X+20 Y+10",
    ]));
    let m = moves(&p);
    // A 45 deg turn: the tangent length is R tan(22.5 deg).
    let t = 4.0 * (22.5_f64).to_radians().tan();
    let s = std::f64::consts::FRAC_1_SQRT_2;
    assert_to(m[1], 10.0 - t, 0.0, -1.0);
    assert_to(m[2], 10.0 + t * s, t * s, -1.0);
    let a = m[2].arc.expect("arc");
    assert!(a.ccw);
    for p in [m[1].to, m[2].to] {
        assert!(
            near((p.x - a.center.x).hypot(p.y - a.center.y), 4.0),
            "radius at {p:?}"
        );
    }
    // Tangency: the radius at each end is normal to its line.
    assert!(near(m[1].to.x - a.center.x, 0.0));
    let (rx, ry) = (m[2].to.x - a.center.x, m[2].to.y - a.center.y);
    assert!(
        near(rx * s + ry * s, 0.0),
        "radius is normal to the line after"
    );
}

#[test]
fn klartext_rnd_between_l_and_lp_lines() {
    let p = ok(&pgm(&[
        "CC X+0 Y+0",
        "L X+0 Y+0 Z-1 R0 F100",
        "L X+10",
        "RND R2",
        "LP PR+10 PA+90",
    ]));
    let m = moves(&p);
    // A 135 deg turn: t = 2 tan(67.5 deg).
    let t = 2.0 * (67.5_f64).to_radians().tan();
    let s = std::f64::consts::FRAC_1_SQRT_2;
    assert_to(m[1], 10.0 - t, 0.0, -1.0);
    assert_to(m[2], 10.0 - t * s, t * s, -1.0);
    assert_arc(m[2], 10.0 - t, 2.0, true);
    assert_to(m[3], 0.0, 10.0, -1.0);
}

#[test]
fn klartext_transition_elements_refuse_what_is_not_exact() {
    let base = "L X+0 Y+0 Z-1 R0 F100";
    // (body, line of the refused block, code)
    let cases: [(&[&str], usize, &str); 11] = [
        // After an arc, before an arc, at the program end.
        (
            &[base, "CC X+0 Y+5", "C X+5 Y+5 DR-", "RND R2", "L X+5 Y+20"],
            5,
            "RND",
        ),
        (
            &[base, "L X+10", "RND R2", "CR X+20 Y+10 R+10 DR+"],
            4,
            "RND",
        ),
        (&[base, "L X+10", "CHF 2"], 4, "CHF"),
        // Too large for the line, on a straight run, two in a row.
        (&[base, "L X+5", "RND R10", "L X+5 Y+20"], 4, "RND"),
        (&[base, "L X+10", "CHF 2", "L X+20"], 4, "CHF"),
        (&[base, "L X+10", "RND R2", "RND R3", "L Y+10"], 4, "RND"),
        // The middle line is 5 mm, but the two roundings need 3 mm each.
        (
            &[base, "L X+10", "RND R3", "L Y+5", "RND R3", "L X+0"],
            6,
            "RND",
        ),
        // A Z change, FMAX, an M function at the block end before, an M function after.
        (&[base, "L X+10", "RND R2", "L Y+10 Z-2"], 5, "RND"),
        (&[base, "L X+10 FMAX", "RND R2", "L Y+10"], 3, "RND"),
        (&[base, "L X+10 M5", "RND R2", "L Y+10"], 3, "RND"),
        (&[base, "L X+10", "CHF 2", "L Y+10 M9"], 5, "CHF"),
    ];
    for (body, line, code) in cases {
        let e = refusal(&pgm(body));
        match &e {
            GcodeError::UnsupportedCode { code: c, .. } => assert_eq!(c, code, "{body:?}: {e}"),
            other => panic!("{body:?}: expected an unsupported-code refusal, got {other}"),
        }
        assert_eq!(e.line(), line, "{body:?}: {e}");
        assert!(
            e.to_string().contains(&format!("block {}:", line - 1)),
            "{e}"
        );
    }
    for bad in ["RND R0", "RND R-2", "RND", "CHF", "CHF 2 3", "RND R2 X+1"] {
        let e = refusal(&pgm(&[base, "L X+10", bad, "L Y+10"]));
        assert_eq!(e.line(), 4, "{bad}: {e}");
    }
}

#[test]
fn klartext_rnd_on_the_first_move_of_a_section_starts_at_the_line_start() {
    // The first move of the program waits for its RND.
    let p = ok(&pgm(&["L X+20 Y+0 F100", "RND R5", "L X+20 Y+20"]));
    let m = moves(&p);
    assert_eq!(m.len(), 3);
    assert!(
        near(m[0].from.x, 0.0) && near(m[0].from.y, 0.0),
        "{:?}",
        m[0].from
    );
    assert_to(m[0], 15.0, 0.0, 0.0);
    assert_arc(m[1], 15.0, 5.0, true);
    assert_to(m[2], 20.0, 20.0, 0.0);
    // The first move after a tool change waits for its RND.
    let p = ok(&pgm(&[
        "L X+0 Y+0 Z-1 R0 F100",
        "TOOL CALL 2 Z S1000",
        "L X+20 Y+0",
        "RND R5",
        "L X+20 Y+20",
    ]));
    assert_eq!(p.sections.len(), 2);
    let s = &p.sections[1].toolpath.moves;
    assert!(
        near(s[0].from.x, 0.0) && near(s[0].from.y, 0.0),
        "{:?}",
        s[0].from
    );
    assert_to(&s[0], 15.0, 0.0, -1.0);
    assert_arc(&s[1], 15.0, 5.0, true);
}
