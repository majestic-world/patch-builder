## Language

- Write code and code comments in English.
- Reply to the user in Brazilian Portuguese (pt-BR).
- Match nouns to the actual count: `1 classe`, `2 classes`; avoid parenthetical plurals such as `classe(s)`.

## Commits

- Commit only when the user asks or a manually invoked skill explicitly requires it.
- Use Conventional Commits with the description in pt-BR.

## Validation

- Preserve validation quality with the least user interference. Prefer checks that do not control the desktop; use the UI when it is necessary to prove behavior.
- During desktop interaction, prefer background input, group predictable actions into short sessions, release control when verification ends, and take focus, mouse, or keyboard only for the indispensable interval.
- In omp sessions, use omp computer use for desktop interaction. Its required use is pre-authorized; proceed without asking. If it is disabled or unavailable, report that once, continue with available checks, and identify any pending UI validation.
- Outside omp, use the environment's available tools under the same least-interference policy. Proceed without requiring or warning about the absence of omp computer use.

## Agent skills

### Issue tracker

Issues are tracked as local markdown files under `.scratch/<feature>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Uses the five default triage labels (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `CONTEXT.md` plus `docs/adr/` at the repo root. See `docs/agents/domain.md`.
