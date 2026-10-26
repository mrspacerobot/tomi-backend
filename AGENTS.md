# Tomi API: rules for anyone writing code here

This is the Rust backend of the Tomi iPhone app. This file is the steering document for people and coding agents. Read it before changing anything. If a rule here is wrong, change the rule in the same pull request; do not quietly ignore it.

The rules follow the Tomi app's own `AGENTS.md`, translated to Rust and a server, plus widely used Rust guidance (sources at the end). Where Rust or a server needs something different, this file says so.

## Before you call anything done

```
./scripts/check.sh
```

It runs `cargo fmt --check`, `cargo clippy` with every warning as an error, and all tests: unit tests, API tests against a real Postgres, and the architecture tests. It needs a Postgres it may create databases in (see the script). It must pass with no warning you introduced. CI runs the same script on every pull request.

Report honestly: if a check fails or you skipped a step, say so.

## How we work: tests first

1. **Write the failing test first.** For a rule, a unit test next to it, in the feature's folder. For an endpoint, an API test in `tests/api` that calls it the way the app would. Watch it fail for the right reason, then write the code.
2. **Prefer API tests for behaviour.** A flow is proven by calling the routes in order (sign in, save a profile, read it back), not by testing a handler's internals. Unit tests are for rules.
3. **Bug fixes start with a reproduction:** an API test that fails the way the app saw it, then the fix.
4. **Tests never depend on the real clock, time zone, network or a model.** Pass the time in. A flaky test is a bug: fix it, never retry or skip it.
5. **Every rule has a test that breaks when the rule breaks.** Name the change that would slip through if one does not.

## Layout

```
src/
  profile.rs  scans.rs  suggestions.rs  progress.rs  recipe.rs  options.rs  ingredients.rs  library.rs  graph.rs
                one root file and one folder per feature
  profile/      routes.rs (HTTP handlers), db.rs (queries), the pure rules, and their tests
  scans/        routes.rs, db.rs, keep.rs (storage step), fridge.rs and photo.rs (rules)
  suggestions/  routes.rs, suggestion.rs (the API answer) and ranking; opens levels by calling `progress`
  progress/     the cook log, game.rs (every game number), the progress rules and the quiz answers (planned,
                docs/progress.md)
  recipe/       graph recipe adaptation: ingredients, steps, amounts and safety checks
  library/      the reviewed library: its rules, and files.rs (reads it from disk)
  graph/        the graph artifact: its rules, and files.rs (reads it from disk)
  kernel/       the vocabulary several features share: ids, language, display names
  meta/         health and bootstrap routes
  http.rs       the shared HTTP pieces (CurrentUser, CookLanguage) and the router that merges every feature
  ai/           the model behind one interface: Gemini, and a fake in tests
  storage.rs    the private photo bucket behind one interface
  auth/         Supabase token checks, and the lookup of the cook's own user id
  db.rs  config.rs  error.rs  lib.rs  main.rs
tests/
  api/          one test binary: main.rs, helpers.rs, one file per feature
  architecture.rs
migrations/     SQL, additive only
```

- **Three layers inside every feature.** `routes.rs` translates HTTP into calls and shapes the answer. The rules decide. `db.rs` stores. A handler never decides a rule, and a rule never touches I/O.
- **The rules of a feature need no web, database or clock.** Every file of a feature except `routes.rs`, `db.rs`, `files.rs` and `keep.rs` takes the time and the data it needs as arguments. `tests/architecture.rs` fails when one of them names `axum`, `sqlx`, `reqwest`, `tokio`, `tower` or reads the current time, and when a new folder under `src` is not listed there as a feature or as plumbing.
- **Group by feature, not by kind.** A new feature is a root file `src/<feature>.rs` and a folder `src/<feature>/` with `routes.rs`, `db.rs` and its rules, and its API tests in `tests/api/<feature>.rs`. Add it to `FEATURES` in `tests/architecture.rs`. What two features share goes in `kernel`. A feature's `db` module is private: another feature calls the few functions the feature re-exports (`profile::read`, `scans::confirm`), never its SQL.
- **Unit tests sit beside the code in their own file:** `src/profile.rs` declares `#[cfg(test)] mod tests;` and the tests live in `src/profile/tests.rs`. That keeps the code file short.
- **API tests are one binary** (`tests/api/main.rs` declares the other files), so they compile once. Shared setup lives in `tests/api/helpers.rs`.

