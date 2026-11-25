## #3751 - `camcraft-cli --skill <folder|claude|copilot> install` (part 2)

The maintainer asked for "a --skill <folder,claude,copiplot> install
creating a AI skill for working with the CLI tool". The issue left the
target paths, the scope and the overwrite rule open. Each answer below
was taken unattended, as the recommended option.

**1. Command shape.** `camcraft-cli --skill <target> install [--dir <path>]
[--user] [--force]`. The lane is in its own file, `crates/app/src/cli_skill.rs`,
so that `cli.rs` stays under 400 lines. It is a `HELP_LANES` entry, so the
help tests check it. Exit: 0 written, 1 refused, 2 usage.

**2. One source for the skill text.** The text is the hand-written body
`crates/app/src/cli_skill.md` (our own words: the lanes, the exit codes,
examples, rules for an agent), followed by a generated "Command reference":
`TOP_USAGE` and each `HELP_LANES` usage text, exactly as the binary prints
them. A new lane or flag thus appears in the installed skill with no edit.
Each target only adds the header that its agent reads.
- Dropped: a copy of `.claude/skills/camcraft-mcp/SKILL.md`. That file is a
  process file of this repository, about MCP, not the CLI. A shipped skill
  that reads it would make each process edit a product change. The body
  points to `camcraft-cli mcp reference` for the tool table instead.
- Dropped: a file under `assets/`. The body is compiled in with
  `include_str!` next to its only reader. No path lookup at run time.
- Relation to #1947 (single-source agent skills): this skill is one more
  single-source text. It does not change the repository's own skill files.

**3. The `folder` target** writes `SKILL.md` (with the Agent Skills header
`name` / `description`) directly into the folder that `--dir` names. The
folder is the skill folder. `--dir` is needed: no default folder.
- Dropped: a `camcraft-cli/` subfolder under `--dir`. "Write the skill
  files into a given folder" reads as the folder itself.

**4. The `claude` target, default scope: the project.** It writes
`.claude/skills/camcraft-cli/SKILL.md` under the current folder (or
`--dir`). Reason: it writes nothing outside the folder that the user is in.
The user scope stays as the explicit flag `--user`
(`~/.claude/skills/camcraft-cli/SKILL.md`; the home folder is `HOME`, else
`USERPROFILE`). `--user` with `--dir`, or with another target, is a usage
error.
- Source: the Claude Code skills layout (`.claude/skills/<name>/SKILL.md`
  for a project, `~/.claude/skills/<name>/SKILL.md` for a user, with YAML
  `name` and `description`). The `description` value is quoted, because it
  holds ": ".
- Dropped: the user scope as the default (it writes into the home folder
  without a flag); no user scope (the issue names "for the user").

**5. The `copilot` target** writes
`.github/instructions/camcraft-cli.instructions.md` under the current
folder (or `--dir`), with the header `applyTo: "**"`.
- Source: the GitHub documentation "Adding repository custom instructions
  for GitHub Copilot"
  (https://docs.github.com/en/copilot/how-tos/configure-custom-instructions/add-repository-instructions,
  read 2026-11-25). Path-specific files are `.github/instructions/NAME.instructions.md`
  with an `applyTo` glob; `"**"` matches all files. When the repository-wide
  `.github/copilot-instructions.md` is there too, Copilot uses both.
- Why not `.github/copilot-instructions.md`: it is one file for the whole
  repository and often holds the user's own text. Our own file next to it
  never collides with it.
- Limit: the page says that GitHub.com reads path-specific files only for
  the Copilot cloud agent and Copilot code review. Editors that read
  `.instructions.md` files read it too.
- Dropped: `.github/skills/` (an Agent Skills folder). The issue asks for
  the place where Copilot reads custom instructions.

**6. Never overwrite silently.** When the target file is there (a file, a
folder or a link), the lane refuses with exit 1, names the path, writes
nothing, and says that `--force` replaces it. `--force` is the only way to
replace it.
- Dropped: a merge into an existing file (we cannot know what the user
  changed); a backup copy (a second file that nobody asked for).

**7. No MCP tool.** The lane writes a text file for an agent. It changes no
document state, has no undo, and is no command in the sense of
"Everything Is a Command". An MCP agent already has `camcraft-cli mcp
reference`. No follow-up issue is filed.

Cost to change later: a target path is one match arm in `Target::file`
and one test assertion. A user who installed the old path keeps the old
file until he removes it.
