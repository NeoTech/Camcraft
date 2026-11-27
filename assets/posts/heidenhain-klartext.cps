// Original CamCraft Klartext post. HEIDENHAIN iTNC 530 Conversational Format,
// 533 190-22 (9/2006), NC 340 490-03: §§4.4, 5.2, 6.4, 7.2, 7.4, 8.9, 14.4.
// https://content.heidenhain.de/doku/tnc_guide/pdf_files/iTNC530/34049x-neu/en/bhb.pdf
// Machine installation selects the qualified language profile and change procedure.
description = "Heidenhain Klartext (metric 3-axis milling)";
vendor = "CamCraft";
legal = "Original clean-room work";
certificationLevel = 1;
minimumRevision = 1;
extension = "H";
setCodePage = "utf-8";
capabilities = 1;
serves = ["mill"];
programUnits = "mm";
coolantModes = true;
failOnWarning = true;
workOffsetIndex = true; floatingTapping = true; // #4227: cycle 206 needs the explicit floating-holder choice
workOffsetMax = 6; manualToolChange = ["manual-retract-top"]; // #4085: it writes the stop and checks for this change

var blockNumber = 0;
var programName = "";
var section = Object.create(null);
var inSection = false;
var moved = false;
var installedTool = null;
var setupSignature = null;
var lastPoint = null;

function refuse(reason) {
  // A script error also prevents the standalone post from saving a partial file.
  throw new Error("Heidenhain Klartext: " + reason);
}
function block(text) {
  writeln(String(blockNumber++) + " " + text);
}
function cleanText(value) {
  // User text appears only in single-line comments.
  return String(value || "").replace(/[^A-Za-z0-9 _.-]/g, " ").trim().slice(0, 48);
}
function required(value, field) {
  if (typeof value !== "number" || !isFinite(value)) refuse("missing or invalid " + field);
  return value;
}
function decimal(value, places, limit, field) {
  required(value, field);
  if (Math.abs(value) > limit) refuse(field + " exceeds the control range");
  // Input coordinates are millimetres, including sub-grid origins and stock.
  // Round once on the control grid after translating into the preset frame.
  return Number(value.toFixed(places));
}
function local(value, axis) {
  return decimal(required(value, axis) - section["Origin" + axis], dialect ? dialect.decimals : 4, dialect && dialect.decimals === 3 ? 99999.999 : 99999.9999, axis);
}
function coord(axis, value) {
  var n = local(value, axis);
  return axis + (n < 0 ? "" : "+") + n;
}
function xyz(x, y, z) {
  return coord("X", x) + " " + coord("Y", y) + " " + coord("Z", z);
}
function rate(value, maximum, field) {
  var word = decimal(value, 3, 99999.999, field);
  if (word <= 0 || (maximum > 0 && Math.max(value, word) > maximum)) {
    refuse(field + " must be positive and within the machine maximum");
  }
  return word;
}
function workOffsetName(n) {
  return Number.isInteger(n) && n >= 1 && n <= workOffsetMax ? "preset " + n : "";
}

