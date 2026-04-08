+++
title = "Before/After Hooks"
description = "Run commands before or after the release pipeline"
weight = 89
template = "docs.html"
+++

Anodize supports running arbitrary shell commands before the pipeline starts and after it completes. These are the same hooks documented in [Global Hooks](/docs/general/hooks/), but this page focuses on their use around the publish phase.

## Config

```yaml
before:
  hooks:
    - "echo 'Starting release'"
    - "cargo test --release"

after:
  hooks:
    - "echo 'Release complete'"
    - "./scripts/deploy.sh"
```

## Structured hooks

For more control, use the structured hook format:

```yaml
before:
  hooks:
    - cmd: "cargo test --release"
      dir: "{{ .Env.PROJECT_ROOT }}"
      env:
        RUST_LOG: info
      output: true
```

### Structured hook fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `cmd` | string | **required** | Command to execute |
| `dir` | string | project root | Working directory (template) |
| `env` | map | none | Additional environment variables |
| `output` | bool | none | Capture and log stdout/stderr |

## Behavior

- Hook commands are rendered through the template engine before execution
- The process environment is inherited; pipeline environment variables (`VERSION`, `TAG`, etc.) are available
- Secrets are automatically redacted from stdout/stderr
- `hooks` is accepted as an alias for `pre` (GoReleaser compatibility)
- Before hooks run sequentially; a failing hook aborts the pipeline
- After hooks run after all stages complete successfully

## Use cases

- **Pre-flight checks**: `cargo fmt --check`, `cargo clippy`, `cargo test`
- **Post-release notifications**: Slack webhooks, deployment triggers
- **Artifact post-processing**: signing, uploading to additional locations
- **Environment setup**: setting up credentials or config before publish
