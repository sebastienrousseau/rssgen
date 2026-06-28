# Migrating from rss-gen 0.0.5 to 0.0.6

This is a short, mechanical migration guide. v0.0.6 introduces three
deliberate API breaks at the `0.0.x` SemVer level. Every other
change is additive.

## 1. `RssError::ValidationErrors` — variant payload changed

**Before (0.0.5):**

```rust
RssError::ValidationErrors(Vec<String>)
```

**After (0.0.6):**

```rust
RssError::ValidationErrors(Vec<ValidationError>)
```

`ValidationError { field, message }` — `field` is a dotted path
(`channel.title`, `item.0.link`, `feed.id`, `entry.2.updated`),
`message` is the human-readable text. `Display` writes the bare
`message`, so `e.to_string()` matches the pre-v0.0.6 string.

| Pre-v0.0.6 pattern | v0.0.6 equivalent |
| :--- | :--- |
| `errors.iter().any(\|e\| e == "channel.title is missing")` | `errors.iter().any(\|e\| e.field == "channel.title")` (preferred) |
| Same, string-compatible | `errors.iter().any(\|e\| e.to_string() == "channel.title is missing")` |
| `errors.iter().any(\|e\| e.contains("Invalid channel.link"))` | `errors.iter().any(\|e\| e.field == "channel.link" && e.message.contains("Invalid"))` |
| `RssError::ValidationErrors(vec!["…".into()])` | `RssError::ValidationErrors(vec![ValidationError::new("channel.title", "channel.title is missing")])` |

Rationale: ADR-0001 (`docs/adr/0001-structured-validation-errors.md`).

## 2. `data::parse_date` / `RssItem::pub_date_parsed` — return type changed

**Before (0.0.5):**

```rust
pub fn parse_date(date_str: &str) -> Result<dtt::datetime::DateTime>
```

**After (0.0.6):**

```rust
pub fn parse_date(date_str: &str) -> Result<time::OffsetDateTime>
```

The returned value now reflects the actual parsed offset rather than
a hard-coded UTC sentinel. Callers that only checked `.is_ok()` are
unaffected. Callers that read fields from the returned value need to
switch to `time::OffsetDateTime` accessors (`.year()`, `.month()`,
`.offset()`, …).

Rationale: ADR-0002 (`docs/adr/0002-time-only-date-stack.md`).

## 3. Removed dependencies and feature flag

| Removed | Why | Replacement |
| :--- | :--- | :--- |
| `dtt` | Experimental 0.0.x, redundant with `time` | `time::OffsetDateTime` directly |
| `commons` (euxis-commons) | Unused in `src/` | n/a |
| `serde_json` | Unused in `src/` (no JSON Feed support yet) | n/a |
| `async = []` feature | No `#[cfg]` paths gated on it | Real Tokio integration arrives in a follow-up |

If your project depended on these crates transitively *through*
`rss-gen` only (i.e. you weren't importing them yourself), no
action is needed. If you were relying on them being in the
dependency tree, declare them directly in your own `Cargo.toml`.

## 4. New surface (no migration needed — purely additive)

- `rss_gen::atom::{AtomFeed, AtomEntry, AtomPerson, AtomLink, AtomTextType, FeedFormat}`
- `rss_gen::atom::generate_atom`
- `rss_gen::atom::detect_feed_format`
- `rss_gen::ValidationError` (re-exported from `error`)

All re-exported from the crate root and `prelude`.

## 5. Validation behaviour changes (no API change)

- Item-level `link` now accepts relative URLs per RSS 2.0 §5.7
  (absolute URLs, root-relative paths like `/tags/`, and bare paths
  like `articles/foo.html`). Channel-level `link` is unchanged.
- `RssFeedValidator::parse_date` no longer requires a literal `" GMT"`
  suffix; any RFC 2822 timezone form is accepted, plus ISO 8601.
- `RssFeedValidator::validate_structure` no longer errors on items
  without a `<link>` (RSS 2.0 §5.7 explicitly allows
  description-only items).

If your test suite was asserting that the old GMT-only / link-required
behaviour fired, those assertions need to be removed.
