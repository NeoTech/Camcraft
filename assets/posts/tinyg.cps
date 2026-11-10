// Synthetos TinyG 3-axis milling post. Provenance: the G-code and M-code
// tables of TinyG's public wiki ("Gcode Support", "Homing and Limits",
// github.com/synthetos/TinyG/wiki - behavior only; the firmware's own
// source is GPL and was not read), the public LinuxCNC G-code reference
// (linuxcnc.org/docs, the dialect TinyG states it follows) and RS-274 /
// ISO 6983 for the words they share - per this repo's Clean-Room
// Protocol. Checked by replaying posted programs through the edge-next
// firmware's own parser and planner (build 450) off the board. The hook
// contract from Autodesk's published post docs. No vendor or CAM post
// script text and no other CAM's output used; original work. Structure
// follows our own grblhal.cps (the other grbl-family dialect); the
// goldens lock this file's decisions.
description = "Synthetos TinyG (3-axis mill, G20/G21 from the machine, G2/G3, G4 dwell; drilling as plain moves - no canned cycles, no G41/G42, no G84, no probing macros; tools T1-T31, M0 after every tool change)";
vendor = "Camcraft";
legal = "Original clean-room work; public documentation only";
certificationLevel = 1;
minimumRevision = 1;
extension = "gcode";
setCodePage = "utf-8";
capabilities = 1; // reference contract: integer feature mask (bit 1 = milling)
coolantModes = true; // reads PARAM Coolant (#112/#150)
afterModes = true;   // reads PARAM AfterOperation (#110)

// Determinism law: no timestamps, no randomness - the same stream
// always posts to the same text (golden files depend on it).

// ---------------------------------------------------------------- state
var wcsWord = "G54";       // from the WorkOffset PARAM
var sRpm = 0;              // spindle/T order follows the stream: PARAMs,
var spindleWait = false;   // TOOL, SECTION - the preamble writes them
var toolWait = false;      // when the first motion needs them legal
var tNo = 0;
var coolant = "";          // the M-code this section started, if any
var coolantAsk = "flood";  // the section's PARAM Coolant (#112), flood when unstated
var afterAsk = "";         // the section's PARAM AfterOperation (#110), none when unstated
var coolantOp = "";        // the section's OperationName, for refusals (#150)
var sectionOpen = false;   // preamble written (a legacy .cl carries no SECTION)
var orgX = 0, orgY = 0, orgZ = 0; // the WCS slot origin (OriginX/Y/Z PARAMs)
var inch = false;          // the machine is programmed in inches (onOpen)

// ------------------------------------------------------------ helpers
// The stream speaks mm and mm/min. Lengths are written at four decimals
// in either unit: TinyG checks that an arc's start and end sit at the
// same radius from the I J K centre, and a tenth of a micron keeps a
// rounded centre inside that check where three decimals would not
// always. An inch machine gets every length and feed divided by 25.4.
// Angles and rpm never scale.
function has(v) { return v !== null && v !== undefined && isFinite(v); }
function trim(s) {
  if (s.indexOf(".") >= 0) s = s.replace(/0+$/, "").replace(/\.$/, "");
  return s === "-0" ? "0" : s;
}
function len(v) { return inch ? v / 25.4 : v; }
function fu(v) {  // a length, in the machine's unit
  if (!has(v)) return "0";
  return trim(Number(len(v)).toFixed(4));
}
function ff(v) {  // a feed: whole mm/min, or in/min at two decimals
  if (!has(v) || v <= 0) return "0";
  return inch ? trim((v / 25.4).toFixed(2)) : String(Math.round(v));
}
function ni(v) { return has(v) ? String(Math.round(v)) : "0"; } // integer word
function w(letter, v) { return has(v) ? letter + fu(v) : ""; }
function fw(v) { return has(v) ? "F" + ff(v) : ""; }
// TinyG's serial receive ring holds 250 bytes, 248 of them usable (wiki,
// "Flow Control and Footers"), and a sender pacing on the board's RTS
// marks has a 192-byte window: a line is kept to 191 content bytes so it
// always arrives whole, and a longer one is refused here by name, never
// cut by the board.
var LINE_CAP = 191;
function block(words) {  // join the non-empty words onto one line
  var a = [];
  for (var i = 0; i < words.length; i++) {
    if (words[i] !== "" && words[i] !== null && words[i] !== undefined) a.push(words[i]);
  }
  if (a.length === 0) return;
  var line = a.join(" ");
  if (line.length > LINE_CAP) { msg("a program line of " + line.length + " bytes is over TinyG's " + LINE_CAP + "-byte line limit"); return; }
  writeln(line);
}
function xyz(p) { return [wo("X", p.X, orgX), wo("Y", p.Y, orgY), wo("Z", p.Z, orgZ)]; }
function wo(letter, v, o) { return has(v) ? letter + fu(v - o) : ""; }  // axis word, origin-relative
function msg(s) {  // counted warning where the host offers message()
  if (typeof message === "function") message(s); else writeln("(" + s + ")");
}
var tapRefused = false;   // the tapping refusal, once per program
var tiltRefused = false;  // one named refusal per program, not per line
function tiltOk(p) {  // this post is 3-axis: a tilted record is refused
  if (!has(p.A)) return true;
  if (!tiltRefused) { tiltRefused = true; msg("tilted move - TinyG post has no RTCP output"); }
  return false;
}

