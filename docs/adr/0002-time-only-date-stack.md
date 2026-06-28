# ADR-0002: Date stack consolidated on `time` (drop `dtt`)

**Status:** Accepted (v0.0.6)
**Date:** 2026-06-28

## Context

Through v0.0.5 the crate depended on three crates for date handling:

- `dtt = "0.0"` — author-maintained custom datetime crate.
- `time = "0.3"` — the de-facto Rust datetime crate, already pulled
  in for RFC 2822 / ISO 8601 parsing.
- `commons = { package = "euxis-commons", … }` — a workspace-shared
  helper crate that internally pulled in the `time` feature set.

The three crates were redundant. `dtt 0.0.x` has no SemVer stability
guarantee (every 0.0.x release can break consumers without warning),
and the existing `parse_date` immediately discarded `time`'s parsed
`OffsetDateTime` by constructing a sentinel UTC `dtt::DateTime` —
callers never saw the actual parsed offset. Meanwhile
`euxis-commons` was declared in `[dependencies]` but `grep -r commons::
src/` returned zero hits.

## Decision

- Drop `dtt` entirely. `parse_date` and
  `RssItem::pub_date_parsed` return `time::OffsetDateTime` directly.
- Drop `commons` entirely (unused).
- Declare `time` with explicit
  `features = ["parsing", "formatting", "macros"]`. Previously the
  parsing feature was pulled in transitively via `commons`; with
  `commons` gone, we need to be explicit so the build doesn't break
  on the next dep tree shake.

## Alternatives considered

1. **Pin `dtt` to the latest 0.0.x and live with it.** Rejected:
   pre-1.0 crates routinely break their callers, and the value
   `dtt` was adding (a UTC sentinel return) was negative.
2. **Wrap `time::OffsetDateTime` in a crate-internal newtype.**
   Deferred: `OffsetDateTime` is already the right abstraction, and
   wrapping it would force users to convert at the boundary for no
   gain.
3. **Add `chrono` as an alternate backend.** Rejected: `time` and
   `chrono` cover overlapping ground and `time` is the lighter
   dependency; doubling the surface is the wrong trade.

## Consequences

- **Breaking API change** on `parse_date` and
  `RssItem::pub_date_parsed` return types (`dtt::DateTime` →
  `time::OffsetDateTime`). Callers that only checked `.is_ok()` are
  unaffected; callers that read fields out of the returned value
  must migrate.
- Compile-time and runtime closer to the metal — no
  parse-then-discard-then-construct round-trip.
- Dependency graph smaller (three crates dropped: `dtt`,
  `euxis-commons`, and their transitives).

## References

- v0.0.6 audit, "Crate Redundancies in Date/Time Crates"
- [`time` crate](https://crates.io/crates/time) feature matrix