## Code style: small enough to read at a glance

A person should understand any function without scrolling and any file without a map. Clippy enforces the limits (`clippy.toml`, `[lints]` in `Cargo.toml`), and `tests/architecture.rs` enforces the file length.

| Limit | Value | Enforced by |
|---|---|---|
| Function body | 15 lines of code | `clippy::too_many_lines` |
| Parameters | 3 (counting `self`) | `clippy::too_many_arguments` |
| Branching | cognitive complexity 7 | `clippy::cognitive_complexity` |
| Nesting | 4 blocks | `clippy::excessive_nesting` |
| File | 200 lines, tests included | `tests/architecture.rs` |
| Line | 120 characters | `rustfmt.toml` |

Rust's formatter spreads a struct literal or a long call over several lines, so the function limit is 15 lines here instead of Tomi's 10. `rustfmt.toml` keeps short literals and calls on one line.

- **One function, one job, one level of detail.** If you need "and" to describe it, split it. If you want a comment to label a block, that block is a function and the comment is its name.
- **When a limit is hit, extract and name.** Never raise a limit to get past it, and never silence a lint to get past a limit. A flat `match` that maps every case to a value may be long.
- **Three parameters at most.** More means a missing type: group them into a struct.

  ```rust
  // No: four loose values the reader must keep in order.
  fn write_profile(pool: &PgPool, user_id: Uuid, expected: i64, profile: &Profile)

  // Yes: one named thing.
  pub struct ProfileWrite { pub user_id: Uuid, pub expected_version: i64, pub profile: Profile }
  pub async fn write(pool: &PgPool, write: &ProfileWrite) -> Result<bool, AppError>
  ```

- **Plain statements over clever expressions.** A reader who knows Java or Kotlin but not Rust should follow it. Unwrap with `let ... else`, `match` or `?`, give each step a name, and let the names tell the story. Do not chain `.ok().filter().map().unwrap_or_else()` into one line.

  ```rust
  // No: four steps packed into one expression.
  env::var(name).ok().filter(|value| !value.is_empty()).ok_or_else(|| format!("{name} is not set"))

  // Yes: each case visible.
  fn required(name: &str) -> Result<String, String> {
      match read(name) {
          Some(value) => Ok(value),
          None => Err(format!("{name} is not set")),
      }
  }
  ```

- **Return early.** Handle the failure first with `let ... else` or `?`, and keep the main path unindented.

  ```rust
  let Some(kid) = header.kid else {
      return Err(AuthError::Invalid);
  };
  ```