// ------------------------------------------------------------ coolant
// The .mch tool-station coolant list carries names + M-codes. A flood
// ask takes the machine's Flood entry; failing one, the first coded
// entry of no other known family; a stated codeless Flood is the
// operator's own valve, silent. A list that states only other kinds has
// no flood code: refused by name (#150). An unstated list stays silent
// (the unstated-limit law). TinyG lists M7 / M8 / M9 and notes that
// mist and flood share one output pin - which code a build wires up is
// the machine file's own statement.
var COOLANT_KINDS = ["flood", "mist", "through-spindle", "through-tool"];
function coolantWho() { return coolantOp ? "operation " + coolantOp + ": " : ""; }
function coolantName(c) {
  return String((typeof c === "string") ? c : (c.name || "")).trim().toLowerCase().replace(/[\s_]+/g, "-");
}
function coolantCode() {
  var cs = (typeof machine !== "undefined" && machine.coolants) || [];
  var other = "", floodNamed = false;
  for (var i = 0; i < cs.length; i++) {
    var c = cs[i];
    var code = (typeof c === "string") ? "" : (c.mCode || "");
    var name = coolantName(c);
    if (name === "flood") { if (code) return code; floodNamed = true; continue; }
    if (code && !other && COOLANT_KINDS.indexOf(name) < 0) other = code;
  }
  if (other || floodNamed) return other;
  if (cs.length > 0) msg(coolantWho() + "the machine states no flood coolant code - pick another coolant for the operation (Off cuts dry) or add a Flood entry to the machine");
  return "";
}
// The op's coolant ask (#112): flood when unstated, or off / mist /
// through-tool - the machine's own entry of that family, refused by
// name (naming the operation) when the machine states none.
function coolantFor(ask) {
  if (ask === "off") return "";
  if (!ask || ask === "flood") return coolantCode();
  var want = ask === "mist" ? ["mist"] : (ask === "through-tool" ? ["through-spindle", "through-tool"] : []);
  if (want.length === 0) { msg(coolantWho() + "unknown coolant ask " + ask); return ""; }
  var cs = (typeof machine !== "undefined" && machine.coolants) || [];
  for (var i = 0; i < cs.length; i++) {
    var c = cs[i];
    var code = (typeof c === "string") ? "" : (c.mCode || "");
    if (code && want.indexOf(coolantName(c)) >= 0) return code;
  }
  msg(coolantWho() + "the machine states no " + ask + " coolant code - pick another coolant for the operation or add it to the machine");
  return "";
}

