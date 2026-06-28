# ADR-0003: Atom 1.0 as a sibling module, not an RSS subtype

**Status:** Accepted (v0.0.6)
**Date:** 2026-06-28

## Context

Issue #24 (P1) asked for Atom 1.0 support alongside RSS. Two
structural options were on the table:

1. **Unified data model.** Generalise `RssData` into a `Feed` struct
   that can serialise to either RSS or Atom, with version-aware
   field elision.
2. **Sibling module.** Ship a new `atom` module with independent
   `AtomFeed` / `AtomEntry` types and a separate `generate_atom`
   serialiser, leaving the RSS surface untouched.

## Decision

Ship the **sibling module**. `AtomFeed` / `AtomEntry` live in
`src/atom.rs` and mirror the `RssData` / `RssItem` ergonomic style
(chained `.title()`, `.link()`, …) without sharing concrete types.
`generate_atom(&AtomFeed)` is a top-level function exported from the
crate root and the `prelude`. A separate
`detect_feed_format(&str) -> FeedFormat` helper handles dispatching
between the two paths on input.

## Alternatives considered

The unified-model approach was rejected because RSS and Atom diverge
in ways the data model would have to flatten:

- **Author shape.** RSS uses a single `<author>` (often an email
  address); Atom requires a structured `<author>` with `name`,
  optional `email`, optional `uri`, and supports multiple authors at
  feed and entry level.
- **Date formats.** RSS uses RFC 822 / RFC 2822; Atom uses
  RFC 3339. Unifying these would force a parse-and-reformat round
  trip on every serialise.
- **Content modelling.** Atom distinguishes `<summary>` from
  `<content>` and carries a `type` attribute (`text` / `html` /
  `xhtml`). RSS has only `<description>`.
- **Identifier semantics.** Atom requires a permanent `<id>` on the
  feed (RFC 4287 §4.2.6) — RSS has no equivalent at the channel
  level.

A unified model would carry every concept from both formats and
expose them as `Option<…>`, leaving consumers to figure out which
fields apply to which output. That trades a one-time learning cost
(two builder APIs) for a permanent ergonomic cost (every field is
optional, every serialise risks dropping data silently).

## Consequences

- **API surface doubled** at the builder level — but each builder
  is internally simpler than a unified one would be, and the
  prelude re-exports both so most callers see a single import.
- **Parser is RSS-only for now.** Atom parsing was deferred from
  the v0.0.6 scope. `detect_feed_format` lets callers reject
  unsupported inputs explicitly until the Atom parser ships.
- **Validation messages are consistent across both paths** — the
  contextual `feed.id` / `entry.<idx>.author` prefixes used by
  `AtomFeed::validate` mirror the `channel.title` / `item.<idx>.link`
  prefixes used by `RssData::validate` (ADR-0001).

## References

- Issue [#24](https://github.com/sebastienrousseau/rssgen/issues/24)
- [RFC 4287 — The Atom Syndication Format](https://www.rfc-editor.org/rfc/rfc4287)
- [ADR-0001](0001-structured-validation-errors.md) (shared error grammar)
