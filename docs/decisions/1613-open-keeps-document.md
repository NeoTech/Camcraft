## #1613 - Keep the open document until its replacement loads

**Decided:** Load disk projects and picked project bytes into a fresh
`new_world()`. Only after the complete load succeeds, use the shared
`reset_for_open(loaded: World)` path to retire the old document's UI state
and history and install the loaded world. Copy the user's theme and color
vision singleton preferences into the replacement before installing it;
these preferences belong to the user across documents. A refusal
names the attempted file and leaves the existing document, history, file
name and document version intact.

**Why:** Project loading includes migrations and semantic checks as well
as JSON parsing. Some checks can fail after the loader has written entities
or document settings. A separate world keeps those partial writes away
from the open document on both entry paths.

**Considered and dropped:** Parsing JSON before resetting catches malformed
text but cannot protect against later semantic failures. Loading into the
current world and rolling back would require restoring entity identities,
undo commands and UI references correctly after partial writes. Staging a
replacement world uses the existing loader once and avoids that recovery
path. No new crate dependencies are needed.

**Cost to change later:** A future loader with a fully validated, infallible
apply phase could replace the temporary world. Until then, opening needs
memory for both documents while the replacement loads; refusal simply drops
the temporary world.
