## #1168 - a shipped build ignores CAMCRAFT_CARRY_ENGINE

**What was decided.** The carry-engine switch (`CAMCRAFT_CARRY_ENGINE=field`,
#637) is honoured only in builds with debug assertions:
`CARRY_ENGINE_SWITCH_HONOURED = cfg!(debug_assertions)` in
`crates/app/src/stock_sim_held.rs`. A release build (what ships) never reads
the variable and always carries stock on the column engine, so one project
file gets the same verdict on every machine. The read itself is the pure
`engine_from_switch(value, honoured)`, pinned with `honoured = false`
(the shipped configuration) by
`carried_engine_agreement::a_shipped_build_ignores_the_carry_engine_switch`.

Wherever the field engine does judge (a dev/test build with the variable
set, or a test's `with_carry_engine` scope), the verdict says so: the post
gate's visible non-blocking line (`sim_clause` -> post result, status line,
G-code header), the setup sheet's `SIM:` line and the GUI Deviate readout
carry "carried stock judged by the field engine (band N, development switch
CAMCRAFT_CARRY_ENGINE), not the shipped column engine". On the column
engine nothing is added, so no golden or shipped verdict text moves.
Pinned by `carried_engine_agreement::a_verdict_the_field_engine_judged_names_the_engine`.

**Why `debug_assertions`, not `cfg(test)` or a cargo feature.**
- `cfg(test)` is set only for a crate's own unit tests; the carried suites
  are integration tests (`crates/app/tests/*`) that link the library built
  without it, so running them whole under the variable would stop working.
- A cargo feature (e.g. `dev-carry-switch`) works, but is a build change
  (Cargo.toml, maintainer review) and every run of the suites on the field
  arm would need `--features ...` on top of the variable - the feature must
  also stay out of the release tier's own `--release` build, which is the
  default anyway. Debug assertions already separate exactly the two worlds
  the issue names: the dev/test profiles (gate quick and batch are debug)
  honour it, the release profile (shipped zips, installers) does not.
- The engine-agreement lane does not depend on the variable at all: it
  selects each engine per call tree with `with_carry_engine`, in release
  too.

**Dropped.** Naming the engine in every verdict but still honouring the
variable in release (the issue's alternative): two machines could still
judge one file differently, only now with a reason printed; the review
preferred the gate. Removing the variable entirely: it is how the carried
suites are run whole on the field arm before #589 flips the default.

**Cost to change later.** When #589 makes the field engine the default,
`default_engine` and `engine_note` invert (the note then names the column
engine when a dev switch selects it); the const and the pure read stay.
A release-mode developer run that needs the field arm whole would need a
cargo feature instead - one `cfg` change here.
