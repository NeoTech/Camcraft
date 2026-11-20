## #1949 - Document the shipped AI workflow

**What was decided**

- Add the task guide under `site/content/guide/automate/ai.md`.
  Preserve `guide/ai/` as an alias.
- Document the native panel's cloud providers and Ollama connection.
  State that Unsloth Studio, LM Studio and llama-server have library
  endpoints but no supported panel setup.
- Use the actual environment and OS keychain credential routes.
  Do not describe an API key form or endpoint field that does not exist.
- Show the external clients' shared stdio command and arguments.
  Leave each client's surrounding configuration format to that client.
- Give four CAM tasks with JSON-RPC tool calls and named inspection steps.
  Check example calls against the live MCP tool schema.
- Keep context budgets (#1944) and single-source agent skills (#1947)
  outside this pass, as the issue's latest scope comment directs.

**Why**

The old MCP page says that the panel has no live transport.
The transport now ships, but its panel exposes fewer endpoints than the
provider library defines. Instructions must follow the available controls.
A tool call completing does not prove that a program is safe. The examples
therefore retain verification, warning inspection and posting approval.

**Considered and dropped**

- Put the page at the old guide root: the current navigation groups
  automation pages under Automate. The alias retains the short URL.
- Invent setup controls for every library endpoint: those controls have
  not shipped. The guide states the limitation instead.
- Copy client-specific configuration files: their formats belong to
  external clients. The shared command and arguments are the stable
  CamCraft interface.
- Use fixed catalog or face identifiers: those values depend on the
  document and runtime catalog. The guide tells readers how to obtain them.

**Cost to change later**

New panel endpoint controls require changes to the setup table and its
limits. New tool schemas or CLI flags must keep the guide's automated
checks green. The guide can link the agent skills when #1947 ships.
