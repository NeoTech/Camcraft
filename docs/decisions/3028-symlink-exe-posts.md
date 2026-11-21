## #3028 - A symlinked camcraft-cli finds the shipped posts

**What was decided.**
- `camcraft_post::data_dir::DataFiles::from_process` resolves the
  executable path with the new `real_exe` before the assets lookup. If the
  path is a symbolic link, `real_exe` calls `std::fs::canonicalize` and the
  lookup starts beside the installed file. A plain path is not changed.
  The posts and the machines use this lookup, so the two kinds are fixed.
- The refusal of a copy of the executable now ends with `COPY_HINT`. The
  hint says to start the program from its install folder, or through a
  symbolic link to the installed executable. There is no setting or
  environment variable that points at an assets folder, so the hint names
  none.

**Why.**
- `std::env::current_exe` gives the path that started the process on macOS
  and on Windows, and that path can be a link. On Linux it reads
  `/proc/self/exe`, which is already the real file: a release build made
  before this change posted through a link on Linux (readings on the
  issue).
- Only a link is resolved because `canonicalize` on Windows adds the
  `\\?\` prefix. Each path in a message would then change for all Windows
  users, also for those who start the program without a link. A link to a
  folder needs no resolution: `<link folder>/assets/posts` follows the
  folder link to the real folder.
- A link that does not resolve keeps the link path. The lookup then fails
  with a refusal that names the folder of the link, as before. It never
  panics.

**Considered and dropped.**
- `canonicalize` on each executable path: dropped for the Windows prefix
  above.
- A walk of two candidate sets (the real file first, then the folder of the
  link): dropped. Assets beside a link are not a shipped layout, and two
  sets make the refusal longer with no gain.
- An environment variable for the assets folder: dropped for this fix. The
  module doc of `data_dir.rs` rejects a second source of shipped files
  that can shadow them with no warning (review of #2293). A variable needs
  its own decision.

**Cost to change later.** Low. The resolution is one function in
`crates/post/src/data_dir.rs`, with its pins in `data_dir_tests.rs`.
