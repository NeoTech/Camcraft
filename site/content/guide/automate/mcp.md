+++
title = "AI agents (MCP)"
description = "Let an AI agent drive CamCraft: setup, run modes, tools, undo and safety."
weight = 2
aliases = ["guide/mcp/"]
+++

CamCraft includes a [Model Context Protocol](https://modelcontextprotocol.io)
(MCP) server. An AI agent connected to it can build a job the same way you
would in the window: import a model, set the stock, add tools and
operations, generate toolpaths, verify them, post the NC program, and undo
what it did. Every change an agent makes runs the same undoable command the
window uses, so the checks and the undo history behave identically.

## Starting the server

The server is part of the command-line tool:

```
camcraft-cli mcp
```

It speaks JSON-RPC 2.0 over standard input and output, one message per
line (the MCP stdio transport), so nothing else is ever printed on stdout.
You normally do not run it by hand: an MCP client starts it as a child
process and talks to it.

### Run modes

| Mode | Start | What the agent drives |
|---|---|---|
| **Headless** | `camcraft-cli mcp` | One document held in memory by the server. No window. |
| **Attached to the open app** | Start the app with `camcraft-ui --mcp-port <port>` (or set `CAMCRAFT_MCP_PORT`), then the client runs `camcraft-cli mcp --connect 127.0.0.1:<port>` | The document in the open window, live: you watch the agent work, and its steps land on the window's undo history. |

The attached mode is off unless you ask for it. The app listens on the
loopback address (127.0.0.1) only, serves one client at a time, and
disconnects a client that sends nothing for five minutes so the next one
can connect. Port `0` picks a free port.

### Setting up an MCP client

Use a **local stdio** server in your client. The client starts
`camcraft-cli`; do not enter the app's TCP address as an HTTP or SSE URL.
Attached mode still uses the CLI as a stdio bridge.

These instructions were checked against CamCraft source after v0.26.0
and the linked client documentation. Client menus can change.

#### Find the installed executable

Use the full path, not a bare command that depends on the client's PATH.
Keep the complete installation, including its assets.

| OS and installation | CLI path | App path for attached mode |
|---|---|---|
| macOS, app moved to `/Applications` | `/Applications/CamCraft.app/Contents/MacOS/camcraft-cli` | `/Applications/CamCraft.app/Contents/MacOS/camcraft-ui` |
| Windows installer, default per-user folder | `%LOCALAPPDATA%\Programs\camcraft\camcraft-cli.exe` | `%LOCALAPPDATA%\Programs\camcraft\camcraft-ui.exe` |
| Windows portable zip | `<extracted folder>\camcraft-cli.exe` | `<extracted folder>\camcraft-ui.exe` |
| Linux DEB or RPM | `/usr/lib/camcraft/camcraft-cli` (also wrapped by `/usr/bin/camcraft-cli`) | `/usr/lib/camcraft/camcraft-ui` (also wrapped by `/usr/bin/camcraft`) |
| Linux tarball | `<extracted folder>/camcraft-cli` | `<extracted folder>/camcraft-ui` |

On macOS, a different app location changes the prefix. The Windows
installer defaults to a per-user install. Its **all users** mode proposes
`%ProgramFiles%\camcraft` instead. Either folder can be changed. Use the
actual folder from the shortcut's target. Expand `%LOCALAPPDATA%` or
`%ProgramFiles%` to the full path before putting it in a client entry;
these table abbreviations are not portable client variables.
A portable archive can be extracted anywhere. For AppImage, Flatpak or Snap,
use that package's launch mechanism; do not assume its internal executable
has the tarball path.

The Windows and Linux folders need `assets/posts` and `assets/machines`
beside the executable. The macOS bundle keeps them under
`Contents/Resources/assets`. Symbolic links to the installed executable
are supported since v0.25.0. A **copy** of the binary alone is not an
installation: it cannot find those assets. Do not move it out of its folder.

The examples below use the macOS path. Replace it with your installed
path before use. In JSON, escape Windows backslashes, for example
`"C:\\CAM Tools\\CamCraft\\camcraft-cli.exe"`. In TOML, a literal string such
as `'C:\CAM Tools\CamCraft\camcraft-cli.exe'` needs no doubled backslashes.
Do not put shell quotes inside a JSON or TOML command value.

#### Choose headless or attached

| Client | Headless configuration | Attached configuration |
|---|---|---|
| Claude Code | `claude mcp add` with final argument `mcp`, shown below | Same command with `mcp --connect 127.0.0.1:7777` |
| Claude Desktop | `mcpServers.camcraft` JSON, `args: ["mcp"]` | Same entry with `args: ["mcp", "--connect", "127.0.0.1:7777"]` |
| Codex | `[mcp_servers.camcraft]` TOML, `args = ["mcp"]` | Same table with `args = ["mcp", "--connect", "127.0.0.1:7777"]` |
| Cursor | `mcpServers.camcraft` stdio JSON, `args: ["mcp"]` | Same entry with `args: ["mcp", "--connect", "127.0.0.1:7777"]` |

For **attached** mode, start the installed app first. In a macOS Terminal:

```sh
"/Applications/CamCraft.app/Contents/MacOS/camcraft-ui" --mcp-port 7777
```

On Linux, quote the full installed app path in the same way. In Windows
PowerShell, use the call operator and your actual path, for example:

```powershell
& 'C:\CAM Tools\CamCraft\camcraft-ui.exe' --mcp-port 7777
```

Keep this app open. Use the same port in the client configuration.
Disconnect other attached clients: the app serves one at a time. For port
`0`, read the selected port from the app's log and replace `7777` below.
Headless mode needs no app and does not share the open window's document
or undo history. Each headless process owns its document in memory. Save
the project explicitly if you need to keep later edits after it exits.

#### Claude Code

Run **one** of these commands in your shell. The options before `--`
belong to Claude Code; the command and arguments after it belong to CamCraft.
User scope stores the entry in `~/.claude.json`, across projects.

Headless:

```sh
claude mcp add --transport stdio --scope user camcraft -- "/Applications/CamCraft.app/Contents/MacOS/camcraft-cli" mcp
```

Attached:

```sh
claude mcp add --transport stdio --scope user camcraft -- "/Applications/CamCraft.app/Contents/MacOS/camcraft-cli" mcp --connect 127.0.0.1:7777
```

On Windows, use the same command with a quoted Windows executable path.
Do not prefix that path with `&`: Claude Code, not PowerShell, launches it.
To switch modes, remove the old entry with
`claude mcp remove --scope user camcraft`, then add the other form.
Start a new Claude Code session. Use `claude mcp get camcraft` or `/mcp`
to inspect connection status and tool permissions.
See the [Claude Code MCP reference](https://code.claude.com/docs/en/mcp).

#### Claude Desktop

Open **Settings > Developer > Edit Config**. Merge the `camcraft` entry
into the existing `mcpServers` object; keep your other servers.

| OS | Configuration file |
|---|---|
| macOS | `~/Library/Application Support/Claude/claude_desktop_config.json` |
| Windows | `%APPDATA%\Claude\claude_desktop_config.json` |
| Linux | `~/.config/Claude/claude_desktop_config.json` |

Headless:

```json
{
  "mcpServers": {
    "camcraft": {
      "command": "/Applications/CamCraft.app/Contents/MacOS/camcraft-cli",
      "args": ["mcp"]
    }
  }
}
```

Attached, after starting the app listener:

```json
{
  "mcpServers": {
    "camcraft": {
      "command": "/Applications/CamCraft.app/Contents/MacOS/camcraft-cli",
      "args": ["mcp", "--connect", "127.0.0.1:7777"]
    }
  }
}
```

Save, fully quit Claude Desktop, then reopen it. Inspect **Connectors**
for the CamCraft tools. CamCraft itself needs no Node.js or `npx` wrapper.
See the official [local-server setup guide](https://modelcontextprotocol.io/docs/develop/connect-local-servers).

#### Codex

Merge **one** table into `~/.codex/config.toml` (or the configuration under
your custom `CODEX_HOME`). Do not declare the same table twice.
A trusted project's `.codex/config.toml` can also hold this entry.

Headless:

```toml
[mcp_servers.camcraft]
command = "/Applications/CamCraft.app/Contents/MacOS/camcraft-cli"
args = ["mcp"]
```

Attached:

```toml
[mcp_servers.camcraft]
command = "/Applications/CamCraft.app/Contents/MacOS/camcraft-cli"
args = ["mcp", "--connect", "127.0.0.1:7777"]
```

Start a new Codex session. `codex mcp list` shows registered servers;
use `/mcp` in the terminal client to inspect available tools.
See the [Codex MCP reference](https://developers.openai.com/codex/mcp).

#### Cursor

Use `.cursor/mcp.json` in the project, or `~/.cursor/mcp.json` for all
projects on this computer. Merge the server into the existing object.
These are local editor/CLI settings, not a hosted Cloud Agent setup.

Headless:

```json
{
  "mcpServers": {
    "camcraft": {
      "type": "stdio",
      "command": "/Applications/CamCraft.app/Contents/MacOS/camcraft-cli",
      "args": ["mcp"]
    }
  }
}
```

Attached:

```json
{
  "mcpServers": {
    "camcraft": {
      "type": "stdio",
      "command": "/Applications/CamCraft.app/Contents/MacOS/camcraft-cli",
      "args": ["mcp", "--connect", "127.0.0.1:7777"]
    }
  }
}
```

Save the file. Inspect the server in Cursor's MCP settings, enable it and
reconnect it if needed. Keep tool approval enabled during the first check.
See the [Cursor MCP reference](https://cursor.com/docs/context/mcp).

#### Connection diagnostics

| Symptom | Check and action |
|---|---|
| Executable not found, `ENOENT`, or process fails to start | Check the full path from the client's environment. Keep spaces inside one command string. Use `camcraft-cli.exe` on Windows. A terminal's PATH can differ from a desktop client's PATH. |
| `no posts folder found` or `no machines folder found` | Restore the complete package. Use its installed executable or a symbolic link, not a copied binary. A successful tool list alone does not prove that post assets are present. |
| `camcraft-cli: mcp --connect ...` reports connection refused | Start the app with `--mcp-port 7777` first. Match the port. Do not configure HTTP/SSE. |
| Attached client waits or disconnects | Disconnect the other client. Reconnect after the five-minute idle timeout. Check the app log and the client's MCP log. |
| No tools after a configuration edit | Check JSON/TOML syntax, restart or reconnect the client, and inspect its server status and permissions. |
| A tool refuses a file path | Read the named reason. Use an absolute path on the computer running the server. Check access and the tool's restrictions. Live `open_project` refuses replacing the open document; use File > Open in the app. Do not weaken a refusal to make a test pass. |

## What an agent can do

The usual flow is `import_model`, `set_stock`, `add_tool`, `add_operation`,
`generate`, `verify`, `post`. The server's reply to the client's first
message says as much, and `get_job` returns the whole document at any
time. The current, authoritative tool list is whatever the client's
`tools/list` call returns; the groups below are the tools in the current
`main`.

| Area | Tools |
|---|---|
| Document and history | `open_project`, `save_project`, `get_job`, `get_selection`, `set_units`, `undo`, `redo` |
| Models | `import_model`, `move_model`, `repair_model`, `set_part_axes`, `lay_part_on_points`, `lay_part_on_face`, `align_part_cylinder`, `delete_model`, `set_model_link`, `vouch_model_fidelity` |
| DXF and SVG profiles | `import_dxf`, `import_svg`, `move_dxf_profile` |
| Engraving text | `insert_text` |
| Outline-font text (closed shapes) | `insert_outline_text` |
| Setups and work offsets | `add_setup`, `activate_setup`, `set_wcs`, `set_wcs_from_features`, `set_wcs_circle_centre`, `place_setup_origin`, `set_work_offset`, `load_machine`, `set_workholding_max_rpm`, `set_sheet_job`, `set_stock`, `fit_stock`, `set_raw_stock_only`, `set_operation_setup`, `delete_setup`, `set_program`, `set_setup_post`, `set_wcs_flip`, `edit_machine`, `get_rotary_stack`, `set_setup_nose_to_axis` |
| Fixtures and workholding | `add_fixture`, `remove_fixture`, `move_fixture`, `locate_part`, `get_fixture_findings`, `add_builtin_vise`, `set_vise`, `add_mod_vise_pair`, `set_mod_jaw`, `refit_mod_vise`, `add_accessory`, `set_accessory`, `add_fixture_plate`, `set_fixture_plate`, `place_on_plate`, `detect_plate_grid`, `get_plate_grid`, `turn_fixture`, `import_dxf_fixture`, `set_fixture_profile`, `derive_fixture_profile`, `get_fixture_profile`, `export_fixture_dxf` |
| Tools and holders | `list_tool_catalog`, `add_tool`, `set_tool`, `import_tool_library`, `export_tool_library`, `list_holder_library`, `set_tool_holder`, `get_tool_cutter_profile`, `set_tool_cutter_profile`, `set_tool_cutter_params`, `edit_tool`, `delete_tool`, `import_tool_holder_dxf`, `export_tool_holder_dxf`, `convert_tool_step_to_profile` |
| Materials and cutting data | `set_work_material`, `set_stock_material`, `suggest_cutting_data`, `apply_cutting_data` |
| Tool bodies and numbers | `set_tool_number`, `get_tool_body`, `set_tool_body`, `set_lathe_tool_body`, `attach_tool_body_piece`, `remove_tool_body_piece`, `set_lathe_insert_zone`, `clear_tool_body` |
| Operations | `list_operation_types`, `add_operation`, `edit_operation`, `delete_operation`, `rename_operation`, `reorder_operation`, `set_operation_coolant`, `get_thread_infeed`, `set_thread_infeed`, `mark_operation_reviewed`, `electrode_from_faces`, `save_cut_recipe`, `list_cut_recipes`, `apply_cut_recipe` |
| Picking geometry | `list_features`, `get_operation_geometry`, `set_operation_geometry`, `set_operation_area`, `get_operation_do_not_touch`, `set_operation_do_not_touch`, `detect_holes`, `detect_pockets`, `pick_in_box` |
| Generate, verify, post | `generate`, `verify`, `render_stock`, `get_post_options`, `set_post_options`, `post` |
| Agie nominal inspection | `create_agie_inspection`, `export_agie_inspection` |

`list_operation_types` returns each operation kind's parameters, their
types and defaults; `list_tool_catalog` and `list_holder_library` return the
built-in tools and holders an agent can add.

### Tool reference and local-model prompt

`camcraft-cli mcp reference` prints a Markdown table of every tool, with its
arguments and its full description. It reads the same table as `tools/list`,
so it is never out of date. `camcraft-cli mcp prompt` prints a compact prompt
for a local model (for example through Ollama, LM Studio or `llama-server`):
the working rules, then one line per tool. The AI Chat panel's Local
Prompt-JSON preset sends the same rules.

### Alignment and origins

`align_part_cylinder` takes a cylinder-wall `pick` from `list_features`.
Choose `target: "x"`, `"y"` or `"z"` (default Z). Optional `flip` reverses
the axis; `angle_deg` turns the aligned part about it. This changes the
part alignment, not the WCS.

`place_setup_origin` keeps the WCS rotation. Set `source` to `model_box`
or `stock_box`, with `point: "corner"` or `"centre"`. Use `source: "vertex"`
with a vertex `pick` to place it on a feature. Both tools accept a named
`setup`; omission uses the active setup. Each change is one undo step.

### Post options

`get_post_options {}` reads the document options. Give `setup` to read a
named setup's resolved options and `follows_document` state.
`set_post_options` uses the same explicit target: no `setup` changes the
document, even when another setup is active. Omitted fields keep their
current values.

For example, `set_post_options {"setup":"Finish","per_op":true}` gives
Finish its own options. `set_post_options {"setup":"Finish","follow":true}`
returns it to following the document. Do not combine `follow: true` with
option fields. The options include `skip_stops`, `per_op`, `threading`
(`g32` or `g76`), `end_units` (`leave`, `inch`, `mm`), `smoothing`,
`between_setups` and `explain`. Read `tools/list` for the accepted values.

`detect_holes` reports holes it does not create in its `skipped` field.
Read that field before assuming that every detected hole became an
operation. A refused Bore is not silently replaced by Drill.

### Agie nominal inspection

Mount the Classic 2S template on the active setup. Place a DXF on the top of a 10 mm reference box and set the work Z origin at its top. Use `list_features` to identify the closed chain. Pass its ID, drawing name, operation name, 0.25 mm wire tool and the centre of the 3 mm hole to `create_agie_inspection`. The start coordinates are world XY in millimetres. The operation has no machining condition and can be saved and reopened.

Call `export_agie_inspection` with the saved operation name and a new directory. The last part of the directory name supplies the uppercase DOS file stem. The CLI equivalent is `camcraft-cli export-agie-inspection <project.camproj> <operation> <new-directory>`. The desktop creation form is under **Setup → Post → Agie nominal inspection**. Select the saved operation to use **Export Agie inspection setup**.

Import the ISO geometry and keep its SBR beside the SBL during script import. Create a new piece and confirm zero working steps. The native piece has zero length and width because physical stock XY is unknown. The contour uses the project work datum. Physical stock Z and travel without stated machine limits remain unverified. The work selects the controller’s perpendicular lead default. Holding, actual entry and exit paths, and effective erosion settings remain unverified.

On the offline Classic 2S, the formal check of the work reports undefined technology (`Technologie: te definiëren`). Checking the whole piece also reports that its dimensions are too small (`Dimensions X, Y, Z: Waarde te laag`), because the inspection piece records zero for its unknown length and width.

### Tool libraries and machine tool tables

`import_tool_library` and `export_tool_library` accept native
`.camtools` files as well as Tormach PathPilot `.csv` tool tables and
Hurco WinMax `.TCM` libraries. Pass optional `format` (`"auto"`,
`"camtools"`, `"pathpilot"` or `"hurco"`) and `units` (`"mm"` or
`"inch"`, defaulting to document units for controller tables). See
[Tools and tool libraries](@/guide/set-up/tools.md#machine-tool-tables-tormach-pathpilot-and-hurco-winmax)
for field mapping and refusals.

## Example session

Each call is a `tools/call`; the answers below are shortened.

1. `import_model { "path": "bracket.step" }` - the model is added to the
   document.
2. `set_stock { "box": { "dx": 100, "dy": 60, "dz": 25 } }` - stock for the
   active setup.
3. `add_tool { "catalog": "<a name from list_tool_catalog>" }` - the tool
   joins the document.
4. `list_operation_types {}`, then `add_operation { "kind": "Pocket", "tool":
   "<tool name>", "source": "bracket" }` - an operation seeded with the
   kind's defaults. It is not generated yet.
5. `generate {}` - toolpaths for every operation that is not current.
6. `verify {}` - findings per operation and the program-level post gate;
   `"ok"` is true only when a post would be written.
7. `post { "output": "bracket.nc", "post": "<built-in post name or .cps path>" }` -
   the written file names, or the refusal reasons.
8. `undo {}` - takes back the last change if the result is not what you
   wanted.

## The AI Chat panel

The window has a dock pane for talking to an AI model without leaving the
app. Open it from **View > Panels > AI Chat**; it opens beside Properties.
It calls the same MCP tools as an outside agent, so every edit is an
undoable command with the same checks.

What the pane offers:

- **Provider** and **Model**. The presets are Anthropic (Cloud), OpenAI
  Responses (Cloud), OpenAI Chat (Cloud), Gemini (Cloud), Offline local
  (Ollama), Local Prompt-JSON and Local Q&A (Read-Only). A line under the
  boxes shows the mode and whether the endpoint is Cloud or Local.
- **Offline only** refuses any cloud preset before anything is sent.
- **Attach selection** sends your current selection with the prompt. It is
  off until you tick it. **Attach screenshot** is disabled: the pane says
  viewport screenshot capture is unavailable.
- A **Prompt** box with **Send** and **Cancel**. Each tool the model calls
  shows as a card with its status.
- **Native tools** mode runs tools directly, but asks first for **post**,
  **open_project**, **save_project**, **import_model**,
  **import_tool_library** and **export_tool_library**. **Prompt JSON
  (confirm every edit)** mode asks before every tool that changes the
  document. **Read-only Q&A** mode runs no tools and shows a banner. A
  prompt shows **Confirmation required**, the tool and its arguments, with
  **Allow** and **Deny**. Deny tells the model the user refused.
- **Undo Turn** reverts all the changes of one chat turn in one click. It
  is disabled, and refuses, once you have made a later edit by hand.

The native desktop pane connects to the cloud providers and local Ollama.
Keys come from environment variables or the OS keychain; there is no key
entry form. All local panel presets use Ollama, and the panel has no
endpoint field. See [Use AI with CamCraft](@/guide/automate/ai.md) for
provider setup, external clients, current limits and four worked CAM tasks.

## Undo and redo

Each mutating tool is one undo step, and `undo` / `redo` walk the same
history as Ctrl+Z / Ctrl+Y. In the attached mode that is the open window's
history, so you can undo an agent's step by hand, and the agent can undo
yours.

## Refusals

A tool that cannot do what it was asked answers with an error naming the
tool and the reason, for example `edit_operation: ...`, and the document is
unchanged. An argument the tool does not know is refused rather than
ignored, so a typo never reads as "done". The same reasons the window gives
apply: see [Reading a refusal](@/guide/reference/refusals.md). An unexpected internal
fault is caught and reported as a refusal; it does not end the session.

`save_project` over the file last opened or saved is refused when that file
on disk changed after the session opened or saved it. For example, a second
CamCraft window, a `git pull` or a colleague in a shared folder wrote it.
The error names the file. Save to a different path, or call the tool again
with `overwrite: true` to replace the file. In the window, Save shows a
prompt for the same case.

## Safety

An agent is held to the same rules as you are. `verify` reports what would
block the post, and `post` refuses with its reasons when an operation is
stale, an operation's own checks found something unsafe, the simulation
finds a gouge, or a machine limit is exceeded. There is no way for an agent
to skip these checks. A posted program is still proved the usual way: look
it over in the G-code viewer, and prove it on the machine (air cut, single
block, feed override down) before cutting material. See
[Posting](@/guide/check-and-post/posting.md).

## First check

Inspect the client's CamCraft server status and tool list first. Tool
listing (`tools/list`) needs no model request or paid provider call.
If you use an agent prompt, its provider can bill that conversation:

> Use only the CamCraft server. List its tools, then call
> `list_operation_types` with `{}`. Report the available operation kinds.
> Do not import, edit, generate, save or post anything.

Success means a nonempty tool list and a result with operation kinds,
parameters and defaults, not just the model saying it connected.
In attached mode, also call `get_job` and compare its setup with the open
window. This is read-only. In headless mode, an empty independent job is
normal. Neither check proves that a machining program is safe.
If it fails, read [Connection diagnostics](#connection-diagnostics) before
asking for any edit.
