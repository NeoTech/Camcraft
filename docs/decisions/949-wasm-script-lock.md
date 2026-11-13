## #949 - web/build-app-wasm.sh leaves Cargo.lock as it found it

**What was decided.** The script saves the tracked `Cargo.lock` to
`target/wasm-tools/Cargo.lock.keep` before its first cargo call, and an
`EXIT` trap copies it back on every exit path (success, failure, Ctrl-C,
TERM). The save and trap moved up from step 3 so that `cargo fetch` and the
wasm-bindgen install are covered too. The header says so.

**Why.** The flecs `--config patch.crates-io...` makes cargo rewrite the
workspace lock's flecs entries. Restoring from a copy is the smallest change
that covers every cargo call the script makes, and it needs no second manifest
dir or `CARGO_TARGET_DIR` split.

**Considered and dropped.**
- Building against a temporary copy of the manifest dir: more moving parts
  (workspace paths, `path` deps, lock per copy), and the build would still
  need the copy kept in sync with the tree.
- Restoring from `git show HEAD:Cargo.lock`: would discard a developer's
  uncommitted lock edits.

**Cost to change later.** Low: one trap and one copy. A SIGKILL still leaves
the lock rewritten; the header tells the operator to `git checkout -- Cargo.lock`.