// ------------------------------------------------------------ the plane
// The arc's plane (G17 XY, G18 XZ, G19 YZ - all three in TinyG's table),
// stated modally: written only on change and taken back to G17 at
// section end.
var planeG = 17;
function planeOf(p) {
  return p.CircularPlane === "XZ" ? 18 : (p.CircularPlane === "YZ" ? 19 : 17);
}
function planeWord(g) {  // "" while modal; the word (and the new state) on change
  if (g === planeG) return "";
  planeG = g;
  return "G" + g;
}
function restorePlane() { var pw = planeWord(17); if (pw) block([pw]); }
// The center words of the arc's plane as increments from the start
// point (G91.1, stated in the header): a shared translation cancels in
// an increment, so the origin must NOT enter them.
function centerWords(p) {
  var c = p.IJKIncrement || {};
  return [
    w("I", has(c.I) && has(p.StartX) ? c.I - p.StartX : null),
    w("J", has(c.J) && has(p.StartY) ? c.J - p.StartY : null),
    w("K", has(c.K) && has(p.StartZ) ? c.K - p.StartZ : null),
  ];
}

// ------------------------------------------------------------ comments
// Program commentary (#257): the setup, the operation and the tool, for
// the operator reading the program - never a word the control moves on.
// User text is cleaned for this dialect: a parenthesis would end the
// comment early and ; opens a comment, so both become spaces; so do the
// characters TinyG's serial input acts on wherever they stand in a line,
// comment included - ! is a feed hold, ~ a cycle start, % a queue flush,
// / a block delete at the line's start, and a line opening with { is a
// JSON command and with $ a setting; printable ASCII only, capped at 40
// characters.
var setupName = "", setupSaid = "", opName = "", opKind = "";
var toolName = "", toolKind = "", toolDia = 0, toolCr = 0;
function clean(s) {
  var t = String(s === null || s === undefined ? "" : s);
  t = t.replace(/[()%;!~\/{}$]/g, " ").replace(/[^\x20-\x7E]/g, "?").replace(/\s+/g, " ").trim();
  return t.length > 40 ? t.substring(0, 40).trim() : t;
}
function joinDash(parts) {  // the non-empty parts, " - " between
  var a = [];
  for (var i = 0; i < parts.length; i++) if (parts[i]) a.push(parts[i]);
  return a.join(" - ");
}
// A comment opening with `msg` is echoed to the operator by TinyG as a
// message (wiki, "Gcode Comments"); a note that would read as one is
// led by "- ", so user text stays a note.
var ACTIVE = /^\s*msg/i;
function note(s) { block(["(" + (ACTIVE.test(s) ? "- " + s : s) + ")"]); }
function commentParam(name, value) {
  if (name === "SetupName") setupName = clean(value);
  else if (name === "OperationName") opName = clean(value);
  else if (name === "OperationKind") opKind = clean(value).toUpperCase();
  else if (name === "ToolKind") toolKind = clean(value).toUpperCase();
  else if (name === "ToolDiameter" && has(value)) toolDia = Number(value);
  else if (name === "ToolCornerRadius" && has(value)) toolCr = Number(value);
}
function opNotes() {  // at the section's preamble: the setup once, the op each time
  if (setupName && setupName !== setupSaid) { note("SETUP: " + setupName); setupSaid = setupName; }
  if (opName) note("OPERATION: " + joinDash([opName, opKind, wcsWord]));
  opName = ""; opKind = "";
}
function toolNote() {  // just before the tool change it describes
  var desc = [];
  if (toolKind) desc.push(toolKind);
  if (toolDia > 0) desc.push("D" + fu(toolDia));
  if (toolCr > 0) desc.push("R" + fu(toolCr));
  note(joinDash(["T" + ni(tNo) + (toolName ? " " + toolName : ""), desc.join(" ")]));
  toolKind = ""; toolCr = 0;
}

// -------------------------------------------------------------- header
// One identity comment, then the modal state every later line relies
// on - the words a program end does NOT reset on TinyG (wiki, "M2, M30
// Program End"), so a previous program's choice would otherwise stand:
// the unit (G20/G21 stays as it was), the plane, absolute positions,
// incremental arc centres (G91.1 is the default, but a G90.1 persists),
// feed per minute, and G64 continuous path (an exact-stop G61.1 persists
// too, and would stop the machine at every block of a contour). No O
// number, no % fence (a line starting with % is a queue flush), no N
// numbers.
//
// No tool length word: TinyG has G43.1 / G49 (build 450), but the offset
// belongs to the operator or sender that measures it at the M0 below -
// a G49 here would throw a measured length away, and the post knows no
// length to restate. No G40 either: TinyG accepts it only to ignore it.
function onOpen() {
  inch = typeof machine !== "undefined" && machine.units === "inch";
  var nm = (typeof machine !== "undefined" && machine.name) || "CAMCRAFT";
  note(clean(nm) || "CAMCRAFT");
  block([inch ? "G20" : "G21", "G17", "G90", "G91.1", "G94", "G64"]);
}

