# Changelog — anodize-stage-announce

## [0.2.0] - 2026-04-19

### Features

* d11f5b587759dde85a6990d7ee29fa2612c4695a add Bluesky provider
* f37eb1036083b5647c2f4edda95b445e8ec4a982 add Discourse provider
* 672d56f1546586ff8802e2a130ec6e3704cfe0b9 add LinkedIn provider
* c6c65f6c59c07013b7b4c76999c4083047dae180 add Mastodon provider
* 2c2fada700085279afaebdaa4bf1aef2a6f59429 add OpenCollective provider
* 0f5e5643126ba019ed8459e477ffff39c3e8a2da add Reddit provider
* 773f455e8983766a0f69e6e2d2da03915e59cc68 add Twitter/X provider with OAuth 1.0a
* 98435ab2c929febe5a299cd38a9af7f1cf79e59d add expected_status_codes to webhook provider
* 016a285a0dc25fd217b1dfb276f50af3af0daa78 add icon_url to Teams provider
* 828b074ab04a18d454ef45262cccfb0c43e08f32 add template-conditional announce.skip field
* b5e075ba2e9a63106b60d5476218815930489339 add title_template to Mattermost provider
* 49edfe115c46f8da69ff8805a0ee9de205bb2ad1 replace sendmail with SMTP transport via lettre
* 0d64c62c4ad80311845f5a40769e1c490378b92b type Slack blocks/attachments for better schema validation
* 4c6d86c7f8d7cb40b97e7983699e3bfa6c2cfe96 Session L — config defaults, ANODIZE_FORCE_TOKEN, announce provider parity
* 62bc47c638445c61fbbcb77a436755be476e540f Stage trait and Context, wire up stage crate stubs
* 01955fd520e6844586f24c5b6a9c5ab9e0ffa957 Discord, Slack, and webhook announcements
* a691cb2c0f01f2e0961f9cbdc5ac9128b0f6ed1d add 8 Pro template variables for GoReleaser parity
* 1b9bee413d89bdb8167e4b8e317ae70adb2d220d Session 5.5 observability + deep audit fixes across entire codebase
* 61f843b35b180577d374d50e6a29e21bffcff3e9 add Telegram, Teams, Mattermost, and email announce providers
* 98765725221efcaa3cc9d57bc5f03cee5eeae1e3 anodize bump --commit bundles changelog + --strict version-pin gate #none
* cc098a9e876be05f586d7ff8ec8b85178f39aa34 complete Phase 1 parity tasks 1-9 (1,736 tests pass)
* 18f300c58484f9007f63ba2749f062cc9bc9b693 scaffold workspace with core and stage crate stubs
* e575fa81397f48df45a77cf678977d86f9470795 v0.1.0 release preparation
* 809ddaeed7e0d4c5b5af2bd1a44edc1b802414f7 wire NSIS and AppBundle stages into workspace, CLI, and pipeline

### Bug Fixes

* 60fc89ee8183e0a916d319bc909047e7507690ab address SMTP email code review findings
* 127f1c2a886dce9fd8cc8b443a33e8fa262b8404 address all review findings for LinkedIn and OpenCollective
* 2cd4a3580122af9fcd8971400d9fcb9ade2f0a9f address all review findings for Reddit, Twitter, Mastodon providers
* 4c9be5e7aa8dbe007716cec79603d02f289c8f14 improve Discourse test quality and env var safety
* e9ddfba278dbae2f553f64255b3f00d382b8c27a template-render Slack blocks/attachments
* 94796c9325749ab69375b4bcca2f02b24c6cfe34 test all options in mattermost all-options test
* be43d3b6a8e58cca4dfb65727b23faafbc446b01 use StringOrBool for skip field and strengthen template test
* 14166a63f315629ae0ce6631ab5ee103c0235868 render template vars in URLs/headers, send raw body for generic webhook
* 8207788a9b4163501014fd12eb3126f95bcc9255 68 GoReleaser parity bugs across all stages
* aac46c4682276d735a5402249a5ed993ac83523e address all 4 findings from Session 5 final review
* f04bab0538d2d02d0498e98b8e94a0c93ef25a5a address all 6 code review findings for announce providers
* c0e62906db01a768a05f754143690b40cc8aae72 cargo fmt, clippy, and add CI auto-tag step
* 2cb51c5d2c04bc12dfe6364087867ece3ade2963 drain known-bugs (W1+W2+S1-S4 safety, S1-S6 pro, S1-S5 dedup) #none
* a7d9766fc991ca3219fdaa1939af3985e8b21ff3 parity sweep — 31 GoReleaser parity fixes across release/sign/changelog/publishers/packaging/announce #none
* 91f7d7f13df7deebe4f54ccca223129f11ff1324 strict-mode bulletproofing + targets subcommand + publisher safety #none
* 5c62e6f3ee5cfb70a09a08963b21e1699db8f351 wire up git variable population and CI improvements for Task 3E

### Others

* 5e14a0390b7e808a3d9bc6c8dd03cae1123c0650 extract render_json_template helper, add expansion test
* ecd50adb6b49550bd9c902e03726389c10a00b57 deep dedup pass + wire all dead CLI flags and config fields
* 6c2c2767aaffc756137c3b7444b2f7ac7ae1df24 replace hand-built RFC 2822 email with Tera template
* 441b3264a59007c448b1ea046f02ba57e982f2f7 unwrap/expect -> ?/context (142 -> 0 non-test lib sites) + publisher cleanup #none
* a6a2f986ccdb28c3ea3fe4d3c33ac6b5dc07858d harden unwrap paths, secret handling, path traversal, and regex injection
