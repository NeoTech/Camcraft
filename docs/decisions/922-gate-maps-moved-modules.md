## #922 - Gate maps name the crate that owns the module

**Decided:** do both. (1) The 96 suite headers that named a module M4.1 moved out of app as
`app::<module>` (`app::post`, `app::verify`, `app::machining2d`, ...) now name the owner
(`post::post`, `sim::verify`, `motion::machining2d`, `tools::...`, `cl::cl`, `geometry::geo2d`); only
header lines changed, so the `#[test]` count is unchanged. `heidenhain_cycles` got a map. (2) xtask learns
the alias: `scan::app_reexports` reads `crates/app/src/lib.rs` for `pub use camcraft_<crate>::...`
re-exports, and `quick::map_modules` also matches `app::<name>` for a change in the owning crate's module.
A pinning test walks the real tree and fails on any suite that still says `app::<moved>`.

**Why:** rewriting alone fixes today's headers but a new test copied from an old one brings the stale name
back, and the quick gate would again pass without the pinning suite (the failure mode is silent). The alias
alone leaves the headers lying and keeps the 86-suite drift. Both together: the gate selects more, never
less, and the tree stays true. The alias is derived from lib.rs, not a hand list, so a future move needs no
xtask edit.

**Considered and dropped:** a hand-written alias table in xtask (rots); matching every `app::<m>` for every
crate's `<m>` (would widen app's own modules, e.g. `app::machine`, which is both app's and post's - that one
is covered because app re-exports `camcraft_post::machine::*`); making `app::machine` header edits (the name
is legitimately app's too). #923 (.cps/.mch-only diffs select nothing) is a separate issue, untouched.

**Cost to change:** drop `legacy` in `quick.rs` once nobody trusts old headers; the real-tree test already
guarantees there are none.
