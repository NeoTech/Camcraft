## #2744 - The Report door sends file names, not full paths

**Decided.** The Report door puts a shared form of the Debug info report
into the GitHub URL (`debug_info::shared_report`). That form names the
project file and the machine file by their file names only. The Debug info
panel in the application keeps the full paths (`debug_info::report`). An
unsaved project still reads `Project: unsaved`. A path with no file name
part reads `(no file name)`, never the path.

**Why.** The browser sends the URL to github.com, and the user can publish
the body without notice. A full path usually holds the user name. The file
name is enough for a triage. The machine line had the same leak (`from
<path>` of a user's own machine file), so it takes the same rule.

**Dropped.**
- A second, separate report text for the door: two sources drift. One
  builder with a "names only" switch keeps the fields the same.
- Only the exact project path replaced in the status (the first form of
  this PR): the review found other status texts with other full paths
  (`open failed: <path>`, `written to <path>`). The shared form now scrubs
  the whole report text (`report_scrub::scrub_paths`): each path-like word
  shrinks to its last component. A path starts with `/`, `\`, UNC `\\`, a drive
  (`C:\`, `C:/`), `~`, `./` or `../`, or has a `\` or two `/`. A path runs
  over spaces and marks to the last later word of the same line that has a
  separator. A mark (`,`, `;`, `)`, a quote) inside a folder name does not
  end the path (`OneDrive - Contoso, Ltd\...`). Two paths on one line merge
  into the last file name. One `/` alone (`mm/min`) is not a path. Known
  limits:
  - A path to a folder keeps the last folder name.
  - A relative path without a root keeps its first word when that word has
    no separator (`jane doe/x/y` keeps `jane`).
  - A non-path text with two `/` (for example a GPU driver text) shrinks
    to its last part.
- Changes at each status message: there are many, and a new one can leak
  again. One scrub at the report builder covers each field.
- `Path::file_name` for the file name: it reads only the separators of the
  running system, so a Windows path in a project opened on Linux came back
  whole. `report_scrub::last_component` reads `/` and `\` on every system.
- A home-directory replace (`~/...`): the directory names under the home
  can still identify the user or the customer. The file name is enough.

**Cost to change later.** Low: one switch in `debug_info::report_with`
and one scrub pass in `debug_info::shared_report`.
