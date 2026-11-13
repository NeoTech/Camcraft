## #1362 - Undoing DeleteSetup and the setup order

**What was decided.** No change to `DeleteSetup`. The setup order is the
name order (`setup::setups` -> `model::named_entities`, sorted by name), and
no index, list component or entity-id order stores it. A setup that undo
re-spawns sorts back into exactly its old place, so the operation order and
the posted program are the same as before the delete. The PR adds pinning
tests (`crates/app/tests/setup_delete_order.rs`) instead of a code change:
delete a middle setup of three, undo, and assert the setup names, the
operation order and the whole posted G-code text, then redo and undo again;
and delete the first setup (the one `None` resolves to) and assert it comes
back first.

**Why.** The report came from PR #1361's MCP test, which made "Front" then
"Back" and saw `["Back", "Front"]` after the undo. That is the name order,
and it was the order before the delete too: the setup did not move. The test
there sorts its list, which hides nothing.

**Considered and dropped.** Adding an explicit setup position field (so
setups can be ordered by hand) - that is a feature, not this defect, and it
would need its own reorder command, file format field and MCP tool. If one is
added later, the tests here fail if its undo loses the position.

**Related commands checked.** `CreateSetup` undo removes the setup it made
(`setups.rs` covers it); a datum-transfer clone is a `CreateSetup` and its
undo is pinned by `datum_transfer_clones_the_world_and_undoes_exactly`.
There is no setup reorder or setup rename command on main, so nothing else
can move a setup.

**Cost to change later.** None; the tests only pin the observed contract.
