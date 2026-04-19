+++
title = "Troubleshooting"
description = "Diagnose and fix common anodizer issues"
weight = 10
template = "docs.html"
+++

# Troubleshooting

When something goes wrong during a release, anodizer provides several flags to help you understand what happened and why.

## Verbosity Flags

### `--quiet` / `-q`

Suppress all non-error output. Useful in CI pipelines where you only want to see failures:

```bash
anodizer release --quiet
```

Only error messages will be printed to stderr. Stdout remains clean for machine-parseable output.

### `--verbose`

Show detailed output from all stages, including:

- External command stdout/stderr (not just on failure)
- Template variable context before rendering
- Environment variables passed to build commands
- File paths being created, copied, or archived

```bash
anodizer release --verbose
```

### `--debug`

Maximum detail. Includes everything from `--verbose`, plus:

- Full HTTP request/response details for GitHub API calls
- The resolved configuration after includes and overrides are merged
- Artifact registry contents at each pipeline stage boundary

```bash
anodizer release --debug
```

## Common Issues

### Build failures

When an external command (cargo, cross, zigbuild) fails, anodizer captures and displays the full stderr output. Look for compiler errors in the output.

If you only see an exit code, run with `--verbose` to see the full command output:

```bash
anodizer release --verbose 2>&1 | less
```

### Missing GitHub token

If you see an error about a missing `GITHUB_TOKEN`, either:

1. Set the environment variable: `export GITHUB_TOKEN=ghp_...`
2. Pass it via CLI: `anodizer release --token ghp_...`

### API errors

When a GitHub API call fails, anodizer displays the HTTP status code and response body. Common causes:

- **401 Unauthorized**: Token is invalid or expired
- **403 Forbidden**: Token lacks required permissions (needs `repo` scope)
- **404 Not Found**: Repository doesn't exist or token can't access it
- **422 Unprocessable Entity**: Tag already exists, or validation failed

Run with `--debug` to see the full HTTP request and response headers.

### Config validation errors

Run `anodizer check` to validate your configuration without running a release:

```bash
anodizer check --verbose
```

This catches issues like:
- Unknown fields (typos in config keys)
- Invalid values (wrong type, unsupported format)
- Circular `depends_on` references
- Missing required fields

### Timeout issues

The default pipeline timeout is 60 minutes. If your release consistently times out:

```bash
anodizer release --timeout 1h
```

### Dry-run mode

Always test with `--dry-run` before a real release:

```bash
anodizer release --dry-run --verbose
```

This runs the full pipeline without creating releases, pushing tags, or publishing packages. Combined with `--verbose`, you can see exactly what would happen.

## Getting Help

- [GitHub Issues](https://github.com/tj-smith47/anodizer/issues) -- report bugs or request features
- `anodizer --help` -- full CLI reference
- `anodizer check` -- validate your configuration
