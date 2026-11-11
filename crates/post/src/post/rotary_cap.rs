//! The rotary feed cap (issue #885): one rule, in the prelude every post
//! script sees, for a feed block that turns a rotary axis while (or
//! instead of) moving the linear ones.
//!
//! Provenance: original; the feed readings are the controls' published
//! programming documentation (behaviour only, per the Clean-Room
//! Protocol) - a Fanuc-family control without tool centre point control
//! times a combined block over one composite length that counts degrees
//! as millimetres (FANUC 0i-F / 30i-B operator's manuals, "feedrate for
//! rotation axes" in feed per minute); under tool centre point control
//! (G43.4) F is the tool tip's speed relative to the work, the rotaries
//! carried along over the same time (the same manuals, "tool center
//! point control"); LinuxCNC's G94 reads F along X Y Z when any of them
//! moves, the rotaries carried along (LinuxCNC G-code reference, "F:
//! Set Feed Rate"). The rates are the machine file's own (`speeds`
//! under each rotary, deg/min - `machine::script_props`); an unstated
//! rate caps nothing, the unstated-limit law every post keeps.

/// Appended to [`super::js::PRELUDE`]: plain JS, no host calls.
pub const ROTARY_CAP: &str = r#"
// The stated rate of rotary `letter` in deg/min: its feed speed, failing
// that its rapid; 0 when the machine states neither (or no such rotary).
function rotaryRate(letter) {
  var rs = (typeof machine !== "undefined" && machine && machine.rotaries) || [];
  var L = String(letter).toUpperCase();
  for (var i = 0; i < rs.length; i++) {
    if (String(rs[i].name || "").toUpperCase() !== L) continue;
    var sp = rs[i].speeds || {};
    if (Number(sp.normal) > 0) return Number(sp.normal);
    if (Number(sp.rapid) > 0) return Number(sp.rapid);
    return 0;
  }
  return 0;
}
// The largest F (never above `f`) under which no axis of the block runs
// past what the machine states. `lin` is the linear length (mm), `turns`
// a list of [degrees turned, stated deg/min (0 = unstated)].
// mode "composite": the control times the block over sqrt(lin^2 + sum
//   d^2), degrees counted as mm - the linear axes run at F * lin / lcmd,
//   each rotary at F * |d| / lcmd.
// mode "path": F is the speed along the linear length (a tool tip under
//   TCP, an arc's chord, LinuxCNC's X Y Z) and the rotaries are carried
//   along over the same time - each at F * |d| / lin. With nothing linear
//   moving, F is read as the rotary rate itself (each rotary at most F).
// `eps` (optional, mm) is the linear length below which nothing linear
// is judged to move; default a nanometre.
function rotaryFeedCap(f, lin, turns, mode, eps) {
  var g = Number(f);
  if (!(g > 0)) return g;
  var e = (eps > 0) ? eps : 1e-9;
  var L = (Number(lin) > 0) ? Number(lin) : 0;
  var fmax = (typeof machine !== "undefined" && machine) ? Number(machine.feedRateMax) : 0;
  var sum = 0, i, d, r;
  for (i = 0; i < turns.length; i++) { d = Number(turns[i][0]) || 0; sum += d * d; }
  if (mode === "composite") {
    var lcmd = Math.sqrt(L * L + sum);
    if (!(lcmd > 0)) return g;
    if (fmax > 0 && L > e) g = Math.min(g, fmax * lcmd / L);
    for (i = 0; i < turns.length; i++) {
      d = Math.abs(Number(turns[i][0]) || 0); r = Number(turns[i][1]);
      if (r > 0 && d > 1e-9) g = Math.min(g, r * lcmd / d);
    }
    return g;
  }
  if (L > e) {
    if (fmax > 0) g = Math.min(g, fmax);
    for (i = 0; i < turns.length; i++) {
      d = Math.abs(Number(turns[i][0]) || 0); r = Number(turns[i][1]);
      if (r > 0 && d > 1e-9) g = Math.min(g, r * L / d);
    }
    return g;
  }
  for (i = 0; i < turns.length; i++) {
    d = Math.abs(Number(turns[i][0]) || 0); r = Number(turns[i][1]);
    if (r > 0 && d > 1e-9) g = Math.min(g, r);
  }
  return g;
}
// A mill feed record's capped F, for a post that writes the letters in
// `letters` from the angles in `prev` ({A, B, C}; a null angle is not
// known to the post). `arc` judges the chord in the record's plane - at
// most the length any reading of an arc's F is measured along, so the
// cap is never too loose. Returns { feed, capped } or { refuse: text }.
function rotaryFeed(p, prev, letters, mode, arc) {
  var f = Number(p.FeedRate);
  var turns = [], i, L, rate, d;
  for (i = 0; i < letters.length; i++) {
    L = letters[i];
    if (p[L] === null || p[L] === undefined || !isFinite(p[L])) continue;
    rate = rotaryRate(L);
    var was = prev[L];
    if (was === null || was === undefined || !isFinite(was)) {
      if (rate > 0) return { refuse: "a feed that turns " + L + " from an angle the post does not know - the stated " + L + " rate cannot be held" };
      continue;
    }
    d = Number(p[L]) - Number(was);
    if (Math.abs(d) > 1e-9) turns.push([d, rate]);
  }
  if (turns.length === 0 || !(f > 0)) return { feed: f, capped: false };
  var sx = p.StartX, sy = p.StartY, sz = p.StartZ;
  var known = function (v) { return v !== null && v !== undefined && isFinite(v); };
  if (!known(sx) || !known(sy) || !known(sz) || !known(p.X) || !known(p.Y) || !known(p.Z)) {
    var any = false;
    for (i = 0; i < turns.length; i++) if (turns[i][1] > 0) any = true;
    if (any) return { refuse: "a feed that turns a rotary from an unstated start point - the stated rotary rate cannot be held" };
    return { feed: f, capped: false };
  }
  var dx = p.X - sx, dy = p.Y - sy, dz = p.Z - sz, lin;
  if (arc) {
    var pl = p.CircularPlane;
    lin = pl === "XZ" ? Math.sqrt(dx * dx + dz * dz) : (pl === "YZ" ? Math.sqrt(dy * dy + dz * dz) : Math.sqrt(dx * dx + dy * dy));
  } else {
    lin = Math.sqrt(dx * dx + dy * dy + dz * dz);
  }
  var g = rotaryFeedCap(f, lin, turns, arc ? "path" : mode);
  return { feed: g, capped: g < f - 1e-9 };
}
// A capped F as a word: floored to 0.1 mm/min (rounding up would pass
// the cap), trailing zeros trimmed; "" when the cap leaves under 0.1.
function rotaryFeedWord(v) {
  var t = Math.floor(Number(v) * 10 + 1e-9) / 10;
  if (!(t >= 0.1)) return "";
  var s = t.toFixed(1).replace(/0+$/, "").replace(/[.]$/, "");
  return "F" + s;
}
"#;
