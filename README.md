# Tomi API

The Rust backend the Tomi iPhone app talks to. It follows the Tomi Rust
backend specification of 8 October 2026, slice by slice: sign-in and the
profile, then the AI functions (fridge scan and tonight's special).

## What exists

| Route | Sign-in | What it does |
|---|---|---|
| `GET /healthz` | no | The process is up |
| `GET /readyz` | no | The database answers |
| `GET /v1/bootstrap` | no | API versions, minimum app version, and the graph version and channel (`graphArtifactVersion`, `graphChannel`). Library and safety versions are `null`: none is published yet. Both graph fields are `null` while `graph.lock` pins no graph |
| `GET /v1/profile` | yes | The cook's profile and its version. A new cook gets the defaults at version 0 |
| `PATCH /v1/profile` | yes | Changes fields of the profile. See below |
| `GET /v1/options` | no | Every list the app offers (diets, things to avoid, cuisines, equipment, experience, units, ingredients, the skill menu) with names in the cook's language |
| `POST /v1/ingredients/resolve` | yes | Typed text to ingredients: `exact`, `ambiguous` (the cook chooses) or `none` |
| `POST /v1/scans` | yes | A fridge photo (`image/jpeg` or `image/png`, up to 10 MB) in, what the model saw out |
| `POST /v1/lessons/suggest` | yes | Confirmed ingredients in, a safe graph recipe and up to two alternatives out |

Sign-in is Supabase Auth. The app sends the Supabase access token as
`Authorization: Bearer <token>`. The API checks the signature, issuer,
audience and expiry, refuses anonymous sessions, and creates its own user
row the first time it sees a cook.

### The profile

The fields and their JSON names are those of Tomi's `SavedProfile`:
`diet`, `avoid`, `cuisinesLoved`, `cuisinesToTry`, `cuisinesNever`,
`cupboardStaples`, `cupboardAnswered`, `equipment`, `experience` (`never`, `basics`,
`comfortable`), `servings`, `units` (`metric`, `cups`), `weeklyGoal` (1 to
7), `focusSkill` and `onboardingDone`. Ids are Tomi's own, such as `nuts`,
`vegetarian` or `olive-oil`.

`cupboardAnswered` is false until the cook has answered the cupboard question: until then the
staples a kitchen is taken to have (`assumed` in `GET /v1/options` `staples`) count, and
`cupboardStaples` is not read. A staple a change adds must be an ingredient the graph knows
(`422 invalid_request`). `GET /v1/options` also lists the `staples` the cupboard offers, and
each avoid item says whether it is an `allergen` or a `choice`.

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

### The fridge scan

The body is the photo itself, with its `Content-Type`. The answer:

```json
{ "scanId": "…", "kept": true,
  "items": [ { "label": "eggs", "ingredient": "egg", "confidence": 0.9 },
             { "label": "a jar of pesto", "ingredient": null, "confidence": 0.7 } ] }
```

The model may only name ingredients the cooking graph knows (every id is listed once, as the allowed values); anything
else comes back as a plain label for the cook to confirm, type or skip.
**Proof of concept: every scanned photo is kept**, as it arrived and with what was seen, in a
private bucket (`kept: true`), when photo storage is set up (`SUPABASE_SECRET_KEY`).
Nothing asks the cook yet. A consent step must come before real cooks use it.
Without a model configured the answer is `503 ai_unavailable`, and the app
falls back to typing. Without a graph it is `503 graph_unavailable`.

### Tonight's special

```json
{ "ingredients": ["egg", "spinach"], "scanId": "…", "staplesLeftOut": ["flour"] }
```

The cook's cupboard staples are added (the assumed ones until they have answered the cupboard
question), less any in `staplesLeftOut` for tonight. A staple left out never removes something
the cook confirmed. The backend filters recipes by open skill levels and "never" cuisines, then
the cooking graph selects a safe plan for the cook's restrictions and ingredients. Without a graph the
answer is `503 graph_unavailable`. The model then reorders the short
list and writes a one-line reason; it cannot add a recipe. Without a model,
or when it fails, the order is by fewest ingredients to buy
(`rankedBy: "overlap"`). Each offer carries `recipeId`, `sourceRecipe` from the graph, and a
`safePlan` with checked ingredients, allowed substitutions (including ratios and caveats),
omissions, missing ingredients, and required equipment. `sourceRecipe` preserves the graph's
original instructions and amounts; it is not rewritten when `safePlan` applies a substitution or
omission. The app must present `safePlan` alongside source instructions and must not describe the
source instructions as adapted. `sourceRecipe` keeps the graph's raw field names, including
snake_case step fields; the surrounding API uses camelCase. Each response includes `graphVersion`
and `graphChannel`.
With a `scanId` of a kept photo, the confirmed ingredients are recorded with it as the dataset's labels.

