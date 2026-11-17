## #989 - Document thread-mill tools and Thread parameters

**Decided:** Add dedicated tool and operation reference rows to the MCP skill for the two thread-mill kinds, catalog entries, pitch fields, handedness, hole interpretation and named refusals. Keep the existing tool and operation rows intact. Leave PROCESS_VERSION to the merging session and declare this as a process change in the PR.

**Why:** The JSON interface already carries these fields, but the skill did not teach agents how to choose a tool or state a legal thread. The implementation allows an unfinished single-form operation with no pitch, then refuses generation, so the text names that stage rather than claiming creation fails. The separate rows keep unrelated same-file PR edits independent.

**Considered and dropped:** Runtime/schema changes are unnecessary because the behavior already ships. A permanent test that checks prose wording would constrain future edits without testing runtime behavior; use scratch documentation pins and the existing MCP test as the behavior reference instead. A PROCESS_VERSION bump or announcement from this issue would duplicate the merging session's responsibility.

**Cost to change later:** Edit the two reference rows when the tool kinds or operation contract changes. No runtime format, dependency or posted output changes.
