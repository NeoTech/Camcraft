// Hurco WinMax (NC) — ISNC (Industry Standard NC) 3-axis mill post processor (#2573).
// Clean-room implementation from public Hurco WinMax Mill NC documentation
// (Hurco WinMax Mill NC Programming Manual 704-0116-101, support.hurco.com ISNC tables).
// No vendor post files or proprietary implementations consulted.

description = "Hurco WinMax (NC)";
vendor = "Hurco";
extension = "nc";
programUnits = "mm inch";
endUnits = "G20 G21";
explains = "paren";
coolantModes = true;
afterModes = true;
millSpindleReverse = true;
floatingTapping = true;
wcsRotation = true;
workOffsetIndex = true;
workOffsetMax = 99;
minimumRevision = 45702;

declaredOptions = {
  showSequenceNumbers: {
    type: "enum",
    title: "Show sequence numbers",
    group: "Format",
    description: "When to write N block numbers",
    choices: ["off", "always", "tool-change"],
    default: "off"
  },
  sequenceNumberStart: {
    type: "integer",
    title: "Sequence number start",
    group: "Format",
    description: "First N number written when sequence numbers are enabled",
    default: 10,
    min: 1,
    max: 999999
  },
  sequenceNumberIncrement: {
    type: "integer",
    title: "Sequence number increment",
    group: "Format",
    description: "Step between N numbers when sequence numbers are enabled",
    default: 5,
    min: 1,
    max: 1000
  },
  separateWordsWithSpace: {
    type: "boolean",
    title: "Space between words",
    group: "Format",
    description: "Separate NC words with spaces on each block",
    default: true
  },
  writeMachine: {
    type: "boolean",
    title: "Write machine comment",
    group: "Header",
    description: "Write a machine name comment at the top of the program",
    default: false
  },
  writeToolList: {
    type: "boolean",
    title: "Write tool comment",
    group: "Header",
    description: "Write a descriptive comment at each tool change",
    default: true
  },
  writeVersion: {
    type: "boolean",
    title: "Write post note",
    group: "Header",
    description: "Write the dialect note at the top of the program",
    default: false
  },
  optionalStop: {
    type: "boolean",
    title: "Optional stop (M1)",
    group: "Behavior",
    description: "Write M1 before tool changes after the first operation",
    default: false
  },
  useRadiusArcs: {
    type: "boolean",
    title: "Use R word for arcs",
    group: "Behavior",
    description: "Emit R instead of I J K on circular moves",
    default: false
  },
  safeRetract: {
    type: "enum",
    title: "Safe retract method",
    group: "Behavior",
    description: "Z retract command before tool changes and at program end",
    choices: ["G28", "G53"],
    default: "G28"
  }
};

function has(v) { return typeof v !== "undefined" && v !== null; }
function num(v, d) { return has(v) && isFinite(Number(v)) ? Number(v) : d; }
function trimNum(s) {
  if (s.indexOf(".") >= 0) s = s.replace(/0+$/, "").replace(/\.$/, "");
  return s === "-0" ? "0" : s;
}
// Fanuc/ISNC requires an explicit decimal point on length, angle, and second-dwell words (#1836).
function pt(s) { return s.indexOf(".") >= 0 ? s : s + "."; }
function pad4(n) {
  var s = String(Math.round(n));
  while (s.length < 4) s = "0" + s;
  return s;
}
function fn(v) {
  var d = unitInch ? 4 : 3;
  var n = Number(v);
  return Math.abs(n) < (unitInch ? 5e-5 : 5e-4) ? "0" : trimNum(n.toFixed(d));
}
function fu(v) { return pt(fn(v * unitScale)); }
function fa(v) {
  var n = Number(v);
  return pt(Math.abs(n) < 5e-4 ? "0" : trimNum(n.toFixed(3)));
}
function ni(v) { return String(Math.round(Number(v))); }
function ff(v) {
  var n = Number(v) * unitScale;
  if (!unitInch) return String(Math.round(n));
  return pt(trimNum(n.toFixed(2)));
}
function cleanComment(s) {
  var c = String(s).replace(/[^\x20-\x7E]/g, " ").replace(/[();*]/g, " ").replace(/\s+/g, " ").trim();
  if (/^\s*(MSG|DEBUG|PROBEOPEN|PROBECLOSE|LOGOPEN|LOGCLOSE|LOGAPPEND|LOG)\s*[,:]/i.test(c)) {
    c = c.replace(/^(\s*(?:MSG|DEBUG|PROBEOPEN|PROBECLOSE|LOGOPEN|LOGCLOSE|LOGAPPEND|LOG))\s*([,:])/i, "$1 -$2");
  }
  return c;
}
function optVal(k, d) {
  return typeof postOptions !== "undefined" && postOptions && has(postOptions[k]) ? postOptions[k] : d;
}
function optBool(k, d) { return Boolean(optVal(k, d)); }
function optInt(k, d) { return Math.round(Number(optVal(k, d))); }
function optStr(k, d) { return String(optVal(k, d)); }
function who() { return opName ? "operation " + opName + ": Hurco WinMax (NC) post: " : "Hurco WinMax (NC) post: "; }
function msg(text) {
  if (typeof message === "function") message(text);
  else writeln("(" + cleanComment(text) + ")");
}

