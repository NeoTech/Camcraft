#!/usr/bin/env bash
# post-merge.yml's reporting (#1451): one rolling issue "main is red
# (post-merge gate)". Red: open it, or comment on the open one, with the
# failing suites, the head sha and the run URL. Green: comment the green sha
# and close it. Never one issue per run.
# Inputs (env): STATUS (success|failure), SHA, RUN_URL, REPO, OUTPUT (the
# gate's stdout file), GH (gh binary override, tests use a fake).
# A failing suite whose name says golden, gcode or post makes the issue P0
# (wrong posted output), otherwise P1.
# Provenance: our own AGENTS.md "Issue Law"; GitHub's public gh CLI docs.
set -uo pipefail
gh_bin="${GH:-gh}"
repo="${REPO:?REPO is required}"
sha="${SHA:?SHA is required}"
title="main is red (post-merge gate)"
short="${sha:0:10}"

open=$("$gh_bin" issue list -R "$repo" --state open --search "\"$title\" in:title" \
  --limit 20 --json number,title \
  --jq "[.[] | select(.title == \"$title\")][0].number // empty") || open=""

if [ "${STATUS:-}" = success ]; then
  if [ -n "$open" ]; then
    "$gh_bin" issue comment "$open" -R "$repo" --body "Green again: the post-merge batch gate passed on \`$sha\` (${RUN_URL:-no run url}). Closing; a new red run opens a new issue." || exit 1
    "$gh_bin" issue close "$open" -R "$repo" --reason completed || exit 1
    echo "closed #$open"
  else
    echo "green, no open red issue"
  fi
  exit 0
fi

# Failing steps: the gate summary's "  FAIL  <secs>s  <step>  <note>" rows.
fails=$(sed -n '/^== gate summary/,$p' "${OUTPUT:-/dev/null}" 2>/dev/null | grep -E '^ +FAIL ' | head -40 || true)
[ -n "$fails" ] || fails="(no FAIL row in the gate summary: the gate did not finish or failed to build; see the run)"
p0=0
if printf '%s\n' "$fails" | grep -qiE 'golden|gcode|g_code|post'; then p0=1; fi

body=$(printf 'The post-merge batch gate failed on `%s`.\n\nRun: %s\n\nFailing steps:\n```\n%s\n```\n\nThe merge that broke it is among the commits up to this head (`git log` since the last green run). Fix it in the normal order; this issue closes itself on the next green run.\n' \
  "$sha" "${RUN_URL:-no run url}" "$fails")

if [ -n "$open" ]; then
  "$gh_bin" issue comment "$open" -R "$repo" --body "Still red on \`$short\` (${RUN_URL:-no run url}).

Failing steps:
\`\`\`
$fails
\`\`\`" || exit 1
  if [ "$p0" = 1 ]; then
    "$gh_bin" issue edit "$open" -R "$repo" --add-label P0 --remove-label P1 || exit 1
  fi
  echo "commented on #$open"
else
  prio=P1; [ "$p0" = 1 ] && prio=P0
  "$gh_bin" issue create -R "$repo" --title "$title" \
    --label "$prio" --label source:agent --label effort:low --label area:build \
    --body "$body" || exit 1
fi