- **Parse, don't validate.** Turn untrusted input into a type that cannot hold a bad value, once, at the edge. After that, code trusts the type. `SavedId::parse` is the example: a `SavedId` is always a well-formed id, so nothing checks it again.
- **Make wrong states unrepresentable.** Prefer an `enum` to a `bool` plus an `Option`, and `Option<T>` to a magic value. `equipment: Option<BTreeSet<SavedId>>` says "not answered" in its type; an empty list is an answer of its own.
- **Names say what, comments say why.** No abbreviations (`configuration`, not `cfg`, except Rust's own `#[cfg]`). No commented-out code, no `TODO` without an issue number. Every module starts with a `//!` line saying what it is for, in about fifteen words.
- **Rust naming** follows the Rust API Guidelines: `UpperCamelCase` types, `snake_case` functions and modules, `SCREAMING_SNAKE_CASE` constants. Getters have no `get_` prefix (`as_str()`, not `get_str()`). Conversions say their cost: `as_` is free and borrows, `to_` copies, `into_` consumes. Error types read verb-object-error, like `ParseIdError`.
- **No duplication.** Search before writing a helper. The second copy is the moment to extract.
- **Formatting is not a matter of taste:** `cargo fmt` settles it.

## Errors, panics and logs

- **No `unwrap`, `expect`, `panic!`, `todo!` or `unimplemented!` in service code.** Return a `Result` and let `?` carry it. Tests may unwrap: a failed unwrap there is a failed test.
- **Errors are enums with `thiserror`,** one per boundary (`AuthError`, `PatchError`, `AppError`), converted with `From` and `?` where a conversion is always the same, and `map_err` only where the call site decides. `AppError` decides the HTTP status and the stable `code` the app acts on. Codes are permanent once released.
- **A lint override uses `#[expect(lint, reason = "...")]`, never `#[allow]`.** `expect` fails when the lint no longer fires, so stale overrides cannot pile up. Clippy refuses `#[allow]` here. The accepted reasons so far: `tracing`'s macros expand into branches that inflate `cognitive_complexity`, so each log line sits in its own small function; and a flat `match` from each case to its value, or a struct literal that names every field's default and does nothing else, may pass `too_many_lines`.
- **Logs are structured `tracing` events, never `println!`.** They hold status, latency, versions, ids and counts. Never a request body, a photo, chat text, an ingredient or a restriction.
- **No `unsafe`.** The crate forbids it.

## Async and the server

- **async/await only,** on Tokio. Never block inside an async function (no `std::thread::sleep`, no blocking file or network calls). Use `tokio::time::sleep` and async clients.
- **Handlers only translate.** They take extractors (`State`, `CurrentUser`, `Json`) and call the feature's rules and `db.rs`. A handler that takes `CurrentUser` can only be reached with a valid token.
- **State is cheap to clone.** `AppState` holds a pool and `Arc`s, nothing that needs a lock on every request.
- **Every request body is limited** (`DefaultBodyLimit`), and every reply carries an `x-request-id`.

## Food safety and the cook's data

- **Fail closed.** An unknown diet or avoid id is kept, never dropped. A missing or unreadable fact, an incompatible version or a withdrawn lesson is a refusal, never an empty restriction or a guess.
- **A restriction is never weakened silently.** Removing a diet or an avoid item needs `allowWeakerRestrictions: true`, which the app sends only after the cook confirmed it on screen. Two copies that disagree are read with the strictest restrictions of both.
- **This server owns the cook log, every progress rule and the quiz,** per Supabase account (the owner's decision of 26 October 2026, contract in `docs/progress.md`). XP, rank, weekly streak, levels, unlocks, mastery, grants and focus are decided here, in the `progress` feature. The phone keeps an outbox of unsent cook entries and shows the last snapshot this server sent; it never computes or predicts XP or rank. The word is "cook entry", never "event".
- **The cook log is the source of truth.** XP, rank, streak and mastery are computed from it on demand. A stored total is a bug. XP, totals and quiz results are never accepted from the app: an entry with such a field is refused as unknown.
- **No result depends on arrival order.** The same entry id twice counts once. The first accepted copy of an entry id wins; a later copy with a different body is answered `conflict`, changes nothing and is logged with the id and the code only.
- **Week and day arithmetic uses the cook's calendar,** with `jiff` and its bundled time zone database (approved by the owner on 26 October 2026), never the server's clock or seven times 24 hours. Each cook entry stores its own IANA time zone. A past week is judged by each entry's own zone; the week in progress uses the time zone and first weekday sent on the request.
- **No Rust file names a specific skill, diet, avoid item, ingredient, equipment or cuisine.** Restrictions, progression and kitchen options come from reviewed library data in `library/` (`library/SOURCE.md`); recipe content and food facts come only from the graph. Explicitly missing appliances tracked by the library filter recipes; other graph equipment is disclosed as a requirement until kitchen options migrate. Game numbers live in one module, `progress::game`, and the app holds none: what it needs for input (the weekly goal range and default) it reads from `GET /v1/options`.
- **The app holds no food knowledge and no lists.** Diets, avoid items, ingredients (from the graph), equipment, cuisines, experience and units, with their names, come from `GET /v1/options` and from each answer. Every offer says which restrictions it was checked against. `library/choices.json` is the server's own list of cuisines, experience answers and units (wording from Tomi's approved mockups).
- **Code decides what may be offered and their order.** Dinner suggestions must make a graph `dish`, never a `preparation`. Recipes are selected from the graph by open skill level and cuisine. The cook's confirmed ingredients are kept separate from assumed cupboard staples and take priority in deterministic ranking. Available ingredients and restrictions go to the graph safety planner; only graph-listed swaps and omissions are used. Copied lesson JSON never gates availability or supplies recipe content. The preproduction `/v1/lessons/suggest` response carries graph `sourceRecipe` separately from its checked `safePlan`; source instructions are not rewritten to pretend a substitution or omission was adapted.
- **AI runs on this server only for fridge scans, behind the `Ai` interface** (`src/ai.rs`), with a fake in every test. A model may name only ingredient ids from a closed list; it never adds an ingredient, diet, recipe or fact, and `domain` checks its answer before anyone sees it. Recipe selection and ranking are deterministic code.
- **Scan vocabularies belong in the prompt, not a schema enum.** The graph's ingredient list can exceed Gemini's structured schema limits. Keep the scan schema small and check every returned id against the graph; unknown ids remain unlinked labels. AI failure logs carry a specific `error_code`, model, operation and provider HTTP status when available, under the HTTP request id. Never log provider bodies or raw client errors: they can contain the cook's data or credentials. The public scan error remains `ai_unavailable`.
- **Proof of concept: there is no consent yet, and every scanned photo is kept.** The owner's decision, for testing with their own photos only. A photo is sent to the model and kept as it arrived, metadata included, whenever photo storage is set up; what the app sends is the app's decision. Never log a photo, and never let the app reach the bucket. **Before any real cook uses this, build consent properly** (what is kept named in a text the cook agrees to, a way to withdraw that deletes what was kept, versions that re-ask when the text changes) and get a privacy review. Until then nobody else's photo may be scanned, and Gemini must not be on its free tier for real cooks' photos. The server still checks the type, size and first bytes of every upload: it does not trust the app.
- **Voice and chat do not reach this service yet.** Adding them needs consent and a change to this file.

- **The cooking graph is one pinned artifact.** `graph.lock` names the version and sha256 this build serves, the image carries it, and start-up refuses anything else: an unknown schema version, a changed file, another version, an unknown channel, a manifest that disagrees with its artifact about the channel, or a draft without `GRAPH_ALLOW_DRAFT`. A lock that names a graph the server cannot load stops the start. Every answer that uses the graph must carry its version and channel. **The graph is the only source of food facts.** An ingredient, fact or recipe it does not list refuses, never passes, and so does a diet or avoid item `restrictions.json` does not list. `restrictions.json` may only name facts the graph has, or the start is refused. Allergen facts no item covers get an avoid item derived from the graph; `library/avoid-items.released.json` is the reviewed list of every avoid item and a test holds the real list to it. A saved avoid id is read by its exact id, never by an alias (`nuts` stays peanuts and tree nuts together), and an id no item has is read as the graph fact of that name, which is only ever stricter. Without a graph, scans and suggestions answer `503 graph_unavailable`. The server's rule is checked against the graph's own reference on every case the release ships (`tests/graph_vectors.rs`, run by CI when a graph is pinned).

## The adapted recipe

- **Teaching is optional graph content, never inferred.** Suggestions carry `teaching` as described in `docs/teaching.md`. Its primary technique is the first sorted actual recipe technique matched to the offered skill level, including graph variants. Missing or malformed introductions leave teaching unavailable without hiding the checked recipe; missing or malformed quizzes are unavailable independently. Text, answers and tiers come unchanged from the pinned graph. Question IDs are scoped by technique. Production requires recipe, technique, introduction and offered questions to be reviewed or higher; a draft graph keeps its explicit testing status.
- **The quiz is run by this server, and correct answers never leave it.** Quiz questions are sent with their choices, without their answer or explanation (teaching schema 2). Each pick is checked here (`POST /v1/quiz-answers`, planned in #43) and answered right or wrong, with the explanation only once right. The first answer to each question is the score; later picks only teach. A cook's quiz result is derived here from those first answers, never sent by the app.
- **Changed recipes need reviewed teaching compatibility.** Until the graph carries compatibility evidence, any applied substitution or omission makes teaching unavailable. A food-safe swap is not proof that the instructions or quiz remain applicable. Teaching schema 2 supplies content for introduction/preparation; graph-native sessions, resume, timers and completion are separate contracts.


- **The cook's recipe is built in code, never by a model.** `recipe::adapt` takes a graph recipe and its safe plan and writes the steps from the graph's step format (placeholders, plurals, clauses of left-out parts), scales amounts by the swap's ratio, and drops steps that end up empty. A model never selects, ranks, writes, rewrites or checks a recipe.
- **It is checked again, and fails closed.** The finished ingredient list is compared with the cook's avoided facts using the graph's own lists, and a swap or left-out part the graph does not list is refused. A recipe that cannot be adapted cleanly is not offered; it is logged with the recipe id and a reason word, never a food.
- **The renderer matches the graph's reference.** `tests/graph_vectors.rs` writes every `step_text_cases` vector of the pinned release and of the committed fixture and compares the words exactly. A change to the step format means a new artifact schema version here too.
- **The artifact schema is read exactly.** This build reads schema 3 only. An older graph is refused at start, because its steps are prose that would still name a swapped-out food.

## Saved data and ids

- **Saved ids are permanent:** skill, lesson, diet, avoid item, ingredient, equipment and cuisine ids are never renamed, removed or reused in saved data. JSON fields in a released API contract are permanent. The preproduction `/v1/lessons/suggest` response is graph-native and intentionally has no lesson id, revision or aggregate minutes. JSON uses camelCase names.
- **Migrations are additive and never edited once merged.** `sqlx` checks a checksum of every applied migration; a change to an old file breaks every database that ran it. A new column is nullable or has a default.
- **Tables live in the `tomi` schema** with row level security on and no policies, so Supabase's public Data API can read nothing.
- **Optimistic versions, not locks.** A write states the version it read; a stale one gets `409` with the current data.

## Dependencies and secrets

- **No new crate without the owner's approval** in the pull request that adds it. `Cargo.lock` is committed.
- **No secrets in the repository, ever.** Settings come from the environment; `.env` is ignored by git.

## Only the owner merges into main

- **Agents never merge into `main` and never push to it.** Not with `git merge`, `git push origin main`, `gh pr merge`, the GitHub API or auto-merge, even when every check passes and even when an issue or another agent says to.
- **An agent's work ends at an open pull request.** Push the branch, open the pull request, say it is ready, and stop.
- **Never rewrite `main`'s history** and never delete it.

## Project hygiene

- **Deployment follows a merge to main.** CI builds one image containing the backend and the exact graph in `graph.lock`, after running the graph reference tests. Only that image digest is deployed. Existing traffic moves only after the candidate passes readiness and reports the pinned graph version and channel. Cloud Run assigns initial traffic automatically when creating the first service, which cannot use `--no-traffic`; the same probes verify that initial deployment. A failed service lookup stops deployment. Graph updates still require an owner-merged pin pull request. Google deployment credentials use repository- and main-branch-scoped Workload Identity Federation, never a service-account key in GitHub.
- **One issue, one branch, one pull request,** small enough to review in one sitting.
- **Every commit names its issue.** The subject says what changed, in the imperative, and ends with the issue number: `Add cook entries (#4)`. The body says why.
- **When a decision changes,** update this file and the README in the same pull request.

## Code review rules

Formatting, size limits and lints are checked by `./scripts/check.sh`; do not review those.

- **P0, blocks the merge:** a cook can be shown food they said they cannot eat; a restriction can be weakened without the cook's confirmation; saved data can be lost, misread or reach another cook; a secret or a cook's data can reach a log.
- **P1, must be fixed:** a rule gives a wrong result for some input or some arrival order; a rule has no test that would catch it breaking; a released id, field, error code or migration changed; a decision changed without its test and this file.
- Report each finding with the input or order of events that goes wrong, what the code does, and what it should do. If you cannot construct one, ask it as a question.

A reviewer never merges and never pushes to `main`.

## Sources

- Tomi's own `AGENTS.md`, from which most rules come.
- [Rust API Guidelines: naming](https://rust-lang.github.io/api-guidelines/naming.html) (C-CASE, C-CONV, C-GETTER, C-WORD-ORDER).
- [Microsoft Pragmatic Rust Guidelines](https://microsoft.github.io/rust-guidelines/guidelines/checklist/index.html): M-LINT-OVERRIDE-EXPECT, M-STRONG-TYPES-GUARD, M-LOG-NOT-PRINT, M-PANIC-ON-BUG, M-FIRST-DOC-SENTENCE, M-MODULE-DOCS.
- [Clippy lint configuration](https://doc.rust-lang.org/clippy/lint_configuration.html) for the thresholds.
- [Comprehensive Rust: parse, don't validate](https://google.github.io/comprehensive-rust/idiomatic/leveraging-the-type-system/newtype-pattern/parse-don-t-validate.html).
- [RFC 3137: let-else](https://rust-lang.github.io/rfcs/3137-let-else.html), for early returns.
- [Zero to Production in Rust](https://github.com/LukeMathWalker/zero-to-production) for the split between pure rules and handlers, and the single API test binary.
