# Where this library comes from

`catalog.json` and `equipment.json` were first copied from Tomi's reviewed
library, `Tomi/Resources/Library` in the `mrspacerobot/tomi` repository at
commit `f9775dc`. The copied recipe lessons have been removed because they
duplicated graph recipe content and incorrectly gated which graph recipes
could be offered. These files supply progression and kitchen option metadata
only. Recipe identity, content, and food facts come from the pinned graph.

**`catalog.json` here is the canonical skill catalog** (the owner's decision
of 26 October 2026, when the cook log and every progress rule moved to this
server, `docs/progress.md`). The unlock and mastery rules read it here, and
the app reads the skill menu from `GET /v1/options`. Change it here, in a
reviewed pull request. **Only a person adds a skill or a level**; a generated
or model-written change that adds one is refused, whatever the checks say.
Skill and course ids and level numbers are permanent: cook entries keep them
forever, so they are never renamed, removed, reused or renumbered. The app's
own copy is retired by mrspacerobot/tomi#156.

`equipment.json` is still a copy of the app's list. Never edit it here: change
it in the Tomi repository, where a person reviews it, and copy it again.

The ingredient table of that commit is not copied. Every food fact now comes
from the cooking graph, and the diets and avoid items are defined over its facts
in `restrictions.json` (below).

## The files written here

`choices.json` is the server's own: the cuisines, experience answers and
units the app offers in onboarding, which the app's library does not hold.
Its wording is taken from the approved onboarding mockups in Tomi's
`docs/design/src/round2.py` and `round3.py`. Experience and unit ids must
stay the raw values the app saves (`never`, `basics`, `comfortable`;
`metric`, `cups`). Change it in a reviewed pull request, like the library.

`restrictions.json` lists the diets and avoid items the cook can choose, each with
the graph facts it rules out (`avoids`). The ids and names are the ones the app's
table had, and ids are permanent: saved preferences keep them. **Only a person
edits it**, because it decides which foods a cook with an allergy is shown. Every
fact it names must exist in the graph, or the server does not start. With
`"allergensFromGraph": true`, every allergen fact of the graph that no item here rules
out gets an avoid item of its own at start (its id is the fact's slug, its name the
graph's), so an allergen the graph knows can never be missing from the list. That is
how celery, mustard, lupin, sulphites and buckwheat appear (mrspacerobot/tomi#123).

`staples.json` is the cupboard question's list: the everyday ingredients a cook can say
they always have (about twenty), each with `assumed`, whether a kitchen is taken to have
it until the cook answers. Only a person edits it. Every ingredient must be one the graph
knows, or the server does not start. Anything else a cook has, they type. Only salt, pepper
and the two oils are assumed for now; the owner has not yet confirmed that.

`catalog.released.json` lists every released skill id with its level numbers.
A test fails when the catalog differs from it, and when a released skill or level
is gone or renumbered, because cook entries keep them forever. Add a new skill or
level to it in the same pull request, by a person, and never remove or renumber
one.

`avoid-items.released.json` lists every avoid item the library ends up with, by id,
with the facts it rules out. A test fails when the real list differs from it, and
when a released item is gone or rules out less, so a graph update that adds or changes
an avoid item shows up in a reviewed diff here. Update it in the same pull request,
and never remove or narrow a released id: saved preferences keep it.
