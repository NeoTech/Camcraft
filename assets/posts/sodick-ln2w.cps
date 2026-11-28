// Sodick wire EDM post, LN2W control dialect (#4045). Provenance: Sodick
// Inc., "Wire EDM Machine Operation Training, LN1W / LN2W Control" (AQ400L,
// AQ600L, AD360L; part number 6300015, January 2012; public at
// fab.cba.mit.edu/content/tools/sodick_edm/sodick_user_manual_small.pdf) as
// this repository recorded it (#4045, #1500, and the #1496 / #2649 / #2686
// decisions): T84 / T85 high / low flushing, T90 / T91 cut / thread the
// wire, M02 ends (no M30); and the statements of the owner of a Sodick
// AQ600L with the LN2W control (2026-11-25, on #4045 and #1500, with his
// round-2 correction on PR #4065): the start sequence below, Z0 is the
// upper head height that the operator sets, the control alarms on G21
// (E00170); and RS-274 / ISO 6983 for the motion words. No other CAM product's post,
// post library or output was read. The Mark 21 post (sodick-wire.cps) is a
// separate file and is not changed: the engine has no include, so the
// shared helpers are copied here. NOT VERIFIED ON A CONTROL in this form:
// check the output at the machine before cutting. What is confirmed and
// what is inferred is in docs/decisions/4045-sodick-ln2w-dialect.md.
//
// Written, in the owner's order for each cut: the setup's G54-G59,
// G00 X Y U0. V0. (the rapid to the start point, the taper axes upright
// on the same line), Z0., T91 (thread; an M00 without automatic
// threading), T84 / T85 and T94 / T89 when the machine file states
// edm.flushing / edm.tank, the C condition word, then G01, G02 / G03 with
// I J; each skim pass's flushing word (edm.skim_flushing) and C word
// before its first move; T90 (cut the wire); M00 / M01; M02.
// The control's wire offset (#4076, the same owner's statements of
// 2026-11-25), only when the stream asks for it (a COMP record): G41 / G42
// Hnnn on its own line before the straight lead-in, H the pass number
// from H001 (C001 always goes with H001; a later pass takes the condition
// that the operation states), G40 on the straight lead-out move. The law
// (a straight lead-in and a straight lead-out, each at least the radius
// long) is the shared wire_offset.rs prelude helper. The header and the
// section notes read programOffsets (the engine, before onOpen), so they
// say which sections use the offset. Without a COMP record the path is
// the wire-centre path with the kerf in it, and the program is as before.
// A constant taper (#4075, the same owner's statements of 2026-11-25, in
// his 4-axis form): after the C word, TP (the cut's bottom above the
// worktable) and TN (the cut's top above the worktable), G74, then each
// move as G01 X Y U V - X Y on the cut's bottom, U V the cut's top minus
// its bottom - and G75 after the last cut. The post option
// partBottomAboveTable is the TP of the program's first tapered cut: it
// ties that cut's bottom (a work Z) to the worktable, and each later
// tapered cut gets TP = the option + (its bottom - the first bottom), so
// TP always names the plane that X Y are on. Each section note names
// that work Z. The stream states U V at the guide planes; the post moves
// them to the part's planes along the same straight wire, so CamCraft
// decides each corner and nothing is left to the control.
// Never written (each a named refusal or an operator's task): G20 / G21
// (the unit is the control's setting; this post writes millimetres only),
// G92 and G29, a 4-axis cut with two profiles (WireFourAxis), an arc in a
// tapered cut, the angle form (G51 / G52 A, G50: the control's corner
// motion and lean side are not confirmed, #4075), the tank fill / drain,
// pump and wire-drive T codes, M03, M30.
description = "Sodick wire EDM, LN2W control (AQ400L / AQ600L; vertical cuts and constant tapers as 4-axis G74; start sequence from an owner's statements; not verified on a control)";
vendor = "Camcraft";
legal = "Original clean-room work; public documentation and a machine owner's statements only";
certificationLevel = 1;
minimumRevision = 1;
extension = "nc";
setCodePage = "utf-8";
capabilities = 1;          // reference contract: integer feature mask
wireEdm = true;            // voices PARAM EdmCondition (the engine's door)
afterModes = true;         // reads PARAM AfterOperation (M01 / M00; home refused)
programUnits = "mm";       // #4045: the control alarms on G21, so no unit word can be written - the engine refuses an inch document by name

// The LN manual's header names C000 the "Approach" condition (#2649). Off
// by default, as in the Mark 21 post.
declaredOptions = {
  approachCondition: {
    type: "boolean",
    title: "C000 approach condition",
    group: "Conditions",
    description: "Cut the approach move (start point to the contour) on condition C000, then select the operation's condition. C000 must exist in the control's condition table",
    default: false,
  },
  // #4075: TP is a height above the worktable, and CamCraft knows no
  // fixture. The user states it; unstated (-1), each taper is refused.
  partBottomAboveTable: {
    type: "number",
    title: "Part bottom above the worktable (mm)",
    group: "Taper",
    description: "TP of the program's first tapered cut: the height of that cut's bottom above the worktable (the fixture height when the cut starts at the part's bottom, often 0). A later tapered cut with a different bottom gets TP moved by the difference, and TN is TP plus the cut's height. Each section note names the work Z that TP stands for. -1 means not stated: each taper is refused. One value for the program: post the tapers of each setup as their own program",
    min: -1,
    max: 1000,
    default: -1,
  },
};

