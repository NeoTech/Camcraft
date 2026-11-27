//! Provenance: issue #4436 and the machine schema's tool_rh/table_rh contract.
//! Gate map: covers post::machine::judge, post::machine::trunnion,
//! post::post::machine_rotary, assets/posts/fanuc.cps, assets/posts/doosan-dnm.cps,
//! assets/posts/tormach.cps.

use camcraft_post::cl::{Move, MoveKind, Toolpath};
use camcraft_post::geometry::Point3;
use camcraft_post::machine::{Machine, MachineCheck, parse_mch};
use camcraft_post::post::{self, Dump, cnc, engine, machine};
use serde_json::json;
use std::path::Path;

const POSTS: [&str; 2] = ["fanuc", "doosan-dnm"];
const STREAM: &str = "\
PARAM ClearancePlane 20
TOOL 1 T1
PARAM SpindleSpeed 6000
SECTION
Rapid X 0 Y 0 Z 20000 A 30000 B 0 C 0
Feed X 10000 Y 0 Z 0 F 300000 A 40000 B 0 C 0
Feed X 20000 Y 0 Z 0 F 300000 A 0 B 20000 C 0
Feed X 30000 Y 0 Z 0 F 300000 A 0 B 0 C 25000
ENDSECTION
";

fn fixture(default: Option<&str>, overrides: [Option<&str>; 3]) -> Machine {
    let parts: Vec<_> = [
        ("A", [1.0, 0.0, 0.0]),
        ("B", [0.0, 1.0, 0.0]),
        ("C", [0.0, 0.0, 1.0]),
    ]
    .into_iter()
    .zip(overrides)
    .map(|((name, axis), sign)| {
        let mut rotary = json!({
            "name": name, "type": "rotary", "axis": axis,
            "travel": { "min": -120.0, "max": 120.0 }
        });
        if let Some(sign) = sign {
            rotary["sign_convention"] = json!(sign);
        }
        rotary
    })
    .collect();
    let mut raw = json!({
        "general": { "model": "SIGN TEST", "kind": "mill" },
        "kinematics": { "default": { "parts": parts } },
        "controller": { "rtcp": true }
    });
    if let Some(sign) = default {
        raw["kinematics"]["default"]["sign_convention"] = json!(sign);
    }
    parse_mch(&raw.to_string()).expect("original user-machine fixture parses")
}

fn run_post(post: &str, m: &Machine, stream: &str) -> engine::PostOutput {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../assets/posts/{post}.cps"));
    let script = std::fs::read_to_string(path).expect("shipped post");
    let options = if post == "doosan-dnm" {
        json!({ "rotaryClamp": "M10", "rotaryUnclamp": "M11" })
    } else {
        json!({})
    };
    engine::run_with(&script, &machine::mirror(m), stream, &options).expect("post runs")
}

fn output(post: &str, m: &Machine, stream: &str) -> String {
    let out = run_post(post, m, stream);
    assert!(
        out.warnings.is_empty(),
        "{post}: {:?}\n{}",
        out.warnings,
        out.program
    );
    assert!(out.program.contains("G43.4"), "{post}: {}", out.program);
    out.program
}

fn has_word(program: &str, letter: char, expected: f64) -> bool {
    program.split_whitespace().any(|word| {
        word.strip_prefix(letter)
            .and_then(|value| value.parse::<f64>().ok())
            .is_some_and(|value| (value - expected).abs() < 1e-6)
    })
}