This preproduction endpoint now returns graph-native recipes in place of the earlier lesson-shaped
offers. It has no lesson ID, lesson revision, or aggregate minutes because the graph supplies none of
those values. Existing skill IDs and profile progress fields remain in use. The backend no longer
loads copied lesson JSON; recipe identity, content, ingredients, techniques, and equipment come
from the pinned graph. `/v1/specials` remains an unsupported old alias.
An explicitly missing appliance from the existing kitchen profile filters a recipe. Equipment
the profile does not yet track, such as pans and utensils, stays visible in `safePlan.equipment`
as a requirement.

### The app holds no lists

The app shows what the server sends and sends back ids. Names come in the
language of the `Accept-Language` header, English when a text has none.
Every offer carries `checked: {diet, avoid}`, the restrictions the server
checked it against: the app hides an offer checked against less than the
cook's own restrictions. Scan items name their ingredient as `{id, name}`;
swaps read `{removed, added}`, as in the app's `AppliedSwap`.

Recipes are offered only at skill levels the cook has reached, by
Tomi's `ProgressRules`. The server has no cook log yet, so for now only the
onboarding answer counts: "comfortable" opens the whole first course.

Errors always look like
`{"error": {"code": "...", "message": "...", "details": {...}}}`.

## The cooking graph

The `cooking-graph` repository builds a versioned, checksummed artifact (`./release` there). This repository
pins exactly one version of it in `graph.lock` (a version and a sha256, or `none`), and the image carries that
artifact in `/graph`. Graph version and image move together, so rolling the image back rolls the graph back.

At start the API reads `GRAPH_DIR`, checks the artifact against its manifest and against the pin, refuses it
unless the schema version, the checksum, the pinned version and the channel all agree, and loads every thing
into memory. A pin that cannot be loaded stops the start. With no pin the API starts without graph features
(scans and suggestions answer `503 graph_unavailable`), unless `GRAPH_REQUIRED=true`, which a deployment sets so that
such a build does not start at all.
A `draft` graph is refused unless `GRAPH_ALLOW_DRAFT=true`; the graph repository can build nothing else yet.
The `bump-graph` workflow checks daily (and on demand) for a newer graph release. When there is one it pins it in
`graph.lock`, runs the conformance tests against that graph, and only then opens a pull request, which a person
merges. `scripts/fetch-graph.sh` fetches the pinned release, or with `--from <checkout>` builds and pins one locally.

See `docs/local-development.md` to run it. Merges into `main` automatically deploy the backend and
the graph pinned in `graph.lock` together to Cloud Run, after the normal checks and graph reference
tests pass. The new revision must report the expected graph version and pass readiness before it
receives traffic. See `docs/cloud-run-deployment.md` for the deployment identity, secrets and rollback.

## Layout and rules

`AGENTS.md` has the layout and the coding rules, with examples. In short:
one folder per feature (`src/profile`, `src/scans`, `src/special` and so on), each with its
`routes.rs`, `db.rs` and pure rules, shared HTTP pieces in `src/http.rs`, API tests as one
binary in `tests/api`.

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

- **The AI was not tried against the real Gemini API** from the session
  that wrote it (no key). Request and answer shapes follow Google's
  documentation and are unit tested; the first real scan will tell.
- **Progress counts no cooks yet,** because the server has no cook log. New cooks see recipes at
  their first open level, and "comfortable" opens the first course.
- **Serving sizes are not scaled:** the graph supplies source amounts, but the API does not adapt
  them to the cook's saved serving count.
- **Ids are checked for shape, not existence.** The spec wants an
  unknown id refused. That needs the reviewed library and safety snapshot,
  which do not exist yet.
- **"Conflicting restrictions merge toward the stricter result"** is handled
  here by refusing a stale write and by refusing silent removals. The merge
  of an imported SwiftData profile is not built.
- **Consent:** none. Every scanned photo is kept (see "The fridge scan"). Build it before real cooks.
- **No rate limits yet.** Cloud Run limits concurrency per instance; real
  limits come with the next slice.
- **The Docker image was not built here** (no Docker daemon in the session
  that wrote it). The binary and all tests were.
- Next slices, from the spec: cook events and progress, then active recipe, ratings and leaderboard.