// #495 / #710: where "straight up" is. TinyG's machine coordinates are
// set by homing (G28.2) or by G28.3; an unhomed board has its machine
// zero wherever it was switched on (`$home` reads 0), so `G53 G0 Z0`
// could rapid down into the part. The same law as grblhal.cps:
// - a machine file stating its Z part's `safe_retract`
//   (`machine.safeRetractZ`, mm) says it is homed and where up is: the
//   lift is `G53 G0 Z<that>`, in machine coordinates;
// - otherwise (no machine, or one stating none) the lift stays in the
//   work offset: `G0 Z<h>`, h the highest Z the program has proven clear -
//   the section's ClearancePlane, or any higher Z it has already
//   commanded. A height that is unknown, or not above the stock top, is
//   refused by name - never a guess about which way is up.
var liftZ = null;          // world mm: the highest proven-clear Z so far
var stockTop = null;       // world mm: the stock's top, when stated
function seeZ(z) { if (has(z) && (liftZ === null || z > liftZ)) liftZ = Number(z); }
function machineSafeZ() {
  var m = typeof machine !== "undefined" ? machine : null;
  return m && has(m.safeRetractZ) ? Number(m.safeRetractZ) : null;
}
var liftRefused = false;   // one named refusal per program
function liftRefuse(why) {
  if (liftRefused) return;
  liftRefused = true;
  msg(why + " - this post lifts Z in the work offset before each tool change and at the end, because an unhomed TinyG has its machine zero at the power-on position; set the operation's clearance height above the stock, or state the Z axis's safe_retract (machine coordinates) in the machine file if the machine homes");
}

// No G28 / G30: their positions are set at the machine with G28.1 /
// G30.1, are not kept across a power cycle, and read machine zero when
// unset - the post cannot see them.
function retract() {
  var mz = machineSafeZ();
  if (mz !== null) { block(["G53", "G0", "Z" + fu(mz)]); return; }
  if (liftZ === null) { liftRefuse("no clearance height is known for the Z lift"); return; }
  if (stockTop !== null && !(liftZ > stockTop)) {
    liftRefuse("the clearance height Z" + fu(liftZ - orgZ) + " is not above the stock top Z" + fu(stockTop - orgZ));
    return;
  }
  block(["G0", wo("Z", liftZ, orgZ)]);
}

// TinyG rejects G41 and G42 (wiki, "Cutter Radius Compensation (not
// supported)"): a Control (or Wear) op is refused by name.
function onRadiusCompensation() {
  msg("control cutter compensation (G41/G42) is not supported by TinyG - choose In computer");
}

function onParameter(name, value) {
  commentParam(name, value);
  if (name === "OperationName") coolantOp = String(value || "");
  if (name === "Coolant") { coolantAsk = String(value || "flood"); return; }
  if (name === "AfterOperation") { afterAsk = String(value || ""); return; }
  if (name === "WorkOffset" && value) wcsWord = String(value);
  else if (name === "OriginX" && has(value)) orgX = Number(value);
  else if (name === "OriginY" && has(value)) orgY = Number(value);
  else if (name === "OriginZ" && has(value)) orgZ = Number(value);
  else if (name === "ClearancePlane") seeZ(value);
  else if (name === "StockZTop" && has(value)) stockTop = Number(value);
}

function onSpindleSpeed() {
  sRpm = Number(properties.SpindleSpeed) || 0;
  spindleWait = true;
  if (typeof machine !== "undefined" && machine.spindleMax > 0 && sRpm > machine.spindleMax) {
    msg("spindle rpm over machine maximum");
  }
}

