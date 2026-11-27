//! The joint angles of a two-rotary tilt chain (#4417): which angles an
//! A rotary about X and a B rotary about Y must hold so that the tool
//! stands on the axis the stream's letters state, for the nest order
//! that the machine file gives.
//!
//! Provenance: the product of rotations along a serial chain (J. J.
//! Craig, "Introduction to Robotics", ch. 2-3) and our own pose law
//! (`cl::pose`: axis = Ry(B)·Rx(A)·Z, B the outer factor). A head chain
//! puts its outer (parent) rotary on the left of the product; a table
//! chain turns the work, so the tool seen from the work takes the
//! inverse and the rotary nearest the work is on the left. One rule,
//! mirrored in the post prelude (`post::tilt_chain`): the machine check
//! judges the same joint angles that a G43.4 post writes.

use super::{AxisLimits, Machine};

/// Below this, an angle (degrees) or a component is zero.
const DUST: f64 = 1e-9;

/// Which letter is the outer factor of the chain's tool-axis product.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TiltNest {
    /// B is the outer factor, as in the pose law: the letters are the
    /// joint angles.
    Letters,
    /// A is the outer factor: the joint angles are solved again.
    AOuter,
    /// The file does not say which rotary carries the other.
    Unstated,
}

/// The rotary named `letter` whose stated axis lies along `k` (0 = X,
/// 1 = Y) within the posts' own 0.998 of its length squared.
fn tilting<'a>(m: &'a Machine, letter: &str, k: usize) -> Option<&'a AxisLimits> {
    let ax = m.axes.iter().find(|a| a.rotary && a.name.eq_ignore_ascii_case(letter))?;
    let d = ax.axis?;
    let n2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
    (d.iter().all(|v| v.is_finite()) && n2 > 0.0 && d[k] * d[k] >= 0.998 * n2).then_some(ax)
}

/// The A rotary about X and the B rotary about Y, when the machine
/// states both. Else None: one tilting rotary has no nest to read.
#[must_use]
pub fn tilt_pair(m: &Machine) -> Option<(&AxisLimits, &AxisLimits)> {
    Some((tilting(m, "A", 0)?, tilting(m, "B", 1)?))
}

/// The nest of the pair: see [`TiltNest`].
#[must_use]
pub fn tilt_nest(a: &AxisLimits, b: &AxisLimits) -> TiltNest {
    let b_on_a = b.parent.as_deref().is_some_and(|p| p.eq_ignore_ascii_case("A"));
    let a_on_b = a.parent.as_deref().is_some_and(|p| p.eq_ignore_ascii_case("B"));
    let outer_a = match (a.turns_part, b.turns_part) {
        // A head chain: the parent is the outer factor.
        (Some(false), Some(false)) if b_on_a => true,
        (Some(false), Some(false)) if a_on_b => false,
        // A table chain: the rotary nearest the work is the outer factor.
        (Some(true), Some(true)) if b_on_a => false,
        (Some(true), Some(true)) if a_on_b => true,
        // A table under a head: the table is the outer factor.
        (Some(true), Some(false)) => true,
        (Some(false), Some(true)) => false,
        _ => return TiltNest::Unstated,
    };
    if outer_a { TiltNest::AOuter } else { TiltNest::Letters }
}

fn snap(v: f64) -> f64 {
    if v.abs() < DUST { 0.0 } else { v }
}

fn wrap(v: f64) -> f64 {
    let mut x = v % 360.0;
    if x > 180.0 {
        x -= 360.0;
    } else if x <= -180.0 {
        x += 360.0;
    }
    x
}

fn in_travel(ax: &AxisLimits, v: f64) -> bool {
    ax.travel.is_none_or(|(lo, hi)| v >= lo - 1e-6 && v <= hi + 1e-6)
}

/// The angles [A, B] of the chain `Rx(A)·Ry(B)·Z` that give the axis of
/// the pose-law letters `a`, `b` (degrees). Two solutions exist away
/// from B = ±90; the first that both travels hold is taken, else the
/// principal one (B in -90..90), which the travel check then refuses.
#[must_use]
pub fn solve_a_outer(a: f64, b: f64, ta: &AxisLimits, tb: &AxisLimits) -> [f64; 2] {
    let (sa, ca) = a.to_radians().sin_cos();
    let (sb, cb) = b.to_radians().sin_cos();
    let v = [sb * ca, -sa, cb * ca];
    let b1 = v[0].clamp(-1.0, 1.0).asin();
    let a1 = if b1.cos() < 1e-12 { 0.0 } else { (-v[1]).atan2(v[2]) };
    let (a1, b1) = (snap(a1.to_degrees()), snap(b1.to_degrees()));
    let (a2, b2) = (snap(wrap(a1 + 180.0)), snap(wrap(180.0 - b1)));
    if !(in_travel(ta, a1) && in_travel(tb, b1)) && in_travel(ta, a2) && in_travel(tb, b2) {
        return [a2, b2];
    }
    [a1, b1]
}

/// The joint angles [A, B, C] that the machine's rotaries hold for the
/// pose-law `letters`, or the reason that no G43.4 post can write them.
/// A machine without an A-about-X and B-about-Y pair, and a pose that
/// turns at most one of the two, keep the letters exactly: then the two
/// products are the same rotation.
///
/// # Errors
/// The refusal text, shared word for word with the post prelude.
pub fn joint_angles(m: &Machine, letters: [f64; 3]) -> Result<[f64; 3], String> {
    let Some((ta, tb)) = tilt_pair(m) else { return Ok(letters) };
    let [a, b, c] = letters;
    for (ax, v, k, name) in [(ta, a, 0, "X"), (tb, b, 1, "Y")] {
        if v.abs() > DUST && ax.axis.is_some_and(|d| d[k] < 0.0) {
            return Err(format!(
                "tilted move turns rotary {}, stated along -{name} in a two-rotary tilt chain - the post writes the pose angle unnegated, so the machine would turn the other way",
                ax.name
            ));
        }
    }
    if a.abs() <= DUST || b.abs() <= DUST {
        return Ok(letters);
    }
    match tilt_nest(ta, tb) {
        TiltNest::Letters => Ok(letters),
        TiltNest::Unstated => Err(
            "tilted move turns A and B - the machine file does not state which of the two rotaries carries the other, so their joint angles are unknown"
                .to_owned(),
        ),
        TiltNest::AOuter if c.abs() > DUST => Err(
            "tilted move turns A, B and C on a chain where A is the outer rotary - the joint angles of that pose are not solved"
                .to_owned(),
        ),
        TiltNest::AOuter => {
            let [ja, jb] = solve_a_outer(a, b, ta, tb);
            Ok([ja, jb, c])
        }
    }
}
