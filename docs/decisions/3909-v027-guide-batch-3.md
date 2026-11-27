## #3909 - v0.27 guide batch 3

**What was decided:** Cover the next eight unticked entries, in issue order.
This is a partial umbrella contribution. It does not close #3909.

| Merged PR | Guide section | Decision |
|---|---|---|
| [#3833](https://github.com/NeoTech/Camcraft/pull/3833) | 2D Contour | Explain the outside ramp, Helix substitution, tab and length limits, and compensation refusal. Do not promise ramped links between all rings. |
| [#3861](https://github.com/NeoTech/Camcraft/pull/3861) | Command line: Control a running window | Give the call, bridge and discovery forms, window selection, undo ownership and refusal exits. Keep syntax separate from captured output. |
| [#3900](https://github.com/NeoTech/Camcraft/pull/3900) | Tools: Create and edit a tool; First run | Replace the dock-pane description with the separate window and Document/Catalog sources. |
| [#3903](https://github.com/NeoTech/Camcraft/pull/3903) | Tools: Create and edit a tool | Explain Tool and Details, Apply, the tool rail and independent drafts. |
| [#3496](https://github.com/NeoTech/Camcraft/pull/3496) | Rotary Offset | Explain the Zigzag closing ring and the skipped duplicate seam edges. |
| [#3516](https://github.com/NeoTech/Camcraft/pull/3516) | 2D Pocket | Explain protected picks, full-tool clearance, rest machining and the MCP setter. |
| [#3579](https://github.com/NeoTech/Camcraft/pull/3579) | 2D Pocket | Quote the missed-relief note and explain that material remains at the boundary corner. |
| [#3762](https://github.com/NeoTech/Camcraft/pull/3762) | Troubleshooting: GPU compute falls back to the CPU | Distinguish compute fallback from window rendering and driver recovery. |

**Why:** The merged PRs and current implementation establish these behaviors.
The existing pages omit them or still describe the former tool dock.
The eight focused site tests pin the missing guide facts. The existing
navigation test checks built links and anchors.

**Considered and dropped:** A new tutorial, new screenshots and CAM changes
are not needed for this prose batch. Existing operation pictures stay as
illustrations, not evidence of the new behavior. Editing the shared MCP
page was dropped because an open contribution changes it. The CLI page
links to it instead. Other open 2D-guide changes affect Chamfer and Bore,
not this batch's Contour and Pocket paragraphs. No stacking is needed.
The current source has a Recipes tab as well as Tool and Details; this
batch does not describe the editor as limited to two tabs.

**Cost to change later:** Update the affected guide sections and their
focused site assertions. No project format, generated program, machine
file, snapshot or process policy changes.
