#!/usr/bin/env bash
# Deploys one checked image and promotes its candidate after graph and readiness probes.
set -euo pipefail

existing=$(gcloud run services list --project "$CLOUD_PROJECT" --region "$REGION" \
  --filter="metadata.name=$SERVICE" --format='value(metadata.name)')
deploy_flags=(--allow-unauthenticated --tag candidate --quiet --format=json)
if [ -n "$existing" ]; then
  deploy_flags+=(--no-traffic)
fi

gcloud run deploy "$SERVICE" --project "$CLOUD_PROJECT" --region "$REGION" \
  --image "$DEPLOY_IMAGE" --service-account "$RUNTIME_ACCOUNT" \
  --update-env-vars "SUPABASE_URL=$SUPABASE_URL,GRAPH_REQUIRED=true,GRAPH_ALLOW_DRAFT=$ALLOW_DRAFT" \
  --update-secrets 'DATABASE_URL=tomi-database-url:latest,GEMINI_API_KEY=tomi-gemini-key:latest,SUPABASE_SECRET_KEY=tomi-supabase-secret:latest' \
  "${deploy_flags[@]}" > "$RUNNER_TEMP/service.json"
url=$(jq -er '.status.traffic[] | select(.tag == "candidate") | .url' "$RUNNER_TEMP/service.json")
revision=$(jq -er '.status.traffic[] | select(.tag == "candidate") | .revisionName' "$RUNNER_TEMP/service.json")
curl --fail --silent --show-error "$url/health"
curl --fail --silent --show-error "$url/readyz"
curl --fail --silent --show-error "$url/v1/bootstrap" > "$RUNNER_TEMP/bootstrap.json"
jq -e --arg version "$GRAPH_VERSION" --arg channel "$GRAPH_CHANNEL" \
  '.graphArtifactVersion == $version and .graphChannel == $channel' "$RUNNER_TEMP/bootstrap.json"
gcloud run services update-traffic "$SERVICE" --project "$CLOUD_PROJECT" --region "$REGION" \
  --to-revisions "$revision=100" --quiet
printf 'Deployed `%s` with graph `%s` (%s).\n' "$GITHUB_SHA" "$GRAPH_VERSION" "$GRAPH_CHANNEL" >> "$GITHUB_STEP_SUMMARY"
