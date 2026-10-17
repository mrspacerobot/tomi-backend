# Automatic Cloud Run deployment

The `ci` workflow deploys every merge to `main`, including changes to `graph.lock`.
Pull requests never deploy. The existing `bump-graph` workflow opens a graph-pin
pull request when a graph release appears; the owner still chooses when to merge it.

The target is service `tomi-api` in `europe-west3` (Frankfurt), project
`tomi-prod-9k3x7q`. Images live in that region's `tomi-api` Artifact Registry
repository. The image tag identifies the Git commit; its `io.tomi.graph.version`
label identifies the graph. Cloud Run receives the immutable image digest.

## What happens after a merge

1. Run `./scripts/check.sh` with a real test Postgres.
2. Download exactly the graph release pinned in `graph.lock`, verify its checksum,
   and run every reference case shipped with it. A missing pin fails this job.
3. Build the backend and graph into one image. Pass that image between jobs as a
   GitHub Actions artifact, without rebuilding it during deployment.
4. After both checks pass, authenticate to Google through Workload Identity
   Federation and push the image to Artifact Registry.
5. Look up the service. For an existing service, deploy a revision tagged
   `candidate` with `--no-traffic`. For the first creation, omit that flag:
   Cloud Run rejects it and assigns initial traffic automatically. A failed
   lookup stops deployment rather than being treated as a missing service.
6. Call its `/health`, `/readyz` and `/v1/bootstrap`. The bootstrap response must name the
   expected graph version and channel. Only then move existing traffic to that
   revision. On the first creation, these probes verify the initial deployment;
   there is no previous revision to keep serving if a probe fails.

Deployments are serialized. A queued run whose commit is no longer `main` is
skipped, so an older completed build cannot replace a newer deployment. If a
check or candidate probe fails, the workflow stops before moving existing traffic.
The deployment script's regression tests run in `./scripts/check.sh` and cover
first creation, existing services, failed lookups, failed health/readiness, and incorrect
graph versions and channels.

Use `/health` for public process checks. Google Frontend intercepts `/healthz`
with a 404 before it reaches the application; the old path remains available
locally for compatibility.

The server also verifies the embedded graph pin at startup, with
`GRAPH_REQUIRED=true`. `GRAPH_ALLOW_DRAFT` is true only when the pinned graph is
a draft. Draft graphs and photo scans are for the owner's testing, as specified
in `AGENTS.md`.

## Google and GitHub configuration

No Google service-account key is stored in GitHub. The identity provider trusts
only repository ID `1411982146`, owner ID `56773331`, the `main` branch, and this
repository's `.github/workflows/ci.yml` workflow.

Repository variables:

| Name | Purpose |
| --- | --- |
| `GCP_WORKLOAD_IDENTITY_PROVIDER` | Full Google Workload Identity provider resource name |
| `GCP_DEPLOY_SERVICE_ACCOUNT` | `tomi-api-deploy@tomi-prod-9k3x7q.iam.gserviceaccount.com` |
| `SUPABASE_URL` | Public HTTPS URL of the Supabase project used by the deployed API |

The deploy account has Cloud Run Admin in this project, Artifact Registry Writer
on the `tomi-api` repository, and Service Account User on the runtime account.
Federated repository identity has Workload Identity User on the deploy account.

The runtime account is
`tomi-api-runtime@tomi-prod-9k3x7q.iam.gserviceaccount.com`. It has Secret Manager
Secret Accessor on these three secrets, individually:

| Cloud Run environment variable | Secret Manager secret |
| --- | --- |
| `DATABASE_URL` | `tomi-database-url:latest` |
| `GEMINI_API_KEY` | `tomi-gemini-key:latest` |
| `SUPABASE_SECRET_KEY` | `tomi-supabase-secret:latest` |

The required Google APIs are Cloud Run, Artifact Registry, IAM, IAM Service
Account Credentials, Security Token Service and Secret Manager. The workflow
deploys a public HTTP service; protected API routes enforce Supabase tokens.

## Redeploy and rollback

Run the `ci` workflow manually on `main` to redeploy the current commit. It runs
the same checks and uses the same pin, never the newest graph automatically.

To roll back, choose a previously working Cloud Run revision. It carries its own
backend and graph together:

```sh
gcloud run services update-traffic tomi-api \
  --project tomi-prod-9k3x7q --region europe-west3 \
  --to-revisions PREVIOUS_REVISION=100
```

Rollback changes traffic, not database migrations or secrets. Migrations must
remain additive. Revert the corresponding commit through a pull request to keep
future merges on the intended version.

## References

- [GitHub identity federation](https://github.com/google-github-actions/auth)
- [Cloud Run revision traffic](https://docs.cloud.google.com/run/docs/rollouts-rollbacks-traffic-migration)
