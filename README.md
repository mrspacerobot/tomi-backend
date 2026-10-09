# Tomi API

The Rust backend the Tomi iPhone app talks to. It follows the Tomi Rust
backend specification of 8 October 2026, slice by slice. This is the first
slice.

## What exists

| Route | Sign-in | What it does |
|---|---|---|
| `GET /healthz` | no | The process is up |
| `GET /readyz` | no | The database answers |
| `GET /v1/bootstrap` | no | API versions, minimum app version. Library, safety and graph versions are `null`: none is published yet |
| `GET /v1/profile` | yes | The cook's profile and its version. A new cook gets the defaults at version 0 |
| `PATCH /v1/profile` | yes | Changes fields of the profile. See below |

Sign-in is Supabase Auth. The app sends the Supabase access token as
`Authorization: Bearer <token>`. The API checks the signature, issuer,
audience and expiry, refuses anonymous sessions, and creates its own user
row the first time it sees a cook.

### Changing the profile

```json
{ "version": 3, "avoid": ["fact/peanuts", "fact/sesame"], "servings": 4 }
```

- `version` is the version the app last read. If someone changed the
  profile since, the answer is `409 version_conflict` with the current
  profile, and the app tries again from that.
- A field that is present replaces the stored one. `null` clears
  `experience` or `focus_skill`.
- Removing a diet or an avoided fact is refused with `409
  weakens_restrictions` unless the request also says
  `"allow_weaker_restrictions": true`. The app sends that only after the
  cook confirmed the removal on screen.
- Ids are typed: `diets` holds `diet/...`, `avoid` holds `fact/...` (the
  cooking graph's safety facts, such as `fact/peanuts`), `cuisines` holds
  `cuisine/...`, `pantry` holds `ingredient/...`, `equipment` holds
  `equipment/...`, `focus_skill` is a `skill/...`. Unknown fields are
  refused.

Errors always look like
`{"error": {"code": "...", "message": "...", "details": {...}}}`.

## Running it

See `docs/supabase-setup.md` for the Supabase side. Then:

```sh
cp .env.example .env     # fill it in
set -a; source .env; set +a
cargo run
```

## Checks

`./scripts/test.sh` runs formatting, lints and all tests. The tests need a
Postgres they can create databases in (see the script). CI runs the same
script on every pull request.

## Not built yet, and doubts

- **Ids are checked for shape and kind, not existence.** The spec wants an
  unknown id refused. That needs the reviewed library and safety snapshot,
  which do not exist yet.
- **Field names are a guess.** They must be matched to Tomi's `SavedProfile`
  before the backend becomes canonical; the spec says saved names stay
  stable. `skill/...` and `diet/...` as id kinds are also my guess.
- **"Conflicting restrictions merge toward the stricter result"** is handled
  here by refusing a stale write and by refusing silent removals. The merge
  of an imported SwiftData profile is not built.
- **Consent** has columns but no route yet.
- **No rate limits yet.** Cloud Run limits concurrency per instance; real
  limits come with the next slice.
- **The Docker image was not built here** (no Docker daemon in the session
  that wrote it). The binary and all tests were.
- Next slices, from the spec: cook events and progress, then the library
  and the graph artifact, then active lesson, ratings and leaderboard.
