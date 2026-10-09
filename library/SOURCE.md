# Where this library comes from

A copy of Tomi's reviewed library, `Tomi/Resources/Library` in the
`mrspacerobot/tomi` repository at commit `f9775dc`. Lessons are copied flat.

Never edit these files here. Change them in the Tomi repository, where a
person reviews them, and copy them again. Until the library is delivered
as a versioned release, this copy is how the server knows the lessons and
the ingredient table.

## The one file written here

`choices.json` is the server's own: the cuisines, experience answers and
units the app offers in onboarding, which the app's library does not hold.
Its wording is taken from the approved onboarding mockups in Tomi's
`docs/design/src/round2.py` and `round3.py`. Experience and unit ids must
stay the raw values the app saves (`never`, `basics`, `comfortable`;
`metric`, `cups`). Change it in a reviewed pull request, like the library.
