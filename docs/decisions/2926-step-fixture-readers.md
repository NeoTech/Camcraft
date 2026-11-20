## #2926 - Select STEP fixture readers with manifest paths

**What was decided:** Recognize `concat!(env!("CARGO_MANIFEST_DIR"), "/...")` in the conservative fixture text scan. Resolve its suffix from the crate directory. Keep the app and UI reader assertions.

**Why:** The suffix alone looks like an absolute path. The scan discards the crate directory when it resolves that suffix. PR #2904 removed other app readers that had hidden this omission. The app and UI libraries still contain fixture readers.

**Considered and dropped:** Removing the reader assertions would hide a missed reader. Fixed lists of fixture suites would require manual updates. Removing every absolute prefix would change the meaning of unrelated paths.

**Cost to change later:** Extend the macro recognition if the source syntax changes. A Rust parser would cost more to add and maintain.