function onOpen() {
  var controller = machine.controllerIdentity;
  if (!controller) {
    refuse("unsupported or missing controller profile; select iTNC 530 NC 340490-03");
  }
  dialectCheck(controller);
  // #4114/#4095: the manual quill procedure (M140, M0 stops) is verified on
  // one installation and NC software only, the same check as the DIN/ISO post.
  if (controller.model !== "iTNC 530" || controller.nc_software !== "340490-03") {
    refuse("controller profile " + dialect.profile + " (" + controller.model + " NC " + controller.nc_software +
      ") is not verified for the manual quill procedure; select iTNC 530 NC 340490-03");
  }
  if (machine.model !== "Fehlmann Picomax 55 (iTNC 530)") {
    refuse("the manual quill procedure requires the Fehlmann Picomax 55 (iTNC 530) installation");
  }
  var axes = Array.isArray(machine.linearAxes) ? machine.linearAxes.slice().sort() : [];
  if (machine.kind !== "mill" || axes.length !== 3 ||
      axes[0] !== "X" || axes[1] !== "Y" || axes[2] !== "Z" ||
      machine.rotaries.length || machine.spindles !== 1) {
    refuse("requires a single-spindle XYZ milling machine");
  }
  if (machine.toolChangeMode !== "manual-retract-top") {
    refuse("requires the manual-retract-top tool change configuration");
  }
  var minimum = required(machine.spindleMin, "minimum spindle speed");
  var maximum = required(machine.spindleMax, "maximum spindle speed");
  if (minimum <= 0 || maximum < minimum) {
    refuse("the installation must state a positive spindle speed range");
  }
  var configuredTolerance = (typeof machine.arcTolerance === "number" && machine.arcTolerance > 0)
    ? machine.arcTolerance
    : (machine.controller && typeof machine.controller.arcTolerance === "number" && machine.controller.arcTolerance > 0)
      ? machine.controller.arcTolerance
      : 0.016;
  if (configuredTolerance > 0.016) {
    refuse("arc tolerance " + configuredTolerance + " mm exceeds the 0.016 mm controller maximum (MP 7431)");
  }
  programName = program.numbered ? String(program.number) : "CAMCRAFT";
  block("BEGIN PGM " + programName + " MM");
}
// #4095: the dialect of the stated control comes from the shared resolver
// (camcraft_post::post::heidenhain_dialect, machine.heidenhainDialect;
// docs/decisions/4095-heidenhain-dialect-resolver.md). This post writes
// L M140 MB MAX, CYCL DEF 247 with Q339 (a preset-table row) and M137, so
// each of these must exist with that meaning on the control. Coordinates
// use the control's decimals.
var dialect = null;
function dialectCheck(controller) {
  var d = machine.heidenhainDialect;
  var who = controller.model + " NC " + controller.nc_software;
  if (!d || typeof d !== "object" || d.error) {
    refuse("unsupported controller profile: " + (d && d.error ? String(d.error) : "no dialect for " + who));
  }
  var lacks = [];
  if (d.m140 !== "yes") lacks.push("M140 MB MAX");
  if (d.cycle247 !== "presetTable") lacks.push("cycle 247 as a preset-table row");
  if (d.m136m137 !== "yes") lacks.push("M137");
  if (d.decimals !== 3 && d.decimals !== 4) lacks.push("3 or 4 coordinate decimals");
  if (lacks.length) {
    refuse("controller profile " + d.profile + " (" + who + ") does not confirm " + lacks.join(", ") +
      ", which this post writes");
  }
  dialect = d;
}
function onParameter(name, value) {
  if (inSection) refuse("unexpected parameter inside a section: " + name);
  // A positive list prevents a newly introduced motion or controller request
  // from silently running through the baseline profile as plain milling.
  switch (name) {
    case "StartX": case "StartY": case "StartZ":
    case "OriginX": case "OriginY": case "OriginZ":
    case "StockXYLeft": case "StockXYRight": case "StockXYFront": case "StockXYBack":
    case "StockZBottom": case "StockZTop": case "ClearancePlane":
    case "SpindleSpeed": case "WorkOffset": case "Coolant":
    case "SetupName": case "OperationName":
      section[name] = value;
      return;
    case "ToolKind": case "ToolDiameter": case "ToolCornerRadius":
      if (section[name] !== undefined && section[name] !== value) {
        refuse("conflicting " + name + " declarations in one section");
      }
      section[name] = value;
      return;
    case "OperationKind":
    case "RetractPlane": case "TopOfStock":
    // #2769: the stock's shape describes the box already stated; this
    // post's BLK FORM reads StockZBottom, not these.
    case "StockShape": case "StockShapeBottom": case "StockShapeDiameter":
      return;
    case "SectionMode":
      if (value === "3-axis") return;
      break;
  }
  refuse("unsupported parameter " + name + " (" + value + ")");
}
function onToolChange(number, name) {
  if (inSection || !Number.isInteger(number) || number < 1 || number > 32767) {
    refuse("missing or invalid tool number");
  }
  // #4087: the machine's tool table (MP 7260 lines on the Picomax).
  if (machine.toolNumberMax > 0 && number > machine.toolNumberMax) {
    refuse("tool number " + number + " is above the machine tool table maximum " + machine.toolNumberMax);
  }
  if (section.tool !== undefined) refuse("multiple TOOL declarations in one section");
  section.tool = number;
  section.toolName = name;
}
var toolDefinitions = Object.create(null);
function checkToolDefinition() {
  var kind = typeof section.ToolKind === "string" ? section.ToolKind.toLowerCase() : "";
  // #4227: a drill or a tap cuts only in a drilling cycle (axialOnly).
  if (["flat end mill", "ball-nose", "bull-nose", "chamfer mill", "drill", "tap"].indexOf(kind) === -1) {
    refuse("unsupported tool kind " + (kind || "(unstated)"));
  }
  var diameter = required(section.ToolDiameter, "ToolDiameter");
  var cornerRadius = section.ToolCornerRadius === undefined ? 0 : section.ToolCornerRadius;
  if (diameter <= 0) {
    refuse("ToolDiameter must be positive");
  }
  if (required(cornerRadius, "ToolCornerRadius") < 0 || cornerRadius > diameter / 2) {
    refuse("ToolCornerRadius requires a compatible ToolDiameter");
  }
  // Compare the original identity, before shortening it for a program comment.
  // Keep every slot for the entire program, including tools used again later.
  var definition = JSON.stringify([section.toolName, kind, diameter, cornerRadius]);
  if (toolDefinitions[section.tool] !== undefined && toolDefinitions[section.tool] !== definition) {
    refuse("conflicting definitions for tool T" + section.tool + " (name, kind, diameter or corner radius)");
  }
  toolDefinitions[section.tool] = definition;
}
function floodRequested() {
  if (section.Coolant === "off") return false;
  if (section.Coolant !== undefined && section.Coolant !== "flood") {
    refuse("unsupported coolant " + section.Coolant);
  }
  // The installation must actually provide the standard flood circuit.
  if (!machine.coolants.some(function(c) {
    return c && typeof c === "object" && String(c.name).toLowerCase() === "flood" &&
      /^M0?8$/i.test(c.mCode);
  })) refuse("flood requires a machine Flood entry with M8");
  return true;
}
function writeSetup() {
  var lo = [section.StockXYLeft, section.StockXYFront, section.StockZBottom];
  var hi = [section.StockXYRight, section.StockXYBack, section.StockZTop];
  for (var i = 0; i < 3; ++i) {
    var axis = "XYZ"[i];
    required(section["Origin" + axis], "Origin" + axis);
    var size = local(required(hi[i], "stock maximum " + axis), axis) -
      local(required(lo[i], "stock minimum " + axis), axis);
    if (size < 0.05 || size > 99999.999) refuse("BLK FORM stock size outside graphical range on " + axis);
  }
  if (!workOffsetName(section.WorkOffset)) refuse("missing or unsupported preset (WorkOffset)");
  var signature = JSON.stringify([section.SetupName, section.WorkOffset, lo, hi,
    section.OriginX, section.OriginY, section.OriginZ]);
  if (setupSignature !== null) {
    if (signature !== setupSignature) refuse("one program requires one setup, blank and preset");
    return;
  }
  setupSignature = signature;
  block("BLK FORM 0.1 Z " + xyz(lo[0], lo[1], lo[2]));
  block("BLK FORM 0.2 " + xyz(hi[0], hi[1], hi[2]));
  if (program.comment) block("; " + cleanText(program.comment));
  block("; CONFIRM XYZ MODE - QUILL COUPLED - W FIXED");
  block("; CONFIRM SPINDLE STOPPED - NO TILT");
  block("; CANCEL ACTIVE COORDINATE TRANSFORMATIONS");
  block("; CONFIRM QUILL AT UPPER CHANGE POSITION");
  block("; VERIFY +Z RETRACT PATH AND CHANGE SPACE");
  block("; CHECK FULL-RETRACT XY PATH FOR ALL TOOLS");
  block("; VERIFY TOOL TABLE AND PRESET " + section.WorkOffset);
  block("; CHECK PRESET X Y Z FILLED AND ROTATION ZERO");
  block("M0");
  block("CYCL DEF 247 DATUM SETTING ~");
  writeln("  Q339=+" + section.WorkOffset);
  block("M137"); // F is mm/min even if the previous program selected feed per revolution.
}
function onSection() {
  if (inSection) refuse("nested operation sections");
  if (section.tool === undefined) refuse("operation has no tool call");
  checkToolDefinition();
  var rpm = rate(section.SpindleSpeed, machine.spindleMax, "SpindleSpeed");
  if (Math.min(section.SpindleSpeed, rpm) < machine.spindleMin) {
    refuse("SpindleSpeed is below the machine minimum");
  }
  var flood = floodRequested();
  writeSetup();
  var clearance = required(section.ClearancePlane, "ClearancePlane");
  var startZ = required(section.StartZ, "StartZ");
  if (clearance <= section.StockZTop || startZ < clearance ||
      local(clearance, "Z") <= local(section.StockZTop, "Z")) {
    refuse("the operation must start at or above ClearancePlane, above the stock");
  }
  var startXY = coord("X", section.StartX) + " " + coord("Y", section.StartY);
  var startHeight = coord("Z", startZ);
  if (section.OperationName) block("; OPERATION " + cleanText(section.OperationName));
  // At cold start the incoming tool and active axis are unknown. The first
  // stop requires the quill at the change position before the operator loads
  // the tool. Later changes retract under the previous explicit Z tool call.
  if (installedTool !== section.tool) {
    if (installedTool !== null) block("L M140 MB MAX");
    // The active traverse range or DCM can stop M140 short of the physical
    // change position. Check the actual quill and drive state at every change.
    block("; CHECK DRIVES OFF BEFORE TOUCHING TOOL");
    block("; CHECK QUILL AT PHYSICAL CHANGE POSITION");
    block("; ABORT IF EITHER CHECK FAILS");
    block("; THEN INSTALL TOOL " + section.tool + " " + cleanText(section.toolName));
    block("M0");
  }
  block("TOOL CALL " + section.tool + " Z S" + rpm);
  installedTool = section.tool;
  block("L M140 MB MAX"); // Positive Z is now established by TOOL CALL.
  // M140 has no completion signal for the intended tip clearance. The tool
  // may extend farther than its predecessor or stop short of upper travel.
  block("; CHECK ACTUAL TIP HEIGHT AND ENTIRE XY PATH");
  block("; ABORT IF TOOL OR HOLDER CANNOT CLEAR");
  block("M0");
  block("M3");
  block(flood ? "M8" : "M9");
  // Establish the programmed start before the first record, including an arc
  // or a plunge as the first record. Stay at the upper limit during XY travel,
  // then descend only in the column of the verified start point.
  block("L " + startXY + " R0 FMAX");
  block("L " + startHeight + " R0 FMAX");
  lastPoint = [section.StartX, section.StartY, startZ];
  moved = false;
  inSection = true;
}

