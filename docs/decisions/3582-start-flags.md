## #3582 - --no-activate and --minimized start options

**Decided.** Two switches in the `cli_options` table:

- `--no-activate` creates the window with `with_active(false)` and skips
  `focus_window()`. winit then shows it with `SW_SHOWNOACTIVATE`.
- `--minimized` sets `--no-activate` too, and calls `set_minimized(true)` right
  after the window is created.

The default start is unchanged: the window shows and takes the focus.

**Why.** A script that opens a project in the window must not take the focus or
cover other windows. winit shows a visible, active window with `SW_SHOW`, and the
shell called `focus_window()`, so the launcher's show state had no effect.

**Considered and dropped.**
- One `--background` switch. Its meaning (minimized or only inactive) is not clear
  from the name. Two switches say what each does.
- Read STARTUPINFO `wShowWindow` on Windows. It changes the default launch for
  every caller (for example a shortcut set to "Minimized"). The requester asked for
  no change to the default launch.
- Create the window hidden, then `set_minimized(true)` and `set_visible(true)`.
  Tested on Windows 11: showing the window restores it, so it was not minimized.
- An environment variable for each switch. Nothing asked for one.

**Cost to change.** Small. The two switches are in one table row each and one
file (`shell_window/start.rs`). On Wayland winit ignores `with_active`, so
`--no-activate` may still take the focus there. That was not tested.
