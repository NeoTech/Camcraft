## #1239 - shortcut hints name the platform's command key

**What was decided.** Every user-visible hint that describes a command
shortcut (ribbon tips, the operation bar's hover hints, the machine-cell
hover, the cleared-toolpath status line) is built from one helper,
`crate::shortcut_keys::chord`, which uses `command_key()` - `Cmd` on macOS,
`Ctrl` elsewhere. That includes the click-to-add pick hints (`edge_hint()`,
the geo pane's "Click selects" hover, the boundary pick help in
`boundary::hints()`): the edge and feature picks read `mods.command`
(`edge_pick.rs`, `interact.rs`, `feature_click.rs`, #1238), so on macOS the
add key is Cmd and the hints say so. They were literal `Ctrl` until the
#1293 review caught it; a const became a fn because the word is chosen at
build time through `command_key()`.

**Why.** The keyboard layer binds egui's `modifiers.command` (panes.rs),
which is Cmd on macOS. The hints said Ctrl there, so a macOS user following
them pressed a dead key.

**Considered and dropped.**
- Keeping a literal `Ctrl` in the pick hints: wrong on macOS, where the
  picks read `mods.command` (Cmd) since #1238.
- egui's `ctx.format_shortcut` / `ModifierNames`: it prints the mac symbols
  (the command glyph) rather than the word the issue asks for, and it needs
  a context at the tip-table call sites. A `const fn` on `cfg!` is simpler
  and testable without a frame; the pure `command_key_for(macos)` tests
  both answers on any host.
- Changing the PNG snapshots: not allowed (release lane).

**Cost to change later.** Low. The word lives in one function; a new
hint with a command key goes through `chord`. If a Cmd-with-Ctrl split is
ever wanted (e.g. Ctrl on macOS for some action), the helper gets a second
function rather than a change to existing callers.
