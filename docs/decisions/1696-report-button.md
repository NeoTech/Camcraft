# Decisions - issue #1696

## #1696 - the Report door: a prefilled GitHub issue and a clipboard screenshot

**What was decided.** A Report door on the status strip opens a small
window. It takes a short title, and its two buttons ("Report a bug",
"Request a feature") open the browser on a prefilled new-issue URL:
`https://github.com/NeoTech/Camcraft/issues/new?title=...&body=...&labels=...`
(no `template` parameter: the prefill query applies to the blank issue,
and the repository enables blank issues in
`.github/ISSUE_TEMPLATE/config.yml`). The body embeds the Debug info
report (#157) - same fields, one source. A "Copy screenshot" button puts
a PNG of the whole application window on the clipboard (see the
full-window follow-up below); the user pastes it into the issue box and
GitHub uploads it. The repository also gained `blank-issue.md` and a
`config.yml` that enables blank issues.

**Why.** The maintainer asked (issue #1696) for one button that prepares
a bug or feature report with a screenshot and the relevant data
attached. GitHub's prefill query is the only documented way to fill a
new issue from an app, and it works only on a blank template. egui's
`Context::open_url` and `Context::copy_image` exist in the pinned
egui 0.36.2 and are already plumbed by the shell's platform-output
handler, so no new dependency is needed.

**Considered and dropped.**
- Opening the existing issue FORMS (`bug.yml`, `feature.yml`) prefilled:
  GitHub ignores the query on issue forms (community discussion
  orgs/community#15477). The blank template plus the `labels` query
  reproduces their `bug` / `enhancement` labels.
- Attaching the PNG file from the app: a browser cannot receive a file
  from another program. The clipboard paste is the shortest path, and
  GitHub uploads a pasted picture on its own.
- The GPU `snapshot_rgba()` path for the picture: the viewport's own
  readback needs a device and is unused by the app. The CPU raster
  (`camcraft_app::raster`, the MCP `render_stock` path) works headless
  and deterministic.
- Writing the report body through a new command: the window writes only
  view state (an open flag, a title string), the same class as the
  Debug info window's state, so "Everything Is a Command" does not
  apply.

**Cost to change later.** The URL and the label names are constants in
`report.rs`; a fork pointing at its own repo changes one constant. If
GitHub later lets a URL prefill an issue form, the app code needs no
change: the same query works on that form.

## #1696 follow-up - the screenshot's ground grid (GUI test finding,
superseded by the full-window design below)

**What was decided.** The maintainer's first GUI test found the CPU-raster
shot's background grid wrong: dark scratch lines over the part. The fix
gave the raster shot its own floor grid. That whole path was then
superseded (next section): the shot is now the real window, so the grid
problem cannot occur. The raster additions it needed (`Canvas::ground_line`,
`Camera::scale`, the `interact` visibility widenings) were reverted;
nothing else used them.

## #1696 follow-up - a full-window shot from the real renderer (GUI test
finding)

**What was decided.** The maintainer's second GUI test said the shot must
be the whole application window, not just the 3D model area. The door now
asks the shell for a window picture the way egui intends: the report path
pushes `egui::ViewportCommand::Screenshot` with a private token. The frame
loop (`shell_window/capture.rs`) sees the command in the root viewport's
output and, in the same frame after the normal egui pass, draws the same
tessellated shapes a second time into a private texture
(`RENDER_ATTACHMENT | COPY_SRC`, the surface config's size and format)
in its own render pass. It then copies the texture to a mapped buffer and
answers egui with `egui::Event::Screenshot`, which the door reads: on a
matching token it copies the picture to the clipboard with
`Context::copy_image` and says so in the status line. SolidCraft
(tangency-labs/solidcraft) was read as mechanism reference only -
their public API use, no code, no look (their brand licence).

**Why.** egui's own screenshot responder lives in eframe; our shell
drives winit and `egui_wgpu::Renderer` itself, so nobody else answers the
command. A second pass in the same frame is the only way to catch the
window as the user just saw it: the report dialog the user pressed is
still up, and its shapes are already tessellated. Reading back the
surface texture itself is not possible: wgpu does not make the swap-chain
texture a copy source. `egui_wgpu`'s own `capture` feature was dropped
without trying: its readback rejects the `Bgra8UnormSrgb` surface format
ours uses, and enabling the feature edits Cargo.toml for one call we must
rewrite anyway. A CPU raster cannot show panels, the ribbon or the dialog -
that was exactly the complaint.

**Sub-decisions taken with it.**
- Token law: `egui::UserData::eq` compares `Arc::ptr_eq`, and Rust
  allocates nothing for a zero-sized payload, so every `UserData::new(())`
  shares one pointer and all tokens compare equal. The shot token is a
  `OnceLock` `UserData` over a non-zero string; the unit test
  `the_shot_token_matches_only_itself` pins both halves of the law.
- Delivery: egui 0.36.2 has no `Context::send_event`, so the map callback
  posts over an `mpsc` channel and the frame loop drains it into
  `raw_input.events` before `ctx.run_ui`. The loop keeps
  `request_redraw` alive while a readback is in flight, so the answer
  lands without user input.
- Format: the swizzle table maps RGBA (0,1,2,3) and BGRA (2,1,0,3);
  the raw read-back of an sRGB format is already sRGB-encoded, so no
  gamma step runs. Any other surface format refuses with a logged status
  line - a wrong-byte-order picture would mis-report colours silently.
- Test law: the kittest harness (wgpu/snapshot features) answers
  screenshot requests itself with the requester's own token. The
  foreign-token test therefore makes no request at all: only the injected
  foreign answer could trigger a copy there.
