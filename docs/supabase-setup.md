# Setting up Supabase for the Tomi API

Supabase gives Tomi three things: sign-in (Supabase Auth), the Postgres
database, and later file storage. The Tomi API is the only program that
reads or writes the database. The iPhone app talks to Supabase only to sign
in.

Dashboard menu names change from time to time. If a name below does not
match, look for the nearest equivalent.

## 1. The project

- Pick a region close to where the API will run. From Zurich, Frankfurt
  (`eu-central-1`) pairs well with Cloud Run in `europe-west3` (Frankfurt) or
  `europe-west6` (Zurich).
- Save the database password in a password manager. It never goes into git,
  a chat or the app.
- The free plan pauses a project after about a week without activity, and
  has no backups. Keep nothing there you cannot lose until you upgrade.

## 2. Keep the database private

The API keeps its tables in a schema called `tomi`, with row level security
on and no policies, so no Supabase client key can read them.

- **Project Settings → Data API:** leave the exposed schemas as `public`
  only. Never add `tomi`. If nothing else uses the Data API, you can turn it
  off.
- The app must never hold the `service_role` or secret key, or the
  database password. It holds only the project URL and the publishable
  (anon) key, which is safe to ship.

## 3. Token signing keys

- **Project Settings → JWT Keys:** check that the project signs tokens with
  an asymmetric key (ECC P-256 is the usual choice). If it still uses the
  legacy shared secret, start the migration to signing keys there.
- With asymmetric keys, the API fetches the public keys itself from
  `https://<project-ref>.supabase.co/auth/v1/.well-known/jwks.json`. Nothing
  to copy.
- Only if you stay on the legacy secret: set `SUPABASE_JWT_SECRET` in the
  API's environment.

## 4. Sign-in providers

Under **Authentication → Sign In / Providers**:

- **Apple:** turn it on. For native sign-in on iPhone, add the app's bundle
  id (for example `ch.yourname.tomi`) to the client ids. In the Apple
  Developer account, the App ID needs the Sign in with Apple capability. The
  app then uses the Supabase Swift library's `signInWithIdToken` with the
  token Apple returns. The secret key and Services ID Apple's web flow needs
  are not required for native-only sign-in. Check this against Supabase's
  current Apple guide when you set it up.
- **Email:** decide whether Tomi offers it. If not, turn off sign-ups by
  email. It is handy to keep it on for a test user while developing (step 7).
- **Anonymous sign-ins:** leave off. The API refuses anonymous sessions.

## 5. The connection string

- Click **Connect** at the top of the dashboard and choose **Session
  pooler**. It looks like
  `postgres://postgres.<project-ref>:<password>@aws-0-<region>.pooler.supabase.com:5432/postgres`.
- Use the session pooler (port 5432), not the transaction pooler (port
  6543): the API's database library uses prepared statements, which the
  transaction pooler does not support. The direct connection only works over
  IPv6 unless you buy the IPv4 add-on, and the session pooler avoids that.

## 6. First run against your project

```sh
cp .env.example .env      # then fill in DATABASE_URL and SUPABASE_URL
set -a; source .env; set +a
cargo run
```

On start the API creates the `tomi` schema and its tables. In **Table
Editor**, switch the schema to `tomi` and you should see `users`,
`profiles` and `_sqlx_migrations`.

```sh
curl localhost:8080/readyz          # 200 when the database answers
curl localhost:8080/v1/bootstrap
```

## 7. Try a real token

Sign in with Apple needs a device, so for a first test use an email user:

1. **Authentication → Users → Add user**, with an email and password, and
   mark the email confirmed.
2. Get a token. The publishable key is under **Project Settings → API
   Keys**.

```sh
curl -s "$SUPABASE_URL/auth/v1/token?grant_type=password" \
  -H "apikey: <publishable key>" -H "content-type: application/json" \
  -d '{"email":"you@example.com","password":"<password>"}' | jq -r .access_token
```