// TinyG keeps the tool number in five bits: T1 to T31 can be selected,
// and a larger number is refused by the board with a status the sender
// may not stop on - so it is refused here by name, never posted.
function onToolChange(number, name) {
  toolName = clean(name);
  tNo = number;
  toolWait = true;
  if (!has(number) || number < 1 || number > 31) {
    msg("tool number " + ni(number) + " is outside TinyG's tool range (1-31)");
  }
}

// The preamble closes over everything a raw motion line needs to be a
// legal program: plane, offset, tool, spindle, coolant. Before a tool
// change Z goes straight up; then `T<n> M6` on a line of its own, and
// `M0` on the next: on TinyG, M6 only makes the T word the current tool
// and moves nothing (wiki, "M6"), so the halt the operator needs to
// change and measure the tool is the program stop - queued with the
// moves, so everything before it has finished when the machine holds,
// and released by the sender's cycle start. The spindle is stopped
// before it by the previous section's M5 (or has not started yet).
function startPreamble() {
  if (sectionOpen) return;
  sectionOpen = true;
  opNotes();
  block(["G17", "G90", wcsWord]);
  planeG = 17;              // the preamble states the plane
  if (toolWait) {
    toolNote();
    retract();
    xyUnknown = true;        // the operator may jog X and Y at the stop
    block(["T" + ni(tNo), "M6"]);
    block(["M0"]);
    toolWait = false;
  }
  // A touch probe states 0 rpm: no spindle start and no cutting fluid.
  var touchProbe = spindleWait && !(sRpm > 0);
  if (spindleWait && sRpm > 0) block(["S" + ni(sRpm), "M3"]);
  spindleWait = false;
  coolant = touchProbe ? "" : coolantFor(coolantAsk);
  if (coolant) block([coolant]);
}

// The op's after-action (#110), written once its spindle has stopped:
// M1 is TinyG's optional stop (it holds only when the board's optional
// stop setting is on), M0 its program stop. A trip home is refused by
// name: G28 / G30 go to positions set at the machine (machine zero when
// unset), and homing itself (G28.2) is not a program's business.
function afterOp() {
  var ask = afterAsk;
  afterAsk = "";
  if (!ask) return;
  if (ask === "optional-stop") { block(["M1"]); return; }
  if (ask === "stop") { block(["M0"]); return; }
  if (ask === "home") {
    msg(coolantWho() + "going home after an operation is not posted by the TinyG post - the G28/G30 positions are set at the machine and it cannot see them; pick Optional stop or Program stop");
    return;
  }
  msg(coolantWho() + "unknown after-operation ask " + ask);
}

function endPreamble() {
  if (!sectionOpen) return;
  owedDwell();
  restorePlane();            // the plane never leaks past its section
  sectionOpen = false;
  if (coolant) block(["M9"]);
  coolant = "";
  coolantAsk = "flood";      // the next section states its own ask
  block(["M5"]);
  afterOp();
}

function onSection() { startPreamble(); }
function onSectionEnd() { endPreamble(); }

// ------------------------------------------------------------- motions
// X and Y are unknown at the program's start and after every tool change
// (the operator may jog the spindle anywhere at the M0): the first move
// from there is preceded by a G0 to its X Y alone, at the current -
// safe, Z-up - height, and only then does Z come down. A rapid positions
// at its target; a feed or arc at the point the stream says it starts
// from (the target when the stream names none).
var xyUnknown = true;
function xyFirst(x, y) {
  if (!xyUnknown || !has(x) || !has(y)) return;
  block(["G0", wo("X", x, orgX), wo("Y", y, orgY)]);
  xyUnknown = false;
}
// A feed or arc as the first move after a tool change starts, in the
// stream, from the generator's own start point (its clearance height).
// The tool stands at the lift height, so feeding from there would crawl
// the whole way down: once X Y are positioned, a rapid takes Z down to
// that start height and the feed runs only the stretch it planned.
function startXY(p) {
  if (!xyUnknown) return;
  if (has(p.StartX) && has(p.StartY)) xyFirst(p.StartX, p.StartY);
  else xyFirst(p.X, p.Y);
  if (!xyUnknown && has(p.StartZ) && has(p.Z) && p.StartZ > p.Z) {
    block(["G0", wo("Z", p.StartZ, orgZ)]);
  }
}

