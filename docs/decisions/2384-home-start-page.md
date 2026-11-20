## #2384 - Home button and the recent projects start page

**Decided:**

- A **Home** button in the top band (the workspace tab row), left of
  Donate. A click shows the start page; a second click, or **Back to the
  project**, hides it.
- The start page stands in for the dock in the central area: the ribbon
  and the status strip stay. It lists the recent projects (the #662 list,
  newest first, from the user's settings file), then **Open project…**,
  **New project** and **Back to the project**.
- A click on a project queues `OpenRecent`, the command of the ribbon's
  Open list. The unsaved-changes prompt and the refusals are Open's own.
  A gone file is greyed, reads "(missing)", and takes no click; its hover
  names the refusal.
- The page remembers the `doc_version` that it opened over
  (`State::home_page`). It closes when the document changes: an open, New,
  an import or any command. A refused open or a cancelled prompt leaves
  the document as it was, so the page stays.
- The page shows at a start with no file on the command line
  (`home::at_start` in the native window). A fresh `State` has no page, so
  the harnesses, the snapshots and the browser build start as before.
- The version label was already the rightmost toolbar item (#556). It
  stays where it is; a test pins that it is painted at the right.
- The page shows each path without the Windows verbatim prefix
  (`home::shown_path`): the list stores canonical paths, which carry it.
  The open still uses the stored path.
- The action row uses the form seam's `action_bar`. The list rows are
  plain buttons and labels: the form helpers have no list-row helper yet
  (#1122).
- The glyph is Tabler's home outline (MIT), with its SVG master in
  `assets/ui/svg/home.svg` and its notice in `assets/ui/LICENSES.md`.

**Why:** the maintainer asked for a Home button that goes to a recent
projects page and for the version in the top bar (#2384). The page and
its door are view state, so they need no command and no MCP tool; the
open goes through the existing `OpenRecent` command.

**Dropped:**

- A separate start window. The single-window law keeps extra surfaces as
  in-app panels, and a page in the central area needs no new window.
- Close the page on each click on it. A cancelled prompt would then drop
  the user on a document that they wanted to leave.
- A second version label in the top band. The toolbar already shows it at
  the right, with the build on hover.
- The start page in the browser build. That build has no recent files.

**Cost to change later:** low. The page is one file
(`crates/ui/src/egui_app/home.rs`) and one branch in `shell_layout`.
