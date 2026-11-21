## #3002 - Scale the post output budget

**Chosen:** Set the output line budget to `max(100_000, 16 * records + 1_024)`.
Use saturating arithmetic and cap the budget at `u32::MAX` so the JavaScript counter remains exact.
Apply one budget to the complete post run, across all hooks and operations.
Initialize it before the user script loads and give it the first operation name.
Update the operation name from the current properties as the engine processes the stream.
Report the limit, the attempted line count, and the operation when output exceeds the budget.

Keep the emitter count private to the host closure.
Keep the refusal latched in Rust so a script cannot hide an exceeded budget by catching the exception.
Check the output buffer separately so direct buffer writes also cause a refusal.
Move the existing output pull helper to `js_host` to keep `js.rs` below 400 lines.
This move does not change the output format.

**Why:** The fixed 100,000-line limit refuses valid Tormach programs whose operations contain more than 100,000 moves.
The input record count gives larger programs a larger finite budget.
The minimum preserves the existing allowance for small programs.
The additional allowance covers headers, tool changes, and records that emit several lines.
The diagnostics identify the operation and the exhausted allowance.

**Dropped:** Unbounded output removes the guard against a runaway script.
A much larger fixed limit still has no relation to the input and permits excessive output for small programs.
Posting each operation separately does not help an operation that exceeds the old limit by itself.
Splitting toolpaths or changing their cutting parameters changes the user's machining plan to satisfy a post implementation limit.

**Cost to change later:** The budget constants and their boundary tests must change together.
A post that legitimately expands records beyond this allowance needs evidence and a revised finite budget.
The change adds no dependencies or user commands and changes no recorded G-code or CL golden files.