function motion() {
  if (!inSection) refuse("motion outside an operation section");
  var p = properties;
  if (p.A !== undefined || p.B !== undefined || p.C !== undefined) refuse("rotary or tilted move");
  // Stream starts can be finer than a serialized motion endpoint. A real
  // discontinuity is an error: C arcs start from the last commanded point.
  if (Math.abs(required(p.StartX, "StartX") - lastPoint[0]) > 1e-8 ||
      Math.abs(required(p.StartY, "StartY") - lastPoint[1]) > 1e-8 ||
      Math.abs(required(p.StartZ, "StartZ") - lastPoint[2]) > 1e-8) {
    refuse("motion does not start at the previous programmed position");
  }
  lastPoint = [p.X, p.Y, p.Z];
  moved = true;
  return p;
}
function onRapid() {
  if (!inSection) refuse("motion outside an operation section");
  if (holeMove(true)) return;
  var p = motion();
  block("L " + xyz(p.X, p.Y, p.Z) + " R0 FMAX");
}
function onLinear() {
  if (!inSection) refuse("motion outside an operation section");
  if (holeMove(false)) return;
  axialOnly("feed");
  var p = motion();
  block("L " + xyz(p.X, p.Y, p.Z) + " R0 F" + rate(p.FeedRate, machine.feedRateMax, "feed"));
  dwellAtDepth();
}
function onCircular() {
  // #3442: an arc whose written ends coincide is no C block (arcFate,
  // PRELUDE): a short one is an L move, or no block (the last programmed
  // point still moves on). A near-full turn keeps the refusal below.
  var fate = arcFate(function (k, v) { return local(v, k); });
  if (fate === "linear") { onLinear(); return; }
  if (fate === "none") { motion(); return; }
  closeHole();
  axialOnly("arc");
  var p = motion();
  if (p.CircularPlane !== "XY" || p.Z !== p.StartZ) refuse("only constant-Z XY arcs are supported");
  var centre = p.IJKIncrement || {};
  var centreWords = coord("X", centre.I) + " " + coord("Y", centre.J);
  var endWords = coord("X", p.X) + " " + coord("Y", p.Y);
  var cx = local(centre.I, "X"), cy = local(centre.J, "Y");
  var startX = local(p.StartX, "X"), startY = local(p.StartY, "Y");
  var endX = local(p.X, "X"), endY = local(p.Y, "Y");
  if (startX === endX && startY === endY && (p.StartX !== p.X || p.StartY !== p.Y)) {
    refuse("rounding would turn the XY arc into a full circle");
  }
  var radius = Math.hypot(startX - cx, startY - cy);
  var endRadius = Math.hypot(endX - cx, endY - cy);
  // MP 7431 circle end point tolerance: default 0.016 mm for the Picomax 55 (iTNC 530).
  var arcTolerance = (typeof machine.arcTolerance === "number" && machine.arcTolerance > 0)
    ? machine.arcTolerance
    : (machine.controller && typeof machine.controller.arcTolerance === "number" && machine.controller.arcTolerance > 0)
      ? machine.controller.arcTolerance
      : 0.016;
  if (arcTolerance > 0.016) {
    refuse("arc tolerance " + arcTolerance + " mm exceeds the 0.016 mm controller maximum (MP 7431)");
  }
  if (radius < 0.001) {
    refuse("inconsistent XY arc centre or radius: radius " + radius.toFixed(4) + " mm is below 0.001 mm limit");
  }
  if (Math.abs(radius - endRadius) > arcTolerance + 1e-6) {
    refuse("inconsistent XY arc centre or radius: start radius " + radius.toFixed(4) + " mm, end radius " + endRadius.toFixed(4) + " mm, limit " + arcTolerance + " mm");
  }
  if (p.CircularDirection !== "CW" && p.CircularDirection !== "CCW") refuse("unknown arc direction");
  var direction = p.CircularDirection === "CW" ? "DR-" : "DR+";
  block("CC " + centreWords);
  block("C " + endWords + " " + direction + " F" + rate(p.FeedRate, machine.feedRateMax, "arc feed"));
}
function onSectionEnd() {
  if (!inSection || !moved) refuse("operation section is empty or has no start");
  closeHole();
  owedDwell();
  cycleSig = "";
  if (lastPoint[2] < section.ClearancePlane ||
      local(lastPoint[2], "Z") < local(section.ClearancePlane, "Z")) {
    refuse("operation ends below ClearancePlane; provide a verified retract before stopping the spindle");
  }
  block("M9");
  block("M5");
  inSection = false;
  section = Object.create(null);
}
function onClose() {
  if (inSection || !setupSignature || Object.keys(section).length) refuse("incomplete program");
  block("M2");
  block("END PGM " + programName + " MM");
}