var unitInch = false;
var unitScale = 1.0;
var seqNext = 10;
var seqInc = 5;
var seqNextForce = false;

var started = false;
var sectionOpen = false;
var secStarted = 0;
var coolant = "";
var coolantAsk = "flood";
var spindleOn = false;
var spindleWait = false;
var spindleRefused = false;
var sRpm = 0;
var spinRev = false;
var kindProbe = false;
var probeSection = false;
var toolWait = false;
var tNo = 0;
var toolName = "";
var toolKind = "";
var toolDia = 0;
var toolCr = 0;

var wcsIndex = 1;
var wcsWord = "G54";
var orgX = 0;
var orgY = 0;
var orgZ = 0;
var wcsRot = 0;
var wcsRotMode = "";
var wcsRotActive = false;

var clearZ = null;
var hopZ = null;
var xyUnknown = false;
var traverseRefused = false;
var seatKnown = false;
var curX = 0;
var curY = 0;
var curZ = 0;
var planeG = 17;

var inCycle = false;
var cycleSig = "";
var cycleDwellConsumed = false;
var cycX = 0;
var cycY = 0;
var cycR = 0;

var compG = "";
var compD = "";
var compOn = false;

var afterAsk = "";
var setupName = "";
var setupSaid = "";
var setupNext = "";
var opName = "";
var opKind = "";
var smoothTol = 0;
var smoothNoted = false;
var safeRetractMode = "G28";
var probeLabelNext = 9001;
var COOLANT_KINDS = ["flood", "mist", "air", "air-blast", "through-spindle", "through-tool", "through-air"];

function seqPrefix() {
  var mode = optStr("showSequenceNumbers", "off");
  var emit = mode === "always" || (mode === "tool-change" && seqNextForce);
  if (!emit) return "";
  seqNextForce = false;
  var n = "N" + seqNext;
  seqNext += seqInc;
  return n + (optBool("separateWordsWithSpace", true) ? " " : "");
}

function block(words) {
  start();
  var sep = optBool("separateWordsWithSpace", true) ? " " : "";
  writeln(seqPrefix() + words.join(sep));
}

function note(text) {
  var c = cleanComment(text);
  if (!c) return;
  start();
  writeln("(" + c + ")");
}

function oWord() {
  var n = (typeof program !== "undefined" && program && has(program.number)) ? program.number : programNumber;
  var raw = Math.round(Number(n));
  if (!isFinite(raw) || raw < 1 || raw > 9999) {
    msg(who() + "program number " + raw + " is out of Hurco WinMax ISNC range O0001..O9999");
    raw = 1;
  }
  return "O" + pad4(raw);
}

function workOffsetName(n) {
  var k = Math.round(Number(n));
  if (!isFinite(k) || Math.abs(Number(n) - k) > 1e-6) {
    msg(who() + "invalid work offset " + n);
    return "G54";
  }
  if (k <= 0) return "G54";
  if (k <= 6) return ["G54", "G55", "G56", "G57", "G58", "G59"][k - 1];
  if (k <= 99) return "G54.1 P" + String(k - 6);
  msg(who() + "work offset " + k + " exceeds Hurco WinMax ISNC maximum (99: G54..G59, G54.1 P1..P93)");
  return "G54";
}

function parseWcs(v) {
  if (typeof v === "number") return Math.round(v);
  var s = String(v || "").trim().toUpperCase();
  var m = /^G(\d+)$/.exec(s);
  if (m) {
    var g = Number(m[1]);
    if (g >= 54 && g <= 59) return g - 53;
  }
  var n = Number(s);
  return isFinite(n) && n > 0 ? Math.round(n) : 1;
}

function resolveSafeRetractMode() {
  safeRetractMode = optStr("safeRetract", "G28").toUpperCase();
  if (safeRetractMode !== "G28" && safeRetractMode !== "G53") {
    msg(who() + "unknown safeRetract option '" + safeRetractMode + "' (expected G28 or G53)");
    safeRetractMode = "G28";
  }
  if (safeRetractMode === "G53") {
    var m = typeof machine !== "undefined" ? machine : null;
    if (!m || !has(m.safeRetractZ)) {
      var mName = m && m.name ? String(m.name) : "the machine";
      msg(who() + mName + " does not state safeRetractZ (needed for G53 safe retract)");
    }
  }
}

function safeRetractBlock() {
  if (safeRetractMode === "G53") {
    var m = typeof machine !== "undefined" ? machine : null;
    var z = m && has(m.safeRetractZ) ? Number(m.safeRetractZ) : 0;
    block(["G53", "G00", "Z" + fu(z)]);
  } else {
    block(["G91", "G28", "Z0."]);
    block(["G28", "X0.", "Y0."]);
    block(["G90"]);
  }
}

