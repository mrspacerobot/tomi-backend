# Tomi API

The Rust backend the Tomi iPhone app talks to. It follows the Tomi Rust
backend specification of 8 October 2026, slice by slice: sign-in and the
profile, then the AI functions (fridge scan and tonight's suggestions).

## What exists

| Route | Sign-in | What it does |
|---|---|---|
| `GET /health` | no | The process is up (`/healthz` remains a local alias; Google Frontend intercepts it on Cloud Run) |
| `GET /readyz` | no | The database answers |
| `GET /v1/bootstrap` | no | API versions, minimum app version, and the graph version and channel (`graphArtifactVersion`, `graphChannel`). Library and safety versions are `null`: none is published yet. Both graph fields are `null` while `graph.lock` pins no graph |
| `GET /v1/profile` | yes | The cook's profile and its version. A new cook gets the defaults at version 0 |
| `PATCH /v1/profile` | yes | Changes fields of the profile. See below |
| `GET /v1/options` | no | Every list the app offers (diets, things to avoid, cuisines, equipment, experience, units, ingredients, the skill menu) with names in the cook's language |
| `POST /v1/ingredients/resolve` | yes | Typed text to ingredients: `exact`, `ambiguous` (the cook chooses) or `none` |
| `POST /v1/scans` | yes | A fridge photo (`image/jpeg` or `image/png`, up to 10 MB) in, what the model saw out |
| `POST /v1/lessons/suggest` | yes | Confirmed ingredients in, a safe graph recipe and up to two alternatives out |
| `POST /v1/cook-entries` | yes | One unsent cook and the cook's calendar in, one `accepted`, `duplicate`, `conflict` or `rejected` result out. The entry id is the idempotency key. Example: [`docs/examples/cook-entries.json`](docs/examples/cook-entries.json). The answer carries the progress snapshot also available through `GET /v1/progress` |
| `GET /v1/progress?timeZone=<IANA>&firstWeekday=<1-7>` | yes | The progress snapshot (XP, rank, weekly streak, skills and levels, focus), computed from the stored cook log on every request in the request's calendar. Example: [`docs/examples/progress.json`](docs/examples/progress.json) |

Planned, not built yet (contract in [`docs/progress.md`](docs/progress.md)):

| Route | Sign-in | What it will do |
|---|---|---|
| `POST /v1/quiz-answers` | yes | One quiz pick in, right or wrong out; correct answers never leave the server |

`GET /v1/options` will also carry the weekly goal range and default (`game.weeklyGoal`).

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

### Tonight's suggestions

Each suggestion can also carry optional graph teaching content for its offered technique: the introduction and
validated quiz questions, with explicit content tiers and the graph identity. Missing teaching leaves the recipe
browsable. Substitutions and omissions make teaching unavailable until their compatibility has been reviewed.
See [the teaching contract](docs/teaching.md) and [its example](docs/examples/teaching.json). This first delivery
supplies introduction/preparation content; guided sessions and completion remain separate deliveries.

```json
{ "ingredients": ["egg", "spinach"], "scanId": "…", "staplesLeftOut": ["flour"] }
```

The cook's cupboard staples are added (the assumed ones until they have answered the cupboard
question), less any in `staplesLeftOut` for tonight. A staple left out never removes something
the cook confirmed. Dinner suggestions only use graph recipes that make dishes, not preparations.
Confirmed ingredients are ranked ahead of assumed cupboard staples. The backend filters recipes by
open skill levels and "never" cuisines, then the cooking graph selects a safe plan for the cook's
restrictions and ingredients. The server deterministically ranks dishes by confirmed ingredient
matches, foods to buy, and pantry overlap. Recipe ranking does not call a model. Without a graph the
answer is `503 graph_unavailable`. `rankedBy` is `"overlap"` for API compatibility. Each offer carries
`recipeId`, `sourceRecipe` from the graph, and a
`safePlan` with checked ingredients, allowed substitutions (including ratios and caveats),
omissions, missing ingredients, and required equipment. `sourceRecipe` preserves the graph's
original instructions and amounts; it is never rewritten. It stays for now because the app still reads it, and
the app must not describe it as adapted. It keeps the graph's raw field names, including snake_case step fields;
the surrounding API uses camelCase. Each response includes `graphVersion` and `graphChannel`.

Each offer also carries the recipe already adapted for the cook, built in code and never by a model:

- `recipe.yields`: `{value, unit}`.
- `recipe.ingredients`: `{part, id, name, amount: {value, unit}, forms: [{id, name}]}`, the foods in use (a
  stand-in where one was swapped in), the amount scaled by the swap's ratio where it has one, no left-out part.
- `recipe.steps`: `{number, text, time, temperature, safetyPoints}`. `text` has the foods in use written in, singular
  or plural, in English (step text is English only for now); the clauses of left-out parts and the steps that were
  only about them are gone and the rest are numbered again from 1. Time, temperature and safety points are the
  graph's, unchanged.
- `changes.swaps`: `{removed, added, ratio, caveat}` for each swap made. `changes.leftOut`: `{part, name,
  ingredients}` for each optional part the cook goes without, `name` being the first food it would have held.

