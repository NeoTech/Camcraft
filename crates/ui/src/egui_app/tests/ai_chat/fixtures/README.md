# Recorded AI replay fixtures (format 1)

These files replay **recorded model output**, not successful machining recipes.
The tests use the real AI panel, worker, argument parser, MCP inbox, document,
and undo history. Only `ChatTransport::complete` is replaced. No network or
model installation is needed.

## Source and correspondence

Source: [mkinney/Camcraft, q08 QA recording](https://github.com/mkinney/Camcraft/tree/d9be74b1408654799446f0e05e2c675fefc619c2/qa/q08/q08-20261122T131017Z).
The source commit is `d9be74b1408654799446f0e05e2c675fefc619c2`.
The transcript blob is `09832301703903e2bc3ced24bd7bc785f4dce4ff`.
Its SHA-256 is `1746eb81dfd556db0c29183a4effc58d16b35db29efd1e8bda6485abd2b9a275`.

The model was `qwen3-coder:latest` on local Ollama. The recorded application
was CamCraft v0.25.0 (`7e2a8119`), Linux x86_64, Native tools mode, with the
`Offline local (Ollama)` preset. The original `meta` object is retained.
The recorder captured the requests and replies, not an application export.
See [the recorder's explanation](https://github.com/NeoTech/Camcraft/issues/3411#issuecomment-7079851491).

| Fixture | Original zero-based turn | Initial state | Recorded outcome |
| --- | --- | --- | --- |
| `q08-finishing.json` | 1 (session 1, step 2) | Original `q08-start.camproj` | Adds `Pocket finishing`; three argument/source refusals; later edits leave 0.1 mm stock and `waterline`. |
| `q08-edit.json` | 3 (session 1, step 4) | Start project plus the finishing operation from `job-03-after-step3.json` | Changes only the finishing operation. The model deletes and re-adds it, then leaves climb=true, stepdown=0.1, stock=0.05. |
| `q08-delete.json` | 7 (session 2, step 6) | Original `q08-after-step5.camproj` | Two native delete calls. Current policy must wait for each Allow/Deny decision. |

Each slice contains the **whole recorded turn**, including all 12 rounds
when the original panel reached its round cap. No final reply is invented.
The finishing and edit requests did **not** achieve their requested zero
stock and 0.2 mm stepdown. The tests pin the actual document, not the model's
claim of success. The roughing operation must retain all recorded parameters.
This is not a new recording of a successful edit-only model workflow.

For the edit slice, `setup` reconstructs the intermediate operation from the
recorded job snapshot. Its `params.pocket` is flattened for `add_operation`,
as that tool requires. This is explicit test setup, not a recorded reply.
The start project supplies the original model, selection, stock and tool.
The smaller catalog tool is added before the reconstructed operation.

The source's historical delete calls ran without confirmation (#3424).
The replay does **not** preserve that bug. Both denial and approval are
tested against the current policy. A denial preserves the document and
history, even though the recorded final text claims that all operations
were deleted. Historical missing system prompts (#3425) are not recreated.

## Format

One JSON object describes one conversation slice:

- `format_version`: currently `1`; unknown versions fail.
- `source`: repository, immutable commit, transcript path, and zero-based
  `turn_indices`. `setup_snapshot` names an intermediate job snapshot.
- `meta`: the original recording metadata, unchanged.
- `initial_project`: one of the checked-in recorded `.camproj` files.
- `setup`: explicit MCP calls that reconstruct the slice's initial state.
- `turns`: original `user`, `session`, `brief_step`, and ordered `rounds`.
  Each round retains `model_reply` and `usage` from the recording.
- `refused_call_ids`: argument/source refusals that must produce failed
  cards, rendered explanations, and real error replies to the model.
- `expected_operations`: the original job snapshot projected to operation
  name, kind, tool, source and parameters, in document order.

`ReplayTransport` wraps the captured message fields in the OpenAI-compatible
`choices[0].message` envelope expected by the shipping parser. It retains
`finish_reason`, call IDs, index, function names, raw argument **strings**,
and content. It never extracts tool calls from plain text or normalizes
argument JSON. Exhaustion and unconsumed replies fail the test.

The original historical `tool_results_sent_this_round` are not replayed.
The real current dispatcher produces those results. The original snapshots
supply expected document values; they are not returned by a fake MCP server.
New additive schema fields are permitted, but every recorded parameter and
source field must match. Operation counts and order must match exactly.
Transient toolpaths, collision results and status caches are not goldens.
Undo and redo are checked against the real before/after document states.
These software tests do not establish physical-machine safety.

Both `.camproj` files are unchanged copies from the source directory.
Their original source paths identify public QA assets; replay uses the
embedded facets and does not read those external paths.

To record another fixture, follow [the recording guide](../../../../../../../docs/agents/ai-chat-replay.md).