function start() {
  if (started) return;
  started = true;
  writeln("%");
  var pc = (typeof program !== "undefined" && program && program.comment) ? cleanComment(program.comment) : "";
  if (!pc) {
    pc = (typeof machine !== "undefined" && machine && machine.name) ? cleanComment(machine.name) : "Hurco WinMax ISNC";
  }
  writeln(oWord() + (pc ? " (" + pc + ")" : ""));
  if (optBool("writeMachine", false) && typeof machine !== "undefined" && machine && machine.name) {
    writeln("(MACHINE: " + cleanComment(machine.name) + ")");
  }
  if (optBool("writeVersion", false)) {
    writeln("(HURCO WINMAX ISNC)");
  }
  writeln(unitInch ? "G20" : "G21");
  // G90 G17 G94 G40 G49 G80: WinMax ISNC has no G91.1 (I/J/K arc centers are always incremental).
  if (wcsIndex <= 6) {
    block(["G90", "G17", "G94", wcsWord, "G40", "G49", "G80"]);
  } else {
    block(["G90", "G17", "G94", "G40", "G49", "G80"]);
    block(["G90", wcsWord]);
  }
}

function onOpen() {
  unitInch = optStr("Units", "mm").toLowerCase() === "inch";
  unitScale = unitInch ? 1.0 / 25.4 : 1.0;
  seqNext = optInt("sequenceNumberStart", 10);
  seqInc = optInt("sequenceNumberIncrement", 5);
  resolveSafeRetractMode();
}

function coolantCodeFor(ask) {
  if (!ask || ask === "off") return "";
  var cs = (typeof machine !== "undefined" && machine && Array.isArray(machine.coolants)) ? machine.coolants : [];
  if (ask === "flood") {
    var other = "", floodNamed = false;
    for (var i = 0; i < cs.length; i++) {
      var c = cs[i];
      var code = typeof c === "string" ? "" : (c.mCode || "");
      var nm = String(typeof c === "string" ? c : (c.name || "")).trim().toLowerCase().replace(/[\s_]+/g, "-");
      if (nm === "flood") { if (code) return code; floodNamed = true; continue; }
      if (code && !other && COOLANT_KINDS.indexOf(nm) < 0) other = code;
    }
    if (other || floodNamed) return other;
    if (cs.length > 0) msg(who() + "the machine states no flood coolant code");
    return cs.length === 0 ? "M8" : "";
  }
  var want = ask === "mist" ? ["mist"] : (ask === "air" ? ["air", "air-blast"] : (ask === "through-tool" ? ["through-spindle", "through-tool"] : []));
  if (want.length === 0) { msg(who() + "unknown coolant mode " + ask); return ""; }
  for (var j = 0; j < cs.length; j++) {
    var cc = cs[j];
    var mcode = typeof cc === "string" ? "" : (cc.mCode || "");
    var name = String(typeof cc === "string" ? cc : (cc.name || "")).trim().toLowerCase().replace(/[\s_]+/g, "-");
    if (mcode && want.indexOf(name) >= 0) return mcode;
  }
  msg(who() + "the machine states no " + ask + " coolant code");
  return "";
}

function onParameter(name, value) {
  if (name === "OperationName") { opName = String(value || ""); spinRev = false; return; }
  if (name === "OperationKind") { opKind = String(value || ""); return; }
  if (name === "SetupName") { setupName = String(value || ""); return; }
  if (name === "SetupChange") { setupNext = String(value || ""); return; }
  if (name === "AfterOperation") { afterAsk = String(value || ""); return; }
  if (name === "MillSpindle") { spinRev = String(value || "").toLowerCase() === "reverse"; return; }
  if (name === "SpindleSpeed") {
    sRpm = Number(value) || 0;
    spindleWait = true;
    return;
  }
  if (name === "Coolant") { coolantAsk = String(value || "flood").toLowerCase(); return; }
  if (name === "ToolKind") {
    toolKind = String(value || "");
    var tk = toolKind.toLowerCase();
    kindProbe = tk === "probe" || tk === "touch probe";
    return;
  }
  if (name === "ToolDiameter") { toolDia = Number(value) || 0; return; }
  if (name === "ToolCornerRadius") { toolCr = Number(value) || 0; return; }
  if (name === "ClearancePlane") { clearZ = has(value) ? Number(value) : null; return; }
  if (name === "RetractPlane") { hopZ = has(value) ? Number(value) : null; return; }
  if (name === "OriginX") { orgX = Number(value) || 0; return; }
  if (name === "OriginY") { orgY = Number(value) || 0; return; }
  if (name === "OriginZ") { orgZ = Number(value) || 0; return; }
  if (name === "WorkOffset") {
    wcsIndex = parseWcs(value);
    wcsWord = workOffsetName(wcsIndex);
    return;
  }
  if (name === "WcsYaw") { wcsRot = Number(value) || 0; return; }
  if (name === "WcsRotationMode") { wcsRotMode = String(value || "").toLowerCase(); return; }
  if (name === "WCSRotation") {
    wcsRot = Number(value) || 0;
    wcsRotMode = Math.abs(wcsRot) > 1e-6 ? "control" : "";
    return;
  }
  if (name === "Smoothing") { smoothTol = Number(value) || 0; return; }
}

function onComment(s) {
  note(s);
}

function onToolChange(number, desc) {
  var p = properties;
  var next = Math.round(has(number) ? Number(number) : num(p.ToolNumber, 0));
  toolName = has(desc) ? String(desc) : (has(p.ToolName) ? String(p.ToolName) : "");
  if (compOn || compG) msg(who() + "tool change before cutter compensation cancel (G40)");
  if (next !== tNo) {
    tNo = next;
    toolWait = true;
  }
}