The steps are written from the graph's step format (`docs/recipe-steps.md` in the graph repository), then the
adapted ingredient list is checked again against the cook's diet and avoid list. A recipe that cannot be adapted
cleanly (a placeholder with no line, a plural the graph lacks, a ratio other than 1 that measures a line another way, a food
the graph does not list for its line, an unsafe food, a required part left out) is not offered, and the server logs
the recipe id and a reason word, never a food. Example, soft scrambled eggs when the cook has crème fraîche but
no cream:

```json
{ "changes": { "swaps": [{ "removed": { "id": "ingredient/cream", "name": "Cream" },
                           "added": { "id": "ingredient/creme-fraiche", "name": "Crème fraîche" },
                           "ratio": null, "caveat": null }], "leftOut": [] },
  "recipe": { "yields": { "value": 2, "unit": "serving" },
    "ingredients": [{ "part": "enrichment", "id": "ingredient/creme-fraiche", "name": "Crème fraîche",
                      "amount": { "value": 1, "unit": "tablespoon" }, "forms": [] }],
    "steps": [{ "number": 3, "time": null, "temperature": null, "safetyPoints": ["raw-or-lightly-cooked-egg"],
                "text": "While the eggs still look wet and loose, take the pan off the heat and stir in the cold butter and the crème fraîche to stop the cooking. Serve at once on warm plates." }] } }
```

(`ingredients` and `steps` are cut to one entry here.) An optional part, such as spinach in the lemon-butter pan
sauce, appears under `changes.leftOut` and its clauses and steps are gone when the cook cannot have it.
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

Recipes are offered only at skill levels the cook has reached. This server owns
the cook log and every progress rule (owner's decision of 26 October 2026,
[`docs/progress.md`](docs/progress.md)); the app holds none. The cook log is
stored (`tomi.cook_entries`, one row per cook per entry id, each with its own
IANA time zone), but suggestions do not read it yet, so for now only the
onboarding answer counts: "comfortable" opens the whole first course.

Errors always look like
`{"error": {"code": "...", "message": "...", "details": {...}}}`.

The shared calendar in `src/kernel/calendar.rs` validates IANA zones and finds local week boundaries; progress owns the streak rules.

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
The artifact layout this build reads is schema 3 (steps in the placeholder format, plural names, `serving.step_text`). An older
layout is refused at start: its steps would still name a food after it was swapped out. The `bump-graph` workflow checks daily (and on demand) for a newer graph release. When there is one it pins it in
`graph.lock`, runs the conformance tests against that graph, and only then opens a pull request, which a person
merges. `scripts/fetch-graph.sh` fetches the pinned release, or with `--from <checkout>` builds and pins one locally.

See `docs/local-development.md` to run it. Merges into `main` automatically deploy the backend and
the graph pinned in `graph.lock` together to Cloud Run, after the normal checks and graph reference
tests pass. Existing traffic moves only after the candidate reports the expected graph version and
passes readiness. Cloud Run assigns initial traffic automatically on the first service creation.
See `docs/cloud-run-deployment.md` for the deployment identity, secrets and rollback.

## Layout and rules

`AGENTS.md` has the layout and the coding rules, with examples. In short:
HTTP request and response types describe the API boundary; validated domain types and rule names describe what they mean.
one folder per feature (`src/profile`, `src/scans`, `src/suggestions` and so on), with HTTP handlers in
`routes.rs`, storage in `db.rs` and pure rules alongside them, shared HTTP pieces in `src/http.rs`, API tests as one
binary in `tests/api`. `src/recipe` holds the recipe adaptation rules. `src/cook_entries` owns the cook log. `src/progress` holds the progress rules
ported from the app (XP, rank, weekly streak, open levels, served and mastered skills) and serves the progress
snapshot of `GET /v1/progress`. Suggestion and recipe
response types use the same names as the frontend: `Suggestion`, `SuggestionsRequest`,
`SuggestionsResponse`, `AdaptedRecipe`, `AdaptedIngredient` and `RecipeChanges`.

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

## Debugging AI requests

Search Cloud Run logs for `jsonPayload.fields.message="AI request failed"`.
Each failure carries `error_code`, `model`, `operation` (`fridge_scan`), and
`provider_http_status` for provider rejections. The
request span carries `request_id`, matching the response's `x-request-id`.

For example, HTTP 400 is `ai_provider_invalid_request`; authentication failures,
missing models, rate limits and provider outages have separate codes. Local
timeouts use `ai_timeout`, connection failures `ai_transport_error`, and bad
JSON, invalid model content, empty responses, blocked responses and exhausted
output limits also have distinct codes. Provider bodies and raw client errors
are never logged. The app still receives the stable `503 ai_unavailable` response
for scan failures. Recipe selection and ranking do not call a model.

The scan's allowed graph ingredient ids are supplied in the prompt, with a small
response schema. Every returned ingredient id is checked against the graph;
unrecognised ids remain plain labels for the cook to confirm or skip.

## Not built yet, and doubts

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
- Next slices: the cook log, progress and the server-run quiz ([`docs/progress.md`](docs/progress.md)), then active recipe, ratings and leaderboard.
