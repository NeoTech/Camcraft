## #3155 - Report the remaining Cargo unit-hash inputs

**Decided:** Keep caller-selected targets and rustflags. Report nonempty
`CARGO_BUILD_TARGET` and `CARGO_TARGET_<triple>_RUSTFLAGS`, as well as the
existing generic rustflags variables. Keep the existing incremental pin and
profile-environment removal. Use that same policy for the docs-images Cargo
command. Save the diagnostics at gate start and repeat them in its summary.

Report selected Cargo config files from every Cargo working directory in the
plan, its ancestors, and Cargo home. Include the UI directory only for plans
with UI steps. Prefer `config` over `config.toml` when both exist, as Cargo
does. Deduplicate paths, not settings. Report relative Cargo home paths from
each command directory. Do not merge files or change precedence.

Every selected file gets a warning about possible rustflags, profile and
target inputs, including files without recognized keys. Add key hints for
ordinary table and dotted assignments: `build.rustflags`, `build.target`,
`target.<target-or-cfg>.rustflags`, and `profile` subtrees. These are source
hints, not effective settings. Do not print values. The scan does not expand
inline tables, escaped key names, config includes, or multiline strings.
Files with multiline strings get only the file warning. Cargo still loads
and interprets its own configuration. No absence of a hint proves absence of
an input. This is deliberately not effective-configuration normalization.

**Why:** A silent target override can change the platform that the caller
wanted to test. Removing rustflags can remove a required toolchain option.
A conservative file warning covers valid configuration syntax that the small
hint scan does not interpret, without adding a TOML loader or a parallel
Cargo configuration system.

**Dropped:** Removing all target/rustflags inputs would change cross builds.
Editing user files would change unrelated builds. A general configuration
loader or new dependency is not needed for diagnostics. Running the heavy UI
renderer cannot prove command environment construction better than inspecting
the real constructed command in an isolated child test.

**Verification:** Regression tests cover dynamic target variables, empty
values, preserved target inputs, selected config paths, extensionless-file
precedence, key hints and unrelated keys. A re-executed test process inherits
real caller profile variables. Its child proves that the variables disappear
and the incremental pin remains. The docs-images test inspects its real Cargo
command under an inherited profile variable without starting the renderer.
A dry gate tests the actual summary. Synthetic files are task-owned; tests do
not edit user configuration. Linux execution does not establish Windows
runtime behavior or physical-machine safety.

**Cost to change later:** Broader key hints need additional syntax fixtures.
Effective configuration normalization needs a separate design with Cargo
precedence and all supported syntax, not stronger claims about this scan.
