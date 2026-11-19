# Dish photos

`dish-photos/manifest.json` records one photo per graph recipe that makes a dish.
Each record names the model, prompt, review status, visible foods and recipe
fingerprint. The fingerprint hashes the compact source recipe JSON, including
the ordered ingredient lines and steps. Recipe changes make a photo stale.
Keep images in the ignored `dish-photos/local/` folder and the public
Supabase `dish-photos` bucket.

1. Fetch the current pin with `./scripts/fetch-graph.sh`, then run
   `./scripts/photos plan`. It lists missing, stale and orphaned recipe IDs.
   Remove orphaned records in the graph update PR. Leave rejected records in
   the manifest to preserve the review decision. To try another plate for an
   unchanged rejected recipe, remove that record before generating again.
2. Run `./scripts/photos generate --recipe recipe/<id>` with
   `OPENAI_API_KEY` available. It fills one fixed prompt with the graph recipe,
   writes the WebP locally and records it as generated. The default is one
   image per command. Set `--limit` explicitly for a batch. To use a WebP from
   another generator, supply `--from-webp <path> --source-model <name>` and
   `--prompt-file <path>` with the recipe ID.
3. Run `./scripts/photos review` and open the local page it prints. Confirm
   the plate matches the graph recipe and the `shows` list accounts for every
   food depicted. Generated records start with all source food lines,
   including preparations and foods in sauces. Keep any food whose presence is uncertain.
   Reject a plate with extra or ambiguous food. Record the owner decision
   with `./scripts/photos review --approve recipe/<id> --reviewer <name>` or
   `./scripts/photos review --reject recipe/<id>`.
4. Run `./scripts/photos upload --project-url https://<project-ref>.supabase.co`
   using the Keychain entry in [Supabase setup](supabase-setup.md#dish-photos).
   Upload before deploying the approved record. Commit the manifest only.

When a photo matches the pinned recipe and its `shows` list is a subset of
the cook's checked adapted ingredient lines, a suggestion can carry a public
HTTPS `photoUrl`. The field is absent otherwise.

The optional batch generator uses the [OpenAI Images API](https://developers.openai.com/api/reference/resources/images/methods/generate)
with a dated model snapshot recorded per image. Section 4.1 of the
[OpenAI Services Agreement](https://openai.com/policies/services-agreement/)
assigns output rights as between the API customer and OpenAI, to the
extent permitted by law.