// Determinism law: no timestamps, no randomness.

// ---------------------------------------------------------------- state
var wcsWord = "";
var orgX = 0, orgY = 0;
var opName = "", opKind = "", setupName = "";
var condition = "", side = "", afterOp = "";
var inSection = false;     // between onSection and onSectionEnd
var sectionOk = false;     // the section is one this post can write
var threaded = false;      // the wire is threaded through the work
var compOff = false;       // the section's first cut has been written
var cutZ = null;           // the section's one programmed plane
var awt = false;
var tankWord = "";         // T94 / T89 from edm.tank ("": not stated)
var flushWord = "";        // T84 / T85 from edm.flushing ("": not stated)
var skimFlushWord = "";    // T84 / T85 from edm.skim_flushing ("": not stated)
var taper = 0;             // the section's stated taper, degrees (0: vertical)
var dialect = "";          // "angle": the stream carries angle words
var angleNext = false;     // the next move states an angle word (WireAngle)
var controlOffset = false; // a refused COMP record refuses this section, even before SECTION
var wireOff = wireOffsetNew(); // #4076: the control's wire offset (wire_offset.rs)
var sectionNo = 0;         // sections ended so far: the index into programOffsets
var fourAxis = "";         // the section's 4-axis pairing rule ("": one profile)
var leaning = false;       // the next move states an upper guide offset (WireU / WireV)
var leanU = null, leanV = null; // that offset, mm at the upper guide
// #4075: a taper's planes from the stream (work Z, mm): the guides that
// U V are stated at, and the part's bottom and top that TP / TN name.
var guideLo = null, guideHi = null, partBot = null, partTop = null;
var tableH = null;         // the first taper's bottom above the worktable (the option), mm
var fourOn = false;        // this section is a G74 taper
var lastU = 0, lastV = 0;  // the last written U V words' values
var taperSetup = null;     // the setup of the program's first taper (one table height)
var taperBase = null;      // that taper's WireBottom (work Z, mm): the plane the option names
// A wire number the stream states but that is no finite number (the
// wire-iso post's #3236 guard): refused, and no cut of the section is written.
var WIRE_NUMBERS = ["WireTaper", "WirePlane", "WireAngle", "WireU", "WireV", "WireGuideLower", "WireGuideUpper", "WireBottom", "WireTop"];
var notFinite = "";        // the name of such a param ("": none)
var machineGood = false;
var approach = false;      // the approachCondition option
var approachPending = false; // C000 is selected; the pass's C word follows the approach
var skim = 0;              // skim passes begun in this section
var skimPending = false;   // a skim's condition arrived; select it at its first move
var refused = {};

