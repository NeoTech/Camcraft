## #4061 - Keep the model arc fixture on straight leads

**Decided:** Set `lead_arc_radius: None` in the `wire_post/model_arcs.rs`
fixture. Keep all existing test assertions and application code unchanged.

**Why:** The merge of #4002 adds this required field to `WireContourOp`.
The model arc fixture from #3215 does not initialize it. The hosted gate
and a local `cargo test -p camcraft-app --test wire_post --no-run` both
report E0063 at this initializer on main `83d52ef6a`.
`None` selects the existing straight lead. Other wire fixtures use the
same value. The model arc tests continue to check their original behavior.

**Considered and dropped:** Do not enable arc leads in this fixture. The
new arc lead option applies to selected DXF profiles, not model profiles.
Do not change the operation type, generation, verification, or post output
to repair a test compilation error. Do not add a separate regression test:
the existing test target cannot compile without this field.

**Cost to change later:** A fixture-only edit and a rerun of `wire_post`.
