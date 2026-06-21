<p align="center">
  <img src="https://kura.pro/rssgen/images/logos/rssgen.svg" alt="RSS Gen logo" width="128" />
</p>

<h1 align="center">RSS Gen</h1>

<p align="center">
  <strong>A Rust library for generating, serializing, and deserializing RSS feeds for various RSS versions.</strong>
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/rssgen/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/rssgen/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/rss-gen"><img src="https://img.shields.io/crates/v/rss-gen.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/rss-gen"><img src="https://img.shields.io/badge/docs.rs-rss-gen-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://codecov.io/gh/sebastienrousseau/rssgen"><img src="https://img.shields.io/codecov/c/github/sebastienrousseau/rssgen?style=for-the-badge&logo=codecov" alt="Coverage" /></a>
  <a href="https://lib.rs/crates/rss-gen"><img src="https://img.shields.io/badge/lib.rs-v0.0.5-orange.svg?style=for-the-badge" alt="lib.rs" /></a>
</p>

---

## Install

```bash
cargo add rss-gen
```

Or add to `Cargo.toml`:

```toml
[dependencies]
rss-gen = "0.0.5"
```

You need [Rust](https://rustup.rs/) 1.79.0 or later. Works on macOS, Linux, and Windows.

---

## Overview

RSS Gen creates, serializes, and deserializes RSS feeds for multiple RSS versions.

- **Feed generation** with a builder API
- **XML serialization** to valid RSS output
- **Deserialization** of existing feeds into Rust structs
- **Multi-version support** — RSS 0.90 through 2.0

---

## Features

| | |
| :--- | :--- |
| **RSS generation** | Create RSS 2.0 feeds programmatically |
| **Serialization** | Serialize feeds to XML strings |
| **Deserialization** | Parse existing RSS feeds into Rust structs |
| **Multiple versions** | Support for RSS 0.90, 0.91, 0.92, 1.0, and 2.0 |
| **Validation** | Validate feed structure and required elements |

---

## Usage

```rust
use rss_gen::{generate_rss, RssData, RssVersion};

fn main() {
    let rss = RssData::new(Some(RssVersion::RSS2_0))
        .title("My Blog")
        .link("https://example.com")
        .description("A blog about Rust");

    println!("{}", generate_rss(&rss).unwrap());
}
```

---

## Development

```bash
cargo build        # Build the project
cargo test         # Run all tests
cargo clippy       # Lint with Clippy
cargo fmt          # Format with rustfmt
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, signed commits, and PR guidelines.

---

**THE ARCHITECT** \u1d2b [Sebastien Rousseau](https://sebastienrousseau.com)
**THE ENGINE** \u1d5e [EUXIS](https://euxis.co) \u1d2b Enterprise Unified Execution Intelligence System

---

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.

<p align="right"><a href="#rss-gen">Back to Top</a></p>