function startPreamble() {
  if (sectionOpen) return;
  sectionOpen = true;
  if (wcsRotActive) {
    block(["G69"]);
    wcsRotActive = false;
  }
  start();
  if (setupName && setupName !== setupSaid) {
    note("SETUP: " + setupName);
    setupSaid = setupName;
  }
  if (opName) note("OPERATION: " + opName + (opKind ? " - " + opKind : "") + " - " + wcsWord);
  opName = ""; opKind = "";

  if (toolWait) {
    if (spindleOn) { block(["M5"]); spindleOn = false; }
    if (coolant) { block(["M9"]); coolant = ""; }
    if (secStarted > 0) safeRetractBlock();
    if (secStarted > 0 && optBool("optionalStop", false)) block(["M1"]);
    if (optBool("writeToolList", true)) {
      var desc = [];
      if (toolKind) desc.push(toolKind);
      if (toolDia > 0) desc.push("D" + fn(toolDia));
      if (toolCr > 0) desc.push("R" + fn(toolCr));
      note("T" + ni(tNo) + (toolName ? " " + toolName : "") + (desc.length ? " - " + desc.join(" ") : ""));
    }
    toolKind = ""; toolDia = 0; toolCr = 0;
    if (optStr("showSequenceNumbers", "off") === "tool-change") seqNextForce = true;
    block(["T" + ni(tNo), "M6"]);
    block(["G43", "H" + ni(tNo)]);
    xyUnknown = true;
    toolWait = false;
    if (sRpm > 0) spindleWait = true;
  }
  block(["G90", wcsWord]);

  if (wcsRotMode === "control" && Math.abs(wcsRot) > 1e-6) {
    block(["G68", "X0.", "Y0.", "R" + fa(wcsRot)]);
    wcsRotActive = true;
  }

  var noSpindle = !(sRpm > 0);
  if (spindleWait && !noSpindle) {
    block(["S" + ni(sRpm), spinRev ? "M4" : "M3"]);
    spindleOn = true;
    spindleWait = false;
  } else if (spindleWait && noSpindle) {
    block(["M5"]);
    spindleOn = false;
    spindleWait = false;
  }

  probeSection = noSpindle && kindProbe;
  var nextCool = noSpindle ? "" : coolantCodeFor(coolantAsk);
  if (nextCool !== coolant) {
    if (coolant) block(["M9"]);
    coolant = nextCool;
    if (coolant) block([coolant]);
  }
  if (smoothTol > 0 && !smoothNoted) {
    note("SMOOTHING NOT WRITTEN - G05.2/G05.3 REQUIRE NC OPTION");
    smoothNoted = true;
  }
  secStarted += 1;
}

function onSection() {
  seatKnown = false;
}

function endCycle() {
  if (inCycle) { block(["G80"]); inCycle = false; cycleSig = ""; }
}

function restorePlane() {
  if (planeG !== 17) { block(["G17"]); planeG = 17; }
}

function onSectionEnd() {
  endCycle();
  restorePlane();
  if (compOn || compG) {
    msg(who() + "section ends with cutter compensation active (missing G40)");
    compG = ""; compD = ""; compOn = false;
  }
  if (wcsRotActive) {
    block(["G69"]);
    wcsRotActive = false;
  }
  sectionOpen = false;
  seatKnown = false;
  hopZ = null; clearZ = null;
  smoothTol = 0; smoothNoted = false;
  coolantAsk = "flood";
  kindProbe = false;
  probeSection = false;
  if (afterAsk) {
    if (coolant) { block(["M9"]); coolant = ""; }
    if (spindleOn) { block(["M5"]); spindleOn = false; }
    if (setupNext && (afterAsk === "optional-stop" || afterAsk === "stop")) {
      safeRetractBlock();
      note("SETUP CHANGE - NEXT: " + setupNext);
    }
    if (afterAsk === "optional-stop") block(["M1"]);
    else if (afterAsk === "stop") block(["M0"]);
    else if (afterAsk === "home") safeRetractBlock();
    afterAsk = ""; setupNext = "";
  } else {
    if (coolant) { block(["M9"]); coolant = ""; }
    if (spindleOn) { block(["M5"]); spindleOn = false; }
  }
  spindleWait = false;
  sRpm = 0;
}

function tiltOk(p) {
  if (has(p.A) || has(p.B) || has(p.C)) {
    msg(who() + "rotary or 5-axis tilted move is not supported by the 3-axis Hurco WinMax (NC) post");
    return false;
  }
  return true;
}

function spindleStopped(what) {
  if (spindleOn || probeSection) return false;
  if (!spindleRefused) {
    spindleRefused = true;
    msg(who() + what + " with the spindle stopped - refused");
  }
  return true;
}

function xyFirst(x, y, z, descend) {
  if (!xyUnknown || !has(x) || !has(y)) return true;
  var c = has(clearZ) ? clearZ : (has(hopZ) ? hopZ : null);
  if (c === null) {
    if (!traverseRefused) {
      traverseRefused = true;
      msg(who() + "first X/Y traverse has no clearance height stated (ClearancePlane or RetractPlane)");
    }
    return false;
  }
  restorePlane();
  var tz = has(z) ? Math.max(c, Number(z)) : c;
  block(["G00", "Z" + fu(tz - orgZ)]);
  if (descend || !has(z) || Math.abs(Number(z) - tz) > 1e-9) {
    block(["G00", "X" + fu(Number(x) - orgX), "Y" + fu(Number(y) - orgY)]);
  }
  if (descend && has(z) && Number(z) < tz - 1e-9) {
    block(["G00", "Z" + fu(Number(z) - orgZ)]);
  }
  xyUnknown = false;
  return true;
}

