#!/usr/bin/env bash
# Exercises the real deployment script with deterministic Cloud Run and HTTP stand-ins.
set -euo pipefail
cd "$(dirname "$0")/../.."
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir "$work/bin"
export PATH="$work/bin:$PATH"
export RUNNER_TEMP="$work" CALLS="$work/calls" GITHUB_STEP_SUMMARY="$work/summary"
export CLOUD_PROJECT=test-project REGION=test-region SERVICE=test-service
export DEPLOY_IMAGE=registry.test/api@sha256:123 RUNTIME_ACCOUNT=runtime@test.invalid
export SUPABASE_URL=https://test.supabase.co ALLOW_DRAFT=true
export GRAPH_VERSION=test-version GRAPH_CHANNEL=draft GITHUB_SHA=test-commit

cat > "$work/bin/gcloud" <<'SHIM'
#!/usr/bin/env bash
set -euo pipefail
case "$*" in
  'run services list '*)
    echo lookup >> "$CALLS"
    if [ "${FAIL_LOOKUP:-false}" = true ]; then exit 23; fi
    if [ "${SERVICE_EXISTS:-false}" = true ]; then echo "$SERVICE"; fi
    ;;
  'run deploy '*)
    echo deploy >> "$CALLS"
    if [ "${SERVICE_EXISTS:-false}" = false ] && [[ " $* " == *' --no-traffic '* ]]; then
      echo '--no-traffic not supported when creating a new service.' >&2
      exit 2
    fi
    if [ "${SERVICE_EXISTS:-false}" = true ] && [[ " $* " != *' --no-traffic '* ]]; then
      echo 'Existing traffic must stay on the working revision.' >&2
      exit 2
    fi
    echo '{"status":{"latestReadyRevisionName":"unrelated-revision","traffic":[{"tag":"candidate","url":"https://candidate.test","revisionName":"candidate-revision"}]}}'
    ;;
  'run services update-traffic '*)
    [[ " $* " == *' --to-revisions candidate-revision=100 '* ]] || exit 2
    echo promote >> "$CALLS"
    ;;
  *) echo "Unexpected gcloud call: $*" >&2; exit 1 ;;
esac
SHIM

cat > "$work/bin/curl" <<'SHIM'
#!/usr/bin/env bash
set -euo pipefail
case "$*" in
  *'/health')
    echo health >> "$CALLS"
    if [ "${FAIL_HEALTH:-false}" = true ]; then exit 22; fi
    echo ok
    ;;
  *'/readyz')
    echo ready >> "$CALLS"
    if [ "${FAIL_READINESS:-false}" = true ]; then exit 22; fi
    echo ok
    ;;
  *'/v1/bootstrap')
    echo bootstrap >> "$CALLS"
    printf '{"graphArtifactVersion":"%s","graphChannel":"%s"}\n' "${REPORTED_VERSION:-$GRAPH_VERSION}" "${REPORTED_CHANNEL:-$GRAPH_CHANNEL}"
    ;;
  *) echo "Unexpected curl call: $*" >&2; exit 1 ;;
esac
SHIM
chmod +x "$work/bin/gcloud" "$work/bin/curl"

fail() { cat "$work/output" >&2; echo "FAIL: $1" >&2; exit 1; }
run() { : > "$CALLS"; ./scripts/deploy.sh > "$work/output" 2>&1; }
promoted() { grep -q '^promote$' "$CALLS"; }

SERVICE_EXISTS=false run || fail 'the first service must be created without --no-traffic'
promoted || fail 'a checked first service must finish deployment'
SERVICE_EXISTS=true run || fail 'an existing service must keep traffic until probes pass'
expected=$(printf 'lookup\ndeploy\nhealth\nready\nbootstrap\npromote')
[ "$(cat "$CALLS")" = "$expected" ] || fail 'traffic must move only after every probe'

if FAIL_LOOKUP=true run; then fail 'a failed lookup must stop deployment'; fi
[ "$(cat "$CALLS")" = lookup ] || fail 'a failed lookup must not be mistaken for a missing service'

if SERVICE_EXISTS=true FAIL_HEALTH=true run; then fail 'health failure must fail deployment'; fi
if promoted; then fail 'health failure must preserve traffic'; fi

if SERVICE_EXISTS=true FAIL_READINESS=true run; then fail 'readiness failure must fail deployment'; fi
if promoted; then fail 'readiness failure must preserve traffic'; fi

if SERVICE_EXISTS=true REPORTED_VERSION=wrong run; then fail 'a different graph must fail deployment'; fi
if promoted; then fail 'a different graph must preserve traffic'; fi

if SERVICE_EXISTS=true REPORTED_CHANNEL=wrong run; then fail 'a different channel must fail deployment'; fi
if promoted; then fail 'a different channel must preserve traffic'; fi
echo 'deployment tests passed'
