#!/usr/bin/env bash
# Pins the newest cooking-graph release in graph.lock, when it is not the one already pinned.
#
#   scripts/bump-graph.sh     needs gh with read access to the graph repository
#
# It changes only graph.lock, and only after the downloaded artifact matches the checksum in its manifest.
# With GITHUB_OUTPUT set (in a workflow) it writes `bumped=true|false` and `version=<version>`.
set -euo pipefail
cd "$(dirname "$0")/.."

repo="${GRAPH_REPO:-mrspacerobot/cooking-graph}"
say() { echo "bump-graph: $1"; }
output() { if [ -n "${GITHUB_OUTPUT:-}" ]; then echo "$1" >> "$GITHUB_OUTPUT"; fi; }

tag=$(gh release list --repo "$repo" --limit 1 --json tagName --jq '.[0].tagName // empty')
if [ -z "$tag" ]; then
  say "no graph release exists yet."
  output "bumped=false"
  exit 0
fi

version=${tag#graph-v}
current=$(awk '$1 == "version" { print $2 }' graph.lock)
if [ "$version" = "$current" ]; then
  say "graph.lock already pins $version."
  output "bumped=false"
  exit 0
fi

folder=$(mktemp -d)
trap 'rm -rf "$folder"' EXIT
gh release download "$tag" --repo "$repo" --dir "$folder" --pattern manifest.json --pattern graph-artifact.json
promised=$(sed -n 's/.*"contentSha256":"\([^"]*\)".*/\1/p' "$folder/manifest.json")
actual=$(shasum -a 256 "$folder/graph-artifact.json" | awk '{ print $1 }')
if [ -z "$promised" ] || [ "$actual" != "$promised" ]; then
  say "the artifact of $tag does not match the checksum in its manifest. Nothing was changed." >&2
  exit 1
fi

printf '# The graph this build serves, pinned by scripts/bump-graph.sh or fetch-graph.sh.\n# A change is a reviewed change.\nversion %s\nsha256 %s\n' "$version" "$promised" > graph.lock
say "bumped $version"
output "bumped=true"
output "version=$version"
