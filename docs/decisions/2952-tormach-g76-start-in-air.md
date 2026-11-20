## #2952 - Tormach lathe G76: refuse a start point above material

**Decided.** `assets/posts/tormach-lathe.cps` refuses the one-block
LinuxCNC `G76` when the thread record's `StartInAir` is not `true`. The
check is the last test in `g76Why`, so it uses the same refusal channel
as the other G76 refusals of the script (`refuse("G76 threading cycle
refused - " + why)`), and it runs before the drive-line rapid and the
`G76` block. The text has the form of the Fanuc refusal of #2878: "the
start point is above material (G76 moves the insert to pass depth at
rapid traverse) - start the thread in free air in front of the end face
or in a relief groove wider than the insert, regenerate the operation,
or choose G33 passes". The value is the one that PR #2892 computes
(`crates/app/src/operation/g76_start.rs`); the post adds no geometry.

**Why.** The public LinuxCNC G-code reference (G76 section) says that,
with no entry taper, the tool rapids to the cut depth and then
synchronizes. This post writes no entry taper (no `E` / `L`). So each
pass moves the insert in at rapid at the start Z. The old Tormach golden
started at Z14 on a bar that runs on to Z40: a rapid into solid material
(Prime Directive). The cycle words do not change.

**Old records.** A cached record or stream without the `AIR` word reads
`false`. The Tormach `G76` of such a project is refused by name until
the threading operation is regenerated - the same rule as the Fanuc post.

**Fixtures.** `tormach_lathe_support` keeps the solid bar
(`thread_world`, start Z14, used by the G33 goldens and the refusal
pins) and adds `g76_world`: the bar runs from Z-26 and ends at the free
end face Z14.693, and the threading top is one lead further out (16.3),
so the cycle starts at Z16 in free air. This is the fixture change of
#2892 for the Fanuc suites. Every test that must post a Tormach `G76`
uses `g76_world`. The G33 goldens do not change. In the two G76 goldens
the start moves from Z14 to Z16 (inch Z0.5512 to Z0.6299), and the
departure before the closing `G30` loses its `G0 Z40` (inch Z1.5748):
the #1604 departure goes +Z only up to the stock face, and the shorter
stock's face (Z14.693) is below the Z24 retract.

**Other posts checked** (each `assets/posts/*.cps` on `main`, for
`G76`, `G92`, `G78`, `G33`, `G32`, `CYCLE97` / `CYCLE98` and the
`onThreadCycle` hook):

| Post | Thread cycle | Start check |
|---|---|---|
| `fanuc-lathe.cps` | `G76` two-block | refuses since #2878 |
| `tormach-lathe.cps` | `G76` one-block | refuses with this change |
| `fanuc-lathe.cps` / `tormach-lathe.cps` passes | `G32` / `G33` per pass, not a canned cycle | not needed: no rapid infeed in a cycle |
| `linuxcnc.cps` | `G33.1` rigid tap, no THREAD record | not a thread cycle |
| `tormach.cps`, `grblhal.cps`, `tinyg.cps` | refuse `G84` | no thread cycle |
| all other posts | no thread word | no thread cycle |

Only the two lathe posts define `onThreadCycle`.

**Dropped.** A start that the post moves out to free air by itself: the
post does not know the part, and a moved start changes the verified
passes. A Tormach pin for a relief groove wider than the insert: the
fixture set has no grooving op for the Tormach suites; the judge itself
is pinned by `g76_start_tests.rs`
(`relief_groove_wider_than_insert_passes_when_grooved_and_refuses_when_uncut_or_narrow`).

**Cost to change.** One `if` in the script and one fixture flag.
