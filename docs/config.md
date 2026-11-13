# Configuration

For basic configuration instructions, see [this documentation](https://developers.openai.com/codex/config-basic).

For advanced configuration instructions, see [this documentation](https://developers.openai.com/codex/config-advanced).

For a full configuration reference, see [this documentation](https://developers.openai.com/codex/config-reference).

## Project instructions

To fail when applicable project instruction files exceed the shared byte budget,
set this in `config.toml`:

```toml
project_doc_overflow = "error"
```

Codex checks the selected environments before using the instructions in a model
request. An exact fit is allowed. The budget is set by `project_doc_max_bytes`
and defaults to 32768 bytes. Error mode requires a positive budget. The default
overflow policy, `"truncate"`, warns and continues with the instructions that fit.
Global and thread instructions have separate size handling.
When config outside the project selects `"error"`, it also controls the project
instruction byte limit and fallback filenames. Repository config can choose
those settings when strict mode is left to the project.

## Lifecycle hooks

Admins can set top-level `allow_managed_hooks_only = true` in
`requirements.toml` to ignore user, project, and session hook configs while
still allowing managed hooks from requirements and managed config layers. This
setting is only supported in `requirements.toml`; putting it in `config.toml`
does not enable managed-hooks-only mode.
