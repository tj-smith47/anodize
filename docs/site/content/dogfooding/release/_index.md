+++
title = "Release pipeline"
description = "Release-pipeline config keys: release.*, changelog.*, announce.*, blobs[], publishers[]."
weight = 30
template = "section.html"
+++

# Release pipeline

The keys that drive the release itself: GitHub/GitLab/Gitea release surface,
changelog generation, announcers, cloud uploads, and custom publishers.

## Release and changelog

| Key | Status | Notes |
|---|---|---|
| `release.github` | ✅ Verified | [anodizer releases](https://github.com/tj-smith47/anodizer/releases). Header/footer/draft/prerelease/make_latest all exercised |
| `release.metadata` | ✅ Verified | [v0.1.1 metadata.json](https://github.com/tj-smith47/anodizer/releases/download/v0.1.1/metadata.json) · [artifacts.json](https://github.com/tj-smith47/anodizer/releases/download/v0.1.1/artifacts.json) |
| `release.name_template` / `tag_template` | ✅ Verified | [cfgd `.anodizer.yaml`](https://github.com/tj-smith47/cfgd/blob/master/.anodizer.yaml) (`tag_template: "core-v{{ Version }}"` / `"v{{ Version }}"` / `"operator-v{{ Version }}"` / `"csi-v{{ Version }}"`) |
| `release.header` / `footer` | ✅ Verified | [cfgd v0.3.5 release body](https://github.com/tj-smith47/cfgd/releases/tag/v0.3.5) (`What's new` header + `Released with anodizer` footer) |
| `changelog.groups` | ✅ Verified | "Features" / "Bug Fixes" / "Others" sections in the [v0.1.1 release body](https://github.com/tj-smith47/anodizer/releases/tag/v0.1.1) |
| `changelog.filters.include` / `exclude` | ✅ Verified | [anodizer `.anodizer.yaml`](https://github.com/tj-smith47/anodizer/blob/master/.anodizer.yaml) (`changelog.filters.include` / `exclude` patterns) |
| `changelog.use: git` | ✅ Verified | [`crates/stage-changelog/src/lib.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-changelog/src/lib.rs) (`use: git` branch) |
| `changelog.use: github-native` | ✅ Verified | [`crates/stage-changelog/src/lib.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-changelog/src/lib.rs) (`use: github-native` branch) |
| `changelog.use: github` | ✅ Verified | [`crates/stage-changelog/src/lib.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-changelog/src/lib.rs) (`use: github` branch) |
| `changelog.use: gitlab` / `gitea` | ✅ Verified | [`crates/stage-changelog/src/lib.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-changelog/src/lib.rs) (`gitlab` / `gitea` branches) |
| `changelog.use: ai` | 🤝 Help wanted | anthropic / openai / ollama implemented; no live release uses it |
| `release.gitlab` | 🤝 Help wanted | We dogfood on GitHub only |
| `release.gitea` | 🤝 Help wanted | We dogfood on GitHub only |
| `milestones[]` | ✅ Verified | [`crates/core/src/config/milestone.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/core/src/config/milestone.rs) |

## Announcers

13 channels implemented. Two are exercised by live cfgd releases; the
others have full test coverage but no live secrets configured.

| Key | Status | Notes |
|---|---|---|
| `announce.webhook` | ✅ Verified | [cfgd `.anodizer.yaml`](https://github.com/tj-smith47/cfgd/blob/master/.anodizer.yaml) (`announce.webhook.endpoint_url: https://tj.jarvispro.io/webhooks/anodizer`) |
| `announce.smtp` | ✅ Verified | [cfgd `.anodizer.yaml`](https://github.com/tj-smith47/cfgd/blob/master/.anodizer.yaml) (`announce.smtp.host: smtp.gmail.com`) |
| `announce.discord` | 🤝 Help wanted | No live workflow has the secrets |
| `announce.slack` | 🤝 Help wanted | No live workflow has the secrets |
| `announce.telegram` | 🤝 Help wanted | No live workflow has the secrets |
| `announce.teams` | 🤝 Help wanted | No live workflow has the secrets |
| `announce.mattermost` | 🤝 Help wanted | No live workflow has the secrets |
| `announce.reddit` | 🤝 Help wanted | No live workflow has the secrets |
| `announce.twitter` | 🤝 Help wanted | No live workflow has the secrets |
| `announce.mastodon` | 🤝 Help wanted | No live workflow has the secrets |
| `announce.bluesky` | 🤝 Help wanted | No live workflow has the secrets |
| `announce.linkedin` | 🤝 Help wanted | No live workflow has the secrets |
| `announce.opencollective` | 🤝 Help wanted | No live workflow has the secrets |
| `announce.discourse` | 🤝 Help wanted | No live workflow has the secrets |

## Blob and artifactory uploads

| Key | Status | Notes |
|---|---|---|
| `blobs[]` (S3 / GCS / Azure) | 🤝 Help wanted | `object_store` SDK wired. No release configures cloud credentials |
| `artifactories[]` | 🤝 Help wanted | Target, mode, TLS, headers wired; no live deployment |
| `uploads[]` | 🤝 Help wanted | Generic HTTP upload wired; no live deployment |
| `furies[]` | 🤝 Help wanted | Implemented; no live credentials |
| `cloudsmiths[]` | 🤝 Help wanted | Wired in [cfgd's config](https://github.com/tj-smith47/cfgd/blob/master/.anodizer.yaml) with a live `CLOUDSMITH_TOKEN`; uploads currently fail at HTTP layer so no package has landed in the `jarvispro/cfgd` repo. Awaiting endpoint debug |

## Custom publishers

| Key | Status | Notes |
|---|---|---|
| `publishers[]` | ✅ Verified | [`crates/cli/src/commands/publisher.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/cli/src/commands/publisher.rs) (custom command per artifact) |

## MCP registry

Final GoReleaser parity item. Publishes an MCP server manifest to
`https://registry.modelcontextprotocol.io`.

| Key | Status | Notes |
|---|---|---|
| `mcp.name` | 🤝 Help wanted | Wired in [anodizer `.anodizer.yaml`](https://github.com/tj-smith47/anodizer/blob/master/.anodizer.yaml) (`mcp.name: io.github.tj-smith47/anodizer`); awaiting first live publish |
| `mcp.packages[]` | 🤝 Help wanted | Wired in [anodizer `.anodizer.yaml`](https://github.com/tj-smith47/anodizer/blob/master/.anodizer.yaml) (`packages[].registry_type: oci`); awaiting first live publish |
| `mcp.auth.type: none` | ✅ Verified | [`crates/stage-publish/src/mcp/auth.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-publish/src/mcp/auth.rs) (None branch) |
| `mcp.auth.type: github` | ✅ Verified | [`crates/stage-publish/src/mcp/auth.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-publish/src/mcp/auth.rs) (PAT exchange branch) |
| `mcp.auth.type: github-oidc` | 🤝 Help wanted | [`crates/stage-publish/src/mcp/auth.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-publish/src/mcp/auth.rs) (OIDC id-token branch); to be dogfooded via anodizer's release workflow |
| `mcp.repository` | ✅ Verified | [`crates/stage-publish/src/mcp/manifest.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-publish/src/mcp/manifest.rs) |
| `mcp.skip` (tera, accepts `disable:` alias) | ✅ Verified | [`crates/stage-publish/src/mcp/mod.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-publish/src/mcp/mod.rs) |
