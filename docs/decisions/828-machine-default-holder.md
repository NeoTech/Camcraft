## #828 - Machine files name a default holder for new tools

**What was decided.**

- **Two optional keys in the `tooling` block**: `tooling.spindle_mount`
  (`BT30`, `BT40`, `CAT40`, `TTS` - the holder library's tapers) and
  `tooling.default_holder` (a holder-library entry by its own name,
  `"BT30 ER20"`; case and spacing ignored, stored canonical). Kept in
  `tooling`, not in the head's `spindle` object, so #807's
  `spindle.rigid_tapping` edits never touch the same lines.
- **Refused by name at both doors** (the plasma rules' pattern): the
  reader (`parse_mch`) refuses the first broken key as
  `machine file: tooling.default_holder: there is no holder 'X' in the
  holder library`; the Machines pane's validator lists every one under
  its key; a project file's stored machine is re-judged by
  `Machine::tooling_refusal` (named by key). Refused: an unknown mount, an unknown holder, a
  non-word or empty value, a holder whose taper is not the stated mount,
  and a straight-shank chuck with a stated mount (it names none).
  `LoadMachine::from_file` / `SetSetupMachine::from_file` now prefix the
  reader's refusal with the file's path (their doc already promised
  "the refusal names the file").
- **Applied only when a tool is added**: `AddTool::with_holder(world,
  name, tool, HolderPick)` - `MachineDefault` (the active setup's
  machine), `Named(entry)` (a stated holder; wins, and a refusal refuses
  the add) or `Bare`. The body is the same drawn spun profile
  `SetToolBody::library` lands (one shared `library_body`), landed in
  the add's own `apply`, so one undo removes the tool and its holder.
  The GUI's add path (`CommandKind::AddTool`) and MCP `add_tool` use
  `MachineDefault`; `add_tool` takes `"holder": "<entry>" | "none"` and
  replies `holder`, `holder_from` and `note`. `AddTool::new` (tool-library
  import, tests) is unchanged: bare.
- **A default that cannot take the tool is left off, named**: a shank
  outside the collet's range (or a lathe-held insert) adds the tool with
  no holder body - as without the key - and the status line / MCP `note`
  says "the machine's default holder TTS ER20 was not applied - ...".
  Never another holder in its place, never a refused add for a default
  the user did not state.
- **Existing tools and documents never change**: mounting a machine
  touches no tool; a project stores its parsed machine, so one saved
  before #828 carries no default until its machine is re-mounted, and
  loads and saves byte-identical.
- **Shipped files** (source: https://www.tormach.com/tormach_mills/ -
  "the MX machines have a BT30 spindle and tool holder"; "the PCNC and M
  machines have an R8 collet in the spindle and use the TTS tool
  holders"; the URL and quote are in each file's `tooling.mount_source`,
  free text the reader ignores, since JSON has no comments):
  - BT30 / `BT30 ER20`: `tormach-770mx`, `-770mx-ml`, `-770mx-microarc4`
    (a 770MX with the 4th axis - not in the issue's list, same spindle),
    `-1100mx`, `-1500mx`.
  - TTS / `TTS ER20`: `tormach-770m`, `-770m-plus`, `-1100m`,
    `-1100m-plus`, `-pcnc440`, `-pcnc770`, `-pcnc770-s3`,
    `-pcnc1100-s1`, `-s2`, `-s3`. Files without a `tooling` block got one
    with only these keys (no invented changer count).
- **No `camcraft-post -> camcraft-tools` dependency** (maintainer,
  2026-11-11, reversing the earlier acceptance: low coupling - a
  dependency between our crates that carries a couple of items is passed
  in by the caller instead). The post crate defines a plain
  `HolderInfo { name, taper }` and `HolderLookup` (`&dyn Fn(&str) ->
  Result<HolderInfo, String>`); `parse_mch_with`, `tooling_refusals`,
  `mount_check` and `Machine::tooling_refusal` take it as a parameter.
  The app passes `machine::holder_lookup` (over
  `camcraft_tools::holder_library::find`) and shadows the glob re-export
  with its own `parse_mch` / `tooling_refusals` / `tooling_refusal`, so
  every app door judges exactly as before (same refusal texts). The post
  crate's own word-only `parse_mch` (the post binary, its internal
  re-reads) keeps the holder as the stated word: it never lands a
  holder. Considered: a dev-only or feature-gated edge (still an edge),
  a global registered lookup (hidden coupling, unset in tests).

**Why.** The holder feeds the collision laws; a default that is silent,
retroactive or substituted would change verdicts unseen. Making it a
named, add-time, undoable part of the add keeps it visible. It does
NOT by itself make the check stricter: the drawn ER20 nut (Ø35) and TTS
collar (Ø38.1) are narrower than the catalog's Ø44 x 35 stand-in
cylinder in the band just above the shank, so a raw default would pass
a wall the bare tool is convicted on (the PR #880 high-tier review: a
wall 20 mm off the axis, 55-65 mm above the tip - bare 30 holder hits,
default 0, on the 770MX and the 770M). Hence the rule below.

**Maintainer's rule (2026-11-11, PR #880 review-fix round).** A
machine-DEFAULT holder is an assumption, not something the user
mounted, so it never judges looser than today's stand-in cylinder: it
is judged by the union of its drawn bands and the tool's
`holder_d` x `holder_len` cylinder, band by band
(`HolderProfile::floored`, applied in `tool_body::judged`; the
single-cylinder laws read the enclosing cylinder of that union). A
holder the user picks EXPLICITLY (`HolderPick::Named`, `SetToolBody`,
the editor, MCP `add_tool` with `holder`) is the real one and is judged
by its real shape. The document knows which: `ToolBody::machine_default`,
set only by `AddTool`'s default door inside the add's undo step, saved
as an additive `machine_default: true` key on the tool body (absent =
explicit, so every older file's holders count as picked; no version
step), and in the operation stamp. Any later explicit edit of the
holder replaces the body with an explicit one. Considered: flooring
every holder body (would overrule a user's real, measured holder) and
skipping the default when it does not enclose the cylinder (loses the
flange check the default adds). Pinned by `crates/tools`
`an_assumed_holder_convicts_the_band_above_the_shank_where_the_bare_tool_does`
and app `an_assumed_default_never_judges_looser_than_the_bare_tool`
(BT30 and TTS machines). A stored default holder that no longer
resolves is refused naming `tooling.default_holder`
(`Machine::tooling_refusal`), not as "a limit that cannot be meant".

**Considered and dropped.** Reading the default inside `AddTool::new`
for every caller (would silently give imported library tools a holder);
a separate `SetToolBody` command after the add (two undo steps, against
the issue); validating the holder only in the app layer (two doors with
different laws); refusing the add when the default does not fit the
shank (blocks adding a perfectly good tool for a default nobody stated);
`spindle_mount: "R8"` for the PCNC/M mills (the holder library has no R8
holders - the mount word names the holder system the spindle takes).

**Older builds.** An older build reading a `.mch` with the new keys
ignores them (the reader and validator read named keys only) and loads
the machine as before; a project saved by this build with such a machine
carries the two fields in its machine object, which older builds' serde
reader ignores (no `deny_unknown_fields`). Not a breaking change.

**Cost to change later.** Low: the keys are optional and self-contained;
a new mount (R8 holders, HSK) is one entry in `SPINDLE_MOUNTS` plus
library holders.
