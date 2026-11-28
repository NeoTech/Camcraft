## #3411 - Replay recorded local-model conversations

**Decision.** Replay the three complete q08 turns for finishing, editing and
bulk deletion. Keep the original Ollama replies and raw argument strings.
Use the real AI panel, inbox, dispatcher and document history. Replace only
the model transport. Store the source commit and zero-based turn mapping.

**Why.** These are authentic local-model replies from the linked QA run.
They include invalid enums, wrong entity names, a nested parameter object,
duplicate operation names, several edits in one turn and two deletes in one
reply. Hand-written successful replies would not cover those shapes.

**Actual outcomes.** The model does not fulfill the requested zero stock or
0.2 mm stepdown. Assert the captured final parameters, not the model's prose.
The edit turn deletes and re-adds only the finishing operation; the roughing
operation must keep its parameters. Reconstruct that turn's initial finishing
operation from its preceding job snapshot, separately from recorded output.

**Safety evolution.** The old recording predates the native-delete
confirmation fix. Test current Allow and Deny paths. Never reproduce the
historical bypass or change production policy to fit the old result.
Do not replay the old missing-system-prompt behavior. Do not assert that
recorded generation or verification results establish machining safety.

**Considered and dropped.** New remote inference and hand-written
transcripts are not needed. Partial successful round prefixes would omit
the actual end of a turn. A production parser change is outside this test
scope. A skill edit is not needed; a docs guide describes recording instead.

**Cost to change later.** Add a fixture and a named test in the same UI
suite. A new recording must identify its source, initial project and expected
job values. Preserve old raw replies when document or tool schemas evolve.
