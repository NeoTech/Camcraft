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
  `Machine::is_sane`. Refused: an unknown mount, an unknown holder, a
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
- `camcraft-post` now depends on `camcraft-tools` (workspace crate, pure
  serde; no cycle) so the one reader can refuse an unknown holder.

**Why.** The holder feeds the collision laws; a default that is silent,
retroactive or substituted would change verdicts unseen. Making it a
named, add-time, undoable part of the add keeps every law reading the
tool's holder exactly as before (no law code changed) and makes the
check stricter, visibly.

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