function onHole(p) {
  return Math.abs(Number(p.X) - cycX) < 1e-6 && Math.abs(Number(p.Y) - cycY) < 1e-6 && Number(p.Z) <= cycR + 1e-6;
}

function onRapid() {
  var p = properties;
  startPreamble();
  if (!tiltOk(p)) return;
  if (inCycle) {
    if (onHole(p)) { seatKnown = true; return; }
    endCycle();
  }
  if (compG || compOn) {
    msg(who() + "cutter compensation active during rapid move (G00)");
    compG = ""; compD = ""; compOn = false;
  }
  seatKnown = true;
  if (!xyFirst(p.X, p.Y, p.Z, false)) {
    if (has(p.X)) curX = Number(p.X);
    if (has(p.Y)) curY = Number(p.Y);
    if (has(p.Z)) curZ = Number(p.Z);
    return;
  }
  restorePlane();
  var w = ["G00"];
  if (has(p.X)) { curX = Number(p.X); w.push("X" + fu(curX - orgX)); }
  if (has(p.Y)) { curY = Number(p.Y); w.push("Y" + fu(curY - orgY)); }
  if (has(p.Z)) { curZ = Number(p.Z); w.push("Z" + fu(curZ - orgZ)); }
  if (w.length > 1) block(w);
}

function onRadiusCompensation(mode) {
  var p = properties;
  var m = String(has(mode) ? mode : (p.Compensation || "")).toLowerCase();
  if (m === "off" || m === "0") {
    compG = compOn ? "G40" : "";
    compD = "";
  } else if (m === "left" || m === "1") {
    compG = "G41";
    compD = "D" + ni(tNo);
  } else if (m === "right" || m === "2") {
    compG = "G42";
    compD = "D" + ni(tNo);
  }
}

function onLinear() {
  var p = properties;
  startPreamble();
  if (!tiltOk(p)) return;
  if (inCycle) {
    if (!onHole(p)) msg(who() + "cutting move leaves hole inside canned-cycle group");
    if (has(p.X)) curX = Number(p.X);
    if (has(p.Y)) curY = Number(p.Y);
    if (has(p.Z)) curZ = Number(p.Z);
    return;
  }
  if (spindleStopped("feed move G01")) return;
  if (!xyFirst(has(p.StartX) ? p.StartX : p.X, has(p.StartY) ? p.StartY : p.Y, p.StartZ, true)) return;

  var w = [];
  if (compG && planeG !== 17) {
    w.push("G17");
    planeG = 17;
  }
  if (compG === "G41" || compG === "G42") {
    w.push(compG, compD, "G01");
    compOn = true; compG = "";
  } else if (compG === "G40") {
    w.push("G40", "G01");
    compOn = false; compG = ""; compD = "";
  } else {
    w.push("G01");
  }
  if (has(p.X)) { curX = Number(p.X); w.push("X" + fu(curX - orgX)); }
  if (has(p.Y)) { curY = Number(p.Y); w.push("Y" + fu(curY - orgY)); }
  if (has(p.Z)) { curZ = Number(p.Z); w.push("Z" + fu(curZ - orgZ)); }
  if (has(p.FeedRate)) {
    var fv = Number(p.FeedRate);
    if (!(fv > 0)) { msg(who() + "non-positive feed rate on linear move"); return; }
    w.push("F" + ff(fv));
  }
  block(w);
}

