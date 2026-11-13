## #1110 - The program sim, setup sheet, tool list and cutting data on a tool-less wire cut

A Wire Contour (#532) mounts no library tool, so its `Operation::tool` is the empty word. The readers below read it as a library tool. These are the points the issue left open.

**Decided:**

- **One test: `KindInfo::mounts_tool()`.** Every reader that changes asks the op's kind whether it mounts a library tool, never "is this a WireContour" or "is the tool name empty". A milling op whose tool was deleted still mounts one, so it is refused or listed exactly as before. A test pins this for every surface touched.
- **The program sim leaves the wire cut out of the plan and refuses the verdict by name.** `plan_program` skips the cut (`stock_sim::wire::skips`), the way the probe and torch arms do, so the stock view and the MCP overlay still walk the rest of the program. `simulate_document` refuses a program that holds a wire cut: `"<op>: a wire cut is verified by the wire laws, not the stock sim"`. The post gate shows that as its non-blocking "not judged" line (the cut carries its generator verdict). Judging the uncut stock instead would report rest material where the wire cut through, which is a wrong answer. A program that mixes a wire EDM setup with a mill setup is already refused by the carry law (two machines), so refusing on any wire cut loses no verdict today.
- **The wire clause has its own file.** `stock_sim_wire.rs` (a `#[path]` submodule, like `rotary`, `lathe`, `carry` and `breaks`) holds the skip predicate, the refusal text and the program walk. `stock_sim.rs` was already 421 lines, and open PR #1140 also edits it, so it gets only three touches: the module line, the skip arm and the one-line call.
- **The setup sheet: the TOOL cell is `cond <id>`, with a condition line under the row.** The cell names the condition the cut was made with. The line under it, `CONDITION C110  WIRE D 0.250  GAP 0.040`, reads through `wire_op::condition_of`, the same door generation uses, so the sheet never states a number the cut was not made with. A condition the machine no longer states prints `! wire condition: <refusal>` instead.
- **The tool list and export hold no row for a wire cut.** A wire EDM has no tool table. A wire cut's travel therefore adds nothing to the tool list's minutes (its moves carry no feed word, so the time estimate had nothing to read anyway). A setup sheet on a wire setup prints "(no tools in this setup)".
- **Cutting data refuses by name.** `cutting::suggest` on a wire cut says it mounts no library tool and that the table covers milling tools only, instead of "tool  is not in the library".

**Considered and dropped:**

- Skipping without a verdict refusal (the torch's way): the verdict would read the uncut stock as rest, and the issue asks for an answer by name.
- Carrying the skipped names out of `plan_program`: that changes the signature its four callers share (stock view, carry, MCP render, the verdict) for one refusal.
- A "wire" row in the tool list: it would list something no tool changer loads.

**Cost to change later:** small. The wire verification laws (#534) can replace the refusal in `stock_sim_wire.rs` with their verdict without touching the plan walk.
