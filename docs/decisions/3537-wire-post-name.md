## #3537 - The native wire EDM post no longer names the machine

**Decided.** The wire post that was "Agie native wire EDM" is now
"Native wire EDM job files" (maintainer decision, 2026-11-22). The name is
one constant, `posting::agie::POST`. The old name is the constant
`posting::agie::LEGACY_POST`, a hidden alias. No list shows it.

- The alias is resolved where a name comes in: the project file (document
  post, setup post), the option keys `builtin:<name>`
  (`PostScriptOptions::normalize`), `SelectPost` and `SetSetupPost`,
  `posting::resolve_cps` (scripts, MCP `post`, CLI), a machine file's
  stated post (`controller.post` check and the post offer), and
  `agie::is_native`. Inside the world only the new name is stored.
- A project saved after the change writes the new name. The decision says
  only that an old project must still open; writing the new name keeps one
  spelling in the world and in new files. An old release cannot read the
  new name, which is the normal direction of a file change.
- Refusals say what is wrong with the job, with "native wire EDM job" or
  "the job" in place of "Agie". The two refusals in `posting_agie.rs` that
  require the machine profile keep the model `AGIECUT CLASSIC 2S`: it is
  the value that the mounted machine file must state, so the refusal must
  name it to say what to do. Only the feature-like words "The Agie job"
  changed. The condition ID `AGIE_QN2` stays: the operator enters it on the
  machine.
- The UI label "Check the effective work quality on Agie Control" now says
  "on the control" (the machine name shows only in the machine library).
  The user guide keeps "Agie Control" and its title: there they describe
  the machine, not a feature of CamCraft.
- The `.cps` option group "Agie technology" is now "Wire EDM technology".
- Kept by the decisions: the script receipt value `"format"`
  (`script/report.rs`), the controller-file goldens, identifiers, module,
  file and test names, the machine file, the LICENSES attribution.

**Dropped.** A project-file version step with a migration (larger, and the
alias at the read gives the same result). Keeping the old name in saved
files (two spellings in the world). Removing the model name from the
machine-profile refusal (it would not tell the user what to mount).

**Cost to change later.** Removing the alias is one constant and the call
sites above; old projects would then show "(built-in, missing)" and refuse
to post.