function emitRadiusArc(p, g, planeNum, sx, sy, sz, ex, ey, ez, cx, cy, cz, ccw, fVal) {
  var axes = planeNum === 18 ? [2, 0, 1] : (planeNum === 19 ? [1, 2, 0] : [0, 1, 2]);
  var u = axes[0], v = axes[1], h = axes[2];
  var from = [sx, sy, sz], to = [ex, ey, ez], center = [cx, cy, cz];
  var org = [orgX, orgY, orgZ];
  var du = from[u] - center[u], dv = from[v] - center[v];
  var eu = to[u] - center[u], ev = to[v] - center[v];
  var r0 = Math.sqrt(du * du + dv * dv);
  var r1 = Math.sqrt(eu * eu + ev * ev);
  if (!(r0 > 0) || !isFinite(r0) || !(r1 > 0) || !isFinite(r1)) return false;
  var r = (r0 + r1) * 0.5;
  if (Math.abs(r0 - r1) > Math.min(0.02, Math.max(0.002, r * 0.005))) return false;
  var a0 = Math.atan2(dv, du);
  var full = Math.abs(to[u] - from[u]) < 1e-5 && Math.abs(to[v] - from[v]) < 1e-5;
  var sweep = 2 * Math.PI;
  if (!full) {
    var a1 = Math.atan2(ev, eu);
    sweep = ccw ? (a1 - a0) : (a0 - a1);
    if (sweep <= 0) sweep += 2 * Math.PI;
  }
  var nearPi = Math.PI / 9;
  var n = 1;
  if (full || sweep >= 2 * Math.PI - nearPi) n = 4;
  else if (Math.abs(sweep - Math.PI) <= nearPi) n = 2;

  function buildSegs(count) {
    var segs = [], dir = ccw ? 1 : -1, prev = [from[0], from[1], from[2]];
    var rSigned = sweep / count > Math.PI + 1e-6 ? -r : r;
    var rp = Number(fu(r));
    if (!(rp > 0) || !isFinite(rp)) return null;
    for (var k = 1; k <= count; k++) {
      var next;
      if (k === count) {
        next = [to[0], to[1], to[2]];
      } else {
        var ak = a0 + dir * sweep * (k / count);
        next = [from[0], from[1], from[2]];
        next[u] = center[u] + r * Math.cos(ak);
        next[v] = center[v] + r * Math.sin(ak);
        if (has(from[h]) && has(to[h])) next[h] = from[h] + (to[h] - from[h]) * (k / count);
      }
      var su = Number(fu(prev[u] - org[u])), sv = Number(fu(prev[v] - org[v]));
      var pu = Number(fu(next[u] - org[u])), pv = Number(fu(next[v] - org[v]));
      var chord = Math.hypot(pu - su, pv - sv);
      if (!(chord > 0) || chord > 2 * rp - 1e-7) return null;
      if (count === 1 && chord > 2 * rp * 0.99) return null;
      segs.push({ pt: next, rSigned: rSigned });
      prev = next;
    }
    return segs;
  }

  var segs = buildSegs(n);
  if (!segs && n === 1) segs = buildSegs(Math.max(2, Math.ceil(sweep / (Math.PI / 2))));
  if (!segs) return false;
  for (var i = 0; i < segs.length; i++) {
    var q = segs[i].pt;
    var blk = [g, "X" + fu(q[0] - orgX), "Y" + fu(q[1] - orgY), "Z" + fu(q[2] - orgZ), "R" + fu(segs[i].rSigned)];
    if (fVal !== null && i === 0) blk.push("F" + ff(fVal));
    block(blk);
  }
  return true;
}

function onCircular() {
  var p = properties;
  startPreamble();
  if (!tiltOk(p)) return;
  if (inCycle) { msg(who() + "circular arc inside canned-cycle group"); return; }
  var ccw = String(p.CircularDirection || "CW").toUpperCase() === "CCW";
  var g = ccw ? "G03" : "G02";
  if (spindleStopped("arc move " + g)) return;
  if (compG === "G41" || compG === "G42" || compG === "G40") {
    msg(who() + "cutter compensation state change (" + compG + ") on arc is not permitted");
    return;
  }
  if (!xyFirst(has(p.StartX) ? p.StartX : curX, has(p.StartY) ? p.StartY : curY, has(p.StartZ) ? p.StartZ : curZ, true)) return;

  var pl = String(p.CircularPlane || "XY").toUpperCase();
  var planeNum = pl === "XZ" ? 18 : (pl === "YZ" ? 19 : 17);
  if (compOn && planeNum !== 17) {
    msg(who() + "plane change to " + pl + " while cutter compensation is engaged (G41/G42)");
    return;
  }
  if (planeNum !== planeG) {
    block(["G" + planeNum]);
    planeG = planeNum;
  }
  var ex = has(p.X) ? Number(p.X) : curX;
  var ey = has(p.Y) ? Number(p.Y) : curY;
  var ez = has(p.Z) ? Number(p.Z) : curZ;
  var sx = has(p.StartX) ? Number(p.StartX) : curX;
  var sy = has(p.StartY) ? Number(p.StartY) : curY;
  var sz = has(p.StartZ) ? Number(p.StartZ) : curZ;
  var ijk = p.IJKIncrement || {};
  var cx = has(ijk.I) ? Number(ijk.I) : sx;
  var cy = has(ijk.J) ? Number(ijk.J) : sy;
  var cz = has(ijk.K) ? Number(ijk.K) : sz;
  var fv = has(p.FeedRate) ? Number(p.FeedRate) : null;
  if (fv !== null && !(fv > 0)) { msg(who() + "non-positive feed rate on arc"); return; }

  if (optBool("useRadiusArcs", false) && emitRadiusArc(p, g, planeNum, sx, sy, sz, ex, ey, ez, cx, cy, cz, ccw, fv)) {
    curX = ex; curY = ey; curZ = ez;
    return;
  }

  var w = [g];
  if (has(p.X)) w.push("X" + fu(ex - orgX));
  if (has(p.Y)) w.push("Y" + fu(ey - orgY));
  if (has(p.Z) && (planeNum !== 17 || Math.abs(ez - sz) > 1e-9)) w.push("Z" + fu(ez - orgZ));
  if (planeNum === 17) {
    w.push("I" + fu(cx - sx), "J" + fu(cy - sy));
  } else if (planeNum === 18) {
    w.push("I" + fu(cx - sx), "K" + fu(cz - sz));
  } else {
    w.push("J" + fu(cy - sy), "K" + fu(cz - sz));
  }
  if (fv !== null) w.push("F" + ff(fv));
  block(w);
  curX = ex; curY = ey; curZ = ez;
}

