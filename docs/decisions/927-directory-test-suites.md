## #927 - Directory-backed integration test suites

**Decision.** Recognize Cargo's `tests/<suite>/main.rs` targets alongside
`tests/<suite>.rs`. Follow declared Rust modules rather than recursively treating
every neighboring source as a test. Preserve the Cargo suite name and qualify
external and inline module paths before emitting libtest `--exact` filters.

**Why.** A directory suite can grow across focused source files without creating
another linked binary or losing quick's changed-function selection. Shared
helpers outside a suite retain the crate-wide fallback. Helper/structural,
new/untracked and deleted module edits inside a directory suite run that suite
whole. Target/feature-conditional source is deliberately not claimed as a precise
static hit: quick runs the suite whole and named selection reports the ambiguity
instead of promising an exact test that may not exist for this target.

**Considered and dropped.** Keeping tests flat does not meet #927. Running every
crate suite for a directory edit discards the existing quick-tier contract. A new
parser dependency or Cargo-target configuration framework is unnecessary for
this conventional layout. Libtest accepts multiple positional filters (rustc
book, Tests / CLI arguments / Filters); retain the existing gate's grouped exact
filters and verify them with real Cargo execution, including duplicate leaf
names in different modules.

**Cost to change later.** Module discovery remains a source scanner, not a full
Rust compiler/cfg evaluator. Keep fixture coverage for flat/directory roots,
qualified nested/inline paths, unused neighboring files, Gate maps, heavy tests,
helpers and lib identity. The real Cargo fixture must assert nonzero selected
execution counts, not just successful argv. No mass test migration, dependency,
CI/cache-policy or product behavior change is included; #919's consolidation
work remains separate. This process change increments PROCESS_VERSION; its
#681 announcement belongs to the maintainer when the PR merges.

Sources: Cargo Book, Package Layout
(https://doc.rust-lang.org/cargo/guide/project-layout.html); rustc Book, Tests
(https://doc.rust-lang.org/rustc/tests/index.html); #919 research linked by #927.
