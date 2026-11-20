## #2303 - a generic LinuxCNC mill post (3-axis), separate from PathPilot

**Decided.** A new built-in post "LinuxCNC" (`assets/posts/linuxcnc.cps`)
and a generic machine "LinuxCNC 3-axis" (`assets/machines/linuxcnc-3axis.mch`),
written from the public LinuxCNC 2.9 G-code and M-code references. The
structure is our own `tormach.cps` (same interpreter); the PathPilot
assumptions are removed: no "machine Z0 is the top" convention, no G30
before each tool change, no belt-range notes, no 1-1000 tool range, no
4th axis. Options (declared post options):

- `retractMode` (default `g53`): `G53 G0 Z<safe_retract>` from the machine
  file; `g28` / `g30` write `G91 G28 Z0` / `G91 G30 Z0` then `G90` (Z only,
  to the stored position); `none` writes no retract (the control's INI
  tool-change position or M6 remap moves Z). With `g53` and no stated
  `safe_retract` - also with no machine at all - the post refuses by name.
  The generic machine file states no `safe_retract`, so a user must state
  it or pick another mode (#435 / #495: never assume Z0 is the top).
- `writeToolLength` (default on): `G43 H<n>` after `T<n> M6`. Off when the
  M6 remap applies the length; then the section safe block also drops
  `G49`, so a repeated tool keeps the length the remap applied.
- `pathControl` (`G64 P<blendTolerance>`, default 0.025 mm, or `G61`).
- `programEnd` (`M30` or `M2`) and `percentDelimiters` (`%` first and last).
- The PathPilot post's sequence-number, header and `useRadiusArcs` options.

Tapping: a rigid tap is `G33.1 Z<bottom> K<pitch>` (pitch = feed / rpm),
written only when the machine file states `rigid_tapping: true` (the
reference requires an encoder on `spindle.N.index-enable`). Otherwise it
is refused by name. A floating-holder tap is `G84` (the reference says G84
is not spindle-synchronized, F = rpm x pitch), so the post declares
`floatingTapping = true`. G33.1 is one move, not a modal cycle: the tool is
seated on R over the hole first and every hole seats again.

Engine: the machine mirror read `rigidTapping` as `true` when the file
states nothing (the unstated-limit law). The post cannot use that as "an
encoder is present", so the engine now also publishes
`machine.rigidTappingStated` (true only for a stated `rigid_tapping: true`).
No other post reads it.

Refused by name: control compensation (G41/G42), a tilted move (3-axis
only), a trip home after an operation, tool numbers that are not positive
whole numbers, G73/G83 with Q <= 0 (the reference makes it an error).

File extension `.ngc` (LinuxCNC's native extension).

**Dropped.** A post option for the G53 Z value (the machine file is the
machine's truth; one place to state it). Expanding a rigid tap into plain
moves (a tap cannot follow unsynchronized moves). G84 for rigid taps (not
synchronized in LinuxCNC). A G0 A indexing 4th axis (not in the issue;
the PathPilot post keeps it). Writing G41/G42 (LinuxCNC supports them,
but no golden or interpreter check exists for them here yet).

**Not verified on a control.** The output was not run in a LinuxCNC sim.
The reporter offered a LinuxCNC 2.9 sim run and an air cut.

**Cost to change.** The post and its goldens
(`crates/app/tests/golden/nc/linuxcnc/`, `synth_linuxcnc.nc`,
`crates/post/tests/recorded/golden/linuxcnc/`).
