<!-- markdownlint-disable MD033 MD041 -->
<img src="https://kura.pro/rssgen/images/logos/rssgen.svg"
alt="RSS Gen logo" height="66" align="right" />
<!-- markdownlint-enable MD033 MD041 -->

# RSS Gen

A Rust library for generating, serializing, and deserializing RSS feeds for various RSS versions.

[![Made With Love][made-with-rust]][14] [![Crates.io][crates-badge]][08] [![lib.rs][libs-badge]][10] [![Docs.rs][docs-badge]][09] [![License][license-badge]][02] [![Build Status][build-badge]][16]

<!-- markdownlint-disable MD033 MD041 -->
<center>
<!-- markdownlint-enable MD033 MD041 -->

• [Website][01] • [Documentation][09] • [Report Bug][04] • [Request Feature][04] • [Contributing Guidelines][05]

<!-- markdownlint-disable MD033 MD041 -->
</center>
<!-- markdownlint-enable MD033 MD041 -->

## Overview

`rss-gen` is a Rust library for generating RSS feeds and serializing and deserializing RSS web content syndication formats. It supports the following RSS versions: RSS 0.90, RSS 0.91, RSS 0.92, RSS 1.0, and RSS 2.0.

## Features

- Generate RSS feeds for multiple RSS versions
- Serialize RSS data to XML format
- Deserialize XML content into RSS data structures
- Support for RSS 0.90, 0.91, 0.92, 1.0, and 2.0
- Flexible API for creating and manipulating RSS feed data
- Comprehensive error handling

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rss-gen = "0.1.0"
```

## Usage

Here's a basic example of how to use the `rss-gen` library:

```rust
use rss_gen::{RssData, generate_rss, RssVersion};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rss_data = RssData::new()
        .title("My Blog")
        .link("https://example.com")
        .description("A blog about Rust programming");

    let rss_feed = generate_rss(&rss_data, RssVersion::RSS2_0)?;
    println!("{}", rss_feed);

    Ok(())
}
```

## Documentation

For full API documentation, please visit [docs.rs/rss-gen][09].

## Supported RSS Versions

- RSS 0.90
- RSS 0.91
- RSS 0.92
- RSS 1.0
- RSS 2.0

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0][02])
- MIT license ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT][03])

at your option.

## Acknowledgments

This crate wouldn't be possible without the valuable open-source work of others, especially:

- [quick-xml](https://crates.io/crates/quick-xml) for fast XML serialization and deserialization.

[01]: https://ssg.com "RSS Gen Website"
[02]: https://opensource.org/license/apache-2-0/ "Apache License, Version 2.0"
[03]: https://opensource.org/licenses/MIT "MIT license"
[04]: https://github.com/sebastienrousseau/rss-gen/issues "Issues"
[05]: https://github.com/sebastienrousseau/rss-gen/blob/main/CONTRIBUTING.md "Contributing Guidelines"
[08]: https://crates.io/crates/rss-gen "Crates.io"
[09]: https://docs.rs/rss-gen "Docs.rs"
[10]: https://lib.rs/crates/rss-gen "Lib.rs"
[14]: https://www.rust-lang.org "The Rust Programming Language"
[16]: https://github.com/sebastienrousseau/rss-gen/actions?query=branch%3Amain "Build Status"

[build-badge]: https://img.shields.io/github/actions/workflow/status/sebastienrousseau/rss-gen/release.yml?branch=main&style=for-the-badge&logo=github "Build Status"
[crates-badge]: https://img.shields.io/crates/v/rss-gen.svg?style=for-the-badge 'Crates.io badge'
[docs-badge]: https://img.shields.io/docsrs/rss-gen.svg?style=for-the-badge 'Docs.rs badge'
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.1.0-orange.svg?style=for-the-badge 'Lib.rs badge'
[license-badge]: https://img.shields.io/crates/l/rss-gen.svg?style=for-the-badge 'License badge'
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust 'Made With Rust badge'