#[test]
fn g43_4_posts_use_machine_and_axis_sign_conventions() {
    for (default, overrides, signs) in [
        (Some("table_rh"), [None; 3], [-1.0; 3]),
        (
            Some("table_rh"),
            [Some("tool_rh"), None, Some("tool_rh")],
            [1.0, -1.0, 1.0],
        ),
        (
            Some("tool_rh"),
            [Some("table_rh"), None, Some("table_rh")],
            [-1.0, 1.0, -1.0],
        ),
        (None, [Some("table_rh"); 3], [-1.0; 3]),
    ] {
        let m = fixture(default, overrides);
        for post in POSTS {
            let program = output(post, &m, STREAM);
            for (letter, angle, sign) in [
                ('A', 30.0, signs[0]),
                ('A', 40.0, signs[0]),
                ('B', 20.0, signs[1]),
                ('C', 25.0, signs[2]),
            ] {
                assert!(
                    has_word(&program, letter, sign * angle),
                    "{post}: expected {letter} {}\n{program}",
                    sign * angle
                );
                assert!(
                    !has_word(&program, letter, -sign * angle),
                    "{post}: wrong sign for {letter} {angle}\n{program}"
                );
            }
        }
    }
}

#[test]
fn g43_4_tool_rh_keeps_the_default_program() {
    for post in POSTS {
        assert_eq!(
            output(post, &fixture(None, [None; 3]), STREAM),
            output(
                post,
                &fixture(Some("tool_rh"), [Some("tool_rh"); 3]),
                STREAM
            ),
            "{post}: stating the default must not change the output"
        );
    }
}

fn leaning_path(letter: char) -> Toolpath {
    let (s, c) = 20f64.to_radians().sin_cos();
    let axis = match letter {
        'A' => [0.0, -s, c],
        'B' => [s, 0.0, c],
        _ => panic!("test only covers pose letters A and B"),
    };
    let p = |x, z| Point3::new(x, 0.0, z);
    Toolpath::new(vec![
        Move::new(MoveKind::Rapid, p(0.0, 30.0), p(0.0, 20.0), None).with_axis(axis),
        Move::new(MoveKind::Feed, p(0.0, 20.0), p(10.0, 0.0), Some(300.0)).with_axis(axis),
    ])
}

fn check(m: &Machine, path: &Toolpath) -> MachineCheck {
    let mut check = MachineCheck::default();
    check.check_path(m, None, path, [0.0; 3], [false; 3], true);
    check
}

#[test]
fn g43_4_travel_check_judges_the_posted_sign() {
    for (default, overrides, signs) in [
        (Some("table_rh"), [None; 3], [-1.0; 3]),
        (
            Some("table_rh"),
            [Some("tool_rh"), None, None],
            [1.0, -1.0, -1.0],
        ),
        (None, [None, Some("table_rh"), None], [1.0, -1.0, 1.0]),
    ] {
        for (index, letter) in [(0, 'A'), (1, 'B')] {
            let mut m = fixture(default, overrides);
            let path = leaning_path(letter);
            let angle = signs[index] * 20.0;
            m.axes[index].travel = Some((angle - 5.0, angle + 5.0));
            let verdict = check(&m, &path);
            assert!(
                verdict.is_ok(),
                "{letter}: the posted {angle} is inside travel: {:?}",
                verdict.problems
            );
            let dump = Dump {
                op_name: "Sign1",
                tool_number: 1,
                tool_name: "T1",
                rpm: 6000.0,
                clearance_z: 20.0,
                ..Default::default()
            };
            let stream = cnc::serialize(&post::dump(&path, &dump));
            for post in POSTS {
                let program = output(post, &m, &stream);
                assert!(
                    has_word(&program, letter, angle),
                    "{post}: checked {angle}, posted:\n{program}"
                );
            }
            m.axes[index].travel = Some((-angle - 5.0, -angle + 5.0));
            let verdict = check(&m, &path);
            assert!(
                !verdict.is_ok(),
                "{letter}: the opposite travel must refuse {angle}"
            );
            assert!(
                verdict
                    .problems
                    .iter()
                    .all(|p| p.contains(&format!("commands {letter} {angle:.3} deg"))),
                "{:?}",
                verdict.problems
            );
        }
    }
}

