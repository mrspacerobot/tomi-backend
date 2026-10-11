# Running the API on your own machine

No Supabase project is needed. The API accepts a token signed with a shared
secret, so you can make a test token yourself. The values below are for this
machine only. Never use them anywhere real.

## With Docker Compose

```sh
docker compose up --build
curl localhost:8080/readyz
```

This starts Postgres and the API (`compose.yaml`). Set `GEMINI_API_KEY` in
your shell first if you want the fridge scan; without it the scan answers
`503 ai_unavailable`. The image builds and starts this way.

## Without Docker for the API

Run only Postgres, then the API from source:

```sh
docker run -d --name tomi-pg -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres:16
docker exec tomi-pg psql -U postgres -c "create database tomi"
cp .env.example .env
```

Set these in `.env`:

```
DATABASE_URL=postgres://postgres:postgres@localhost:5432/tomi
SUPABASE_URL=http://localhost:54321
SUPABASE_JWT_SECRET=local-dev-secret-at-least-32-characters-long
```

```sh
set -a; source .env; set +a
cargo run
```

## A test token

The API checks the issuer (`$SUPABASE_URL/auth/v1`), the audience and role
`authenticated`, and the expiry. This makes a token valid for 30 days:

```sh
python3 - <<'PY'
import base64, hashlib, hmac, json, time
def b(x): return base64.urlsafe_b64encode(x).rstrip(b"=")
head = b(json.dumps({"alg": "HS256", "typ": "JWT"}).encode())
body = b(json.dumps({
    "sub": "11111111-1111-1111-1111-111111111111",
    "role": "authenticated", "aud": "authenticated",
    "iss": "http://localhost:54321/auth/v1",
    "exp": int(time.time()) + 30 * 86400}).encode())
secret = b"local-dev-secret-at-least-32-characters-long"
sig = b(hmac.new(secret, head + b"." + body, hashlib.sha256).digest())
print((head + b"." + body + b"." + sig).decode())
PY
```

```sh
TOKEN=<the token>
curl -s localhost:8080/v1/profile -H "authorization: Bearer $TOKEN"
```

## The cooking graph

Build a draft artifact from a local checkout of `cooking-graph`, pin it, and start with it:

```sh
scripts/fetch-graph.sh --from ../cooking-graph   # writes graph/ and a local graph.lock
docker compose up --build
curl localhost:8080/v1/bootstrap                 # graphArtifactVersion and graphChannel
```

`compose.yaml` allows a draft graph. Without that switch the start is refused. The pin is embedded in the
binary, so changing `graph.lock` needs a rebuild. Do not commit a pin made with `--from`: commit one only
when it names a published release (`scripts/fetch-graph.sh` with no argument downloads it).

## The iPhone app against this API

Build the app with the server's address as build settings, and start it in
the simulator with the token in `SIMCTL_CHILD_TOMI_DEV_TOKEN`. The app has
no such switch yet: Sign in with Apple cannot work on a local setup, so a
local-only change to the app's sign-in is needed. See the app's pull request
for the current state.

```sh
xcodebuild ... TOMI_API_BASE_URL=http://127.0.0.1:8080 \
  TOMI_SUPABASE_URL=http://127.0.0.1:54321 TOMI_SUPABASE_PUBLISHABLE_KEY=local-dev
```

## Tests

`./scripts/check.sh` needs a Postgres it may create databases in. The
container above on port 5432 is enough.
