## #1239 - shortcut hints name the platform's command key

**What was decided.** Every user-visible hint that describes a command
shortcut (ribbon tips, the operation bar's hover hints, the machine-cell
hover, the cleared-toolpath status line) is built from one helper,
`crate::shortcut_keys::chord`, which uses `command_key()` - `Cmd` on macOS,
`Ctrl` elsewhere. Hints that describe a real Control modifier (Ctrl+click
edge and feature picks: `EDGE_HINT`, the geo pane's "Click selects" hover,
the boundary pick help) keep the literal `Ctrl`, because the binding reads
`mods.ctrl` and is Control on every platform.

**Why.** The keyboard layer binds egui's `modifiers.command` (panes.rs),
which is Cmd on macOS. The hints said Ctrl there, so a macOS user following
them pressed a dead key.

**Considered and dropped.**
- Changing the Ctrl+click bindings to `command` on macOS: out of scope
  (a binding change, not a hint change) and it would move the pick gesture.
- egui's `ctx.format_shortcut` / `ModifierNames`: it prints the mac symbols
  (the command glyph) rather than the word the issue asks for, and it needs
  a context at the tip-table call sites. A `const fn` on `cfg!` is simpler
  and testable without a frame.
- Changing the PNG snapshots: not allowed (release lane).

**Cost to change later.** Low. The word lives in one function; a new
hint with a command key goes through `chord`. If a Cmd-with-Ctrl split is
ever wanted (e.g. Ctrl on macOS for some action), the helper gets a second
function rather than a change to existing callers.
