## Language

- Code and code comments: English.
- Replies to the user: Brazilian Portuguese (pt-BR).

## Commits

Commit only when the user asks. Then:

- Follow Conventional Commits: `type(scope): description`. `type` and `scope` stay in English (spec keywords); the description and body are in pt-BR.
- The user is the sole author: end the message without `Co-authored-by` trailers.

## Tests

Unit tests live in `tests/unit/`, mirroring the `src/` path of the module under test (`src/foo/bar.rs` → `tests/unit/foo/bar.rs`). Attach each file from its module so the tests keep access to private items:

```rust
#[cfg(test)]
#[path = "../../tests/unit/foo/bar.rs"] // relative to the module file's directory
mod tests;
```

Top-level `tests/*.rs` files are Cargo integration tests (public API only).

## Agent skills

### Issue tracker

Issues are tracked as local markdown files under `.scratch/<feature>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Uses the five default triage labels (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `CONTEXT.md` plus `docs/adr/` at the repo root. See `docs/agents/domain.md`.
