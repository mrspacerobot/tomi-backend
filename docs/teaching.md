# Graph teaching contract

`POST /v1/lessons/suggest` adds an optional `teaching` field to each suggestion. Existing recipe fields and browsing remain available when teaching is null. Older responses without this field mean teaching is unavailable.

Teaching schema 2 contains:

- `schemaVersion`: integer 2.
- `graphVersion`, `graphChannel`: the exact pinned graph identity; channel is `draft` or `production`.
- `language`: `en`; graph teaching prose currently exists only in English.
- `technique`: `{id, name, tier}`. The ID is `technique/<slug>` for the actual technique used by the recipe and matched to its offered skill level. A slug has at most 64 ASCII lowercase letters/digits joined by single hyphens, with no empty segments. Selection sorts matching technique IDs, independent of source order. The first matching technique must have a valid introduction; no unrelated fallback is guessed.
- `introduction`: `{concept, what, why, tier}`. Text is copied unchanged from graph `intro.idea`, `intro.what`, and `intro.why`.
- `quiz`: null, or an array of `{id, question, choices, tier}`. IDs are scoped as `<technique-id>/quiz/<graph-question-id>`, with the same slug rules for each local question ID. Answers and explanations stay in the pinned graph for server-side scoring. Missing, duplicate, malformed, or policy-incompatible questions make the whole quiz unavailable. On a draft graph a valid introduction is still offered without it, so content can be written step by step. On production the technique is not taught at all without a complete, reviewed quiz: a gap in the content is refused, never shown as a normal lesson, and recipe browsing is unaffected.

Tiers preserve graph values: `drafted`, `sourced`, `reviewed`, `verified`, `cook-tested`. Unknown tiers refuse content. Production teaching requires the recipe, selected technique, introduction, and each offered question to be at least reviewed. A draft graph has already passed the existing explicit `GRAPH_ALLOW_DRAFT` startup policy; its teaching must be labelled as draft/testing content on the phone and must not claim review.

For this first delivery, any applied substitution or omission makes teaching unavailable. The graph does not yet provide reviewed instruction/quiz compatibility for adaptations. In particular, a safe ingredient swap is insufficient evidence that a verb such as “melt” remains applicable. No food compatibility rule or rewritten prose is invented here.

The frontend decodes the optional teaching payload separately from the checked recipe. Unknown schemas, malformed text, unknown tiers, invalid question IDs, and incompatible channel/tier combinations leave teaching unavailable. They must not turn recipe browsing into an error or weaken the existing restriction guard.

Introduction and preparation may be browsed, but live guided steps wait for the graph-native session contract. Starting preparation requires a successful profile refresh and an exact match between the suggestion's checked diet/avoid set and the latest server profile. A change in either direction requires new suggestions; browsing keeps its existing restriction-coverage guard. This contract adds no lesson identity, saved session field, total time, illustration, serving scaling, progress claim, or XP award.
