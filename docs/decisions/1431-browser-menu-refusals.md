## #1431 - Disable unsupported browser menu commands

**Chosen:** Use `browser_files::refusal` for Theme Browse and the Fixtures
menu file commands. Disable unsupported rows and show the same reason before
a click. Keep supported rows and desktop commands available. Correct the
stale release-note claim that the browser file picker is still owed.

**Why:** The command dispatch already refuses these actions in the browser,
but live menu rows hide that limitation until after a click. The existing
refusal table keeps the visible state and command handling consistent.
Model/project pickers and project/program downloads already exist.

**Dropped:** Removing the rows would hide the available desktop workflow.
Adding new pickers or download implementations would duplicate shipped
work and extend this fix beyond the remaining menu-state defect.

**Cost to change later:** Enable a supported command by updating its refusal
entry and structural assertions. No project format or posted program changes
are needed.
