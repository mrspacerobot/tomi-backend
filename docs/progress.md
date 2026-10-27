# Cook log, progress and quiz contract

On 26 October 2026 the owner decided that this server owns the cook log, every game and progress rule (XP, rank,
weekly streak, levels, unlocks, mastery, grants and focus) and the quiz, per Supabase account. The phone becomes a
thin client: it keeps an outbox of unsent cook entries, shows the last progress snapshot this server sent, and never
computes or predicts XP or rank. The quiz is run here: the phone gets questions and choices, sends each pick, and is
told right or wrong. Correct answers never leave the server.

**Status: planned.** None of these routes exists yet; only `game.weeklyGoal` in `GET /v1/options` and the pinned
released catalog (#39) are built. This document is the contract the following pull requests implement, and each of
them keeps it current:

| Issue | Delivers |
|---|---|
| mrspacerobot/tomi-backend#36 | The `progress` feature and the game rules, ported from the app with its tests |
| mrspacerobot/tomi-backend#37 | The weekly streak in the cook's time zone and first weekday (`jiff`) |
| mrspacerobot/tomi-backend#38 | `POST /v1/cook-entries` and the stored cook log |
| mrspacerobot/tomi-backend#39 | `game.weeklyGoal` in `GET /v1/options`, and the released skill catalog pinned by a test |
| mrspacerobot/tomi-backend#40 | `GET /v1/progress`, and the snapshot in the cook-entries answer |
| mrspacerobot/tomi-backend#41 | Suggestions open levels from the cook's own log |
| mrspacerobot/tomi-backend#42 | Quiz questions sent without their answers (teaching schema 2) |
| mrspacerobot/tomi-backend#43 | `POST /v1/quiz-answers` |
| mrspacerobot/tomi-backend#44 | Each cook's quiz result derived from its recorded first answers |

All routes need sign-in. JSON uses camelCase, times are RFC 3339 instants in UTC, and errors use the usual shape
(`{"error": {"code", "message", "details"}}`). Every status, problem code and field named here is permanent once
released.

## The rules in one place

- **The cook log is the source of truth.** Progress is computed from the stored entries on every request. A stored
  total is a bug. The app never sends XP, a total or a quiz result.
- **Game numbers live in `progress::game`** and nowhere else, the app included: XP per finished cook (50), the quiz
  bonus (10), the rank thresholds and the XP per tier past the top, the weekly goal range (1 to 7, default 3), the
  cooks to complete a level (2), the tolerance for a cook in the future (10 minutes), and `RULES_VERSION`, which
  changes whenever a game number or rule does.
- **The skill catalog is `library/catalog.json`** here, the canonical reviewed copy (`library/SOURCE.md`).
  `library/catalog.released.json` lists every released skill id with its level numbers; a test fails when the
  catalog differs from it, and when a released skill or level is gone or renumbered.
- **Arrival order never matters.** The same entry id twice counts once. The first accepted copy of an entry id wins;
  a later copy with a different body is answered `conflict`, changes nothing, and is logged with the id and the code
  only. Quiz answers recorded before or after their entry give the same result.
- **A stopped-early cook counts for nothing.** A skill or level the catalog does not know still earns XP and counts
  for the streak, and opens nothing.

## The cook's calendar

```json
{ "timeZone": "Europe/Amsterdam", "firstWeekday": 2 }
```

- `timeZone` is an IANA identifier and `firstWeekday` is 1 (Sunday) to 7 (Saturday), Foundation's numbering. Both
  are required. A missing or unknown value is `422 invalid_request`, never a silent UTC default.
- The arithmetic is Gregorian, done with `jiff` and its bundled time zone database, by calendar steps and never by
  seven times 24 hours, so a week across a daylight saving change is right.
- **Each cook entry stores its own time zone.** A week before the current one is judged by each entry's own zone: a
  cook made in Tokyo counts on its Tokyo day. The week in progress uses the zone on the request. The first weekday
  on the request is used for every week.
- "Now" is the request time, passed into the rules by the route.

## `POST /v1/cook-entries`

The phone sends the cooks in its outbox, possibly late and possibly twice. The phone makes a UUID for each cook when
cooking begins; that id is the idempotency key, so a retried batch is always safe.

```json
{
  "calendar": { "timeZone": "Europe/Amsterdam", "firstWeekday": 2 },
  "entries": [
    {
      "schemaVersion": 1,
      "id": "4e2c2a0e-7e6b-4c7a-9c2b-6e0a3d1f9a11",
      "cookedAt": "2026-10-26T18:42:00Z",
      "timeZone": "Europe/Amsterdam",
      "dish": { "recipeId": "recipe/omelette-french", "graphVersion": "2026.10.20" },
      "skill": "eggs",
      "level": 1,
      "outcome": "finished",
      "weeklyGoal": 3
    }
  ]
}
```

- At most 100 entries per batch.
- `dish` is exactly one of `recipeId` with `graphVersion`, for a graph recipe, or `lessonId`, for the old
  development logs already on phones.
- `outcome` is `finished` or `stoppedEarly`.
- `weeklyGoal` is the goal when the cook was made, so raising the goal later never judges a past week again.
- **No `quizCorrect`, no XP and no totals.** Unknown fields are refused per entry. The quiz result is derived here
  from the answers recorded under the entry's id (see "The quiz").

The answer is `200` whenever the batch itself could be read, with one result per entry and the snapshot after the
batch:

```json
{
  "results": [
    { "id": "4e2c2a0e-7e6b-4c7a-9c2b-6e0a3d1f9a11", "status": "accepted" },
    { "id": "9b1f0c4d-2a3e-4f5b-8c6d-7e8f9a0b1c2d", "status": "rejected", "problems": ["cooked_in_future"] }
  ],
  "progress": { "schemaVersion": 1 }
}
```

(`progress` is cut short here; it is the full snapshot below.)

| Status | Meaning | The phone |
|---|---|---|
| `accepted` | Stored now | Removes it from the outbox |
| `duplicate` | The same id with the same body is already stored | Removes it |
| `conflict` | The same id with a different body is already stored. The first copy stays and counts; nothing changes | Removes it, logs the id and the code only |
| `rejected` | Not stored. `problems` holds permanent codes | Marks it refused, never retries, logs the codes |

Problem codes of a rejected entry:

| Code | When |
|---|---|
| `unsupported_schema_version` | `schemaVersion` is not one this server reads |
| `invalid_skill_id` | The skill is not a well-formed id |
| `level_out_of_range` | The level is not a valid level number |
| `weekly_goal_out_of_range` | The goal is outside the game's weekly goal range |
| `cooked_in_future` | `cookedAt` is later than the receipt by more than the game's tolerance |
| `invalid_time_zone` | The entry's `timeZone` is not a known IANA zone |
| `recipe_does_not_teach_level` | The pinned graph knows the recipe, and its levels do not include this skill and level |

A skill or level the catalog does not know is **accepted**: it is ignored for mastery, still earns XP and counts for
the streak, so a newer catalog can recognise it later.

Batch errors: `401` and `403` as for every route; `422 invalid_request` for an unreadable body, more than 100 entries
or a bad `calendar`. The phone keeps its entries on any batch error and tries again later; it never drops them.

## `GET /v1/progress?timeZone=Europe/Amsterdam&firstWeekday=2`

Returns the snapshot, computed from the stored log, the cook's profile (`experience`, `weeklyGoal`, `focusSkill`)
and the catalog. The phone asks on launch, when it becomes active, and after a profile change; the cook-entries
answer carries the same snapshot.

```json
{
  "schemaVersion": 1,
  "rulesVersion": 1,
  "catalogVersion": 1,
  "computedAt": "2026-10-26T18:43:02Z",
  "calendar": { "timeZone": "Europe/Amsterdam", "firstWeekday": 2 },
  "xp": 360,
  "rank": { "id": "commis", "name": "Commis", "tier": 1, "startsAt": 150, "nextAt": 400 },
  "streak": {
    "weeks": 2, "weeksBeforeThisWeek": 2, "daysThisWeek": 1, "goal": 3, "daysLeftInWeek": 4,
    "thisWeekMet": false, "atRisk": false, "lost": false,
    "weekStartsAt": "2026-10-25T23:00:00Z", "weekEndsAt": "2026-11-01T23:00:00Z"
  },
  "skills": [
    {
      "skill": "eggs", "status": "mastered", "missingPrerequisites": [], "currentLevel": 2,
      "levels": [
        { "number": 1, "cooks": 2, "cooksToComplete": 2, "open": true, "granted": false, "complete": true },
        { "number": 2, "cooks": 2, "cooksToComplete": 2, "open": true, "granted": false, "complete": true }
      ]
    }
  ],
  "masteredCount": 4,
  "focusSkill": "roux",
  "counted": { "finishedCooks": 6, "ignoredUnknownSkillOrLevel": 0 }
}
```

- Every number a screen shows is here, so the phone does no game arithmetic. The rank bar's fraction is drawn from
  `xp`, `startsAt` and `nextAt` as given.
- `rank.id` is permanent; `rank.name` follows `Accept-Language`, like the names in `GET /v1/options`.
- `streak.weekEndsAt` lets the phone tell that a kept snapshot's week is over.
- `skills` is in catalog order and keyed by the skill ids `GET /v1/options` sends. `status` is `locked`, `served`
  or `mastered`. `experience: comfortable` grants the first course, shown as `granted`, with no XP.
- `schemaVersion` is the shape; an added field does not change it. `rulesVersion` is `progress::game::RULES_VERSION`.
  `catalogVersion` is the catalog's `libraryVersion`.
- A new cook gets a complete snapshot of zeros, never missing fields.
- A missing or unknown `timeZone`, or a `firstWeekday` outside 1 to 7, is `422 invalid_request`.

## The quiz

Suggestions carry each quiz question as `{id, question, choices, tier}`, **without its answer or explanation**
(teaching schema 2, `docs/teaching.md`). The answers stay in the pinned graph, read by question id. Until #42 is
merged, teaching schema 1 still sends them.

### `POST /v1/quiz-answers`

```json
{
  "entryId": "4e2c2a0e-7e6b-4c7a-9c2b-6e0a3d1f9a11",
  "recipeId": "recipe/omelette-french",
  "graphVersion": "2026.10.20",
  "questionId": "technique/omelette/quiz/heat",
  "choice": 1
}
```

```json
{ "questionId": "technique/omelette/quiz/heat", "correct": true, "scored": true, "explanation": "…" }
```

- `entryId` is the id the phone made when cooking began and later sends as the cook entry's `id`, so the answers
  and the entry meet by id, in either order.
- One pick in, right or wrong out. `explanation` is sent only once the pick is right; the correct index is never
  sent.
- **The first answer to each question is the score.** `scored` is true only for the first recorded pick of that
  question in that cook; later picks are checked and answered, but only teach, and are not stored.
- A `graphVersion` other than the pinned graph is `409 graph_version_changed`. A question that is not part of the
  quiz taught for that recipe, or a choice that is not one of its choices, is `422 invalid_request` and stores
  nothing.
- The quiz needs a network connection.

### How a cook's quiz result is derived

A cook **passed the quiz** when its entry is `finished`, its answers are for the same recipe as the entry, every
question of that quiz has a recorded first answer, and every first answer is right. It then earns the quiz bonus
from `progress::game`. A wrong first answer stays wrong after a right retry, an unanswered question means no bonus,
and answers with no entry earn nothing.

## `GET /v1/options` additions

```json
"game": { "weeklyGoal": { "minimum": 1, "maximum": 7, "default": 3 } }
```

Read from `progress::game`, never typed twice. `PATCH /v1/profile` checks `weeklyGoal` against the same values and
refuses one outside them with `422 invalid_request`. A new cook's goal is the default. Built (#39).