function dwellWord(rawSec, cycleLabel) {
  if (!(rawSec > 0)) return "";
  var s4 = rawSec.toFixed(4);
  if (!(Number(s4) > 0)) {
    msg(who() + cycleLabel + " dwell " + rawSec + "s rounds to 0 on Hurco WinMax ISNC");
    return "";
  }
  return "P" + pt(trimNum(s4));
}

function onDwell(seconds) {
  if (cycleDwellConsumed) { cycleDwellConsumed = false; return; }
  startPreamble();
  endCycle();
  var s = Number(seconds);
  if (!(s > 0)) return;
  var pw = dwellWord(s, "G04");
  if (pw) block(["G04", pw]);
}

function onCycle() {
  var p = properties;
  startPreamble();
  if (!tiltOk(p)) return;
  if (spindleStopped("canned cycle")) return;
  var fRaw = has(p.FeedRate) ? p.FeedRate : p.ChainFeedRate;
  if (!has(fRaw) || !(Number(fRaw) > 0)) {
    msg(who() + "canned cycle has no feed rate");
    return;
  }
  if (!xyFirst(p.X, p.Y, has(p.R) ? p.R : p.Z, false)) return;
  restorePlane();

  var kind = String(p.CycleType || "DRILL").toUpperCase();
  var tapMode = String(p.TappingMode || "rigid").toLowerCase();
  var rawDwell = has(p.P) ? Number(p.P) : 0;
  cycleDwellConsumed = rawDwell > 0;
  var g = "G81";
  var qStr = "";
  var pStr = "";

  if (kind === "DRILL" || kind === "SPOT") {
    if (rawDwell > 0) {
      g = "G82";
      pStr = dwellWord(rawDwell, "G82");
    } else {
      g = "G81";
    }
  } else if (kind === "PECK" || kind === "DEEP") {
    if (rawDwell > 0) {
      msg(who() + "dwell P is not valid on Hurco WinMax ISNC " + (kind === "PECK" ? "G73" : "G83") + " peck cycle");
      return;
    }
    var q = has(p.Q) ? Number(p.Q) : 0;
    if (!(q > 0)) { msg(who() + kind + " peck cycle requires positive peck Q"); return; }
    g = kind === "PECK" ? "G73" : "G83";
    qStr = "Q" + fu(q);
  } else if (kind === "TAP" && tapMode === "floating") {
    if (rawDwell > 0) {
      msg(who() + "dwell P is not valid on Hurco WinMax ISNC floating tapping (G84)");
      return;
    }
    g = "G84";
  } else if (kind === "TAP") {
    if (rawDwell > 0) {
      msg(who() + "dwell P is not valid on Hurco WinMax ISNC rigid tapping (G84.2/G84.3)");
      return;
    }
    var m = typeof machine !== "undefined" ? machine : null;
    if (m && m.rigidTapping === false) {
      var mn = m.name ? String(m.name) : "the machine";
      msg(who() + "rigid tapping (" + mn + ") is unavailable because rigidTapping is false (use FloatingTap for G84)");
      return;
    }
    g = spinRev ? "G84.3" : "G84.2";
  } else if (kind === "BORE") {
    if (rawDwell > 0) {
      msg(who() + "G86 (bore) has no dwell word on Hurco WinMax ISNC (" + trimNum(rawDwell.toFixed(4)) + " s)");
      return;
    }
    g = "G86";
  } else if (kind === "BACKBORE" || kind === "REAM") {
    if (rawDwell > 0) {
      g = "G89";
      pStr = dwellWord(rawDwell, "G89");
    } else {
      g = "G85";
    }
  } else {
    msg(who() + "unsupported canned cycle type '" + kind + "'");
    return;
  }

  var xVal = Number(p.X);
  var yVal = Number(p.Y);
  var zVal = Number(p.Z);
  var rVal = has(p.R) ? Number(p.R) : 0;
  var fVal = Number(fRaw);
  var sig = [g, fu(zVal - orgZ), fu(rVal - orgZ), qStr, pStr, ff(fVal)].join("|");

  if (!inCycle || sig !== cycleSig) {
    var blk = ["G99", g, "X" + fu(xVal - orgX), "Y" + fu(yVal - orgY), "Z" + fu(zVal - orgZ)];
    if (qStr) blk.push(qStr);
    if (pStr) blk.push(pStr);
    blk.push("R" + fu(rVal - orgZ), "F" + ff(fVal));
    block(blk);
    inCycle = true;
    cycleSig = sig;
  } else {
    block(["X" + fu(xVal - orgX), "Y" + fu(yVal - orgY)]);
  }
  cycX = xVal; cycY = yVal; cycR = rVal;
  curX = xVal; curY = yVal; curZ = rVal;
}

function onCycleEnd() {
  cycleDwellConsumed = false;
}

function probeAxisInfo(p) {
  var dx = Math.abs(num(p.X, 0) - num(p.StartX, 0));
  var dy = Math.abs(num(p.Y, 0) - num(p.StartY, 0));
  var dz = Math.abs(num(p.Z, 0) - num(p.StartZ, 0));
  if (dy >= dx && dy >= dz) return { v: "#5062", exp: num(p.ExpectY, num(p.Y, 0)) - orgY };
  if (dz > dx && dz > dy) return { v: "#5063", exp: num(p.ExpectZ, num(p.Z, 0)) - orgZ };
  return { v: "#5061", exp: num(p.ExpectX, num(p.X, 0)) - orgX };
}

