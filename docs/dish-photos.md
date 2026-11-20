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
3. Run `./scripts/photos review` and open the page it prints. Use `--bundle`
   to hand the reviewer a ZIP containing the page and its photos. They can
   extract it and open `review.html` on their own computer. The reviewer checks
   each photo against its food list, including preparations and sauces, and
   rejects plates containing extra or uncertain foods. They enter their name,
   choose a decision on each plate, then download the decisions file. Apply
   it with `./scripts/photos review --apply <decisions.json>`. Import refuses
   the whole file if a decision refers to a photo, food list, or recipe that
   has changed. An individual decision can also be entered with
   `--approve recipe/<id> --reviewer <name>` or `--reject recipe/<id>`.
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
