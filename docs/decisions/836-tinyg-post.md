## #836 - Synthetos TinyG post processor (3-axis mill)

**Decided:** add the built-in `TinyG` post and a generic `TinyG 3-axis`
machine. The original post is based on TinyG's public G-code wiki and the
public LinuxCNC G-code reference, with the CamCraft grblHAL post (#710) as
its structural starting point. Compatibility corrections also use the
user-supplied, implementation-free TinyG behavior review dated 2026-11-12:
master 440.21 (`d0418390`) and edge-next 450.x (`998c02eb`). No GPL firmware
source was consulted by the CamCraft implementation agents.

**Evidence scope:** the supplied review reports source-reviewed behavior
and selected hardware findings; that is distinct from this PR's host
verification. Original TinyG goldens and compatibility fixtures have been
replayed against an existing edge-next parser/planner executable whose
recorded source is dirty at `998c02eb`. The available host adapters do not
build against master, so this PR does not claim an independent master
replay or hardware test. The final PR body records the exact tested head
and commands; earlier replay results are not evidence for later edits.

- **No canned cycles.** Drilling uses the stream's expanded moves. A cycle
  dwell is written as `G4 P<seconds>` at the hole bottom; an absent bottom
  or an unsupported dwell is refused. Tapping is refused because the post
  has no spindle-synchronized tapping implementation.
- **Manual tool changes stop outputs explicitly.** The sequence stops
  coolant and spindle with M9 and M5, lifts Z, writes `T<n> M6`, then M0.
  The first tool also requires confirmation. M0 is a program stop, not an
  implicit spindle/coolant stop. After continuation the post repeats the
  Z-only safe retract before any lateral move, since the operator may have
  jogged while measuring the tool. It then approaches XY and Z separately.
  A first cutting move requires a known start position and seats its stated
  Z even for a horizontal or rising cut; a start at or below known stock
  is refused pending a safe rapid approach. A legacy stream without stock
  metadata retains the contract that its stated start is a safe approach.
  Tool numbers are integral 1-31, matching the stricter edge-next limit.
  *Dropped:* relying on M6 or M0 to switch outputs, or trusting the
  pre-pause position after the operator can move the tool.
- **Tool offsets belong to the operator/sender.** No G43.1 or G49 is
  written. A sender that measures or changes offsets during M0 must hold
  the stream at that stop: M0 stops execution, not necessarily parsing or
  planning of following lines. This post does not implement sender flow
  control or an automatic tool-length measurement protocol.
- **Header states units, XY plane, absolute endpoints, incremental arc
  centers, feed per minute and G64.** The behavior review reports that
  master accepts but ignores G91.1 and always uses incremental IJK;
  edge-next applies it. Incremental center words serve both. G64 selects
  TinyG's exact-path behavior and clears a previous exact-stop choice; it
  does not promise tolerance-controlled blending. No percent fence,
  O number, N number, guessed G64 P tolerance, or implicit G28/G30 return.
- **Smoothing requests are named once, not silently discarded.** A
  positive per-operation smoothing tolerance adds one explanatory comment
  and no motion or tolerance word. The request clears at section end.
- **Feed formatting refuses zero.** A positive stated feed that would
  round to F0 is refused instead of emitted. Existing modal feed semantics
  remain for records that omit a new feed. Raising precision alone was
  rejected because it only moves the unsafe rounding threshold.
- **Native arcs are XY and at most a half turn per block.** Full circles
  and full-turn helices are split, preserving the endpoint and helical Z
  progression, rather than flattened into a line or plunge. This avoids
  the reported master full-CCW-circle refusal. Non-XY arcs are emitted as
  a chord approximation bounded to 0.005 mm, with output rounding included
  in that bound, avoiding the reported master G18 direction defect.
  Unsupported or degenerate geometry is refused, never silently dropped.
  *Dropped:* a firmware-specific G18/G19 fast path and treating equal
  rounded endpoints as permission to replace an arc with a straight move.
- **Spindle wait is configurable and follows M3.** The script accepts
  `postOptions.SpindleDwellSeconds`, default zero, in seconds from 0 to
  4294. A positive request must not round down to zero. The M0 before M3
  does not provide spin-up time because the spindle is stopped during that
  pause. A fixed guessed wait was rejected; the machine's needs determine
  the requested wait. Availability in the app's Setup UI is not implied by
  this script option.
- **Lengths retain four decimal places in the output unit.** Coordinate,
  center and feed words convert from the stream's millimeters for an inch
  machine. Real-time control characters are removed from comments and
  lines longer than 191 content bytes are refused.
- **Retract height is explicit.** A stated machine `safe_retract` selects
  a G53 Z-only lift. Without it, use the section's work-offset clearance,
  refused when missing or not above the stock. Machine zero at power-on
  is not a safe home position. The generic machine states no safe retract.
- **Unsupported operations are refused.** Control cutter compensation,
  probing, unsupported tool-axis motion and untrusted go-home requests do
  not produce a guessed program. The post is registered as `TinyG`, uses
  `.gcode`, and the generic router/mill follows the existing machine schema.

**Deferred:** short-segment simplification belongs in a separately verified
geometry pass. It must preserve endpoints, feeds, significant corners,
clearance and operation boundaries. This post does not silently remove
short segments, rewrite rapid moves as feeds, or claim to eliminate old
firmware's segment-length or feedhold limitations. Non-XY linearization
may produce more short segments; host replay is not a timing or hardware
correctness proof.

**Cost to change later:** exposing the script's spindle wait through Setup
requires an app option and persistence/UI work. Supporting native non-XY
arcs would require an explicit firmware capability and fresh verification.
Changing the chord bound or adding simplification requires geometric and
clearance evidence, regenerated goldens, and controller replay. A 4-axis
post is separate scope.
