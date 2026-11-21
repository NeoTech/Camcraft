## #3119 - The AI guide test reads a CRLF checkout

**Decided:** `crates/app/tests/ai_guide.rs` changes CRLF to LF after it reads
`site/content/guide/automate/ai.md`. A new test makes a CRLF copy of the
guide and checks that it gives the same `tools/call` examples as the LF text.

**Why:** a Windows checkout with `core.autocrlf` writes the guide with CRLF.
The test split the text on "```json\n", found no fence, and failed. The fix is
in the one place that reads the file, and it works in each checkout that
exists now, with no new checkout or `git add --renormalize` step.

**Dropped:** a `.gitattributes` rule (`site/content/** text eol=lf`). It
makes Git write LF only in new checkouts. An existing Windows clone keeps
CRLF until the user checks out the files again, so the test stays red for
them. The rule also changes the bytes of the site sources for each Windows
author, which is a larger change than a test fix.

**Cost to change later:** low. A `.gitattributes` rule can come later. The
normalization in the test then has no effect, and the test stays correct.
