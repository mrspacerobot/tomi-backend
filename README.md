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

### The profile

The fields and their JSON names are those of Tomi's `SavedProfile`:
`diet`, `avoid`, `cuisinesLoved`, `cuisinesToTry`, `cuisinesNever`,
`cupboardStaples`, `equipment`, `experience` (`never`, `basics`,
`comfortable`), `servings`, `units` (`metric`, `cups`), `weeklyGoal` (1 to
7), `focusSkill` and `onboardingDone`. Ids are Tomi's own, such as `nuts`,
`vegetarian` or `olive-oil`.

```json
{ "version": 3, "diet": "vegetarian", "avoid": ["nuts", "sesame"], "servings": 4 }
```

- `version` is the version the app last read. If someone changed the
  profile since, the answer is `409 version_conflict` with the current
  profile, and the app tries again from that.
- A field that is present replaces the stored one. `null` clears `diet`,
  `equipment` ("not answered"), `units` or `focusSkill`.
- Removing an avoid item, or clearing or changing the diet, is refused with
  `409 weakens_restrictions` unless the request also says
  `"allowWeakerRestrictions": true`. The app sends that only after the cook
  confirmed the change on screen.
- A cuisine in two lists, an id of the wrong shape, an unknown field or a
  number out of range is refused with `422 invalid_request`.

Errors always look like
`{"error": {"code": "...", "message": "...", "details": {...}}}`.

## Layout and rules

`AGENTS.md` has the layout and the coding rules, with examples. In short:
pure rules in `src/domain`, one file per feature in `src/routes` and
`src/db`, API tests as one binary in `tests/api`.

## Running it

See `docs/supabase-setup.md` for the Supabase side. Then:

```sh
cp .env.example .env     # fill it in
set -a; source .env; set +a
cargo run
```

## Checks

`./scripts/check.sh` runs formatting, lints and all tests. The tests need a
Postgres they can create databases in (see the script). CI runs the same
script on every pull request.

## Not built yet, and doubts

- **Ids are checked for shape, not existence.** The spec wants an
  unknown id refused. That needs the reviewed library and safety snapshot,
  which do not exist yet.
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
