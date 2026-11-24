## #3533 - Keep the CPU path after a shared GPU failure

**What was decided:** Store the first shared GPU failure in a `OnceLock<String>`.
Install callbacks for uncaptured errors and device loss when the shared device is made.
Use `log_fallback_once` to log the reason without duplicate messages.
Keep the existing CPU fallback and its exact output values.

**Why:** An uncaptured GPU error must not panic on a path that a user can start.
After a driver failure, the shared device must stay unused for the rest of the process.
The first reason gives the caller a stable diagnostic.

**Considered and dropped:** Do not restart or retry the GPU after a failure.
The driver state is uncertain, and the CPU path remains available.
Do not catch a panic from the default handler.
Installing the handler prevents that panic directly.

**Cost to change later:** Small.
The policy is in the shared device lifecycle module.
Isolated regression tests check both callbacks, the retained reason, and the CPU result.
