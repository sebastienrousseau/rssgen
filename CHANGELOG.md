# Changelog

All notable changes to `rss-gen` are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.6](https://github.com/sebastienrousseau/rssgen/compare/v0.0.5...v0.0.6) - 2026-06-28

### Added

- *(atom)* add Atom 1.0 feed generation, validation, format detection (closes #24)

### Dependencies

- *(deps)* bump time from 0.3.49 to 0.3.51

### Documentation

- *(readme)* full review against noyalib reference style + cloudcdn.pro logo swap

### Fixed

- *(v0.0.6)* close 5 spec/correctness bugs + drop dead deps
- context-prefixed validation errors + relative item links ([#34](https://github.com/sebastienrousseau/rssgen/pull/34))

### Tooling

- *(msrv)* bump floor from 1.85.0 to 1.88.0 to match transitive deps
- *(v0.0.6)* unblock MSRV + Miri + codecov gates on PR #35

### Added

- **Atom 1.0 feed format support** (closes #24). A new `atom` module
  provides `AtomFeed`, `AtomEntry`, `AtomPerson`, `AtomLink`, and the
  `generate_atom` serializer, covering RFC 4287 required elements
  (`id`, `title`, `updated` at both feed and entry level), feed-level
  vs entry-level author resolution, media enclosures via
  `<link rel="enclosure" type="..." length="...">`, plain-text and
  HTML payloads for `<summary>` / `<content>`, multiple authors and
  contributors, categories, `xml:lang`, icon/logo/rights/subtitle, and
  RFC 3339 timestamp validation. Validation errors carry `feed.` and
  `entry.<idx>.` context prefixes, matching the contextual error work
  done for the RSS path in issue #34.
- **Format auto-detection** via `detect_feed_format(&str) -> FeedFormat`
  for dispatching between RSS (`<rss>` / `<rdf:RDF>`) and Atom inputs
  without parsing the full document.
- Re-exports added to the crate root and prelude: `generate_atom`,
  `AtomFeed`, `AtomEntry`, `AtomLink`, `AtomPerson`, `AtomTextType`,
  `FeedFormat`, `detect_feed_format`.

### Fixed

- **Validation error messages now carry context prefixes** (closes
  #34). Channel-level errors are prefixed with `channel.` and
  item-level errors with `item.`, replacing the previous bare
  `Link is missing` / `Title is missing` strings that gave downstream
  tooling no way to distinguish where the failure occurred.
- **Relative item links are now accepted** per RSS 2.0 §5.7 via a new
  `validate_link_field` helper. Absolute URLs, root-relative paths
  (`/tags/`), and bare paths (`articles/foo.html`) all validate;
  whitespace, control characters, and empty strings are still
  rejected. Channel-level `link` retains its absolute-URL strictness
  as the spec requires.
- **`RssFeedValidator::parse_date` is no longer GMT-only** (P0). The
  pre-v0.0.6 implementation hard-required a literal `" GMT"` suffix
  and rejected every spec-compliant feed produced outside of GMT
  (`+0000`, `+0530`, `EST`, …). It now delegates to
  `data::parse_date`, which accepts the full RFC 2822 timezone
  grammar plus ISO 8601 — staying aligned with the channel-level
  path used by `generate_rss`.
- **`RssFeedValidator::validate_structure` no longer rejects empty
  or relative item links** (P0). RSS 2.0 §5.7 explicitly allows
  items to ship without a `<link>` so long as they carry a `<title>`
  or `<description>`. The structural pass now skips empty item
  links and delegates populated ones to `validate_link_field`,
  matching `RssData::validate`. Pre-v0.0.6 the validator unconditionally
  fed every item link to `Url::parse`, so feeds with description-only
  items always failed.

### Changed

- **`RssError::ValidationErrors` now carries `Vec<ValidationError>`
  rather than `Vec<String>`.** Each entry exposes structured
  `field` (a dotted path: `channel.title`, `item.0.link`, `feed.id`,
  `entry.2.updated`) and `message` properties so callers (CI gates,
  IDE integrations, JSON error responses) can dispatch on `field`
  without parsing strings. `Display` writes the bare `message`, so
  `e.to_string()` keeps the pre-v0.0.6 string format and
  `errors.iter().any(|e| e.to_string() == "channel.title is missing")`
  still works. `ValidationError` is now re-exported from the crate
  root and prelude.
- Date stack consolidated on `time`: dropped the `dtt` dependency
  (was `0.0.x`, no SemVer stability) and removed `commons`
  (euxis-commons) which was declared but never used in `src/`.
  `parse_date` / `RssItem::pub_date_parsed` now return
  `time::OffsetDateTime` directly rather than a `dtt::DateTime`
  UTC sentinel — callers see the actual parsed offset.
- Dropped the unused `serde_json` runtime dependency (it was never
  imported under `src/`).
- Removed the no-op `async = []` feature flag from `Cargo.toml`;
  it gated zero `#[cfg]` code paths. A real Tokio / async-write
  integration will return behind a properly wired flag in a
  follow-up release.
- Bumped `time` from `0.3.49` to `0.3.51` (supersedes Dependabot PR
  #33). `0.3.50` added the `Timestamp` type and improved RFC 2822 /
  ISO 8601 parsing/formatting throughput; `0.3.51` is the build-fix
  for the macros-feature regression in `0.3.50`. No API changes
  affecting `rss-gen`.
- **MSRV bumped from `1.79.0` to `1.88.0`.** The effective floor
  through current crates.io is 1.88: `time 0.3.51` and
  `time-core 0.1.9` declare `rust-version = "1.88"`, the `icu_*`
  chain (transitives of `url`/`idna`) and `criterion 0.8.2`
  declare 1.86, and the lowest version that satisfies every
  transitive is 1.88.0. CI now pins a dedicated MSRV lane to
  this floor so future dep updates can't silently push the
  effective MSRV without anyone noticing.

### Performance

- Stress benchmark added (`examples/stress_huge_feed`): 50 000-item
  RSS 2.0 emit runs in ~15 ms (14 MiB output) and 50 000-entry
  Atom 1.0 emit in ~114 ms (8.7 MiB output) on an M-series Mac in
  release mode — both well under the < 1 s / generate budget.
  `detect_feed_format` classifies a 23 MiB combined payload in
  ~7 µs.

### Tooling

- Examples: `examples/example_atom.rs`,
  `examples/example_detect.rs`, and
  `examples/example_validation_errors.rs` cover Atom 1.0 emit
  (multi-author, contributor, enclosure, HTML content), root-element
  classification, and structured validation diagnostics.
- Benchmarks: `benches/criterion.rs` extended with `Generate Atom`,
  `Detect feed format`, and `Validate` groups in addition to the
  existing `Generate RSS` / `Parse RSS` coverage.

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
