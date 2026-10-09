# Rules for anyone writing here

- **Only the owner merges into `main`.** Work on a branch, push it, open a
  pull request and stop.
- **Tests first.** Every rule the API enforces has a test that breaks it.
- **`./scripts/test.sh` must pass** before anything is called done. Report
  honestly if it fails or a step was skipped.
- **No secrets in the repository, ever.** Settings come from the
  environment; `.env` is ignored by git.
- **Never log a request body,** a photo, voice, chat, an ingredient or a
  restriction. Logs hold status, latency, versions and counts.
- **Fail closed.** A missing fact, an unknown id or an incompatible version
  is a refusal, never an empty restriction or a guess.
- **XP is never accepted from the app.** The server computes it from cook
  events.
- Code is small and plain: one function, one job, names that say what,
  comments that say why.