#[test]
fn lone_x_travel_applies_convention_after_the_axis_direction() {
    for (default, axis_sign, sign) in [
        (Some("table_rh"), None, -1.0),
        (Some("table_rh"), Some("tool_rh"), 1.0),
        (Some("tool_rh"), Some("table_rh"), -1.0),
    ] {
        for direction in [-1.0, 1.0] {
            let mut m = fixture(default, [axis_sign, None, None]);
            m.axes.truncate(1);
            m.axes[0].axis = Some([direction, 0.0, 0.0]);
            let angle = direction * sign * 20.0;
            m.axes[0].travel = Some((angle - 5.0, angle + 5.0));
            let verdict = check(&m, &leaning_path('A'));
            assert!(
                verdict.is_ok(),
                "direction {direction}, signed {angle}: {:?}",
                verdict.problems
            );
            m.axes[0].travel = Some((-angle - 5.0, -angle + 5.0));
            let verdict = check(&m, &leaning_path('A'));
            assert!(!verdict.is_ok());
            assert!(
                verdict
                    .problems
                    .iter()
                    .all(|p| p.contains(&format!("commands A {angle:.3} deg"))),
                "{:?}",
                verdict.problems
            );
        }
    }
}

#[test]
fn tormach_refuses_its_own_word_outside_the_signed_travel() {
    let mut m = fixture(Some("table_rh"), [None; 3]);
    m.axes.truncate(1);
    m.axes[0].travel = Some((-25.0, -15.0));
    m.safe_retract_z = Some(50.0);
    let path = leaning_path('A');
    let verdict = check(&m, &path);
    assert!(
        verdict.is_ok(),
        "the signed -20 is within travel: {:?}",
        verdict.problems
    );
    let dump = Dump {
        op_name: "Sign1",
        tool_number: 1,
        tool_name: "T1",
        rpm: 6000.0,
        clearance_z: 20.0,
        ..Default::default()
    };
    let stream = cnc::serialize(&post::dump(&path, &dump));
    let script = include_str!("../../../../assets/posts/tormach.cps");
    let out = engine::run(script, &machine::mirror(&m), &stream).expect("Tormach post runs");
    assert!(
        out.warnings.iter().any(|warning| warning
            .contains("A20 in operation Sign1 is beyond the A axis travel -25..-15 degrees")),
        "Tormach must refuse its own +20 before it writes that word: {:?}\n{}",
        out.warnings,
        out.program
    );
    assert!(!has_word(&out.program, 'A', 20.0), "{}", out.program);
}

#[test]
fn g43_4_posts_refuse_explicit_c_live_tool_work() {
    let mut path = leaning_path('A');
    for mv in &mut path.moves {
        mv.axis = None;
        mv.rotary = Some(20.0);
    }
    let dump = Dump {
        op_name: "SignC",
        tool_number: 1,
        tool_name: "T1",
        rpm: 6000.0,
        clearance_z: 20.0,
        ..Default::default()
    };
    let records = post::dump(&path, &dump);
    assert!(records.iter().any(|record| matches!(record,
        cnc::Record::Param { name, value } if name == "LiveTool" && value == &json!("radial")
    )));
    let stream = cnc::serialize(&records);
    for (default, overrides) in [
        (None, [None; 3]),
        (Some("table_rh"), [None; 3]),
        (None, [None, None, Some("table_rh")]),
        (Some("table_rh"), [None, None, Some("tool_rh")]),
    ] {
        for center in [-20.0, 20.0] {
            let mut m = fixture(default, overrides);
            m.axes[2].travel = Some((center - 5.0, center + 5.0));
            let verdict = check(&m, &path);
            assert_eq!(
                verdict.is_ok(),
                center > 0.0,
                "the distinct live-tool C check keeps its direction-only contract: {:?}",
                verdict.problems
            );
            for post in POSTS {
                let out = run_post(post, &m, &stream);
                assert!(
                    out.warnings.iter().any(|warning| warning
                        .contains("operation SignC: is C-axis live-tool work")
                        && warning.contains("does not declare liveTooling")),
                    "{post}: actual Move::rotary input must return the blocking capability warning: {:?}",
                    out.warnings
                );
            }
        }
    }
}
