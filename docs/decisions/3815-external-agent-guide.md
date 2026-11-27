## #3815 - One external agent drives SolidCraft and CamCraft

**Decided:** Add one guide under `docs/user-guide/` and link it from the
README. Use Claude Code's project configuration as the concrete client
example. Give both CLI bridge arguments and separate ports. Use a one-body
millimetre plate with one through hole, followed by a CamCraft drill
operation on pre-sized stock. Document STEP as the fallback when the native
cache is incompatible.

**Why:** This documents the external-agent path that exists now. The live
CamCraft registry, source restrictions and public SolidCraft documentation
supply the contracts. Separate server labels prevent ambiguous calls such
as `undo`. Explicit verification, file-link and approval steps avoid an
implied automatic CAD-to-NC pipeline.

**Considered and dropped:** Do not implement the proposed in-app MCP client
or its privacy policy from research #3793. Those are separate issues. Do
not add client-specific examples for every agent, duplicate the full tool
reference, install SolidCraft, or run a broad app build for prose. A
source/help check is not a two-app runtime acceptance result. State that
limit in the guide and PR. Do not imply in-app cloud controls govern an
external client. No process files or runtime code change.

**Cost to change later:** Update one guide when a CLI, MCP schema or native
cache contract changes. A future tested two-app example can replace the
source-checked walkthrough without changing application code. The
repository's review-time light lane is not run for this prose-only slice;
focused link and schema checks are the verification for this task.
