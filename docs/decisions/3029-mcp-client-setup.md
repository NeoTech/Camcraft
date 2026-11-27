## #3029 - One MCP client setup reference

**Decided:** Keep the installed-path table and four client configurations
in `guide/automate/mcp.md`. Link there from `ai.md`. Give complete headless
and attached examples, and end both pages with a read-only first check.

**Why:** The client starts a local stdio process in both modes. The attached
process needs `--connect`; the app's loopback listener is not HTTP.
Explicit command paths avoid differences between shell and desktop PATH.
Package-specific paths preserve asset discovery. Windows installer and
portable paths remain user-selected, not universal defaults. Symbolic
links already work; copied executables without assets do not.

**Considered and dropped:** Duplicating client instructions in the AI guide
would make updates inconsistent. A single generic JSON object would leave
Codex's TOML and Claude Code's CLI setup unspecified. Installing clients or
changing user settings would add state without validating the guide's text.

**Verification boundary:** Check syntax, links, current CLI parsing, package
layouts and primary client references. Run focused existing documentation
checks where cached tools permit. Do not claim end-to-end client or
cross-platform acceptance from those checks. No application behavior changes.

**Cost to change later:** Update the canonical examples when a client or
package layout changes. Keep the two first-check sections consistent.
