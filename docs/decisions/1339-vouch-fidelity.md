# 1339 - Vouch for a mesh of unknown fidelity

Issue: #1339 (a follow-up of #1056).

## What was chosen

- A new undoable command, `VouchModelFidelity { model, by }`. It clears the
  "fidelity unknown" mark (`model_fidelity::UNKNOWN_MM`) of one model and of
  the stocks copied straight from it. Undo puts each mark back.
- The command applies to the unknown sentinel only. It refuses by name:
  - a model with a recorded coarse chord (#864);
  - an incomplete model (#1610, a mesh with a hole);
  - a model that is not marked;
  - a name that is not a model;
  - an empty name of the person who vouches.
- The record of who vouched is a new component, `FidelityVouched { by, rev }`.
  `rev` is the geometry revision of the model at the time of the vouch.
- The record speaks only while the model has that revision and no fidelity
  flag (`model_vouch::by_of`). An import, an update or a repair lands other
  facets, and the record is then silent. No other command changes for this.
- The history names who vouched. `VouchModelFidelity::run` executes the
  command as one labelled undo group (`History::top_label`). It also logs
  the vouch at `info` level.
- The project file keeps the record in the optional key `fidelity_vouched`
  of the model row. The loader refuses the key on a row that is not a
  model, beside a fidelity flag, or with an empty name.
- The MCP tool is `vouch_model_fidelity { model, by }`. `get_job` shows
  `fidelity_vouched` on the model. Scripts use the tool through
  `cam.call("vouch_model_fidelity", ...)`, which is the script seam for
  each undoable MCP tool (#2393).
- The model page shows a name field and the button "Vouch for this mesh"
  below the "Fidelity unknown" line. A vouched model shows who vouched.

## Why

- A recorded chord is a measurement. A hole is a fact about the mesh. The
  word of a person does not replace either, so the command refuses them.
- The gate refuses each operation on an unknown mesh, and the laws read
  that mesh untightened at generation. The toolpaths that exist are thus
  the toolpaths of an exact mesh. The command regenerates nothing.
- The revision in the record keeps a stale vouch silent with no edit to
  `UpdateModel`, `RepairModel` or `ImportModel`. Open PR #2024 changes
  those files, so this also prevents a conflict.

## What was dropped

- No project version step. An old reader that drops `fidelity_vouched`
  loads the same unmarked model and loses only the name, as with
  `mesh_open`. The gate result is the same.
- No change to `UNKNOWN_NOTE`. The refusal text still says to re-import the
  model. Many tests pin that text. The model page and the MCP tool
  description tell the user about the vouch.
- No vouch for a stock copy whose source model is deleted. The command
  takes a model name. Such a stock still needs a new stock definition.
- No vouch through a chain of copies. Only a stock copied straight from the
  model, with the unknown mark of its own, is cleared. A copy with a
  recorded chord keeps it.
- No identity check of `by`. The application has no user accounts. The
  name is plain text that the person types.
- No dedicated script verb. `cam.call` is the seam.
