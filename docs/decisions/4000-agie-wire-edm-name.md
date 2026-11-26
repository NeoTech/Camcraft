## #4000 - The AgieVision post is named "Agie Wire EDM (AgieVision)"

**Decided.**
- The post that writes AgieVision ISO, SBR and SBL job files for the
  AGIECUT Classic 2S is named "Agie Wire EDM (AgieVision)". The name
  follows the other wire posts: brand, process, control ("Sodick Wire EDM
  (Mark 21)"). The contributor and the maintainer agreed the name on
  Discord on 25 Nov 2026.
- Its refusals say "The Agie Wire EDM job requires ...". The post dialog
  says "AgieVision job files (ISO, SBR, SBL)" and "the AgieVision
  control". The `.cps` option group is "AgieVision technology".
- The two old names "Agie native wire EDM" (#3537) and "Native wire EDM
  job files" stay hidden aliases. One table, `posting::aliases`
  (`crates/app/src/posting_aliases.rs`), holds them. The project loader,
  the option-key normalizer, `SelectPost`, `SetSetupPost`, `resolve_cps`
  (scripts, MCP, CLI) and the machine `controller.post` check call it. No
  generic module calls `posting::agie` to resolve a post name.
- A save writes the new name. The project file version does not change.
- The UI command `CommandKind::ImportAgieDrawing` is now
  `ImportInspectionDrawing`: it runs a plain DXF import. It is internal to
  the UI; no MCP tool, CLI command or script verb names it.
- Kept: the condition ids, the MCP tool names, the CLI command, the
  module names, the golden folder names, and the script receipt value
  `"format": "Agie native wire EDM"` (scripts can read it).

**Why.** #3537 removed the machine brand from the post name. The generic
result "Native wire EDM job files" made a one-control post look like a
feature of all wire machines. The control name in the post name tells the
user which control the job is for. The name keeps "Wire EDM", because
Agie also builds die-sinking machines.

**Dropped.**
- "AgieVision job files" (the audit's short form). Dropped: it does not
  match the brand + process + control form of the other wire posts.
- A neutral refusal in `crates/geometry/src/die_opening.rs` with the post
  name added by the app caller. Dropped: the issue asks only for the
  rename; the refusal now says "The Agie Wire EDM job requires ...".
- Change the script receipt `format` value. Dropped: scripts read it, so
  a change is a breaking change.
- Post the legacy project in `wire_post_alias.rs` (the #4022 review asked
  for a posting round-trip). Dropped: the control-checked punch fixture
  and the job goldens live in the `agie_native` suite. The posting
  round-trip is the submodule `agie_native/legacy_name.rs`, which reuses
  that fixture. A copy of the fixture in a second test binary would
  drift.
- Keep the site anchor `#native-wire-edm-job-files`. Dropped: the one
  link to it (`posting.md`) now points to the new heading anchor
  `#agie-wire-edm-agievision`.

**Cost to change.** Small: the name is `posting::agie::POST`; a later
rename adds one row to `POST_ALIASES`.
