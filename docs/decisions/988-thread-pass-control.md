## #988 - Thread milling pass control: decreasing passes, finish and spring passes, depth allowance, M4 cutters

**What was decided**

1. **One value, `ThreadPasses`, carries every control** (`machining2d::thread_passes`,
   field `passes` on `OpParams::Thread` and the kernel's `ThreadParams`):
   schedule (`Equal` | `ConstantArea`), `finish` (radial depth of a
   finishing pass of its own, 0 = none), `spring` (repeats of the final
   radius, at most 4), `allowance` (radial depth left short of the
   standard depth), `spindle_reverse` (M4) and `cutter_left` (left-hand-cut
   mill). Its default is the #894 ride, so every existing Thread op,
   golden and digest is unchanged.
2. **Constant chip area = cumulative depth `D x sqrt(i/n)`** with `infeed`
   read as the FIRST (largest) pass: `n = ceil((D/infeed)^2)`. `D` is the
   roughing depth (thread depth - allowance - finish). Capped at 64
   passes, refused by name past that ("raise the infeed").
3. **The last pass lands exactly on depth - allowance**: the last roughing
   pass of a constant-area schedule is `D x sqrt(1) = D`; the finish and
   spring passes are written as that value itself.
4. **Allowance is one-sided and bounded**: negative (deeper than ISO 724 /
   ISO 965-1's depth, i.e. past the major diameter of a hole or the design
   root of a boss) is refused by name - the crest law judges at that
   circle and a deliberate over-cut would only be convicted later and
   more opaquely. Above a quarter of the thread depth is refused ("would
   not gauge"): ISO 965-1 pitch-diameter tolerances (6H/6g) are a small
   fraction of the depth; a quarter is a generous, stated bound.
5. **Spindle and cutter hand are two statements and must agree**: a
   right-hand-cut mill only under M3, a left-hand-cut one only under M4;
   any other pair is refused by name at the door (create, edit, retool)
   and by the kernel. Climb under M4 flips the orbit
   (`ccw = (climb != external) != spindle_reverse`); the helix still lays
   the drawing's hand (the rise follows the orbit).
6. **M4 reaches the post through a mill-spindle record of its own**:
   `post::ask_reverse_spindle` adds `PARAM MillSpindle reverse` before
   the section; a script voices it by declaring `millSpindleReverse`
   (the Fanuc mill post now does, writing `M4`), and the engine refuses
   the section by name for any other script. The lathe's #629
   `SpindleDirection` / `threadHands` was not reused: it states a turned
   thread's hand, and declaring it on the Fanuc mill post would have let
   a reverse lathe thread through a mill dialect (pinned by
   `lathe_post_g76::the_hand_is_never_dropped`).
7. **File/MCP keys are flattened and additive**: `pass_schedule`,
   `finish_depth`, `spring_passes`, `depth_allowance`, `spindle_reverse`,
   `cutter_left_hand`, each written only when not default (no version
   step); the digest gains a clause only when the control is not
   default. The MCP params door carries them through `edit_operation` /
   `add_operation`. The page shows them on Passes (schedule, finish,
   spring, allowance) and Feeds (spindle, cutter hand).

**Sources (public)**

- Peter Smid, "CNC Programming Handbook", thread cutting chapter:
  decreasing depth per pass for a constant cut area, `D_n = D_1 x sqrt(n)`;
  spring pass.
- Machinery's Handbook, "Thread milling" / "Thread cutting": finishing
  pass, climb direction relative to cutter rotation, cutter hand vs
  spindle direction.
- ISO 724, ISO 965-1 (basic depths; tolerance grades as the scale for the
  allowance bound). Elementary geometry: a V form's area grows with the
  square of its depth.

No HSMWorks / Fusion material, output or documentation was consulted.

**Considered and dropped**

- *A pass count instead of a first-pass infeed for the decreasing
  schedule*: keeps one meaning of "infeed" (the largest bite the cutter
  takes) across both schedules.
- *Cutter hand on the Tool*: the right home long term, but a tool file
  key and a catalog change for one operation; the op states it and the
  door refuses a mismatch. Moving it later needs a reader alias.
- *Allowing a negative allowance down to the standard's maximum major
  diameter*: the crest and root laws judge at the basic profile; opening
  them is a separate safety change.
- *Voicing M4 in every mill post*: only Fanuc is touched; the others
  refuse by name until each dialect is checked.

**Left open**

- The issue's fifth bullet (honouring the operation's `top_z` R plane as
  the feed start instead of each hole's entry) is not in this PR.