// ------------------------------------------------------------ cycles
// #4227: iTNC 530 Cycle Programming (NC 340 49x): cycles 9 DWELL TIME,
// 200 DRILLING, 206 TAPPING NEW and 207 RIGID TAPPING NEW, at the
// parameter set of the resolver's drillC200 "q211" level (no Q395).
// A native cycle replaces only the hole chain that it does itself, and
// each replaced move is compared with the cycle's own motion:
// - DRILL / SPOT, cycle 200 with Q202 = the whole depth: one feed to the
//   depth, the dwell Q211, FMAX back to Q200 (= R). Q204 = Q200, so the
//   second clearance move is no move.
// - TAP, cycle 207 (rigid) or 206 (floating): feed to the depth at the
//   pitch feed, the spindle reverses, feed back to Q200 (= R).
// PECK and DEEP post the checked chain as L blocks: the native peck
// re-approach is not provably the chain (#4224,
// docs/decisions/4224-heidenhain-peck-chain.md). Their dwell, and only
// theirs, is cycle 9 after the last feed to the depth.
// docs/decisions/4227-klartext-cycles.md records the choices.
var CYCLE_TOL = 1e-6;
var hole = null;          // the open hole: {x, y, r, steps, native, name}
var cycleSig = "";        // the last CYCL DEF written in this section
var pendingDwell = null;  // {z, s}: cycle 9 owed after the feed to the depth
var dwellTaken = 0;       // the dwell the open cycle wrote (Q211 or cycle 9)
function qword(n, v) {
  // Increments and Q values carry four decimals, with an explicit sign.
  var s = Number(v).toFixed(4).replace(/0+$/, "").replace(/\.$/, "");
  if (s === "-0" || !s) s = "0";
  return "Q" + n + "=" + (s.charAt(0) === "-" ? s : "+" + s);
}
function toolKindWord() {
  return typeof section.ToolKind === "string" ? section.ToolKind.toLowerCase() : "";
}
function cycleDef(text, words) {
  block("CYCL DEF " + text + " ~");
  for (var i = 0; i < words.length; ++i) writeln("  " + words[i] + (i + 1 < words.length ? " ~" : ""));
}
function sameHole(p) {
  return hole !== null && Math.abs(p.X - hole.x) < CYCLE_TOL && Math.abs(p.Y - hole.y) < CYCLE_TOL;
}
// A move of the open hole's chain. Native: compare it with the cycle's next
// own move and write nothing (true). Explicit chain: false, the caller posts it.
function holeMove(rapid) {
  var p = properties;
  if (hole === null) return false;
  if (!sameHole(p) || p.Z > hole.r + CYCLE_TOL) {
    if (hole.steps.length) refuse(hole.name + " ends before the hole chain returns to R");
    hole = null;
    return false;
  }
  if (!hole.native) return false;
  // The tool stands at R after the cycle: a rapid there (the hop when the
  // hop plane is R) moves nothing.
  if (!hole.steps.length && rapid && Math.abs(p.Z - hole.r) <= CYCLE_TOL) { motion(); return true; }
  var step = hole.steps.shift();
  var feed = rapid ? null : p.FeedRate;
  if (!step || step.rapid !== rapid || Math.abs(p.Z - step.z) > CYCLE_TOL ||
      (!rapid && !(Math.abs(feed - step.f) <= CYCLE_TOL * Math.max(1, step.f)))) {
    refuse("the hole chain is not the motion of " + hole.name);
  }
  motion();
  return true;
}
function closeHole() {
  if (hole !== null && hole.steps.length) refuse(hole.name + " ends before the hole chain returns to R");
  hole = null;
}
// A drill or tap cuts only along the axis of a drilling cycle's hole.
function axialOnly(what) {
  var kind = toolKindWord();
  if (kind !== "drill" && kind !== "tap") return;
  var p = properties;
  if (kind === "tap" || !sameHole(p) || Math.abs(p.StartX - p.X) > CYCLE_TOL ||
      Math.abs(p.StartY - p.Y) > CYCLE_TOL || what === "arc") {
    refuse("a " + kind + " tool cuts only inside its drilling cycle (" + what + ")");
  }
}
function dwellWords(s) {
  if (dialect.dwell !== "cycle9") {
    refuse("controller profile " + dialect.profile + " does not confirm cycle 9 DWELL TIME, which this post writes");
  }
  var t = Number(s.toFixed(3));
  if (!(t > 0) || t > 3600) refuse("dwell " + s + " s is outside 0.001 to 3600 s");
  block("CYCL DEF 9.0 DWELL TIME");
  block("CYCL DEF 9.1 DWELL " + t);
  cycleSig = ""; // define the machining cycle again before its next call
}
function dwellAtDepth() {
  var p = properties;
  if (pendingDwell === null || !sameHole(p) || Math.abs(p.Z - pendingDwell.z) > CYCLE_TOL) return;
  dwellWords(pendingDwell.s);
  pendingDwell = null;
}
function owedDwell() {
  if (pendingDwell !== null) refuse("drilling dwell without a feed move to the hole bottom");
}
function onCycle() {
  if (!inSection) refuse("drilling cycle outside an operation section");
  closeHole();
  owedDwell();
  dwellTaken = 0;
  var p = properties, type = String(p.CycleType);
  var tap = type === "TAP", floating = tap && p.TappingMode === "floating";
  var name = tap ? (floating ? "cycle 206" : "cycle 207") : (type === "PECK" || type === "DEEP" ? type + " chain" : "cycle 200");
  function bad(why) { refuse("drilling cycle " + type + ": " + why); }
  if (["SPOT", "DRILL", "PECK", "DEEP", "TAP"].indexOf(type) === -1) bad("unsupported cycle kind");
  var kind = toolKindWord();
  if (tap !== (kind === "tap")) bad(tap ? "a tapping cycle requires a tap tool" : "a tap tool cuts only in a tapping cycle");
  if (p.A !== undefined || p.B !== undefined || p.C !== undefined) bad("rotary or tilted hole");
  var x = required(p.X, "hole X"), y = required(p.Y, "hole Y"), r = required(p.R, "hole R");
  var bottom = required(p.Z, "hole depth"), entry = required(p.EntryZ, "hole entry");
  // The tool stands over the hole at R: the stream rapids there first.
  if (Math.abs(lastPoint[0] - x) > CYCLE_TOL || Math.abs(lastPoint[1] - y) > CYCLE_TOL ||
      Math.abs(lastPoint[2] - r) > CYCLE_TOL) {
    bad("the tool is not at the hole's R plane when the cycle starts");
  }
  var clearance = r - entry, depth = bottom - entry;
  if (clearance < 0 || clearance > 99999.9999 || depth >= 0 || depth < -99999.9999) {
    bad("needs a nonnegative R clearance and a negative depth below the hole entry");
  }
  if (finite(p.ChainBottomZ) && Math.abs(p.ChainBottomZ - bottom) > CYCLE_TOL) bad("the depth disagrees with the hole chain bottom");
  var dwell = finite(p.P) ? Number(p.P) : 0;
  if (!(dwell >= 0) || dwell > 3600) bad("invalid dwell");
  if (type === "PECK" || type === "DEEP") {
    // #4224: the checked chain posts as L blocks; cycle 9 dwells at its depth.
    hole = { x: x, y: y, r: r, steps: [], native: false, name: name };
    if (dwell > 0) pendingDwell = { z: bottom, s: dwell };
    dwellTaken = dwell;
    return;
  }
  var feedIn = finite(p.FeedRate) ? p.FeedRate : p.ChainFeedRate;
  var feed = rate(feedIn, machine.feedRateMax, "cycle feed");
  var words = [qword(200, clearance), qword(201, depth)];
  var text;
  if (tap) {
    var rpm = rate(section.SpindleSpeed, machine.spindleMax, "SpindleSpeed");
    if (rpm !== Math.round(rpm)) bad("tapping needs an integral spindle speed to state the pitch");
    if (floating) {
      if (Math.abs(feedIn - feed) > 0.0000005) bad("the floating tapping feed cannot be written at the control's feed resolution");
      if (dwell > 0.5) bad("cycle 206 dwell must be at most 0.5 s to avoid wedging the tap");
      text = "206 TAPPING NEW";
      words.push(qword(206, feed), qword(211, dwell));
    } else {
      var pitch = feedIn / rpm;
      if (pitch < 0.0001 || pitch > 99.9999 || Math.abs(pitch - Math.round(pitch * 10000) / 10000) > 0.0000005) {
        bad("the tapping feed and spindle speed give no pitch at the control's resolution");
      }
      if (machine.rigidTapping === false) bad("rigid tapping is unavailable on this machine; select a floating holder explicitly if installed");
      if (dwell !== 0) bad("cycle 207 has no dwell parameter");
      text = "207 RIGID TAPPING NEW";
      words.push(qword(239, pitch));
    }
  } else {
    if (dialect.drillC200 !== "q211") {
      bad("controller profile " + dialect.profile + " has cycle 200 level " + dialect.drillC200 + ", not the q211 level that this post writes");
    }
    // One plunge: the plunging depth is the whole depth.
    text = "200 DRILLING";
    words.push(qword(206, feed), qword(202, -depth), qword(210, 0));
  }
  words.push(qword(203, local(entry, "Z")), qword(204, clearance));
  if (!tap) words.push(qword(211, dwell));
  dwellTaken = tap && !floating ? 0 : dwell;
  var sig = text + "|" + words.join("|");
  if (sig !== cycleSig) { cycleDef(text, words); cycleSig = sig; }
  block("CYCL CALL");
  hole = {
    x: x, y: y, r: r, native: true, name: name,
    steps: [{ rapid: false, z: bottom, f: feedIn }, tap ? { rapid: false, z: r, f: feedIn } : { rapid: true, z: r }]
  };
}
// The engine sends the cycle's dwell after onCycle: the cycle wrote it.
function onDwell(seconds) {
  if (!(Math.abs(Number(seconds) - dwellTaken) <= CYCLE_TOL)) refuse("dwell " + seconds + " s outside a drilling cycle that writes it");
  dwellTaken = 0;
}
function finite(v) { return typeof v === "number" && isFinite(v); }

