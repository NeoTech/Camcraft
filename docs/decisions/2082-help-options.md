# 2082 - one option table for `camcraft-ui` (--help)

- Chosen: a table in `crates/ui/src/cli_options.rs` (options and environment variables). The checker and the `--help` text read it. No new dependency (no clap).
- Chosen: the value checks stay with the old readers (`gpu::pick_from`, `gpu::adapter_from`, `mcp::port_from`). The table only gives the names.
- Chosen: an argument that starts with a dash and is not in the table gives exit code 2. An argument without a dash passes, so a file path from the system does not fail the start.
- Chosen: `CAMCRAFT_CARRY_ENGINE` and `CAMCRAFT_REQUIRE_GPU` are not listed. They are for tests and debug builds only.
- Dropped: a separate parser per option. It lets the help text and the parser disagree.
- Chosen: `CAMCRAFT_PROFILE_PROJECTION` (read in `crates/geometry/src/geometry/slice.rs`) is not listed. It is a developer timing switch.
- Chosen: the `--gpu-backend` help names the synonyms that `parse_backend` accepts (`d3d12`, `gles`, `opengl`). Dropped: narrowing the parser, because scripts may use them.
