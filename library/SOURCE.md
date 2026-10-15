# Where this library comes from

A copy of Tomi's reviewed library, `Tomi/Resources/Library` in the
`mrspacerobot/tomi` repository at commit `f9775dc`: the catalog, the equipment
list and the lessons. Lessons are copied flat.

Never edit those files here. Change them in the Tomi repository, where a
person reviews them, and copy them again. Until the library is delivered
as a versioned release, this copy is how the server knows the lessons.

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
fact it names must exist in the graph, or the server does not start. Facts that
have no avoid item yet (celery, mustard, lupin, sulphites, buckwheat) are
tracked in mrspacerobot/tomi#123.
