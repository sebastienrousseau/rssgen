# Changelog

All notable changes to `rss-gen` are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.5] - 2026-06-21

### Added

- CI: unified `.github/workflows/ci.yml` that delegates to the
  `sebastienrousseau/pipelines` reusable workflows (`rust-ci.yml`,
  `security.yml`, `docs.yml`), replacing the previous fan-out of
  `audit`, `check`, `coverage`, `document`, `lint`, `release`, and
  `test` workflows.
- `.github/labeler.yml` for automatic PR labelling.
- Dependabot `minor-and-patch` group rule for cargo updates, keeping
  weekly noise low.
- Expanded test coverage in `data`, `error`, `generator`, `parser`,
  and `validator`: size-limit checks, builder-field coverage, item
  field setters, version-specific validation, GMT-suffix date
  parsing, and assorted edge cases.

### Changed

- Bumped MSRV from **1.68.0** to **1.79.0** to accommodate
  `quick-xml 0.40`.
- Updated `quick-xml` requirement from `0.39` to `0.40` (adds UTF-16
  and ISO-2022-JP decoding via `DecodingReader`, normalised XML 1.0
  EOL handling, attribute normalisation APIs). Supersedes
  Dependabot PR #28.
- Updated `criterion` dev-dependency to `0.8` (previously bumped to
  `0.8` in tree). Supersedes Dependabot PR #22.
- Refactored CI: GitHub Actions Dependabot PRs for
  `actions/upload-artifact` (#25), `actions/download-artifact` (#26),
  `codecov/codecov-action` (#27), and `peaceiris/actions-gh-pages`
  (#29) are obsolete; the actions they target no longer exist in the
  consolidated workflow.

### Fixed

- `validator::parse_date`: removed a redundant
  `date.offset = time::UtcOffset::UTC;` write that broke against
  `dtt 0.0.10` (the `DateTime` parsed from a stripped GMT suffix is
  already UTC; the field was no longer writable on the new type).
- `README.md` usage example now matches the current `RssData::new`
  signature (`Option<RssVersion>`) and uses `generate_rss` instead of
  the removed `.build()` / `.to_xml()` methods. This unblocks the
  doctest that runs via `#![doc = include_str!("../README.md")]`.

### Notes

- Lib test suite: **163 tests passing**; doctests: **8 passing**.
- `cargo update` bumps a wide set of transitive crates (notably
  `tokio`, `wasm-bindgen`, `web-sys`, `zerocopy`) — no behaviour
  change observed.
- Coverage failure on Dependabot PR #29 was caused by the legacy
  `coverage.yml` using deprecated `-Zprofile` plus
  `actions-rs/cargo@v1`; it disappears with the unified CI pipeline.

[0.0.5]: https://github.com/sebastienrousseau/rssgen/releases/tag/v0.0.5