function onRapid() {
  startPreamble();
  seeZ(properties.Z);
  if (!tiltOk(properties)) return;
  xyFirst(properties.X, properties.Y);
  block(["G0"].concat(xyz(properties)));
}
function onLinear() {
  startPreamble();
  seeZ(properties.Z); seeZ(properties.StartZ);
  if (!tiltOk(properties)) return;
  startXY(properties);
  block(["G1"].concat(xyz(properties), [fw(properties.FeedRate)]));
  dwellAtBottom(properties.Z);
}
function onCircular() {
  startPreamble();
  if (!tiltOk(properties)) return;
  var p = properties;
  seeZ(p.Z); seeZ(p.StartZ);
  startXY(p);
  block([
    planeWord(planeOf(p)),
    p.CircularDirection === "CW" ? "G2" : "G3",
    wo("X", p.X, orgX), wo("Y", p.Y, orgY), wo("Z", p.Z, orgZ),
  ].concat(centerWords(p)).concat([fw(p.FeedRate)]));
}

// ------------------------------------------------------- drilling
// TinyG has no canned cycles (G80 is the only word of that group it
// lists; G81-G89 are rejected like any other unlisted G code), so a
// CYCLE record writes nothing and the hole's expanded chain posts as the
// plain moves it is - the generator's own rapid onto the retract plane,
// plunges, peck retracts and exit, the exact motion the simulation
// verified. A dwell becomes `G4 P<seconds>` (wiki, "G4 dwell duration":
// P is seconds, 0 to 4294) right after the move that reaches the hole's
// bottom. Tapping cannot be plain moves (the feed out of a tap needs the
// spindle reversed in step with it, and TinyG has no G84 and no spindle
// synchronisation), so TAP is refused by name.
var DWELL_TOL = 1e-6;
var pendingDwell = null; // {z, s}: the open hole's dwell, owed at its bottom
function onCycle() {
  startPreamble();
  owedDwell();
  var p = properties;
  seeZ(p.R);
  if (p.CycleType === "TAP") {
    if (!tapRefused) {
      tapRefused = true;
      msg("tapping (G84) is not in TinyG's G-code set and cannot be plain moves - refused, not posted; use a tapping head with a plain drill cycle, or thread milling");
    }
    return;
  }
  if (has(p.P) && p.P > 0) {
    if (p.P > 4294) { msg("a drilling dwell of " + trim(Number(p.P).toFixed(3)) + " s is over TinyG's 4294 s dwell limit"); return; }
    pendingDwell = { z: Number(p.Z), s: Number(p.P) };
  }
}
function dwellAtBottom(z) {
  if (pendingDwell === null || !has(z) || Math.abs(z - pendingDwell.z) > DWELL_TOL) return;
  block(["G4", "P" + trim(Number(pendingDwell.s).toFixed(3))]);
  pendingDwell = null;
}
// A dwell whose chain never reached its bottom is refused, never lost.
function owedDwell() {
  if (pendingDwell === null) return;
  pendingDwell = null;
  msg("drilling dwell without a feed move to the hole bottom - refused, not dropped");
}

// Probing: TinyG has G38.2, but no parameters, expressions or
// conditionals (a # in a line is stripped before parsing), so the
// result test and the offset arithmetic a probe cycle needs cannot be
// written. This post defines no probe hook and the engine refuses a
// probing operation by name.

// ------------------------------------------------------------ footer
// Z straight up, then M30 (no % fence). TinyG's M30 stops the spindle
// and coolant and resets the plane, G90 and G94, but not the unit, so
// the user's post option `EndUnits` ("inch" / "mm"; absent = leave the
// program's unit) names the unit the control should be left in, written
// after the last move, right before M30.
function endUnitsWord() {
  var want = typeof postOptions !== "undefined" && postOptions ? postOptions.EndUnits : undefined;
  if (want === "inch" && !inch) return "G20";
  if (want === "mm" && inch) return "G21";
  return "";
}
function onClose() {
  endPreamble();
  retract();
  var u = endUnitsWord();
  if (u) block([u]);
  block(["M30"]);
}
