## #1189 - a P1-P9 priority scale replaces P0/P1/P2 (2026-11-12)

**Decided (maintainer, question dialog, 2026-11-12).** "We might need a P1-P9
prioritization system now.. we're having issues with scope creep throughout
the issue list" -> "P1 to P9 scale: replace P0/P1/P2 with nine levels;
agents keep filing and rating their own issues as today". Read as "P4 is our
normal; P1 to P3 is fix now", with **only `P1` and `P2` blocking a release
cut**; a `P3` is worked ahead of planned work but does not hold a cut back.

**Chosen.** The nine-level table now in AGENTS.md "Priorities" (lower
number first). Rules changed: Pick order, enhancement-lane order, safety
floor (every `P1` is at least `effort:high`), red-gate filing (`P1` wrong
output / gouge, else `P2`), cut blocker (`P1` or `P2` the gate opened;
a `P3` never), MCP issues `P5`, filing rules (`P1`-`P3` defects only; `P4`
only with the maintainer's own words quoted; a follow-up found on the way is
`P7` unless it is a defect; never `P?`). `P?` is unchanged.

**Considered and dropped.**
- Keep `P0`-`P2` and add a maintainer-set scope marker label: leaves three
  levels to order 185 issues, and a second label per issue to keep in sync.
- A scale where only the maintainer may set `P1`-`P4`: would stall agent
  filing on his attention; the table's "defects only" and "his own words"
  rules carry the same restraint without a gate.
- Widening the cut blocker to "any open `P1`/`P2`": the old text scoped it
  to issues the red gate opened; that scope is kept, the numbers changed.

**Mapping.** old `P0` -> `P1`. Old `P1`: red-main bugs -> `P2`; other bugs
-> `P3`; enhancements the maintainer asked for -> `P4`; `area:mcp` and
process -> `P5`; other enhancements -> `P6`. Old `P2`: bugs and polish ->
`P7`; enhancements -> `P8` (`P6` where an open PR works them).

**Watch.**
1. The meaning of `P1` flips (old "user hits wrong behaviour" becomes
   "unsafe output"). The relabel must be one pass at merge, old `P1` moved
   BEFORE old `P0` is renamed, or an old-`P1` issue reads as unsafe.
2. Agents still rate their own issues, so the scale orders the list but does
   not slow its growth.

**Migration script.** Not run by the PR. The merging session runs
`python migrate.py --dry-run`, reads the list, then runs it without the
flag. It touches open issues only; closed ones keep their history. The
`P4` set is the maintainer's list from the issue (#530-#548, #617, #661,
#1103, #1104; the follow-ups of #1089 are added by hand from the dry-run).

```python
#!/usr/bin/env python3
# One-shot P0/P1/P2 -> P1..P9 relabel for #1189. Usage: migrate.py [--dry-run]
import json, subprocess, sys
DRY = "--dry-run" in sys.argv
P4 = set(range(530, 549)) | {617, 661, 1103, 1104}

def gh(*a, check=True):
    return subprocess.run(["gh", *a], capture_output=True, text=True, check=check).stdout

# 1. labels P3..P9 and re-described P1, P2 (red to grey)
LABELS = [
    ("P1", "B60205", "Unsafe or wrong output: gouge, collision, wrong G-code, crash, data loss"),
    ("P2", "D93F0B", "Main is broken or a regression with no workaround; blocks a cut"),
    ("P3", "E99695", "Wrong behaviour a user hits, with a workaround; fix now"),
    ("P4", "F9A03F", "Work the maintainer asked for, for the next cut"),
    ("P5", "FBCA04", "Agent surface and tooling keep up with what shipped"),
    ("P6", "C2E0C6", "Accepted enhancement, not scheduled"),
    ("P7", "BFD4F2", "Owed polish and small fixes"),
    ("P8", "C5DEF5", "Larger idea needing design or research before code"),
    ("P9", "D4D4D4", "Someday: not picked while anything else is free"),
]
existing = {l["name"] for l in json.loads(gh("label", "list", "--limit", "200", "--json", "name"))}
for name, colour, desc in LABELS:
    verb = "edit" if name in existing else "create"
    cmd = ["label", verb, name, "--color", colour, "--description", desc]
    print("gh", *cmd)
    if not DRY:
        gh(*cmd)

issues = json.loads(gh("issue", "list", "--state", "open", "--limit", "1000",
                       "--json", "number,title,labels"))
def names(i): return {l["name"] for l in i["labels"]}
log = []
def move(i, old, new):
    log.append((i["number"], old, new))
    print(f"#{i['number']}: {old} -> {new}")
    if not DRY:
        gh("issue", "edit", str(i["number"]), "--remove-label", old, "--add-label", new)

# 2. old P1 FIRST (while "P1" still means the old thing)
for i in issues:
    n = names(i)
    if "P1" not in n: continue
    if "enhancement" not in n and ("blocked-main" in n or "red-gate" in n):
        new = "P2"
    elif "area:mcp" in n or "area:build" in n or "area:process" in n:
        new = "P5"
    elif "enhancement" in n:
        new = "P4" if i["number"] in P4 else "P6"
    else:
        new = "P3"
    move(i, "P1", new)
# 3. old P0 -> P1, then delete P0
for i in issues:
    if "P0" in names(i):
        move(i, "P0", "P1")
if not DRY:
    gh("label", "delete", "P0", "--yes", check=False)
print("gh label delete P0 --yes")
# 4. old P2 -> P7 / P8 / P6   (P2 issues that were just created above are not in `issues`)
for i in issues:
    n = names(i)
    if "P2" not in n: continue
    new = "P7"
    if "enhancement" in n:
        new = "P8" if "effort:max" in n else "P6"
    move(i, "P2", new)
# 5. open issues with no priority, for a by-hand pass
print("\nNo priority (set by hand with the AGENTS.md table):")
for i in issues:
    n = names(i)
    if not any(x == "P?" or (x.startswith("P") and x[1:].isdigit()) for x in n):
        print(f"  #{i['number']} {i['title']}")
# 6. the audit list
print("\n(issue, old, new):")
for row in log: print(row)
```

Notes on the script: step 2 reads `issues` taken BEFORE any change, so an
old-`P1` issue moved to `P2`/`P3` is never confused with a renamed old
`P0`; step 4 runs from the same snapshot, so an old `P2` is never an issue
that step 2 just labelled `P2`. The `blocked-main`/`red-gate`/`area:process`
label names are guesses at the red-main marker: the dry-run list is read by
the maintainer, and the red-main `P2`s (#1146, #1184 and the like) are fixed
by hand there if no such label exists.
