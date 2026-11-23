## #3583 - build the stock eye on a worker thread at start

**Decided.** The native shell builds `StockEye` on a thread named `stock-eye`
(`shell_window/eye_boot.rs`). The shell takes the result on a later frame. Until
then the viewport draws the stock sheet, as the browser shell does. If the thread
cannot start, the shell builds the eye on the UI thread as before.

**Why.** `StockEye::new` compiled the composite and trace shaders on the UI thread
before the first frame: about 430 ms in the release build on one Windows DX12
adapter. The window was shown and did not answer during that time. The project
load took 14 ms for a 5 MB file, so it was not the cause.

**Considered and dropped.**
- Build the eye on the UI thread after the first frame. This moves the freeze to
  just after the first frame. It does not remove it.
- Move all GPU bring-up off the UI thread (about 200 ms more). That is a larger
  change. It is asked as an open question on the issue and is not part of this PR.
- Load the project in the background. The load is 14 ms, so there is little to gain.

**Cost to change.** Small. One new file and a field in `Shell`. Open risks, said
plainly: the thread is not joined at shutdown (the process ends with
`std::process::exit`), and the thread keeps a clone of the window while it runs.
