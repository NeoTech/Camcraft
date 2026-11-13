## #1514 - operation picking is a mode

**Decided.** The open operation's selection is locked whenever its dialog
shows a tab other than Geometry and no pick row is armed
(`interact::op_pick_locked`, derived from `State::op_tab`, the boundary
pick and the setup-page drafts; no new state, never a command). A locked
viewport click is swallowed in `pick_at` before the model/feature routing,
so it neither edits geometry nor selects a model that would swap the
dialog; a Ctrl/Shift box drag is not armed and hover lights nothing. The
dialog shows one line under the tab bar: "Picking on - clicks edit the
selection" or "Picking off - selection locked".

**Why.** A click meant to orbit or focus silently dropped a picked face.
Deriving the lock from the tab makes it follow the tab with nothing to
reset; selection changes remain `SetOperationGeometry` commands (undoable).

**Dropped.** A separate stored "pick armed" flag (a second source of truth
beside the tab); making Model mode exempt (it would still close the dialog
from a locked tab); locking the setup-page picks (locate, points, plane,
axis, WCS are their own modes and stay live).

**Cost to change.** One predicate in `interact.rs`.
