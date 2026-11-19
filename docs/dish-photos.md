# Dish photo publishing contract

The cooking graph supplies dish recipes. The review service owns photo creation,
approval and publishing. A graph release and its backend deployment can go out
while photos are being prepared.

The authoring service compares the pinned graph's dish recipes with
`tomi.dish_photos`. Hash the compact source recipe JSON using SHA-256, with
object keys in lexicographic order and ingredient and step arrays in their
original order. Key a candidate by `recipe_id`, `recipe_fingerprint` and
`object_name`. Use the lowercase SHA-256 of the WebP bytes plus `.webp` as its
object name. Store its depicted graph food IDs in `shows`, its provenance in
`model` and `prompt`, and initially leave its status at `generated`.

After review, record `reviewed_by` and `reviewed_at`. Upload approved bytes to
the public Supabase `dish-photos` bucket without overwriting. Fetch the public
object and check its hash before setting `published_at`. Keep old published
records for pinned graph rollbacks. Replacing a photo for the same fingerprint
updates publication of the old and new rows together. The review service uses
a server-side DB credential. The app receives no write credential or table
access.

A suggestion may contain `photoUrl` when the database record is approved and
published, its recipe fingerprint matches the backend's graph, and all the
foods it depicts appear in the cook's adapted recipe. The iPhone downloads
directly from that URL. A missing, stale or unusable photo leaves the recipe
card available.
