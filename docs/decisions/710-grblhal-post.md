## #710 - grblHAL post processor (3-axis mill)

**Decided:** a new built-in post, `assets/posts/grblhal.cps`, specced from the
G-code list in grblHAL's public core README and the LinuxCNC G-code reference
(behaviour only; grblHAL's source is GPL and was not read). Structure from
`tormach.cps`, coolant and after-operation handling from `mazak.cps`.

- **No tool length word at all** (no G43 H, no G43.1, no G49). The issue
  proposed G43.1 as the default, but G43.1 needs a length value the post cannot
  know. On grblHAL the length offset belongs to the controller's tool-change
  handling (tool setter, or a sender that measures and sets G43.1), and a G49
  in a preamble would throw that measured length away. *Dropped:* `G43 H<n>`
  (needs a tool table, which is driver-dependent); a G43/G43.1 post option
  (can be added later without changing the default).
- **`T<n> M6` alone on its line**, after the Z lift (below). Some senders hold
  the M6 block and drop the rest of it. No G28/G30 before the change: their
  stored positions are controller settings, machine zero when unset.
- **No control compensation**: grblHAL lists G40 only, so Control and Wear ops
  are refused by name, like Heidenhain and Tormach.
- **Canned cycles always on**: G73/G81/G82/G83/G86/G89 and G98 are in the core
  list without a configuration marker, so the issue's "optional, else
  expanded" switch was dropped. Dwell P in seconds.
- **Tapping refused by name**: G84 is not in the list and a tap cannot be
  plain moves.
- **Probing refused by the engine** (no probe hooks): G38.2-G38.5 exist, but
  the result test and offset arithmetic need parameters and expressions the
  core does not list.
- **Go home after an operation refused by name**: same reason as the G30 above.
- **The Z lift trusts machine Z0 only on a homed machine.** With homing off
  (`$22=0`, common on routers without limit switches) grblHAL takes its
  power-on position as machine zero, so `G53 G0 Z0` goes to wherever Z stood
  at switch-on, possibly down into the part (the first version of this PR
  did exactly that on its generic machine). Rule: a machine file stating `safe_retract` (the statement
  "this machine homes, and up is here") lifts with `G53 G0 Z<that>`;
  otherwise the lift is `G0 Z<h>` in the work offset, where h is the highest
  Z the program has proven clear (the section's `ClearancePlane`, or any
  higher Z it already commanded). The lift is refused by name when h is
  unknown or not above `StockZTop`. A G-code program cannot test "only if
  Z is lower", so h must be a clearance, never an arbitrary park height.
  *Dropped:* `G53 G0 Z0` as the default (unsafe without homing); a fixed
  park height above the work zero (could sit inside a tall stock); a post
  option for the mode (the machine file already carries the one fact that
  decides it).
- **Header without G64, O word, % or N numbers**: grblHAL lists G61 only, and
  O words need its macro plugins.

- **Generic machine `grblhal-3axis.mch` ("grblHAL 3-axis") states no
  `safe_retract`**, so it assumes no homing and lifts in the work offset.
  A homed build adds `safe_retract` (0.0 with grbl's default Z-up homing) to
  its copy of the file. *Dropped:* stating 0.0 (first version of this PR):
  it would have rapided to the power-on height on every machine without
  limit switches.
- **Generic machine figures** (300 x 300 x 100 mm stroke written `-L..L`,
  spindle 1000-24000 rpm, X/Y 4000/5000 and Z 1500/2000 mm/min, Flood M8 and
  Mist M7): a typical small router, meant to be copied and edited. No tool
  count and no spindle power: unstated rather than invented (manual changes
  have no magazine).
- **Post registered as "grblHAL"** before "Fanuc Lathe" in `BUILTINS`: the
  conformance test expects the mill posts first, then the lathe post;
  entries are addressed by name, so the lathe post's index moving is
  harmless. The synth golden drops only the tap record (G55 stays).
- **End units option** (`EndUnits`) is read by the script, but the Setup
  pane's end-units row still shows only for Tormach; widening it is UI
  work left out of this PR.
- **Per-operation smoothing (#616) is named, not written** (review of
  PR #715). grblHAL's core lists G61 (exact path) only - no G64 P and no
  other word bounds the deviation by a tolerance. With smoothing on, the
  program carries one `(SMOOTHING NOT WRITTEN - GRBLHAL STATES NO
  SMOOTHING TOLERANCE)` note, the way Fanuc and Mazak name an ask they
  cannot bound; the ask resets at each section end. Dropped: writing G64
  without P (unbounded blending - worse than none), and staying silent
  (the user's ask would vanish). Pinned by
  `smoothing::fanuc_and_mazak_name_the_ask_once_and_write_no_unbounded_word`.

**Cost to change later:** low - a post option for the length offset or for G30
before M6 adds words without moving any existing golden.
