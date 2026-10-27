# Cook log, progress and quiz contract

On 26 October 2026 the owner decided that this server owns the cook log, every game and progress rule (XP, rank,
weekly streak, levels, unlocks, mastery, grants and focus) and the quiz, per Supabase account. The phone becomes a
thin client: it keeps an outbox of unsent cook entries, shows the last progress snapshot this server sent, and never
computes or predicts XP or rank. The quiz is run here: the phone gets questions and choices, sends each pick, and is
told right or wrong. Correct answers never leave the server.

**Status: in progress.** `POST /v1/cook-entries` stores the cook log (#38); its answer gains the progress
snapshot with #40. The other routes are planned. This document is the contract the following pull requests
implement, and each of them keeps it current:

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
- Precisely: a cook whose day on the request's clocks falls in the week in progress (or later) counts on that day.
  Any other cook counts on its day in its own zone. A cook made just before the week in progress whose own day
  already falls in it (1 am Monday in Tokyo is still Sunday in Amsterdam) counts once, on that own day.
- A week is met when its days with a finished cook reach the goal recorded with its latest cook; at the same
  instant the higher goal decides, so arrival order never matters. The week in progress never breaks the streak:
  it adds a week once met, and the streak ends only when a finished week was not met.
- "Now" is the request time, passed into the rules by the route.
- The rules are `progress::CookCalendar` (parsed once; an unknown zone or a first weekday outside 1 to 7 is
  refused) and `progress::StreakRules` (mrspacerobot/tomi-backend#37).

## `POST /v1/cook-entries`

The phone sends the cooks in its outbox, possibly late and possibly twice. The phone makes a UUID for each cook when
cooking begins; that id is the idempotency key, so a retried request is always safe.

```json
{
  "calendar": { "timeZone": "Europe/Amsterdam", "firstWeekday": 2 },
  "entry": {
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
}
```

- `dish` contains `recipeId` and `graphVersion` for the graph recipe cooked.
- `outcome` is `finished` or `stoppedEarly`.
- `weeklyGoal` is the goal when the cook was made, so raising the goal later never judges a past week again.
- **No `quizCorrect`, no XP and no totals.** Unknown fields, in the entry or its `dish`, are refused per entry as
  `unknown_field`. The quiz result is derived here from the answers recorded under the entry's id (see "The quiz").
- Each entry is stored in `tomi.cook_entries` (primary key: the cook and the entry id) with its own time zone, by its
  canonical IANA name. Nothing else is stored with it: no total, no XP, no quiz result.
- Two copies are the same body when they say the same thing: the instant is compared in UTC and the zone by its
  canonical name, so `2026-10-26T19:42:00+01:00` and `2026-10-26T18:42:00Z` are one cook.

The answer is `200` whenever the request could be read, with the result for the entry and the snapshot after it
(added by #40; [`docs/examples/cook-entries.json`](examples/cook-entries.json)
is the request and answer as they are today, checked by a test):

```json
{
  "result": { "id": "4e2c2a0e-7e6b-4c7a-9c2b-6e0a3d1f9a11", "status": "accepted" },
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

Problem codes of a rejected entry, listed in this order when an entry has several:

| Code | When |
|---|---|
| `unsupported_schema_version` | `schemaVersion` is missing or not one this server reads. Nothing else of the entry is read |
| `unknown_field` | The entry or its `dish` has a field schema 1 does not have, such as `quizCorrect`, `xp` or a total |
| `unreadable_entry` | A field is missing or of the wrong type, `cookedAt` is not an RFC 3339 instant, `outcome` is another word, or `dish` lacks a recipe id or graph version |
| `invalid_skill_id` | The skill is not a well-formed id |
| `level_out_of_range` | The level is not a valid level number |
| `weekly_goal_out_of_range` | The goal is outside the game's weekly goal range |
| `cooked_in_future` | `cookedAt` is later than the receipt by more than the game's tolerance |
| `invalid_time_zone` | The entry's `timeZone` is not a known IANA zone |
| `recipe_does_not_teach_level` | The entry's `graphVersion` is the pinned graph, that graph knows the recipe, and its levels do not include this skill and level. A cook of another graph version is not judged by this one |

An entry without a readable `id` is answered with `"id": null` and `unreadable_entry`. The conflict log line holds the entry id and `"code": "conflict"`, never what the
entry says.

A skill or level the catalog does not know is **accepted**: it is ignored for mastery, still earns XP and counts for
the streak, so a newer catalog can recognise it later.

Request errors: `401` and `403` as for every route (a deleted account is `403 account_deleted`); `422 invalid_request`
for an unreadable body, an unknown top-level field, a missing entry or a bad `calendar` (a missing or unknown
`timeZone`, or a `firstWeekday` outside 1 to 7). Nothing of a refused request is stored. The phone keeps the entry on a request error and tries again later; it never drops it.

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

Read from `progress::game`, never typed twice. `PATCH /v1/profile` checks `weeklyGoal` against the same values.
