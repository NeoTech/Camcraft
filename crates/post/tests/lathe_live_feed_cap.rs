//! A feed held under a machine rate is judged on its POSTED word
//! (#1392): the Fanuc lathe post's C-axis live-tool section caps F so
//! the X/Z part stays within the machine's maximum feed and the C part
//! within its C rate, then writes F at one decimal. Rounding to nearest
//! could lift a feed within half a step of its cap up to 0.05 mm/min
//! over it; the word is floored to the step at or under the cap instead.
//! A feed far from its cap posts exactly as before.
//!
//! Gate map (M4.4): covers post::post (engine on fanuc-lathe.cps's
//! liveMove/capWord), post::machine (the mirror the script reads).
//! Lane shape (M4.4): all tests here run in milliseconds.

use camcraft_post::cl::MoveKind;
use camcraft_post::machine::{AxisLimits, Coolant, LiveToolCodes, Machine};
use camcraft_post::post::{cnc, engine};
use serde_json::json;

const LATHE_CPS: &str = include_str!("../../../assets/posts/fanuc-lathe.cps");

/// A test-bench mill-turn whose C rate is `c_rate` deg/min and whose
/// maximum linear feed is `fmax` mm/min. Placeholder live-tool words.
fn mill_turn(c_rate: f64, fmax: f64) -> serde_json::Value {
    let mut v = camcraft_post::post::machine::mirror(&Machine {
        model: "TestBench mill-turn".into(),
        kind: Some("mill-turn".into()),
        spindle_max: Some(6000.0),
        coolants: vec![Coolant { name: "Flood".into(), m_code: "M8".into() }],
        axes: vec![
            AxisLimits { name: "X".into(), feed: Some(fmax), ..Default::default() },
            AxisLimits { name: "Z".into(), feed: Some(fmax), ..Default::default() },
            AxisLimits {
                name: "C".into(),
                rotary: true,
                axis: Some([0.0, 0.0, 1.0]),
                feed: Some(c_rate),
                rapid: Some(2.0 * c_rate),
                ..Default::default()
            },
        ],
        live_tool: Some(LiveToolCodes {
            c_axis_on: "M1001".into(),
            c_axis_off: "M1002".into(),
            spindle_on: "M1003".into(),
            spindle_off: "M1004".into(),
        }),
        ..Default::default()
    });
    // The bench states its maximum feed outright, so the test does not
    // lean on how the mirror folds the axis feeds into it.
    v["feedRateMax"] = json!(fmax);
    v
}

/// A live-tool move: the tip at radius `r` turned onto the station by
/// `c` degrees (part frame: the tip sits at angle -c), height `z`.
fn cmv(kind: MoveKind, r: f64, c: f64, z: f64, feed: Option<f64>) -> cnc::Record {
    let a = -c.to_radians();
    cnc::Record::Move(cnc::MoveRec {
        kind,
        x: r * a.cos(),
        y: r * a.sin(),
        z,
        feed,
        center: None,
        ccw: false,
        plane: Default::default(),
        abc: Some([0.0, 0.0, c]),
    })
}

/// One live-tool section: position at C0 r10 z1, then `cut`.
fn live_post(machine: &serde_json::Value, cut: cnc::Record) -> engine::PostOutput {
    let r = vec![
        cnc::Record::Param { name: "SpindleSpeed".into(), value: json!(3000.0) },
        cnc::Record::Param { name: "WorkOffset".into(), value: json!("G54") },
        cnc::Record::Param { name: "LiveTool".into(), value: json!("radial") },
        cnc::Record::Tool { number: 2, name: "EM".into() },
        cnc::Record::Section { begin: true },
        cmv(MoveKind::Rapid, 10.0, 0.0, 1.0, None),
        cut,
        cnc::Record::Section { begin: false },
    ];
    engine::run(LATHE_CPS, machine, &cnc::serialize(&r)).expect("the script runs")
}

