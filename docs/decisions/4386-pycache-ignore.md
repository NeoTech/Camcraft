## #4386 - Python bytecode is never tracked

**What was decided.** The 17 tracked `.pyc` blobs under
`site/tests/__pycache__/` and `site/tools/__pycache__/` are removed from the
index, and `.gitignore` gets the rules `__pycache__/` and `*.pyc` for the
whole tree.

**Why.** The blobs came in with the laser stack's icons commit
(`1d174e09c`). Each run of the site tests rewrites them, so a worktree that
ran the tests showed changed tracked files, and the blobs were the only
conflicts of #4288. A rule for the whole tree also covers a new Python
folder.

**Considered and dropped.** A rule only for `site/`: other folders can get
Python tools later, and no tracked file anywhere needs bytecode.

**Cost to change later.** Two lines of `.gitignore`.
