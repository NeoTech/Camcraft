## #3121 - Post-processor API reference

**What was decided:** Add one user reference page for the current CamCraft
post contract. Link it from the posting workflow. Cover manifest values,
hooks, records, properties, helpers, refusal causes, and standalone debugging.
Use an explicitly non-executable trace example with exact output.

**Why:** Users need to diagnose their own posts without reading Rust.
The reference follows our documents and implementation only. It distinguishes
the app's refusal gate from the standalone tool, which still writes output
when warnings exist. The manifest coverage test detects missing reference
rows when the authoring contract changes.

**Dropped:** Do not copy another product's post reference. Do not present
a short example as a machine-safe milling post. Do not change posting
behavior, dependencies, controller scripts, or golden output for a docs task.

**Cost to change later:** Update the reference when the API changes. Extend
its coverage check if the manifest description adopts a different syntax.
The page has its own URL and can later split into linked reference pages.
