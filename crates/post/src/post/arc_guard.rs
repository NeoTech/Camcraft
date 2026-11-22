//! The coincident arc guard (issue #3442): one rule, in the prelude every
//! post script sees, for an arc record whose end point, as the script
//! writes it, is its start point.
//!
//! Provenance: original. RS-274 / ISO 6983 circular interpolation: an
//! arc block whose end point equals the current position, with a centre
//! off that point, is a full circle (the reading of the Fanuc-family,
//! LinuxCNC and grbl controls' published programming manuals, behaviour
//! only). The output resolution is the script's own word format, so the
//! comparison is made on the words as written, never on the stream's
//! millimetres.
//!
//! The rule (decision `docs/decisions/3442-coincident-arc-blocks.md`):
//! - the written in-plane ends differ: the arc posts as before;
//! - they coincide and the record turns more than half a circle (or its
//!   ends are equal in the stream: a full turn stated as one record),
//!   or it states no centre (the script refuses it by name):
//!   the full circle is the arc within the output resolution, and the
//!   arc posts as before;
//! - they coincide on a short arc: the tool moves less than the output
//!   resolution in the plane, so the record posts as a straight move
//!   when the plane's normal axis (a helix) or a rotary still moves, and
//!   as no block at all when nothing changes as written.

/// Appended to [`super::js::PRELUDE`]: plain JS, no host calls.
pub const ARC_GUARD_JS: &str = r#"
// #3442 (arc_guard.rs): what an arc record may become in the output.
// `word(axis, value)` is the script's own text for a position on "X", "Y"
// or "Z" (origin, unit and precision applied; "" for an axis it never
// writes). Returns "arc", "linear" (post the record as a straight move)
// or "none" (write no motion block).
function _arcNum(v) { return typeof v === "number" && isFinite(v); }
function arcShapeOf(p) {
  var c = p.IJKIncrement || {};
  return {
    plane: p.CircularPlane || "XY",
    ccw: p.CircularDirection !== "CW",
    start: { X: p.StartX, Y: p.StartY, Z: p.StartZ },
    end: { X: p.X, Y: p.Y, Z: p.Z },
    center: { X: c.I, Y: c.J, Z: c.K },
    rotary: _arcNum(p.A) || _arcNum(p.B) || _arcNum(p.C)
  };
}
function arcFateOf(a, word) {
  // The right-handed in-plane frame (u, v), u x v = +normal: CCW (seen
  // from the positive normal) is the ordinary atan2 sense in each plane.
  var ax = a.plane === "XZ" ? ["Z", "X", "Y"] : (a.plane === "YZ" ? ["Y", "Z", "X"] : ["X", "Y", "Z"]);
  var u = ax[0], v = ax[1], h = ax[2], s = a.start, e = a.end, c = a.center;
  if (!_arcNum(s[u]) || !_arcNum(s[v]) || !_arcNum(e[u]) || !_arcNum(e[v])) return "arc";
  function same(k, p, q) { return String(word(k, p)) === String(word(k, q)); }
  if (!same(u, s[u], e[u]) || !same(v, s[v], e[v])) return "arc";
  // The written ends coincide. With no centre the record is not an arc
  // this rule can read: the script's own code refuses it by name.
  if (!_arcNum(c[u]) || !_arcNum(c[v])) return "arc";
  // A centre that stands off the start as written makes the block a
  // full circle: right for a full turn. A full turn is a record whose
  // ends are equal in the stream (the one-record form the reader takes,
  // and the form the writer gives a near-full turn on its lattice), or
  // whose sweep from its direction and centre is more than half a turn.
  // Never ends that are only close: a true short arc under any band is
  // still short.
  var round = !(same(u, c[u], s[u]) && same(v, c[v], s[v]));
  if (round) {
    var full = e[u] === s[u] && e[v] === s[v];
    if (!full) {
      var a0 = Math.atan2(s[v] - c[v], s[u] - c[u]), a1 = Math.atan2(e[v] - c[v], e[u] - c[u]);
      var sweep = a.ccw ? a1 - a0 : a0 - a1;
      if (sweep < 0) sweep += 2 * Math.PI;
      full = sweep > Math.PI;
    }
    if (full) return "arc";
  }
  // A short arc under the output resolution: no arc block.
  if (_arcNum(e[h]) && !(_arcNum(s[h]) && same(h, s[h], e[h]))) return "linear";
  if (a.rotary) return "linear";
  return "none";
}
function arcFate(word) { return arcFateOf(arcShapeOf(properties), word); }
"#;
