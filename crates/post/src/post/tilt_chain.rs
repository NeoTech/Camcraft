//! The joint angles of a two-rotary tilt chain (#4417), in the prelude
//! every post script sees: `chainAngles(p)` turns the stream's pose
//! letters into the angles that the machine's A and B rotaries hold for
//! the nest order of the machine file.
//!
//! Provenance: the JS mirror of `machine::tilt_chain` (the product of
//! rotations along a serial chain, J. J. Craig, "Introduction to
//! Robotics", ch. 2-3, and our `cl::pose` law). The rule and the refusal
//! texts are the same word for word, so the machine check judges the
//! angles that a G43.4 post writes.

/// Appended to [`super::js::PRELUDE`]: plain JS, no host calls.
pub const TILT_CHAIN_JS: &str = r#"
// The rotary of `letter` whose direction lies along component k (0 X,
// 1 Y) within 0.998 of its length squared, else null.
function _tiltRotary(letter, k) {
  var rs = (typeof machine !== "undefined" && machine && machine.rotaries) || [];
  for (var i = 0; i < rs.length; i++) {
    if (String(rs[i].name || "").toUpperCase() !== letter) continue;
    var d = rs[i].direction;
    if (!d || d.length !== 3) return null;
    var x = Number(d[0]), y = Number(d[1]), z = Number(d[2]);
    if (!isFinite(x) || !isFinite(y) || !isFinite(z)) return null;
    var n2 = x * x + y * y + z * z, c = Number(d[k]);
    return (n2 > 0 && c * c >= 0.998 * n2) ? rs[i] : null;
  }
  return null;
}
function _tiltHas(v) { return v !== null && v !== undefined && isFinite(v); }
function _tiltSnap(v) { return Math.abs(v) < 1e-9 ? 0 : v; }
function _tiltWrap(v) { var x = v % 360; if (x > 180) x -= 360; else if (x <= -180) x += 360; return x; }
function _tiltIn(r, v) {
  var t = r.travel;
  if (!t || t.length !== 2 || !isFinite(Number(t[0])) || !isFinite(Number(t[1]))) return true;
  return v >= Number(t[0]) - 1e-6 && v <= Number(t[1]) + 1e-6;
}
// "letters" (B is the outer factor, the pose law), "aOuter", or
// "unstated" (the file does not say which rotary carries the other).
function _tiltNest(ra, rb) {
  var pa = String(ra.parent || "").toUpperCase(), pb = String(rb.parent || "").toUpperCase();
  var ta = ra.turnsPart, tb = rb.turnsPart;
  if (ta === false && tb === false) return pb === "A" ? "aOuter" : (pa === "B" ? "letters" : "unstated");
  if (ta === true && tb === true) return pb === "A" ? "letters" : (pa === "B" ? "aOuter" : "unstated");
  if (ta === true && tb === false) return "aOuter";
  if (ta === false && tb === true) return "letters";
  return "unstated";
}
var tiltChainWhy = "";  // the refusal of the last chainAngles that returned null
// The joint angles {A, B, C} that the machine's rotaries hold for the
// pose letters of `p`, or null (the reason in tiltChainWhy). A machine
// without an A-about-X and B-about-Y pair, and a pose that turns at most
// one of the two, keep the letters exactly.
function chainAngles(p) {
  var a = _tiltHas(p.A) ? Number(p.A) : 0, b = _tiltHas(p.B) ? Number(p.B) : 0, c = _tiltHas(p.C) ? Number(p.C) : 0;
  var same = { A: p.A, B: p.B, C: p.C };
  var ra = _tiltRotary("A", 0), rb = _tiltRotary("B", 1);
  if (!ra || !rb) return same;
  var turns = [[ra, a, 0, "X"], [rb, b, 1, "Y"]];
  for (var i = 0; i < 2; i++) {
    if (Math.abs(turns[i][1]) > 1e-9 && Number(turns[i][0].direction[turns[i][2]]) < 0) {
      tiltChainWhy = "tilted move turns rotary " + String(turns[i][0].name).toUpperCase() + ", stated along -" + turns[i][3] + " in a two-rotary tilt chain - the post writes the pose angle unnegated, so the machine would turn the other way";
      return null;
    }
  }
  if (Math.abs(a) <= 1e-9 || Math.abs(b) <= 1e-9) return same;
  var nest = _tiltNest(ra, rb);
  if (nest === "letters") return same;
  if (nest === "unstated") {
    tiltChainWhy = "tilted move turns A and B - the machine file does not state which of the two rotaries carries the other, so their joint angles are unknown";
    return null;
  }
  if (Math.abs(c) > 1e-9) {
    tiltChainWhy = "tilted move turns A, B and C on a chain where A is the outer rotary - the joint angles of that pose are not solved";
    return null;
  }
  var r = Math.PI / 180;
  var vx = Math.sin(b * r) * Math.cos(a * r), vy = -Math.sin(a * r), vz = Math.cos(b * r) * Math.cos(a * r);
  var b1 = Math.asin(Math.max(-1, Math.min(1, vx)));
  var a1 = Math.cos(b1) < 1e-12 ? 0 : Math.atan2(-vy, vz);
  a1 = _tiltSnap(a1 / r); b1 = _tiltSnap(b1 / r);
  var a2 = _tiltSnap(_tiltWrap(a1 + 180)), b2 = _tiltSnap(_tiltWrap(180 - b1));
  if (!(_tiltIn(ra, a1) && _tiltIn(rb, b1)) && _tiltIn(ra, a2) && _tiltIn(rb, b2)) return { A: a2, B: b2, C: p.C };
  return { A: a1, B: b1, C: p.C };
}
"#;
