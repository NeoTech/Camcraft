## #1233 - MCP set_setup_post

**Decided:** `set_setup_post {post, setup?}` runs the existing `SetSetupPost` through the session History (one undo step). `post` is a string (a `.cps` suffix means a file post, else a built-in name) or null to clear; `setup` defaults to the active one, like `set_work_offset`. `get_job` setups gain `post` (own post label or null) and `posts_with` (the effective post).
**Why:** mirrors the Setup pane; the command already refuses unknown names/setups, so the tool adds no logic.
**Dropped:** a separate `clear` flag (null is the repo's idiom); judging post-vs-machine kind here (#1265's post gate owns that).
**Cost to change:** one tool entry and two job fields.
