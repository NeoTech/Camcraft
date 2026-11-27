//! #4417: a G43.4 post writes the joint angles of the machine's own
//! tilt chain, not the stream's pose letters. On the tilt head (A carries
//! B) the tool axis of the posted words is `Rx(A)·Ry(B)·Z`; the pose law
//! of the letters is `Ry(B)·Rx(A)·Z`. Each posted block must give back
//! the stream's tool axis on the machine's forward kinematics, and the
//! machine check judges the same angles that the post writes.
//!
//! Gate map: covers post::tilt_chain (the prelude `chainAngles`),
//! machine::tilt_chain, machine::judge, assets/posts/fanuc.cps,
//! assets/posts/doosan-dnm.cps.

use camcraft_post::cl::{Move, MoveKind, Toolpath};
use camcraft_post::geometry::Point3;
use camcraft_post::machine::{parse_mch, tilt_chain, Machine, MachineCheck};
use camcraft_post::post::{engine, machine::mirror};
use serde_json::Value;
use std::path::Path;

fn read(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

/// The tilt head with its two rotaries' names and axes swapped: B carries A.
fn swap_nest(text: &str) -> String {
    let mut v: Value = serde_json::from_str(text).expect("tilthead is JSON");
    let a = v.pointer_mut("/kinematics/default/parts/0/parts/1/parts/0").expect("the A part");
    let (an, aa) = (a["name"].clone(), a["axis"].clone());
    let b = &mut a["parts"][0];
    let (bn, ba) = (b["name"].clone(), b["axis"].clone());
    b["name"] = an;
    b["axis"] = aa;
    let a = v.pointer_mut("/kinematics/default/parts/0/parts/1/parts/0").expect("the A part");
    a["name"] = bn;
    a["axis"] = ba;
    v.to_string()
}

fn with_travel(text: &str, a: f64, b: f64) -> String {
    let mut v: Value = serde_json::from_str(text).expect("JSON");
    let outer = v.pointer_mut("/kinematics/default/parts/0/parts/1/parts/0").expect("outer rotary");
    let set = |p: &mut Value, t: f64| p["travel"] = serde_json::json!({ "min": -t, "max": t });
    let lim = |p: &Value| if p["name"] == "A" { a } else { b };
    let t = lim(outer);
    set(outer, t);
    let inner = &mut outer["parts"][0];
    let t = lim(inner);
    set(inner, t);
    v.to_string()
}

const POSES: [(f64, f64); 5] = [(30.0, 0.0), (-40.588, 68.859), (25.0, -50.0), (0.0, 45.0), (-60.0, -30.0)];

fn stream() -> String {
    let mut s = String::from("PARAM ClearancePlane 20\nTOOL 1 T1\nPARAM SpindleSpeed 6000\nSECTION\nRapid X 0 Y 0 Z 20000 A 30000 B 0 C 0\n");
    for (k, (a, b)) in POSES.iter().enumerate().skip(1) {
        let x = 10_000 * k;
        s.push_str(&format!("Feed X {x} Y 0 Z 0 F 300000 A {} B {} C 0\n", (a * 1000.0).round(), (b * 1000.0).round()));
    }
    s.push_str("ENDSECTION\n");
    s
}

fn pose_axis(a: f64, b: f64) -> [f64; 3] {
    let (sa, ca) = a.to_radians().sin_cos();
    let (sb, cb) = b.to_radians().sin_cos();
    [sb * ca, -sa, cb * ca]
}

/// Forward kinematics of a head chain: `a_outer` is Rx(A)·Ry(B)·Z.
fn chain_axis(a: f64, b: f64, a_outer: bool) -> [f64; 3] {
    if !a_outer {
        return pose_axis(a, b);
    }
    let (sa, ca) = a.to_radians().sin_cos();
    let (sb, cb) = b.to_radians().sin_cos();
    [sb, -sa * cb, ca * cb]
}

fn word(line: &str, k: char) -> Option<f64> {
    line.split(' ').find_map(|w| w.strip_prefix(k).and_then(|r| r.parse::<f64>().ok()))
}

/// The modal (A, B) of each block that writes an X word and an angle word.
fn posted_angles(program: &str) -> Vec<(f64, f64)> {
    let (mut a, mut b) = (0.0, 0.0);
    let mut out = Vec::new();
    for l in program.lines() {
        a = word(l, 'A').unwrap_or(a);
        b = word(l, 'B').unwrap_or(b);
        if word(l, 'X').is_some() && (l.contains(" A") || l.contains(" B")) {
            out.push((a, b));
        }
    }
    out
}

fn post(cps: &str, mch: &Machine) -> engine::PostOutput {
    let options = serde_json::json!({ "rotaryBrake": true });
    let out = engine::run_with(cps, &mirror(mch), &stream(), &options).expect("post");
    assert!(out.warnings.is_empty(), "{:?}\n{}", out.warnings, out.program);
    out
}

fn assert_axes(out: &engine::PostOutput, mch: &Machine, a_outer: bool) {
    let got = posted_angles(&out.program);
    assert_eq!(got.len(), POSES.len(), "one angle block per pose: {}", out.program);
    for ((pa, pb), (ja, jb)) in POSES.iter().zip(&got) {
        let want = pose_axis(*pa, *pb);
        let have = chain_axis(*ja, *jb, a_outer);
        let err = (0..3).map(|k| (want[k] - have[k]).abs()).fold(0.0, f64::max);
        assert!(err < 1e-4, "pose A{pa} B{pb}: posted A{ja} B{jb} gives {have:?}, the stream states {want:?}\n{}", out.program);
        // The machine check judges these very angles.
        let j = tilt_chain::joint_angles(mch, [*pa, *pb, 0.0]).expect("judged");
        assert!((j[0] - ja).abs() < 1.5e-3 && (j[1] - jb).abs() < 1.5e-3, "check {j:?} vs posted A{ja} B{jb}");
    }
}

#[test]
fn fanuc_posts_the_joint_angles_of_a_chain_where_a_carries_b() {
    let mch = parse_mch(&read("assets/machines/tilthead.mch")).expect("tilthead parses");
    let out = post(&read("assets/posts/fanuc.cps"), &mch);
    assert_axes(&out, &mch, true);
}

#[test]
fn doosan_dnm_posts_the_joint_angles_of_a_chain_where_a_carries_b() {
    let mch = parse_mch(&read("assets/machines/tilthead.mch")).expect("tilthead parses");
    let out = post(&read("assets/posts/doosan-dnm.cps"), &mch);
    assert_axes(&out, &mch, true);
}

#[test]
fn a_chain_where_b_carries_a_posts_the_pose_letters() {
    let mch = parse_mch(&swap_nest(&read("assets/machines/tilthead.mch"))).expect("swapped tilthead parses");
    for cps in ["assets/posts/fanuc.cps", "assets/posts/doosan-dnm.cps"] {
        let out = post(&read(cps), &mch);
        assert_axes(&out, &mch, false);
        assert!(out.program.contains("A-40.588 B68.859"), "{cps}: the letters as they come: {}", out.program);
    }
}

#[test]
fn a_chain_with_no_stated_nest_is_refused_by_name() {
    let mch = serde_json::json!({
        "kinematics": { "static": { "rotaries": [
            { "letter": "A", "type": "rotary", "direction": [1.0, 0.0, 0.0] },
            { "letter": "B", "type": "rotary", "direction": [0.0, 1.0, 0.0] } ] } }
    });
    let options = serde_json::json!({ "rotaryBrake": true });
    for cps in ["assets/posts/fanuc.cps", "assets/posts/doosan-dnm.cps"] {
        let out = engine::run_with(&read(cps), &mch, &stream(), &options).expect("post");
        assert_eq!(
            out.warnings,
            vec!["tilted move turns A and B - the machine file does not state which of the two rotaries carries the other, so their joint angles are unknown"],
            "{cps}"
        );
    }
}

fn leaning(a: f64, b: f64) -> Toolpath {
    Toolpath::new(vec![Move::new(
        MoveKind::Feed,
        Point3 { x: 0.0, y: 0.0, z: 0.0 },
        Point3 { x: 10.0, y: 0.0, z: 0.0 },
        Some(100.0),
    )
    .with_axis(pose_axis(a, b))])
}

#[test]
fn the_machine_check_judges_the_joint_angles_not_the_letters() {
    // A -40.588 B 68.859 holds A -67.2 B 45.1 on the tilt head: inside
    // the letters' 60/70 travel, outside the joint's.
    let head = read("assets/machines/tilthead.mch");
    let tight = parse_mch(&with_travel(&head, 60.0, 70.0)).expect("parses");
    let mut out = MachineCheck::default();
    out.check_path(&tight, None, &leaning(-40.588, 68.859), [0.0; 3], [false; 3], false);
    assert!(out.problems.iter().any(|p| p.contains("commands A -67.")), "{:?}", out.problems);
    // The same travel on the chain where B carries A holds the letters.
    let swapped = parse_mch(&with_travel(&swap_nest(&head), 60.0, 70.0)).expect("parses");
    let mut out = MachineCheck::default();
    out.check_path(&swapped, None, &leaning(-40.588, 68.859), [0.0; 3], [false; 3], false);
    assert!(out.problems.is_empty(), "{:?}", out.problems);
}

#[test]
fn a_pose_past_ninety_takes_the_second_solution_inside_the_travel() {
    // A 0 B 120 is the same rotation in either order; A 10 B 110 is not,
    // and its principal solution (B in -90..90) holds A 152.7, outside
    // the 120 travel - the other solution (A -27.3 B 112.3) is inside it.
    let mch = parse_mch(&read("assets/machines/tilthead.mch")).expect("tilthead parses");
    assert_eq!(tilt_chain::joint_angles(&mch, [0.0, 120.0, 0.0]).expect("ok"), [0.0, 120.0, 0.0]);
    let j = tilt_chain::joint_angles(&mch, [10.0, 110.0, 0.0]).expect("ok");
    assert!(j[0].abs() <= 120.0 && j[1].abs() <= 120.0, "{j:?}");
    let want = pose_axis(10.0, 110.0);
    let have = chain_axis(j[0], j[1], true);
    assert!((0..3).all(|k| (want[k] - have[k]).abs() < 1e-9), "{have:?} vs {want:?}");
}
