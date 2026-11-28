## #3052 - Split the viewport along existing domain seams

**Decided:** Keep `viewport` as the public facade and resource-state owner.
Move construction and target lifecycle to `viewport/init.rs`, uploads and
settings to `viewport/upload.rs`, pass encoding to `viewport/render.rs`,
and diagnostic readback to `viewport/readback.rs`. Move the WGSL literal
verbatim to `viewport/shader.rs`. Keep the existing pipeline and wide-line
modules unchanged.

Move camera state, movement and basis methods to `viewport/camera.rs`.
Put rays, cursor dolly and projection matrices in its private `projection`
child. Re-export `Camera` and `ProjectionMode` at their existing public
paths. Keep the crate-visible draw-order helpers and orthographic distance
constant in the facade. No caller or test needs an import change.

**Why:** These are existing seams, not new abstractions. Child implementation
modules can read the facade's private resource fields. The projection child
can read the camera's private basis methods. No resource lifetime, draw order,
method body, shader text, public visibility or safety check needs to change.
Each viewport source file is below 400 lines after the move.

**Considered and dropped:** A single camera file would still exceed the
limit. A new renderer-state wrapper would require delegation and broader
visibility changes. Extracting or rewriting shader functions would add
semantic risk to a code-only move. None is needed here.

**Evidence:** Compare against base `9fb406cd646c55ce12058dfd2930f509892240f1`.
The facade shrinks from 1,313 to 166 lines. Exact contiguous source blocks
retain every nonstructural original line, including shader composition.
Existing viewport children and test sources remain byte-identical. The UI
inventory keeps 1,956 test functions and 9,187 assertion macros. Existing
GPU tests cover display modes, depth, overlays, shading and MSAA. Separate
before/after captures use the unchanged catalog renderer on `2d/first-part`;
no snapshot goldens are refreshed. Commands and measured results are in the
PR description.

**Cost to change later:** A different grouping only moves private modules
and their imports. The public facade and all existing callers can stay.