function onProbeMove() {
  var p = properties;
  startPreamble();
  endCycle();
  if (wcsRotActive || Math.abs(wcsRot) > 1e-6) {
    msg(who() + "probing with active G68 rotation is refused");
    return;
  }
  if (!xyFirst(p.X, p.Y, p.Z, false)) return;
  restorePlane();
  block(["G31", "X" + fu(Number(p.X) - orgX), "Y" + fu(Number(p.Y) - orgY), "Z" + fu(Number(p.Z) - orgZ), "F" + ff(Number(p.Feed))]);
  var slot = Math.round(num(p.Measure, -1));
  if (slot >= 0) {
    var info = probeAxisInfo(p);
    var lbl = probeLabelNext++;
    block(["IF[ABS[" + info.v + "-" + trimNum(info.exp.toFixed(3)) + "]GE0.001]GOTO" + lbl]);
    block(["#3000=2(PROBE NO CONTACT)"]);
    block(["N" + lbl]);
    block(["#" + (100 + slot) + "=" + info.v]);
  }
  curX = Number(p.X); curY = Number(p.Y); curZ = Number(p.Z);
}

function onProbeGuard() {
  var p = properties;
  startPreamble();
  endCycle();
  if (!xyFirst(p.X, p.Y, p.Z, false)) return;
  restorePlane();
  block(["G31", "X" + fu(Number(p.X) - orgX), "Y" + fu(Number(p.Y) - orgY), "Z" + fu(Number(p.Z) - orgZ), "F" + ff(Number(p.Feed))]);
  curX = Number(p.X); curY = Number(p.Y); curZ = Number(p.Z);
}

function onProbeRetreat() {
  var p = properties;
  startPreamble();
  endCycle();
  block(["G91", "G01", "X" + fu(num(p.DeltaX, 0)), "Y" + fu(num(p.DeltaY, 0)), "Z" + fu(num(p.DeltaZ, 0)), "F" + ff(Number(p.Feed))]);
  block(["G90"]);
}

function onProbeResult() {
  var p = properties;
  startPreamble();
  endCycle();
  var axes = Array.isArray(p.Axes) ? p.Axes : [];
  var tol = num(p.Tolerance, 0.5);
  var setX = false, setY = false, setZ = false;
  for (var i = 0; i < axes.length; i++) {
    var ax = axes[i];
    var letter = String(ax.Axis || "X").toUpperCase();
    var devVar = letter === "Z" ? 192 : (letter === "Y" ? 191 : 190);
    var ms = Array.isArray(ax.Measures) ? ax.Measures : [];
    if (ms.length === 0) continue;
    var terms = [];
    for (var j = 0; j < ms.length; j++) terms.push("#" + (100 + Math.round(Number(ms[j]))));
    var avg = ms.length === 1 ? terms[0] : "[" + terms.join("+") + "]/" + ms.length;
    var org = letter === "Z" ? orgZ : (letter === "Y" ? orgY : orgX);
    var exp = trimNum(((Number(ax.Expected) - org) * unitScale).toFixed(unitInch ? 4 : 3));
    block(["#" + devVar + "=" + avg + "-" + exp]);
    var lbl = probeLabelNext++;
    block(["IF[ABS[#" + devVar + "]LE" + trimNum((tol * unitScale).toFixed(3)) + "]GOTO" + lbl]);
    block(["#3000=3(PROBE TOLERANCE EXCEEDED)"]);
    block(["N" + lbl]);
    if (letter === "X") setX = true;
    if (letter === "Y") setY = true;
    if (letter === "Z") setZ = true;
  }
  var target = has(p.SetWorkOffset) ? Math.round(Number(p.SetWorkOffset)) : -1;
  if (target < 0) {
    note("CHECK PROBE RESULT #190..");
    block(["M00"]);
    return;
  }
  if (target > 5) {
    msg(who() + "G10 L2 probe offset target " + (target + 1) + " exceeds Hurco WinMax ISNC G54..G59 range (1..6)");
    return;
  }
  var base = 5201 + wcsIndex * 20;
  var g10 = ["G10", "L2", "P" + (target + 1)];
  if (setX) g10.push("X[#" + base + "+#190]");
  if (setY) g10.push("Y[#" + (base + 1) + "+#191]");
  if (setZ) g10.push("Z[#" + (base + 2) + "+#192]");
  block(g10);
  note("SET " + workOffsetName(target + 1) + " FROM PROBE");
}

function onClose() {
  start();
  endCycle();
  restorePlane();
  if (wcsRotActive) {
    block(["G69"]);
    wcsRotActive = false;
  }
  if (compOn || compG) {
    msg(who() + "program ends with cutter compensation active (missing G40)");
  }
  if (coolant) { block(["M9"]); coolant = ""; }
  if (spindleOn) { block(["M5"]); spindleOn = false; }
  safeRetractBlock();
  var eu = optStr("EndUnits", "").toLowerCase();
  if (eu === "inch" && !unitInch) block(["G20"]);
  else if (eu === "mm" && unitInch) block(["G21"]);
  block(["M30"]);
  writeln("%");
}
