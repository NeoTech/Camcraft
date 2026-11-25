## #4041 - UI link: release the owner lock by unlock, never by a close alone

**Decided.** Each place that releases a UI link owner lock (#3995) calls
`File::unlock` before the file closes:

- `link_owner::owner`, when it takes a free lock to prove that a window
  has ended. If the unlock fails, the entry is unproved and not dialed.
- `Registration::drop`, when a window closes, after it removed its files.
- `link::open`, when the bind fails after the lock was taken.
- The tests end a stand-in window with `release_lock` (an unlock), not
  with `drop`.

**Why.** On Unix, `flock` belongs to the open file, not to the
descriptor. When a thread starts a child process, the child holds a copy
of each descriptor until its `exec` closes it. A close during that time
does not release the lock. Then the next test of the lock reads an ended
window as alive, and the CLI dials its address. The post-merge gate of
`3bb6647f6` showed this in `mcp_live`. A loop of the unchanged binary
failed in 44 of 150 runs. After this change it passed 200 of 200.

**Dropped.**

- A retry or a sleep in the test: it hides the race and keeps the product
  defect in `owner`.
- POSIX record locks (`fcntl`), which a child does not inherit: they need
  `libc` (a new direct dependency) and are not in std.

**Residual.** When a window process crashes while one of its own threads
is starting a child, the lock stays held until that child runs `exec`.
This is a short time and the OS ends it. The post-hello test of the lock
stays in place.

**Cost to change later.** None: the calls are local to the three places.
