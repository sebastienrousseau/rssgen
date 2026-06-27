<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<p align="center">
  <img src="https://cloudcdn.pro/rssgen/v1/logos/rssgen.svg" alt="RSS Gen logo" width="128" />
</p>

<h1 align="center">rss-gen</h1>

<p align="center">
  An RSS and Atom 1.0 syndication library for Rust — generate, parse, and
  validate feeds across every shipping RSS version plus Atom 1.0, with
  <code>#![forbid(unsafe_code)]</code> and zero panics on the validated path.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/rssgen/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/rssgen/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/rss-gen"><img src="https://img.shields.io/crates/v/rss-gen.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/rss-gen"><img src="https://img.shields.io/badge/docs.rs-rss--gen-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://codecov.io/gh/sebastienrousseau/rssgen"><img src="https://img.shields.io/codecov/c/github/sebastienrousseau/rssgen?style=for-the-badge&logo=codecov" alt="Coverage" /></a>
  <a href="https://lib.rs/crates/rss-gen"><img src="https://img.shields.io/badge/lib.rs-rss--gen-orange.svg?style=for-the-badge" alt="lib.rs" /></a>
</p>

---

## Contents

- [Install](#install) — Cargo, MSRV, source
- [Quick Start](#quick-start) — generate an RSS 2.0 feed in ten lines
- [Library Usage](#library-usage)
  - [RSS generation](#rss-generation)
  - [Atom 1.0 generation](#atom-10-generation)
  - [Parsing existing feeds](#parsing-existing-feeds)
  - [Format auto-detection](#format-auto-detection)
  - [Validation diagnostics](#validation-diagnostics)
  - [`quick_rss` helper](#quick_rss-helper)
  - [Constants and limits](#constants-and-limits)
- [Modules](#modules)
- [Features](#features)
- [Development](#development)
- [Security](#security)
- [Documentation](#documentation)
- [License](#license)

---

## Install

### As a Rust library

```bash
cargo add rss-gen
```

Or pin explicitly in `Cargo.toml`:

```toml
[dependencies]
rss-gen = "0.0.6"
```

### MSRV

| Surface | Minimum Supported Rust Version |
| :--- | :--- |
| Library | **1.79.0** (bumped from 1.68.0 in 0.0.5 to track `quick-xml 0.40`) |

CI runs `stable` on every push. Cross-platform: macOS, Linux, Windows.

### Build from source

```bash
git clone https://github.com/sebastienrousseau/rssgen.git
cd rssgen
cargo test --workspace --all-features    # full suite, ~200 tests
```

---

## Quick Start

```rust
use rss_gen::{generate_rss, RssData, RssItem, RssVersion};

fn main() -> Result<(), rss_gen::RssError> {
    // Construct an RSS 2.0 channel via the builder API.
    let mut feed = RssData::new(Some(RssVersion::RSS2_0))
        .title("My Rust Blog")
        .link("https://example.com")
        .description("A blog about Rust");

    // Items are appended in the order they should appear.
    feed.add_item(
        RssItem::new()
            .title("Hello, world")
            .link("https://example.com/hello")
            .description("First post")
            .guid("https://example.com/hello"),
    );

    let xml = generate_rss(&feed)?;
    assert!(xml.contains("<rss version=\"2.0\""));
    assert!(xml.contains("<title>My Rust Blog</title>"));
    Ok(())
}
```

---

## Library Usage

### RSS generation

`rss-gen` covers every shipping RSS version. Pick the wire format via
`RssVersion`; the builder is the same for all of them.

```rust
use rss_gen::{generate_rss, RssData, RssItem, RssVersion};

fn main() -> Result<(), rss_gen::RssError> {
    let mut feed = RssData::new(Some(RssVersion::RSS2_0))
        .title("Engineering")
        .link("https://example.com")
        .description("Posts from the engineering team")
        .language("en-GB")
        .pub_date("Sat, 27 Jun 2026 00:00:00 GMT")
        .generator("rss-gen")
        // `atom_link` advertises the canonical feed URL via the
        // `<atom:link rel="self">` element on the channel.
        .atom_link("https://example.com/feed.xml");

    feed.add_item(
        RssItem::new()
            .title("Release notes")
            .link("https://example.com/release")
            .description("What shipped this week")
            .guid("https://example.com/release")
            .pub_date("Sat, 27 Jun 2026 00:00:00 GMT")
            .author("editor@example.com"),
    );

    let xml = generate_rss(&feed)?;
    // The emitted document is well-formed XML and includes the
    // standard atom:self link required by most feed readers.
    assert!(xml.contains(r#"<atom:link href="https://example.com/feed.xml""#));
    Ok(())
}
```

`RssVersion::{RSS0_90, RSS0_91, RSS0_92, RSS1_0, RSS2_0}` are all
supported. RSS 1.0 emits the RDF wrapper (`<rdf:RDF>`); the others
emit a `<rss>` root.

### Atom 1.0 generation

Atom is a sibling code path — independent types, the same ergonomics.
RFC 4287 required elements (`id`, `title`, `updated`) are checked at
serialise time; entries inherit feed-level authors when none are
declared per-entry, per §4.1.1.

```rust
use rss_gen::{generate_atom, AtomEntry, AtomFeed};

fn main() -> Result<(), rss_gen::RssError> {
    let feed = AtomFeed::new()
        .id("https://example.com/feed")
        .title("My Atom Feed")
        .updated("2026-06-27T00:00:00Z")
        .author_name("Jane Doe")
        // Convenience: emit `<link rel="self" href="...">`.
        .self_link("https://example.com/atom.xml")
        .add_entry(
            AtomEntry::new()
                .id("https://example.com/post-1")
                .title("First Post")
                .updated("2026-06-27T00:00:00Z")
                // Plain-text summary; use `summary_html` for an
                // explicit `type="html"` payload.
                .summary("Hello, Atom"),
        );

    let xml = generate_atom(&feed)?;
    assert!(xml.contains(r#"<feed xmlns="http://www.w3.org/2005/Atom">"#));
    Ok(())
}
```

Media enclosures (podcasts, attached video) follow Atom's typed-link
model:

```rust
use rss_gen::{generate_atom, AtomEntry, AtomFeed};

fn main() -> Result<(), rss_gen::RssError> {
    let feed = AtomFeed::new()
        .id("https://example.com/podcast")
        .title("Pilot")
        .updated("2026-06-27T00:00:00Z")
        .author_name("Producer")
        .add_entry(
            AtomEntry::new()
                .id("https://example.com/ep-1")
                .title("Episode 1")
                .updated("2026-06-27T00:00:00Z")
                .summary("Pilot episode")
                // Emits <link rel="enclosure" type="audio/mpeg" length="12345678" .../>
                .add_enclosure(
                    "https://example.com/ep-1.mp3",
                    "audio/mpeg",
                    12_345_678,
                ),
        );

    let xml = generate_atom(&feed)?;
    assert!(xml.contains(r#"rel="enclosure""#));
    assert!(xml.contains(r#"type="audio/mpeg""#));
    Ok(())
}
```

### Parsing existing feeds

`parse_rss` reads any supported RSS version into the same `RssData`
struct used by the generator — round-tripping is symmetric.

```rust
use rss_gen::parse_rss;

fn main() -> Result<(), rss_gen::RssError> {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Example</title>
    <link>https://example.com</link>
    <description>Sample feed</description>
    <item>
      <title>Post</title>
      <link>https://example.com/post</link>
      <description>Body</description>
    </item>
  </channel>
</rss>"#;

    let feed = parse_rss(xml, None)?;
    assert_eq!(feed.title, "Example");
    assert_eq!(feed.items.len(), 1);
    assert_eq!(feed.items[0].title, "Post");
    Ok(())
}
```

### Format auto-detection

When the caller does not know upfront whether the document is RSS or
Atom, `detect_feed_format` peeks the root element and returns a
classifier value. It does not parse the rest of the document.

```rust
use rss_gen::{detect_feed_format, FeedFormat};

let rss_xml  = r#"<?xml version="1.0"?><rss version="2.0"><channel/></rss>"#;
let atom_xml = r#"<?xml version="1.0"?><feed xmlns="http://www.w3.org/2005/Atom"><id/></feed>"#;

assert_eq!(detect_feed_format(rss_xml),  FeedFormat::Rss);
assert_eq!(detect_feed_format(atom_xml), FeedFormat::Atom);
```

The classifier returns `FeedFormat::RssRdf` for RSS 1.0 (`<rdf:RDF>`)
and `FeedFormat::Unknown` for documents that match neither root
element — including hand-rolled `<feed>` documents that omit the Atom
namespace declaration, so a misclassified payload does not silently
flow into the Atom code path.

### Validation diagnostics

Every validation error is prefixed with the element that produced it,
so downstream tools (static site generators, CI gates, IDE
integrations) can point at the offending field rather than a bare
"missing field" message.

| Context | Example error string |
| :--- | :--- |
| RSS channel | `channel.title is missing`, `Invalid channel.link: …`, `Invalid channel.pub_date: …` |
| RSS item | `item.title is missing`, `item.link is missing`, `Invalid item.link: …` |
| Atom feed | `feed.id is missing`, `feed.updated is not a valid RFC 3339 timestamp: …` |
| Atom entry | `entry.0.author is missing (and feed has no feed-level author)`, `entry.2.updated is missing` |

```rust
use rss_gen::{generate_rss, RssData, RssError};

fn main() {
    // Channel without a title, link, or description.
    let feed = RssData::new(None);

    match generate_rss(&feed) {
        Err(RssError::ValidationErrors(errors)) => {
            assert!(errors.iter().any(|e| e == "channel.title is missing"));
            assert!(errors.iter().any(|e| e == "channel.link is missing"));
            assert!(errors.iter().any(|e| e == "channel.description is missing"));
        }
        other => panic!("expected ValidationErrors, got {other:?}"),
    }
}
```

Item-level `link` follows RSS 2.0 §5.7 — absolute URLs, root-relative
paths (`/tags/`), and bare paths (`articles/foo.html`) are all
accepted; whitespace, control characters, and empty strings are
rejected. Channel-level `link` retains absolute-URL strictness because
the spec requires it.

### `quick_rss` helper

For one-shot generation of a minimal feed, `quick_rss` validates input
bounds (length caps, URL scheme) and returns the serialised XML
directly. Useful inside `build.rs`, snippet generators, and tests.

```rust
use rss_gen::quick_rss;

fn main() -> Result<(), rss_gen::RssError> {
    let xml = quick_rss(
        "My Rust Blog",
        "https://example.com",
        "A blog about Rust",
    )?;
    assert!(xml.contains("<title>My Rust Blog</title>"));
    assert!(xml.contains("<item>")); // an example item is included
    Ok(())
}
```

### Constants and limits

`rss-gen` exposes its hard input bounds as `pub const` so callers can
validate ahead of time rather than discover the limit at serialise
time.

| Constant | Value | Applies to |
| :--- | ---: | :--- |
| `MAX_TITLE_LENGTH` | 256 | `RssData::title`, `RssItem::title` |
| `MAX_LINK_LENGTH` | 2 048 | `RssData::link`, `RssItem::link` |
| `MAX_DESCRIPTION_LENGTH` | 100 000 | `RssData::description`, `RssItem::description` |
| `MAX_GENERAL_LENGTH` | 1 024 | `RssData::category` and similar single-line fields |
| `MAX_FEED_SIZE` | 1 048 576 (1 MiB) | Combined serialised feed size, enforced by `RssData::validate_size` |

The `VERSION` constant resolves to the crate's `CARGO_PKG_VERSION` at
compile time.

```rust
use rss_gen::{MAX_FEED_SIZE, VERSION};

assert!(VERSION.starts_with(char::is_numeric));
assert_eq!(MAX_FEED_SIZE, 1_048_576);
```

---

## Modules

| Module | Surface |
| :--- | :--- |
| `rss_gen::atom` | `AtomFeed`, `AtomEntry`, `AtomPerson`, `AtomLink`, `AtomTextType`, `FeedFormat`, `generate_atom`, `detect_feed_format`. |
| `rss_gen::data` | `RssData`, `RssItem`, `RssVersion`, plus the `RssDataField` / `RssItemField` enums used by `set_field` / `set_item_field`. |
| `rss_gen::error` | `RssError` (the crate-wide error enum) and the `Result<T> = std::result::Result<T, RssError>` alias. |
| `rss_gen::generator` | `generate_rss`, `sanitize_content`, `write_element` — the RSS serialisation pipeline. |
| `rss_gen::parser` | `parse_rss`, plus the `ElementHandler` trait for callers that need to plug in custom-element extraction. |
| `rss_gen::validator` | Standalone validation helpers used by the generator and exposed for callers that want to validate before constructing. |
| `rss_gen::macros` | Procedural shortcuts (`macro_set_rss_data_fields!`, `macro_write_element!`, …) for terse builder code. |
| `rss_gen::prelude` | Re-exports the surface most callers need — `RssData`, `RssItem`, `RssVersion`, `AtomFeed`, `AtomEntry`, `generate_rss`, `generate_atom`, `parse_rss`, `quick_rss`, `detect_feed_format`, and the error types. |

---

## Features

| | |
| :--- | :--- |
| **RSS generation** | Author RSS 0.90, 0.91, 0.92, 1.0, and 2.0 feeds through a single `RssData` builder. RSS 2.0 emits the standard `xmlns:atom` declaration so feed readers can recognise the `<atom:link rel="self">` element on the channel. |
| **Atom 1.0 generation** | `AtomFeed` / `AtomEntry` cover RFC 4287 required elements, multi-author and contributor lists, categories, `xml:lang`, icon/logo/rights/subtitle, plain-text and HTML payloads for `<summary>` / `<content>`, and `<link rel="enclosure">` media attachments. |
| **Parsing** | `parse_rss` reads RSS 0.9x / 1.0 / 2.0 into `RssData` and is symmetric with the generator — `parse_rss(generate_rss(x)?, None)?` round-trips structurally for valid inputs. |
| **Format detection** | `detect_feed_format` classifies a document as `Rss`, `RssRdf`, `Atom`, or `Unknown` from the first start element, without parsing the body. |
| **Validation** | Required-element checks at both feed and item / entry level, with `channel.`, `item.`, `feed.`, and `entry.<idx>.` context prefixes on every error message. RFC 3339 timestamp validation for Atom; relative item links for RSS 2.0 §5.7. |
| **Sanitisation** | `sanitize_content` strips invalid XML control characters and is idempotent — round-tripping through it does not double-encode entities. |
| **Robust XML backend** | `quick-xml` 0.40 with the `serialize` feature. Output is UTF-8, well-formed, and includes the `<?xml version="1.0" encoding="utf-8"?>` declaration. |
| **Memory safety** | `#![forbid(unsafe_code)]` at the crate root; no FFI, no raw-pointer dereferences. |
| **Lint posture** | `#![deny(clippy::all, clippy::cargo, clippy::pedantic)]` and `#![deny(missing_docs)]` — every public item is documented; every public surface passes pedantic Clippy. |
| **Test posture** | Unit tests, integration tests, doctest coverage, property tests (`proptest`, `quickcheck`), and structure-aware fuzzers (`cargo fuzz`) for parsing, generation, date parsing, content sanitisation, and URL validation. |

---

## Development

```bash
cargo build                                  # build the library
cargo test --workspace --all-features        # full test suite
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo bench --bench benchmark                # Criterion harness
cargo +nightly fuzz run fuzz_rss_parsing     # structure-aware fuzzing
```

CI runs through a reusable workflow that gates Clippy, formatting,
tests, `cargo audit`, `cargo deny`, dependency review, and `CodeQL` on
every push and pull request. See [CONTRIBUTING.md](CONTRIBUTING.md)
for signed-commit policy and PR guidelines.

---

## Security

- **No `unsafe` blocks.** `#![forbid(unsafe_code)]` is enforced at the
  crate root — no FFI to a C XML parser, no raw-pointer dereferences,
  no `unsafe` blocks in the generator, parser, validator, or
  sanitiser.
- **Input bounds.** `MAX_TITLE_LENGTH`, `MAX_LINK_LENGTH`,
  `MAX_DESCRIPTION_LENGTH`, `MAX_GENERAL_LENGTH`, and `MAX_FEED_SIZE`
  cap every text dimension; `quick_rss` and `RssData::validate` /
  `RssData::validate_size` enforce them. Set these before exposing
  feed generation to untrusted input.
- **Content sanitisation.** `sanitize_content` removes invalid XML
  control characters (everything below `0x20` except `\n`, `\r`, `\t`)
  and escapes `&`, `<`, `>`, `"`, and `'` so user-supplied content
  cannot break feed well-formedness or smuggle injected markup.
- **URL hygiene.** Channel-level `link` requires an absolute
  `http` / `https` URL. Item-level `link` follows RSS 2.0 §5.7 and
  permits relative paths, but rejects whitespace and control
  characters that would otherwise break feed-reader parsers.
- **Supply chain.** `cargo audit` and `cargo deny` run on every push
  via the shared security workflow. Dependabot is wired for the
  `minor-and-patch` group on `Cargo.toml`. `CodeQL` static analysis
  runs on every push and pull request.

Coordinated-disclosure contact and policy live in `CONTRIBUTING.md`.

---

## Documentation

| Document | Covers |
| :--- | :--- |
| [`CHANGELOG.md`](CHANGELOG.md) | Per-release notes following Keep a Changelog 1.1.0. |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Setup, signed-commit policy, PR guidelines. |
| [`AUTHORS.md`](AUTHORS.md) | Contributor roll. |
| [docs.rs/rss-gen](https://docs.rs/rss-gen) | Generated API reference for the published version. |
| [`rssgen.co`](https://rssgen.co) | Long-form documentation, examples, and design notes. |

---

**THE ARCHITECT** — [Sebastien Rousseau](https://sebastienrousseau.com)

**THE ENGINE** — [EUXIS](https://euxis.co) — Enterprise Unified Execution Intelligence System

---

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.

<p align="right"><a href="#contents">Back to Top</a></p>
