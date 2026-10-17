# Where this library comes from

A copy of Tomi's reviewed library, `Tomi/Resources/Library` in the
`mrspacerobot/tomi` repository at commit `f9775dc`: the catalog and equipment
list. The copied recipe lessons have been removed because they duplicated graph
recipe content and incorrectly gated which graph recipes could be offered.

Never edit those files here. Change them in the Tomi repository, where a
person reviews them, and copy them again. Until the library is delivered
as a versioned release, this copy supplies progression and kitchen option
metadata only. Recipe identity, content, and food facts come from the pinned graph.

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

`avoid-items.released.json` lists every avoid item the library ends up with, by id,
with the facts it rules out. A test fails when the real list differs from it, and
when a released item is gone or rules out less, so a graph update that adds or changes
an avoid item shows up in a reviewed diff here. Update it in the same pull request,
and never remove or narrow a released id: saved preferences keep it.
