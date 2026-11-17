## #1499 - Sodick wire post: tank and flushing words from the machine file

**What was decided.** Two optional wire-machine keys, `edm.tank`
(`submerged` | `non-submerged`) and `edm.flushing` (`high` | `low`), new
module `crates/post/src/machine/edm_dielectric.rs`. They are set in the
Machine window's wire section (two combos, "not stated" by default) or
with the MCP `edit_machine` merge patch. Both routes apply as the one
undoable machine mount (`EditMachineApply` / `LoadMachine`), so no new
command or MCP tool was needed. A word outside the list refuses the
`.mch` file, the project's machine (serde) and the `edit_machine` patch
by name. `sodick-wire.cps` writes the matching Sodick T word:

| key | value | word | Sodick 1998 manual, T-code table |
|---|---|---|---|
| `flushing` | `high` | T84 | High Pressure Flushing Mode On |
| `flushing` | `low` | T85 | High Pressure Flushing Mode Off (low flush) |
| `tank` | `submerged` | T94 | Switch To Submersed Machining |
| `tank` | `non-submerged` | T89 | Switch To Non-Submersed Machining |

They are written after each threading (T91, or the operator's M00) and
before the C condition word, flushing first, in the order of the
manual's Program Example 1 (`T84`, `T94`, `C451`). The header line
"TANK FILL AND FLUSHING ARE NOT PROGRAMMED" is replaced by one line per
key, saying what is programmed and what is still the operator's. When
neither key is stated the program is byte-identical to before (the #617
golden `nc/sodick-wire/wire_slug_punch.nc` is unchanged). The new golden
`nc/sodick-wire/wire_slug_punch_flushed.nc` differs from it only in
those header lines and the T84 / T94 lines. The post still refuses by
name a word it does not know, in case a mirror arrives without passing
the parser.

**Why the machine file, not the operation.** The issue allows either.
The machine route already has an undoable command, an MCP tool, a UI and
a project round trip, so this PR adds no new command surface. A
per-operation choice can be added later as an override if a shop needs
one (#1499 follow-up only if asked).

**Why after threading.** The manual's T91 / T84 / T94 relative order is
not printed in one example: its Program Example 1 has no T91. Threading
first, then flushing and tank, then the condition, keeps the example's
own order (T84, T94, C) and never switches the flushing on during
threading. Note that issue #1499's text says the example writes "T94,
T91, T84". The example pages read for this PR (A600 pp. 6-5 and 6-8) do
not show that order, so the read pages were followed.

**Considered and dropped.**
- Sodick-specific key names (`edm.sodick_tank`), the earlier Codex
  draft: the choices are machine facts, and the words are the post's.
- Writing T96 / T97 (fill pump), T82 / T83 (drain), T86 / T87 (flushing
  pump) and T80 / T81 (wire drive): the manual lists them, but no read
  page shows when a program writes them. The T94 header line tells the
  operator that no fill or drain word is written.
- Switching back (T85 / T89) at section or program end: nothing in the
  source asks for it.
- A refusal in `wire-iso.cps` when a tank or flushing is stated: that post
  has no `edm.codes` word for either. It writes a header note naming the
  stated ask as the operator's instead. The ask is not a geometry safety
  matter, and refusing the whole program would block a legitimate post.

**Cost to change later.** A per-operation override would add two
`WireContourOp` fields with their command and MCP DTO, plus a PARAM in
the stream, and the post would prefer the PARAM. Changing the word order
would touch only the post and the two goldens.
