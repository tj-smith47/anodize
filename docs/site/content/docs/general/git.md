+++
title = "Git"
description = "Configure tag sorting, filtering, and version detection from git"
weight = 7
template = "docs.html"
+++

Anodize detects the current version from git tags. The `git` section lets you control how tags are sorted and which tags are considered.

## Minimal config

```yaml
git:
  tag_sort: "-version:refname"
```

## Git config fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `tag_sort` | string | `-version:refname` | How to sort tags: `"-version:refname"` (semver-aware sort) or `"-version:creatordate"` (by creation date) |
| `ignore_tags` | list | none | Glob patterns for tags to exclude from version detection (supports templates) |
| `ignore_tag_prefixes` | list | none | Prefixes for tags to exclude from version detection (supports templates) |
| `prerelease_suffix` | string | none | Suffix identifying pre-release tags for sort ordering |

## Tag sorting

The default `"-version:refname"` sorts tags using Rust-side semver-aware comparison. Use `"-version:creatordate"` to sort by the tag's creation date instead (newest first):

```yaml
git:
  tag_sort: "-version:creatordate"
```

## Ignoring tags

Filter out tags that should not be considered for version detection:

```yaml
git:
  ignore_tags:
    - "nightly*"
    - "legacy-*"
    - "{{ .Env.IGNORE_PATTERN }}"
  ignore_tag_prefixes:
    - "internal/"
    - "test-"
```

Both `ignore_tags` and `ignore_tag_prefixes` support template rendering, so you can use environment variables or other template expressions.

## Pre-release suffix

When set, tags ending with this suffix are treated as pre-releases and sorted accordingly:

```yaml
git:
  prerelease_suffix: "-rc"
```

Setting `prerelease_suffix` also forces git-delegated sorting (via `git -c versionsort.suffix=<suffix>`) rather than Rust-side semver sorting.

## Detected git info

Anodize detects the following information from git, all available as template variables:

| Variable | Description |
|----------|-------------|
| `{{ .Tag }}` | Current git tag |
| `{{ .Commit }}` | Full commit SHA |
| `{{ .ShortCommit }}` | Short commit SHA |
| `{{ .Branch }}` | Current branch name |
| `{{ .CommitDate }}` | Commit date (ISO 8601) |
| `{{ .CommitTimestamp }}` | Commit timestamp (Unix) |
| `{{ .PreviousTag }}` | Previous git tag |
| `{{ .Summary }}` | `git describe` output |
| `{{ .TagSubject }}` | Tag annotation subject |
| `{{ .TagBody }}` | Tag annotation body |
| `{{ .TagContents }}` | Full tag annotation |
| `{{ .IsSnapshot }}` | Whether this is a snapshot build |
| `{{ .IsGitDirty }}` | Whether the working tree has uncommitted changes |
| `{{ .IsGitClean }}` | Inverse of `IsGitDirty` |
| `{{ .GitTreeState }}` | `"clean"` or `"dirty"` |
| `{{ .GitURL }}` | Git remote URL (credentials stripped) |
| `{{ .Version }}` | Semver version (tag without `v` prefix) |
| `{{ .RawVersion }}` | Raw version string before normalization |
| `{{ .Major }}` | Semver major version number |
| `{{ .Minor }}` | Semver minor version number |
| `{{ .Patch }}` | Semver patch version number |
| `{{ .Prerelease }}` | Semver pre-release suffix (e.g., `rc1`) |

## Full example

```yaml
git:
  tag_sort: "-version:creatordate"
  ignore_tags:
    - "nightly*"
    - "legacy-*"
  ignore_tag_prefixes:
    - "internal/"
    - "test-"
  prerelease_suffix: "-rc"
```
