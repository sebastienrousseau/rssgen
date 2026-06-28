# ADR-0001: Structured `ValidationError` payload

**Status:** Accepted (v0.0.6)
**Date:** 2026-06-28

## Context

Through v0.0.5 the crate emitted `RssError::ValidationErrors(Vec<String>)` —
a flat list of human-readable strings. Downstream tooling
(static-site generators, CI gates, IDE plugins) had to grep against
those strings to surface field-level diagnostics. That coupling was
fragile, and the strings themselves didn't distinguish channel- from
item-level failures (issue #34: `staticdatagen` produced cryptic
`Invalid link: …` errors that named neither the failing field nor
the surrounding `RssData` instance).

## Decision

Change the variant payload to `Vec<ValidationError>`, where each
`ValidationError` is

```rust
pub struct ValidationError {
    pub field: String,    // dotted path, e.g. "channel.title", "item.0.link"
    pub message: String,  // human-readable text, e.g. "channel.title is missing"
}
```

`field` uses dotted-path notation
(`channel.<name>` / `item.<idx>.<name>` for RSS; `feed.<name>` /
`entry.<idx>.<name>` for Atom). `Display` writes the bare `message`,
so `e.to_string()` keeps the pre-v0.0.6 string format and existing
callers using `errors.iter().any(|e| e.to_string().contains("…"))`
continue to compile and pass.

## Alternatives considered

1. **Add a parallel `validate_structured()` returning the new
   shape, leaving `validate()` on `Vec<String>`.** Rejected as
   long-term debt: every callsite would need to choose between two
   APIs, and the string-based one would silently rot.
2. **Use a richer error enum
   (`MissingTitle`, `InvalidLink(String)`, …) instead of a struct.**
   Rejected because the field-name space is open (custom XML
   extensions, future spec additions). A `String` field keeps the
   API forward-compatible without enum-variant churn.
3. **Add a `code: &'static str` for typed dispatch.** Deferred —
   the dotted path covers the majority case, and `code` can be
   layered on later as a `#[non_exhaustive]` enum without breaking
   anyone.

## Consequences

- **Breaking API change** at the `RssError::ValidationErrors`
  variant — callers that pattern-matched on `Vec<String>` need to
  migrate (see [`docs/MIGRATION-0.0.5-to-0.0.6.md`](../MIGRATION-0.0.5-to-0.0.6.md)).
  Accepted: the crate is `0.0.x` and the new shape is the
  right long-term primitive.
- Downstream tools can now dispatch on `e.field == "channel.title"`
  rather than string parsing.
- `Display` is a thin wrapper over `message`, so the human-readable
  output is unchanged.

## References

- Issue [#34](https://github.com/sebastienrousseau/rssgen/issues/34)
- v0.0.6 audit, P2 "RssError::ValidationErrors flattens structure"