3. Call the API with it:

```sh
TOKEN=<access token>
curl -s localhost:8080/v1/profile -H "authorization: Bearer $TOKEN"
curl -s -X PATCH localhost:8080/v1/profile -H "authorization: Bearer $TOKEN" \
  -H "content-type: application/json" -d '{"version":0,"avoid":["nuts"]}'
```

Delete the test user and turn email sign-up back off when you are done, if
Tomi will not offer email.

## 8. The photo bucket, for the dataset

Needed for the photos every scan keeps (proof of concept: there is no consent step yet). Without it, scans still work and nothing is kept.

1. **Storage → New bucket**, named `fridge-photos`. Leave **Public bucket**
   off. Set the file size limit to 10 MB and the allowed types to
   `image/jpeg, image/png`.
2. Add no storage policies. Only the server writes and reads, with the
   secret key, which bypasses them; the app's keys can do nothing there.
3. **Project Settings → API Keys:** copy the secret key (`sb_secret_...`,
   or the legacy `service_role` key) into `SUPABASE_SECRET_KEY` on the
   server. It can read and delete everything in the project: never put it
   in the app, a chat or git.

The server sends the key both as `apikey` and as a bearer token. Check a
first upload in **Storage → fridge-photos** after a test scan; I could not
test against a real project.

The free plan has 1 GB of storage. A fridge photo is around 1 to 3 MB, so
plan to upgrade before the dataset grows.

## 9. Gemini

1. Create a key in Google AI Studio (aistudio.google.com) and put it in
   `GEMINI_API_KEY` on the server.
2. `GEMINI_MODEL` defaults to `gemini-3.8-flash`. If Google says the model
   is not found or not free, pick a current Flash model from Google's model
   list.
3. **Use only your own test photos on the free tier.** Google may use what
   is sent there to improve its models. Switch the project to paid billing
   before real cooks scan, and before keeping anything you plan to sell.

## 10. Deploying to Cloud Run

Merges into `main` deploy automatically after the normal checks and graph reference
tests pass. CI packages the exact pinned graph with the backend and deploys that
image by digest. See [automatic deployment](cloud-run-deployment.md) for the Google
identity, GitHub variables, runtime secrets and rollback procedure.

Store `DATABASE_URL`, `GEMINI_API_KEY` and `SUPABASE_SECRET_KEY` in Secret Manager,
never as plain values in a script you commit. `SUPABASE_URL` is a public repository
variable. The runtime service account reads the three secrets when Cloud Run starts
the revision.

`GRAPH_REQUIRED=true` makes a build that pins no graph refuse to start, so a deploy
without a graph fails its health check and never replaces a working one. Every scan and
suggestion needs the graph. `GRAPH_ALLOW_DRAFT=true` is for the proof of concept only: the graph
has no production release until its safety gates pass, and then it must be removed.

Public Cloud Run access lets the app reach the API. The API checks
every cook's token itself.

## Not done yet

- Account deletion. App Store rules require deleting an account from inside
  the app. That needs a route that marks the Tomi user deleted and removes
  the Supabase user with the secret key, kept only on the server.
- Account recovery, and what happens to a cook who signs in with a second
  method.
- Backups, which the free plan does not include.

## Dish photos

Create a separate public Storage bucket named `dish-photos`. Allow the `image/webp`
MIME type. Leave object writes restricted to the operator's service credential,
with no client write policy. Keep the scan bucket private.

Save the Supabase secret key as a macOS Keychain generic password with service
name `tomi-dish-photos`. Run `./scripts/photos upload --project-url https://<project-ref>.supabase.co`
after review. The command verifies SHA-256, uploads without overwriting, and
checks that the uploaded bytes are publicly readable. The API constructs public
URLs from its configured Supabase origin and needs only the manifest to serve
them. See [the dish photo workflow](dish-photos.md) for the order of operations.
