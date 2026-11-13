## #1296 - every camcraft-cli subcommand answers --help

**What was decided.** `script`, `process` and `mcp` each print their own
usage to stdout and exit 0 when `--help` or `-h` appears anywhere in their
arguments. The check runs in `run` before the lane parses its arguments, so
`mcp --help` never starts the server or reads stdin. The top-level `help`
gained one line pointing at `camcraft-cli <command> --help`. Parsing stays
hand-rolled; no dependency added.

**Why.** The issue asked for `--help` on each subcommand, and the CLI docs
page (#1294) needs a command to point at. Checking before the lane runs is
the only way `mcp --help` can be guaranteed not to start the JSON-RPC server.

**Considered and dropped.**
- Only accepting `--help` in the first position: rejected, since
  `process proj.camproj --help` is the natural spelling and would otherwise
  fall into the unknown-option refusal.
- A `help <command>` form: left out to keep one way to ask (the flag);
  `help` still prints the top-level usage.

**Cost to change later.** Low. The three usage texts sit next to the
`command_usage` mapping in `crates/app/src/cli.rs`; a new lane adds one arm.
