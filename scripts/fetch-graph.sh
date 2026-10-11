#!/usr/bin/env bash
# Puts the cooking graph artifact in graph/ and keeps graph.lock in step with it.
#
#   scripts/fetch-graph.sh                      download the version graph.lock pins (needs gh and read access)
#   scripts/fetch-graph.sh --from ../cooking-graph
#                                               build a draft artifact from a local checkout and pin it
#
# A pin made with --from is for your own machine. Commit a pin only when it names a published release.
set -euo pipefail
cd "$(dirname "$0")/.."

field() { sed -n "s/.*\"$1\":\"\([^\"]*\)\".*/\1/p" "$2"; }

if [ "${1:-}" = "--from" ]; then
  source_dir="${2:?usage: fetch-graph.sh --from <cooking-graph checkout>}"
  "$source_dir/release"
  (cd "$source_dir/out/release" && shasum -a 256 -c SHA256SUMS)
  cp "$source_dir/out/release/graph-artifact.json" "$source_dir/out/release/manifest.json" graph/
  version=$(field graphVersion graph/manifest.json)
  sha256=$(field contentSha256 graph/manifest.json)
  printf '# Pinned by scripts/fetch-graph.sh --from. For local use only.\nversion %s\nsha256 %s\n' "$version" "$sha256" > graph.lock
  echo "fetch-graph: pinned $version"
  exit 0
fi

version=$(awk '$1 == "version" { print $2 }' graph.lock)
sha256=$(awk '$1 == "sha256" { print $2 }' graph.lock)
if [ -z "$version" ]; then
  echo "fetch-graph: graph.lock pins no graph, nothing to fetch."
  exit 0
fi
gh release download "graph-v$version" --repo mrspacerobot/cooking-graph --dir graph --clobber \
  --pattern graph-artifact.json --pattern manifest.json
actual=$(shasum -a 256 graph/graph-artifact.json | awk '{ print $1 }')
if [ "$actual" != "$sha256" ]; then
  echo "fetch-graph: the downloaded artifact is not the pinned one." >&2
  rm -f graph/graph-artifact.json graph/manifest.json
  exit 1
fi
echo "fetch-graph: fetched $version"