// Refuse before the host can substitute an ordinary motion or comment for a
// record that requests a material change, spindle synchronization or probing.
function onRadiusCompensation() { refuse("unsupported control cutter compensation"); }
function onThreadCycle() { refuse("unsupported threading cycle"); }
function onThreadCycleEnd() { refuse("unsupported threading cycle"); }
function onPartLoading() { refuse("unsupported part loading"); }
function onPartUnloading() { refuse("unsupported part unloading"); }
function onPartTransfer() { refuse("unsupported part transfer"); }
function onProbe() { refuse("unsupported probing"); }
function onProbeMove() { refuse("unsupported probing"); }
function onProbeRetreat() { refuse("unsupported probing"); }
function onProbeGuard() { refuse("unsupported probing"); }
function onProbeResult() { refuse("unsupported probing"); }
function onPlasma() { refuse("unsupported plasma operation"); }
function onTorchOn() { refuse("unsupported torch operation"); }
function onTorchOff() { refuse("unsupported torch operation"); }
// The host also sends informational program notes here. Preserve their complete
// text (including the accuracy bound) and strip control-language delimiters.
// A WARN record still blocks the program through the failOnWarning manifest.
function onComment(text) {
  // Preserve word boundaries in multiline notes before the host replaces
  // non-printable characters with question marks.
  var singleLine = String(text || "").replace(/\s+/g, " ");
  var words = commentText(singleLine).match(/.{1,64}(?: |$)|.{1,64}/g) || [];
  for (var i = 0; i < words.length; ++i) block("; " + words[i].trim());
}
