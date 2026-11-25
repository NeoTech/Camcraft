## #3995 - UI link: a link file counts only while its window holds its lock

**Decided.** Each window holds an exclusive OS file lock on `<pid>.lock`
in the link folder for its whole life (`std::fs::File::try_lock`: `flock`
on Unix, `LockFileEx` on Windows). The window takes the lock before it
writes its link file and removes both when it closes. The CLI tests the
lock of each candidate entry two times: before it dials the address and
after the hello.

- The lock is held: the window runs, and the entry is used.
- The lock is free: the window has ended (a crash included, because the
  OS releases the lock with the process). The entry is stale. The CLI
  removes `<pid>.json`, `<pid>.sock` and `<pid>.lock` (fixed names in the
  folder, never a path read from the file), logs it, and never dials it.
- No lock can be tested (no lock file, an error): the entry is unproved.
  The CLI logs why, keeps the file and never dials it. When nothing else
  answers, the CLI exits 3 and the message names each skipped entry.

A false server on the freed port of a crashed window therefore gets no
connection, so no hello and no tool call, even when it read the token and
can compute every proof. The test after the hello closes the gap where
the window ends during the exchange.

**Why.** The issue asks that only a window that is proved alive gets the
calls, with no silent fall-back. A lock is a stronger proof than a pid
check: a reused pid cannot hold the lock of an ended process. It needs
only std (MSRV 1.90; `File::try_lock` is stable since 1.89), so it adds no
dependency and builds the same on Windows, Linux and macOS.

**Dropped.**

- A pid-alive check (`kill(pid, 0)`, `OpenProcess`) as on the branch of
  PR #3971: it needs the new direct dependencies `libc` and `windows-sys`
  (a maintainer decision, which closed #3971), and a reused pid passes it.
- The Windows named pipe with a user-only ACL and a check of the server
  process id. It needs `windows-sys` (std has no pipe API) and is a
  transport design choice for the maintainer. It is the follow-up issue
  filed with this PR. The gap that it closes: while a window runs on
  Windows, a local process could share its TCP port (`SO_REUSEADDR`) and
  take a connection. The lock proves that the window runs; it does not
  prove which socket on the port answers. On Unix the socket is in a 0700
  folder, so only the same user can replace it.
- Compatibility with a window of an older build: such a window writes no
  lock file, so a new CLI does not dial it (exit 3, with the reason). A
  silent fall-back to an unproved window is the defect that this issue
  closes, so it is not kept.

**Cost to change later.** Small. The proof is one module
(`crates/app/src/mcp/link_owner.rs`) and one call site in `connect`. A
named-pipe transport adds a second proof beside it and keeps this one.
