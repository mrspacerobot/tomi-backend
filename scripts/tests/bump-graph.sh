#!/usr/bin/env bash
# Tests scripts/bump-graph.sh against a stand-in for `gh` that serves tests/fixtures/graph as a release.
set -euo pipefail
cd "$(dirname "$0")/../.."
root=$(pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# A `gh` that knows two calls: `release list` prints $FAKE_TAG, `release download` copies the fixture files.
mkdir "$work/bin"
cat > "$work/bin/gh" <<'SHIM'
#!/usr/bin/env bash
if [ "$1 $2" = "release list" ]; then
  [ -n "${FAKE_TAG:-}" ] && echo "$FAKE_TAG"
  exit 0
fi
if [ "$1 $2" = "release download" ]; then
  while [ $# -gt 0 ]; do [ "$1" = "--dir" ] && dir="$2"; shift; done
  cp "$FAKE_RELEASE"/manifest.json "$FAKE_RELEASE"/graph-artifact.json "$dir"/
  exit 0
fi
echo "unexpected gh call: $*" >&2; exit 1
SHIM
chmod +x "$work/bin/gh"
export PATH="$work/bin:$PATH"

fail() { echo "FAIL: $1" >&2; exit 1; }
fresh() { rm -rf "$work/repo"; mkdir -p "$work/repo/scripts"; cp scripts/bump-graph.sh "$work/repo/scripts/"; printf '%s\n' "$1" > "$work/repo/graph.lock"; }
run() { (cd "$work/repo" && ./scripts/bump-graph.sh 2>&1); }

export FAKE_RELEASE="$root/tests/fixtures/graph"

fresh none
FAKE_TAG="" run | grep -q "no graph release" || fail "no release should say so"
[ "$(cat "$work/repo/graph.lock")" = "none" ] || fail "no release must leave the lock alone"

fresh none
FAKE_TAG="graph-vfixture" run | grep -q "bumped fixture" || fail "a new release should be pinned"
grep -q "^version fixture$" "$work/repo/graph.lock" || fail "the lock should name the version"
grep -q "^sha256 $(sed -n 's/.*"contentSha256":"\([^"]*\)".*/\1/p' "$FAKE_RELEASE/manifest.json")$" "$work/repo/graph.lock" || fail "the lock should name the checksum"

FAKE_TAG="graph-vfixture" run | grep -q "already pins fixture" || fail "a current pin should be left alone"

fresh none
mkdir "$work/bad"; cp "$FAKE_RELEASE/manifest.json" "$work/bad/"; echo '{"changed":true}' > "$work/bad/graph-artifact.json"
if FAKE_RELEASE="$work/bad" FAKE_TAG="graph-vfixture" run; then fail "a checksum mismatch must fail"; fi
[ "$(cat "$work/repo/graph.lock")" = "none" ] || fail "a mismatch must leave the lock alone"
echo "bump-graph tests passed"
