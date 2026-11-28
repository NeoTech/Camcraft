//! The `fixture_laws` suite of `camcraft-sim` (#3188): the fixture, chuck and machine-model collision laws on straight, posed and turned moves.
//! Provenance: our own test layout (AGENTS.md "New tests join a directory suite", #919); each
//! module keeps its own provenance and Gate map.

mod c_axis_fixture;
mod fixture_arc_sagitta;
mod fixture_mesh;
mod fixture_mesh_column;
mod fixture_mesh_convict;
mod fixture_mesh_posed;
mod fixture_stack_span;
mod lathe_chuck;
mod lathe_chuck_jaws;
mod machine_model;
mod machine_model_refusals;
mod posed_band_cylinder;
mod posed_fixture;
mod posed_fixture_midpose;
mod undercut_fixture_envelope;
