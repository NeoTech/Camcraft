# Record an AI chat replay fixture

Replay tests run offline. Recording is a separate, explicitly authorized QA
activity. Do not provision a model or send project data to a provider merely
to run the test suite.

## Record

1. Use a synthetic project without private geometry, credentials or business data.
2. Save the initial `.camproj`. Record the application commit, platform,
   provider, model/version, preset, tool mode, system prompt and offered schemas.
3. Capture each user message and every raw model reply for that turn. Preserve
   call IDs, content, function names and the original argument strings.
   Do not repair invalid arguments or convert text into calls.
4. Save `get_job` after each turn. Record any Allow/Deny decision and each
   Undo/Redo result. Save another project before starting a fresh chat session.
5. Keep immutable source links and hashes. Review all files for secrets and
   private data before publishing. Document any redaction without changing a
   tool's argument shape. Never use an account key as fixture data.

The q08 source used a separate wire capture because the panel had no export.
This guide does not authorize traffic capture. Use an approved recording
method in an isolated QA session; never intercept another person's traffic.

## Make the fixture

Use the format and source mapping in
[`fixtures/README.md`](../../crates/ui/src/egui_app/tests/ai_chat/fixtures/README.md).
Choose complete turns, not only the successful calls. Preserve the original
reply objects. Keep setup reconstruction separate from recorded output.
Document why the slice has the same relevant initial state as its source.

Project schemas evolve. Use the normal project loader and explain migrations.
Update expected values only for a reviewed, intentional change. Do not alter
raw replies to fit the current parser. Do not preserve an old authorization
bug by weakening confirmation or refusal expectations.

Project the original job snapshots to stable operation fields. Assert the
operation count, names, tools, sources and parameters after **each** turn.
Assert that one Undo reverts the whole mutating turn and one Redo restores it.
For rejected destructive calls, assert the unchanged document and history.
Check that refusals appear in the panel and return to the model as errors.
A model's success text is not evidence that its requested edit succeeded.

## Run and challenge

From the repository root:

```sh
cargo test -p camcraft-ui --lib recorded_q08 -- --nocapture
cargo test -p camcraft-ui --lib egui_app::tests::ai_chat
cargo xtask gate quick --base origin/main
cargo clippy -p camcraft-ui --all-targets -- -D warnings
```

Add one named test per fixture in the existing `ai_chat` directory suite.
The transport must consume only its recorded replies and fail on an extra
round. Never fall back to the HTTP transport or a live model.

Temporarily perturb the test transport's arguments, run the focused test,
and verify that its document assertion fails. Restore the perturbation and
run the same test again. Keep both logs outside the source tree. Do not
commit the perturbation or change production parsing to make the test pass.

These tests check the application workflow. They are not toolpath goldens,
proof of collision safety, or approval to run a physical machine.