/// The F word of the one feed G01 that turns C.
fn f_word(out: &engine::PostOutput) -> f64 {
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    let line = out
        .program
        .lines()
        .find(|l| l.starts_with("G01") && l.contains(" C"))
        .unwrap_or_else(|| panic!("a G01 with C: {}", out.program));
    let f = line.split_whitespace().find_map(|w| w.strip_prefix('F')).unwrap_or_else(|| panic!("F in {line}"));
    f.parse().unwrap_or_else(|_| panic!("F number in {line}"))
}

/// The planned feed whose composite (C-degrees-as-mm) rate is `target`
/// for a pure C turn of `dc` degrees at radius `r`.
fn planned_for(target: f64, r: f64, dc: f64) -> f64 {
    let chord = 2.0 * r * (dc.to_radians() / 2.0).sin();
    target * chord / dc
}

#[test]
fn a_feed_capped_at_a_c_rate_between_steps_posts_at_or_under_it() {
    // C rate 6000.06 deg/min: a pure C turn at a planned feed far over
    // it is capped to 6000.06 exactly. Rounded to nearest that wrote
    // F6000.1 - 0.04 deg/min over the machine's stated rate.
    let m = mill_turn(6000.06, 4000.0);
    let out = live_post(&m, cmv(MoveKind::Feed, 10.0, 90.0, 1.0, Some(50000.0)));
    let f = f_word(&out);
    assert!(f <= 6000.06, "F{f} passes the C rate 6000.06: {}", out.program);
    assert_eq!(f, 6000.0, "floored to the step under the cap: {}", out.program);
}

#[test]
fn a_feed_one_step_under_its_c_cap_posts_at_or_under_it() {
    // The issue's case: the planned composite feed 6000.06 is UNDER the
    // C rate 6000.08 (so the cap does not bind), but rounds to 6000.1.
    let m = mill_turn(6000.08, 4000.0);
    let out = live_post(&m, cmv(MoveKind::Feed, 10.0, 90.0, 1.0, Some(planned_for(6000.06, 10.0, 90.0))));
    let f = f_word(&out);
    assert!(f <= 6000.08, "F{f} passes the C rate 6000.08: {}", out.program);
    assert_eq!(f, 6000.0, "{}", out.program);
}

#[test]
fn a_feed_one_step_under_the_linear_cap_posts_at_or_under_it() {
    // C turns a little while Z runs far: the linear part binds. The
    // composite feed sits under the maximum feed's cap but rounds over.
    let m = mill_turn(1.0e6, 4000.07);
    let (r, dc, dz) = (10.0_f64, 0.5_f64, -40.0_f64);
    let a = dc.to_radians();
    let ltrue = ((r * a.cos() - r).powi(2) + (r * a.sin()).powi(2) + dz * dz).sqrt();
    let lcmd = (dz * dz + dc * dc).sqrt();
    let cap = 4000.07 * lcmd / dz.abs();
    // The composite rate 0.02 under the cap, and still over its step.
    let target = cap - 0.02;
    assert!(format!("{target:.1}").parse::<f64>().expect("number") > cap, "the bench must round over: {target} vs {cap}");
    let out = live_post(&m, cmv(MoveKind::Feed, r, dc, 1.0 + dz, Some(target * ltrue / lcmd)));
    let f = f_word(&out);
    assert!(f <= cap, "F{f} passes the linear cap {cap}: {}", out.program);
    assert!(cap - f < 0.1, "floored by one step, no more: F{f} vs {cap}");
}

#[test]
fn a_feed_far_from_its_cap_posts_unchanged() {
    // 1234.56 composite under a 6000 C rate: rounded to nearest as ever.
    let m = mill_turn(6000.0, 4000.0);
    let out = live_post(&m, cmv(MoveKind::Feed, 10.0, 90.0, 1.0, Some(planned_for(1234.56, 10.0, 90.0))));
    assert_eq!(f_word(&out), 1234.6, "{}", out.program);
    // And a feed exactly at its cap keeps the cap's own word.
    let out = live_post(&mill_turn(6000.0, 4000.0), cmv(MoveKind::Feed, 10.0, 90.0, 1.0, Some(50000.0)));
    assert_eq!(f_word(&out), 6000.0, "{}", out.program);
}