// ------------------------------------------------------------ helpers
function has(v) { return v !== null && v !== undefined && isFinite(v); }
// A number always carries its decimal point: the control reads a number
// without one in its smallest unit, so "X5" would be a different place.
function fu(v) {
  var s = Number(v).toFixed(3).replace(/0+$/, "");
  return (s === "-0." || s === "-.") ? "0." : s;
}
function wo(letter, v, o) { return has(v) ? letter + fu(v - o) : ""; }
function arcWord(k, v) { return k === "Z" ? "" : wo(k, v, k === "X" ? orgX : orgY); }  // #3442: a position word as written (no Z), for arcFate
function block(words) {
  var a = [];
  for (var i = 0; i < words.length; i++) {
    if (words[i] !== "" && words[i] !== null && words[i] !== undefined) a.push(words[i]);
  }
  if (a.length > 0) writeln(a.join(" "));
}
function msg(s) { if (typeof message === "function") message(s); else note(s); }
function refuse(key, s) {
  if (refused[key]) return;
  refused[key] = true;
  msg(s);
}
function who() { return opName ? "operation " + opName + ": " : ""; }
function clean(s) {
  var t = String(s === null || s === undefined ? "" : s);
  t = t.replace(/[()%;\/]/g, " ").replace(/[^\x20-\x7E]/g, "?").replace(/\s+/g, " ").trim();
  return t.length > 48 ? t.substring(0, 48).trim() : t;
}
// #3213: the control rejects hashes, brackets and escapes in every comment.
function note(s) { block(["(" + String(s).replace(/[#\[\]\\]/g, " ") + ")"]); }
// #3214: the posted condition's row in the machine file. The path's kerf
// offset used its gap; the program states that gap so the operator can
// compare it with the control's own row, and says so loudly when the row
// is an example (notes starting "EXAMPLE").
function conditionRow(id) {
  var e = typeof machine !== "undefined" && machine ? machine.edm : null;
  var list = e && e.conditions ? e.conditions : [];
  var key = String(id).replace(/\s+/g, "").toUpperCase();
  for (var i = 0; i < list.length; i++) {
    if (list[i] && String(list[i].id).replace(/\s+/g, "").toUpperCase() === key) return list[i];
  }
  return null;
}
function conditionNotes() {
  var c = conditionRow(condition);
  if (!c) return;
  var gap = has(c.gap) ? Number(c.gap).toFixed(3) + " MM" : "NOT STATED";
  // #4076: on the control's offset the path holds no kerf, so the row's
  // gap is not in it. The H register at the control sets the part size.
  if (offsetMode() === "control") {
    var eg = /^\s*EXAMPLE/i.test(String(c.notes || "")) ? "WARNING CONDITION " + condition + " IS AN EXAMPLE ROW - " : "CONDITION " + condition + " - ";
    note(eg + "THE MACHINE FILE GAP " + gap + " IS NOT IN THE PATH - THE H REGISTER AT THE CONTROL SETS THE KERF");
    return;
  }
  if (/^\s*EXAMPLE/i.test(String(c.notes || ""))) {
    note("WARNING CONDITION " + condition + " IS AN EXAMPLE ROW - THE KERF USES AN EXAMPLE GAP OF " + gap + " - REPLACE IT BEFORE CUTTING");
  } else {
    note("CONDITION " + condition + " KERF GAP " + gap + " - COMPARE WITH THE CONTROL ROW");
  }
}
// #4076: programOffsets (the engine, before onOpen) states for each
// section if a COMP record engages the control's offset in it:
// "control", "wear" or "". The notes come before the COMP record, so
// they read it here. offsetMode is the current section's entry.
function offsetList() { return typeof programOffsets !== "undefined" && programOffsets ? programOffsets : []; }
function offsetMode() { return offsetList()[sectionNo] || ""; }
function offsetProgram() {
  var l = offsetList();
  for (var i = 0; i < l.length; i++) if (l[i]) return true;
  return false;
}
// Program notes use this dialect without truncating the stated bound.
function onComment(s) { note(commentText(s)); }

// ------------------------------------------------------------- machine
function machineName() {
  var m = typeof machine !== "undefined" ? machine : null;
  return (m && (m.model || m.name)) || "no machine";
}
// The machine check, once per program: a wire EDM with no rotary.
function machineOk() {
  var m = typeof machine !== "undefined" ? machine : null;
  if (!m || m.kind !== "wire_edm") {
    refuse("kind", "the Sodick LN2W wire post writes wire words only a wire EDM reads - machine " + machineName() + " is not declared kind wire_edm");
    return false;
  }
  awt = !!(m.edm && m.edm.awt === true);
  if (!dielectricOk(m.edm || {})) return false;
  if (m.rotaries && m.rotaries.length > 0) {
    refuse("rotary", "machine " + machineName() + " states a rotary axis - the Sodick LN2W wire post writes no rotary word");
    return false;
  }
  return true;
}

// The machine file's tank and flushing, as the manual's T words (#1499,
// #2686). Unstated writes nothing (the operator sets it at the control);
// a word this post does not know is refused by name, never guessed.
function dielectricOk(e) {
  var ok = true;
  if (e.tank !== undefined && e.tank !== null) {
    if (e.tank === "submerged") tankWord = "T94";
    else if (e.tank === "non-submerged") tankWord = "T89";
    else {
      refuse("tank", "machine " + machineName() + ": edm.tank " + JSON.stringify(e.tank) + " is not submerged or non-submerged - the Sodick LN2W wire post writes T94 or T89 only");
      ok = false;
    }
  }
  if (e.flushing !== undefined && e.flushing !== null) {
    if (e.flushing === "high") flushWord = "T84";
    else if (e.flushing === "low") flushWord = "T85";
    else {
      refuse("flushing", "machine " + machineName() + ": edm.flushing " + JSON.stringify(e.flushing) + " is not high or low - the Sodick LN2W wire post writes T84 or T85 only");
      ok = false;
    }
  }
  // A T word is modal: a skim's flushing would stay active for the next
  // operation's rough cut if that cut selected none of its own.
  if (e.skim_flushing !== undefined && e.skim_flushing !== null) {
    if (e.skim_flushing === "high") skimFlushWord = "T84";
    else if (e.skim_flushing === "low") skimFlushWord = "T85";
    else {
      refuse("skimflushing", "machine " + machineName() + ": edm.skim_flushing " + JSON.stringify(e.skim_flushing) + " is not high or low - the Sodick LN2W wire post writes T84 or T85 only");
      ok = false;
    }
    if (e.flushing === undefined || e.flushing === null) {
      refuse("skimflushing", "machine " + machineName() + ": edm.skim_flushing is stated without edm.flushing - the skim's flushing word would stay active for the next rough cut; state edm.flushing (high or low) too");
      ok = false;
    }
  }
  return ok;
}

// The header's word on tank and flushing: what is programmed, what is
// left to the operator.
function dielectricNotes() {
  if (tankWord === "" && flushWord === "") {
    note("TANK FILL AND FLUSHING ARE NOT PROGRAMMED - SET THEM AT THE CONTROL");
    return;
  }
  if (tankWord === "T94") note("T94 SUBMERGED MACHINING AFTER THREADING - NO FILL OR DRAIN WORD IS WRITTEN - CHECK THE TANK IS FULL");
  else if (tankWord === "T89") note("T89 NON-SUBMERGED MACHINING AFTER THREADING");
  else note("TANK IS NOT PROGRAMMED - SET IT AT THE CONTROL");
  if (flushWord === "T84") note("T84 HIGH-PRESSURE FLUSHING AFTER THREADING");
  else if (flushWord === "T85") note("T85 LOW-PRESSURE FLUSHING AFTER THREADING");
  else note("FLUSHING IS NOT PROGRAMMED - SET IT AT THE CONTROL");
  if (skimFlushWord === "T84") note("T84 HIGH-PRESSURE FLUSHING BEFORE EACH SKIM PASS");
  else if (skimFlushWord === "T85") note("T85 LOW-PRESSURE FLUSHING BEFORE EACH SKIM PASS");
}

// -------------------------------------------------------------- header
// Absolute positions; no unit word (the control alarms on G21). The
// header states what the program leaves to the operator.
function onOpen() {
  machineGood = machineOk();
  approach = typeof postOptions !== "undefined" && postOptions && postOptions.approachCondition === true;
  var h = typeof postOptions !== "undefined" && postOptions ? postOptions.partBottomAboveTable : undefined;
  tableH = has(h) && Number(h) >= 0 ? Number(h) : null;
  note(clean(machineName()));
  note("SODICK LN2W WORDS - NOT VERIFIED ON A CONTROL - CHECK BEFORE CUTTING");
  note("PROGRAM IS IN MM - NO G20 OR G21 IS WRITTEN - THE CONTROL UNIT SETTING MUST BE MM");
  note(awt ? "T91 THREADS THE WIRE AT EACH START POINT" : "WIRE NOT THREADED AT START - THREAD IT AT EACH M00");
  note("WORK OFFSET SET AT THE CONTROL - NO G92 IS WRITTEN");
  note("Z0. FOLLOWS EACH MOVE TO A START POINT - SET Z0 AT THE UPPER HEAD HEIGHT ABOVE THE PART BEFORE START");
  dielectricNotes();
  if (offsetProgram()) note("WIRE OFFSET G41 G42 IS USED IN EACH SECTION THAT NAMES AN H REGISTER - SET EACH H REGISTER AT THE CONTROL BEFORE START - A SECTION WITHOUT ONE IS A WIRE-CENTRE PATH");
  else note("WIRE-CENTRE PATH - NO WIRE OFFSET G41 G42 IS USED");
  note("EACH C CONDITION NAMED BELOW MUST EXIST IN THE CONTROL CONDITION TABLE BEFORE START");
  if (approach) note("C000 APPROACH CONDITION ON EACH APPROACH MOVE - C000 MUST EXIST IN THE CONTROL CONDITION TABLE");
  block(["G90"]);
}

function onParameter(name, value) {
  if (WIRE_NUMBERS.indexOf(name) >= 0 && !has(value)) {
    sectionOk = false;
    notFinite = name;
    refuse("nf" + name + opName, who() + "PARAM " + name + " is not a finite number (" + (value === null || value === undefined ? "null" : clean(value)) + ") - refused, the Sodick LN2W wire post writes no cut from it");
    return;
  }
  if (name === "WorkOffset" && value) wcsWord = String(value);
  else if (name === "OriginX" && has(value)) orgX = Number(value);
  else if (name === "OriginY" && has(value)) orgY = Number(value);
  else if (name === "OperationName") opName = clean(value);
  else if (name === "OperationKind") opKind = clean(value);
  else if (name === "SetupName") setupName = clean(value);
  else if (name === "EdmCondition") {
    // With the wire threaded, a condition is the next skim pass's (#538,
    // #2649): startSkim writes it before that pass's first move.
    condition = String(value === null || value === undefined ? "" : value);
    if (inSection && threaded) skimPending = true;
  }
  else if (name === "WireSide") side = clean(value);
  else if (name === "AfterOperation") afterOp = String(value);
  else if (name === "WireTaper") taper = Number(value);
  else if (name === "WireFourAxis") fourAxis = clean(value);
  else if (name === "WireTaperDialect") dialect = clean(value);
  else if (name === "WireAngle") angleNext = true;
  else if (name === "WireU") { leaning = true; leanU = Number(value); }
  else if (name === "WireV") { leaning = true; leanV = Number(value); }
  else if (name === "WireGuideLower") guideLo = Number(value);
  else if (name === "WireGuideUpper") guideHi = Number(value);
  else if (name === "WireBottom") partBot = Number(value);
  else if (name === "WireTop") partTop = Number(value);
  else if (name === "SpindleSpeed" && has(value) && Number(value) > 0) {
    refuse("rpm" + opName, who() + "states " + Math.round(Number(value)) + " rpm - a wire EDM turns no spindle");
  }
}

function onToolChange(number, name) {
  refuse("tool" + opName, who() + "changes to tool " + number + " - a wire EDM mounts no tool (the wire is the condition's)");
}

// #4076: the control's wire offset. The shared law (wire_offset.rs)
// keeps the state; the words are written at the next move (offsetStep,
// offsetWords). A refused record refuses the section; OFF cannot make a
// refused section safe again. Section end resets the refusal.
function onRadiusCompensation(side) {
  var p = typeof properties !== "undefined" && properties ? properties : {};
  var why = wireOffsetRecord(wireOff, side, p.CompensationRadius, p.CompensationType);
  if (why === "") return;
  controlOffset = true;
  sectionOk = false;
  refuse("offset" + opName, who() + why);
}

// The law's verdict on one move, before any word of it is written; null
// refuses the section. The LN2W rules on top of the law: H001 goes with
// C001 (the owner, #4076), so the first pass must be on C001 and the C000
// approach option cannot cut the lead-in.
function offsetStep(p, straight) {
  if (!inSection || !sectionOk) return { engage: "", cancel: false, error: "" };
  var d = wireOffsetMove(wireOff, straight, p.StartX, p.StartY, p.X, p.Y);
  if (d.error === "" && d.engage !== "") {
    var pass = !threaded ? 1 : skim + (skimPending ? 2 : 1);
    if (fourOn) {
      d.error = "the control's wire offset (G41 / G42) in a tapered cut is not confirmed on the LN2W - post the taper on the wire-centre path (no wire offset)";
    } else if (approach) {
      d.error = "the C000 approach condition cuts the lead-in, but the wire offset H001 goes with C001 - set the C000 approach condition off to post a wire offset";
    } else if (pass === 1 && condition !== "C001") {
      d.error = "the wire offset H001 always goes with condition C001 on the LN2W control, and the first pass is on " + condition + " - cut the first pass on C001";
    } else if (pass > 999) {
      d.error = "pass " + pass + " has no H register (H001 to H999) - refused";
    }
  }
  if (d.error === "") return d;
  refuse("offmove" + opName, who() + d.error);
  sectionOk = false;
  return null;
}

// The offset's own line before the lead-in (the owner's own-line form):
// G41 / G42 and the pass's H register, after the pass's C word.
function offsetWords(d) {
  if (d.engage === "") return;
  var h = "H" + ("00" + (skim + 1)).slice(-3);
  var r = Number(wireOff.radius).toFixed(3) + " MM";
  if (wireOff.wear) note("WIRE OFFSET " + h + " WITH CONDITION " + condition + " HOLDS THE WEAR DELTA ONLY - THE STREAM STATES " + r + " - SET " + h + " AT THE CONTROL");
  else note("WIRE OFFSET " + h + " WITH CONDITION " + condition + " - THE PATH IS THE PART CONTOUR, NOT THE WIRE CENTRE - SET " + h + " AT THE CONTROL TO THE WIRE RADIUS PLUS THE GAP - THE STREAM STATES " + r);
  block([d.engage === "left" ? "G41" : "G42", h]);
}

// ------------------------------------------------------------- section
function onSection() {
  inSection = true;
  sectionOk = false;
  threaded = false;
  compOff = false;
  cutZ = null;
  skim = 0;
  skimPending = false;
  approachPending = false;
  if (condition === "") {
    refuse("op" + opName, who() + "is not wire EDM work (no EDM condition) - the Sodick LN2W wire post posts Wire Contour operations only");
    return;
  }
  var ok = machineGood && !controlOffset && notFinite === "";
  // The condition word: C and three digits (C000 to C999). The operation's
  // condition id is that word; anything else is refused.
  if (!/^C[0-9]{3}$/.test(condition)) {
    refuse("cid" + opName, who() + "condition id " + JSON.stringify(condition) + " is not a Sodick condition word (C and three digits, C000 to C999) - name the machine file's condition that way");
    ok = false;
  }
  if (fourAxis !== "") {
    refuse("four" + opName, who() + "is a 4-axis cut - the Sodick LN2W wire post writes vertical 2-axis cuts only; no G74 / G75 or U V profile words");
    ok = false;
  } else if (taper !== 0 && dialect !== "") {
    // #4075: the angle form leaves each corner to the control, and the
    // LN2W control's corner motion and lean side under G51 / G52 are not
    // confirmed. Never written from an unconfirmed rule.
    refuse("taper" + opName, who() + "is tapered (" + String(Math.abs(taper)) + " deg) as an angle word - the Sodick LN2W wire post writes a taper as 4-axis G74 U V moves only: the control's corner motion and lean side under G51 / G52 are not confirmed (#4075); state no edm.taper_angle in the machine file");
    ok = false;
  } else if (taper !== 0) {
    var why = taperWhy();
    if (why !== "") {
      refuse("taper" + opName, who() + "is tapered (" + String(Math.abs(taper)) + " deg) - " + why + " (#4075)");
      ok = false;
    } else fourOn = true;
  } else if (dialect !== "") {
    refuse("angle0" + opName, who() + "states an angle-word dialect without a taper angle - refused");
    ok = false;
  }
  if (!/^G5[4-9]$/.test(wcsWord)) {
    refuse("wcs" + opName, who() + "work offset " + JSON.stringify(wcsWord) + " is not G54 to G59 - the Sodick LN2W wire post writes no G92, so the setup must state one");
    ok = false;
  }
  if (!ok) { fourOn = false; return; }
  sectionOk = true;
  if (fourOn && taperSetup === null) { taperSetup = setupName; taperBase = partBot; }
  note(opName + (opKind ? " - " + opKind : ""));
  if (setupName) note("SETUP " + setupName);
  var mode = offsetMode();
  if (mode === "control") note("WIRE OFFSET G41 G42 IN THIS SECTION - THE PATH IS THE PART CONTOUR WITHOUT THE KERF");
  else if (mode === "wear") note("WIRE OFFSET G41 G42 IN THIS SECTION - THE H REGISTER HOLDS THE WEAR DELTA ONLY");
  if (side) note("SIDE " + side.toUpperCase() + (mode === "control" ? " - THE CONTROL WIRE OFFSET SETS THE KERF" : " - KERF IN THE PATH"));
  // The C word selects a row of the control's own condition table; the
  // program carries no table, so the operator checks the row exists.
  note("CONDITION " + condition + " MUST EXIST IN THE CONTROL CONDITION TABLE - CHECK ITS VALUES");
  conditionNotes();
  if (fourOn) note("TAPER " + fu(Math.abs(taper)) + " DEG - 4-AXIS G74 - TP " + fu(tpOf()) + " IS WORK Z" + fu(partBot) + " ABOVE THE WORKTABLE - THE POST OPTION STATES WORK Z" + fu(taperBase) + " AT " + fu(tableH) + " - CHECK IT ON THE MACHINE");
  block([wcsWord]);
}

// #4075: why a taper section cannot be written as G74 ("" when it can):
// each plane the conversion reads, the stated table height, and one
// table height for the program.
// TP of this section: X Y are on its own WireBottom, so TP is that
// plane's height above the worktable. The option states it for the
// program's first taper; a later bottom moves TP by the difference.
function tpOf() { return tableH + partBot - (taperBase === null ? partBot : taperBase); }
function taperWhy() {
  var planes = [["WireGuideLower", guideLo], ["WireGuideUpper", guideHi], ["WireBottom", partBot], ["WireTop", partTop]];
  for (var i = 0; i < planes.length; i++) {
    if (!has(planes[i][1])) return "the stream states no PARAM " + planes[i][0] + " - the post cannot move U V from the guides to the part's planes; generate and post it again";
  }
  if (guideHi - guideLo <= 0.001) return "the guide planes (Z" + fu(guideLo) + " .. Z" + fu(guideHi) + ") have no span";
  if (partTop - partBot <= 0.001) return "the part's top (Z" + fu(partTop) + ") is not above its bottom (Z" + fu(partBot) + ")";
  if (tableH === null) return "the part's bottom above the worktable (TP) is not stated - set the post option partBottomAboveTable (Part bottom above the worktable, mm) to the fixture height";
  if (taperSetup !== null && taperSetup !== setupName) return "the post option partBottomAboveTable states one table height for the program, and setup " + JSON.stringify(setupName) + " is tapered after setup " + JSON.stringify(taperSetup) + " - post the tapers of each setup as their own program";
  if (tpOf() < -0.0005) return "its bottom (work Z" + fu(partBot) + ") is below the worktable: the post option partBottomAboveTable states work Z" + fu(taperBase) + " (the bottom of the program's first tapered cut) at " + fu(tableH) + " mm above the worktable - check the option and the bottom heights";
  return "";
}

// #4075: a move's words on the part's planes. The stream's X Y are the
// lower guide's and U V the upper guide's offset from it; the wire is
// one straight line, so its point on the part's bottom is X Y plus the
// slope over (bottom - lower guide), and U V the slope over the part's height.
function onPart(p, lean) {
  var span = guideHi - guideLo;
  var tx = lean ? lean[0] / span : 0, ty = lean ? lean[1] / span : 0;
  var d = partBot - guideLo, h = partTop - partBot;
  return { x: p.X + tx * d, y: p.Y + ty * d, u: tx * h, v: ty * h };
}
function uvWords(w) {
  lastU = w.u;
  lastV = w.v;
  return ["U" + fu(w.u), "V" + fu(w.v)];
}

// The first motion of a section, in the order the owner of an AQ600L
// (LN2W) gave (his round-2 correction on PR #4065): G00 to the start
// point with U0. V0. on the same line (the taper axes upright), Z0. (the
// upper head height the operator set), T91 (or the operator at an M00),
// flushing, tank, the condition, then the first cut.
function openRun() { return side === "left" || side === "right"; }
function startCut(p) {
  if (threaded) {
    if (skimPending) return startSkim();
    // The approach (the section's first move) was cut on C000; the
    // operation's own condition takes over for the contour (#2649).
    if (approachPending && compOff) {
      approachPending = false;
      block([condition]);
    }
    return true;
  }
  if (!sectionOk) return false;
  if (!has(p.StartX) || !has(p.StartY)) {
    refuse("start" + opName, who() + "the path states no threading point - regenerate it");
    return false;
  }
  block(["G00", wo("X", p.StartX, orgX), wo("Y", p.StartY, orgY), "U0.", "V0."]);
  block(["Z0."]);
  if (awt) block(["T91"]);
  else {
    note(openRun() ? "THREAD THE WIRE AT THE RUN-IN POINT OUTSIDE THE STOCK" : "THREAD THE WIRE AT THE START HOLE");
    block(["M00"]);
  }
  if (flushWord !== "") block([flushWord]);
  if (tankWord !== "") block([tankWord]);
  if (approach && condition !== "C000") {
    block(["C000"]);
    approachPending = true;
  } else block([condition]);
  // #4075: the 4-axis form's planes and its start, after the condition.
  if (fourOn) {
    block(["TP" + fu(tpOf())]);
    block(["TN" + fu(tpOf() + partTop - partBot)]);
    block(["G74"]);
    lastU = 0;
    lastV = 0;
  }
  threaded = true;
  cutZ = has(p.StartZ) ? p.StartZ : null;
  return true;
}

// A skim pass (#2649): the previous pass ended back at the start point
// with the wire threaded; the skim's flushing word, then its
// own C word, come before its first move. Its offset is in the path.
function startSkim() {
  skimPending = false;
  approachPending = false;
  if (!/^C[0-9]{3}$/.test(condition)) {
    refuse("skim" + opName, who() + "skim condition id " + JSON.stringify(condition) + " is not a Sodick condition word (C and three digits, C000 to C999) - name the machine file's condition that way");
    sectionOk = false;
    return false;
  }
  // #2686: the rough cut's flushing is programmed and the machine file
  // states none for a skim: never cut the skim on the rough cut's flushing.
  if (flushWord !== "" && skimFlushWord === "") {
    refuse("skimflush" + opName, who() + "has a skim pass, and machine " + machineName() + " states edm.flushing but no edm.skim_flushing - the Sodick LN2W wire post does not cut a skim on the rough cut's flushing; state edm.skim_flushing (high or low) in the machine file");
    sectionOk = false;
    return false;
  }
  skim++;
  note("SKIM " + skim + " - CONDITION " + condition + " MUST EXIST IN THE CONTROL CONDITION TABLE");
  conditionNotes();
  if (skimFlushWord !== "") block([skimFlushWord]);
  block([condition]);
  return true;
}

// A cut: one programmed plane, no pose, no guide-offset lean, no angle
// word, from a threaded wire.
function cutOk(p, ang) {
  var lean = leaning;
  leaning = false;
  if (!inSection || condition === "") {
    refuse("free" + opName, who() + "a move outside a wire section - refused");
    return false;
  }
  if (!sectionOk || notFinite !== "") return false;
  if (has(p.A) || has(p.B) || has(p.C)) {
    refuse("tilt" + opName, who() + "a posed move - the Sodick LN2W wire post writes vertical 2-axis cuts only");
    sectionOk = false;
    return false;
  }
  if (lean && !fourOn) {
    refuse("uv" + opName, who() + "a leaning move (upper guide offset) - the Sodick LN2W wire post writes no U V lean");
    sectionOk = false;
    return false;
  }
  if (ang) {
    refuse("ang" + opName, who() + "an angle word (a leaning move) - the Sodick LN2W wire post writes vertical 2-axis cuts only (#4075)");
    sectionOk = false;
    return false;
  }
  if (!startCut(p)) return false;
  if (has(p.Z) && cutZ !== null && Math.abs(p.Z - cutZ) > 0.0005) {
    refuse("z" + opName, who() + "a move changes Z - a wire cut rides one programmed plane");
    return false;
  }
  return true;
}

function onRapid() {
  var lean = leaning;
  leaning = false;
  angleNext = false;
  if (threaded) {
    refuse("rapid" + opName, who() + "a rapid with the wire threaded through the work - refused");
    return;
  }
  // Before the first cut the wire is not threaded: the G00 to the first
  // cut's start (written by startCut) stands for every positioning move.
  if (!inSection || condition === "") {
    refuse("free" + opName, who() + "a move outside a wire section - refused");
    return;
  }
  // A positioning move that states an upper guide offset leans the wire
  // before it is threaded (#1497): refuse the section by name.
  if (lean) {
    refuse("uvrapid" + opName, who() + "a leaning positioning move (upper guide offset) - the Sodick LN2W wire post writes no U V lean");
    sectionOk = false;
  }
}

// The lean the next move states (null: upright), consumed by it.
function takeLean() {
  var l = leaning ? [leanU, leanV] : null;
  leanU = null;
  leanV = null;
  return l;
}

function onLinear() {
  var p = properties;
  var ang = angleNext;
  angleNext = false;
  var lean = takeLean();
  var d = offsetStep(p, true);
  if (d === null) return;
  if (!cutOk(p, ang)) return;
  offsetWords(d);
  // No F word: the condition sets the rate. G40 rides the lead-out move.
  if (fourOn) {
    if (lean && (!has(lean[0]) || !has(lean[1]))) {
      refuse("uvpair" + opName, who() + "a leaning move states only one of WireU / WireV - refused");
      sectionOk = false;
      return;
    }
    var w = onPart(p, lean);
    block(["G01", wo("X", w.x, orgX), wo("Y", w.y, orgY)].concat(uvWords(w)));
  } else block([d.cancel ? "G40" : "", "G01", wo("X", p.X, orgX), wo("Y", p.Y, orgY)]);
  compOff = true;
}

function onCircular() {
  var p = properties;
  var ang = angleNext;
  angleNext = false;
  if (p.CircularPlane && p.CircularPlane !== "XY") {
    refuse("plane" + opName, who() + "an arc in the " + p.CircularPlane + " plane - a wire cut is flat");
    return;
  }
  // #4075: the 4-axis form's arc words (the upper arc) are not confirmed.
  if (fourOn && sectionOk && inSection) {
    takeLean();
    leaning = false;
    refuse("arctaper" + opName, who() + "an arc in a tapered cut - the Sodick LN2W wire post writes a taper as straight G74 moves only");
    sectionOk = false;
    return;
  }
  if (!threaded && !compOff && sectionOk && inSection && condition !== "") {
    refuse("arcfirst" + opName, who() + "the first cut is an arc - the Sodick LN2W wire post starts every cut with a straight move from the start point; give the cut a straight lead-in");
    sectionOk = false;
    return;
  }
  // An arc never engages or cancels the offset (the law refuses it).
  if (offsetStep(p, false) === null) return;
  if (!cutOk(p, ang)) return;
  // #3442: an arc whose written ends coincide is no arc block (arcFate, PRELUDE).
  var fate = arcFate(arcWord, arcIncOf(fu));  // #3474: I J K are increments from the start
  if (fate !== "arc") { if (fate === "linear") onLinear(); return; }
  var c = p.IJKIncrement || {};
  if (!has(c.I) || !has(c.J) || !has(p.StartX) || !has(p.StartY)) {
    refuse("arcc" + opName, who() + "an arc states no centre - refused");
    return;
  }
  block([
    p.CircularDirection === "CW" ? "G02" : "G03",
    wo("X", p.X, orgX), wo("Y", p.Y, orgY),
    "I" + fu(c.I - p.StartX), "J" + fu(c.J - p.StartY),
  ]);
}

function onCycle() {
  refuse("cycle" + opName, who() + "a canned cycle - a wire EDM has no drilling cycle");
}

// The section's end: cut the wire (T90, or the operator at an M00), then
// the operator's stop if the op asks one.
function onSectionEnd() {
  var on = wireOffsetEnd(wireOff);
  if (on !== "" && sectionOk) refuse("offend" + opName, who() + on);
  // #4075: G75 ends the 4-axis form with the wire upright, before T90.
  if (threaded && fourOn) {
    if (fu(lastU) !== "0." || fu(lastV) !== "0.") {
      refuse("leanend" + opName, who() + "ends with the wire leaning (U" + fu(lastU) + " V" + fu(lastV) + ") - a tapered cut must stand the wire upright before G75; give it an upright lead-out");
    }
    block(["G75"]);
  }
  if (threaded) {
    if (awt) block(["T90"]);
    else {
      note(openRun() ? "CUT THE WIRE AT THE RUN-OUT POINT" : "CUT THE WIRE");
      block(["M00"]);
    }
  }
  if (afterOp === "optional-stop") block(["M01"]);
  else if (afterOp === "stop") block(["M00"]);
  else if (afterOp === "home") {
    refuse("home" + opName, who() + "asks a trip home - the Sodick LN2W wire post states no home position; set the operation to Continue");
  }
  sectionNo++;
  inSection = false;
  sectionOk = false;
  threaded = false;
  compOff = false;
  skim = 0;
  skimPending = false;
  approachPending = false;
  condition = "";
  side = "";
  afterOp = "";
  taper = 0;
  dialect = "";
  angleNext = false;
  controlOffset = false;
  fourAxis = "";
  leaning = false;
  leanU = null;
  leanV = null;
  guideLo = null;
  guideHi = null;
  partBot = null;
  partTop = null;
  fourOn = false;
  notFinite = "";
}

// ------------------------------------------------------------ footer
function onClose() {
  if (threaded) refuse("open", "the program ends with the wire threaded - refused");
  block(["M02"]);
}
