## #2190 - Apply stored script options when posting a named post, or warn clearly when not applied

**What was decided.**

- `posting::resolve_cps` returns `(PostRef, String)` (while `resolve_cps_text` stays a thin wrapper), and new `post_script_for_post_with_options` / `post_mounts_for_post_with_options` entry points carry that resolved `PostRef` through both single-mount (`post_script_as`) and multi-mount (`posting_mounts::post_each`) posting into `program_options_for_script`.
- `setup_post_options::same_post` and `setup_post_options::script_for_post` unify `PostRef::Builtin` with `PostRef::Named("Fanuc")` via `post_key`, and also match `PostRef::File` paths by `std::fs::canonicalize` when exact path spelling differs (e.g. `./custom.cps` vs `/abs/custom.cps`).
- When `script::api::v_post` (used by MCP `post`, JS `cam.post`, and `camcraft-cli process --cps`) posts with an explicit post, `setup_post_options::unapplied_script_warning` checks every setup in the posted mounts: if the document or a setup's own configured post holds non-empty stored `script_options` that did not apply (because a different post ran, or a combined single-file mount used the first setup's options), the warning names the setup (when multi-setup), its post, and the unapplied keys, and attaches it as `"warning"` on `PostResult` (and pushes it to `env.logs` for CLI runs). When every configured post's stored options applied (or none were stored), `"warning"` is omitted.

**Why.** Passing only script text (`post: None`) through `post_mounts_script_with_options` silently threw away all stored `PostScriptOptions` even when naming the document's own post (e.g. `"Fanuc"`), dropping configured `safeRetract` or `useRadiusArcs` unseen. Carrying the resolved `PostRef` preserves one seam across MCP, JS scripting, and CLI without breaking raw-text callers of `post_script`, and returning `"warning"` on mismatch lets users test another dialect without losing visibility into unapplied settings.

**Considered and dropped.**

- Inferring `PostRef` by matching raw `.cps` text bytes against `BUILTINS`. Dropped: fails for user `.cps` files on disk (including a copied or edited `fanuc.cps` whose options are keyed under `file:<path>` rather than `builtin:Fanuc`).
- Refusing the post whenever stored `script_options` exist on a different configured post. Dropped: prevents posting a configured project temporarily to another dialect (e.g. `"Haas"` or a custom `.cps`), whereas reporting a clear `"warning"` matches existing MCP non-blocking diagnostics (`coolant_tools`, `clearance_warning`) and satisfies the issue's "says it did not apply them".

**Cost to change later.** Isolated to `posting`, `posting_mounts`, `setup_post_options`, and `script::api`/`report`/`mod` (`PostResult::warning`); if `post_per_op_script` later gains a named-post caller, it can take an explicit `Option<&PostRef>` the same way.
