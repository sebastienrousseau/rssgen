<!-- markdownlint-disable MD033 MD041 -->
<img src="https://kura.pro/rssgen/images/logos/rssgen.svg"
alt="RSS Gen logo" height="66" align="right" />
<!-- markdownlint-enable MD033 MD041 -->

# RSS Gen

A comprehensive Rust library for generating, parsing, serializing, and deserializing RSS feeds across various RSS versions.

[![Made With Love][made-with-rust]][14] [![Crates.io][crates-badge]][08] [![lib.rs][libs-badge]][10] [![Docs.rs][docs-badge]][09] [![License][license-badge]][02] [![Build Status][build-badge]][16]

<!-- markdownlint-disable MD033 MD041 -->
<center>
<!-- markdownlint-enable MD033 MD041 -->

• [Website][01] • [Documentation][09] • [Report Bug][04] • [Request Feature][04] • [Contributing Guidelines][05]

<!-- markdownlint-disable MD033 MD041 -->
</center>
<!-- markdownlint-enable MD033 MD041 -->

## Overview

`rss-gen` is a powerful Rust library designed for working with RSS feeds. It provides functionality for generating, parsing, serializing, and deserializing RSS content across multiple RSS versions. It supports the following RSS versions: RSS 0.90, RSS 0.91, RSS 0.92, RSS 1.0, and RSS 2.0. The library offers a flexible and efficient way to handle RSS feeds in your Rust projects.

## Features

- Support for RSS versions 0.90, 0.91, 0.92, 1.0, and 2.0
- Generation of RSS feeds from structured data
- Parsing of existing RSS feeds into structured data
- Serialization and deserialization of RSS data
- Extensible elements for managing standard and optional RSS fields
- Atom link support for modern syndication compatibility
- Image embedding for RSS 2.0 feeds
- Comprehensive error handling and validation
- Performance-optimized XML processing

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rss-gen = "0.0.1"
```

## Usage

Here's a basic example of how to use the `rss-gen` library to generate an RSS feed:

```rust
use rss_gen::{RssData, RssVersion, generate_rss};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rss_data = RssData::new(Some(RssVersion::RSS2_0))
        .title("My Rust Blog")
        .link("https://myrustblog.com")
        .description("A blog about Rust programming and tutorials.");

    match generate_rss(&rss_data) {
        Ok(rss_feed) => println!("Generated RSS feed:\n{}", rss_feed),
        Err(e) => eprintln!("Error generating RSS feed: {}", e),
    }

    Ok(())
}
```

For parsing an existing RSS feed:

```rust
use rss_gen::parse_rss;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rss_content = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0">
            <channel>
                <title>My Rust Blog</title>
                <link>https://myrustblog.com</link>
                <description>A blog about Rust programming and tutorials.</description>
            </channel>
        </rss>
    "#;

    match parse_rss(rss_content) {
        Ok(parsed_data) => println!("Parsed RSS data: {:?}", parsed_data),
        Err(e) => eprintln!("Error parsing RSS feed: {}", e),
    }

    Ok(())
}
```

## Macros

RSS Gen provides a set of convenient macros to simplify RSS feed generation and data manipulation tasks:

- `macro_generate_rss!`: Generates a complete RSS feed in XML format from a given `RssData` struct.
- `macro_write_element!`: Writes an individual XML element with a given name and content.
- `macro_set_rss_data_fields!`: Sets multiple fields of an `RssData` struct in one go.
- `macro_get_args!`: Retrieves a named argument from a `clap::ArgMatches` object, useful for command-line interfaces.
- `macro_metadata_option!`: Extracts an option value from metadata, typically used with `HashMap<String, String>`.

Refer to the [documentation][09] for more details on how to use these macros.

## Documentation

For full API documentation, please visit [docs.rs/rss-gen][09].

## Supported RSS Versions

- RSS 0.90
- RSS 0.91
- RSS 0.92
- RSS 1.0
- RSS 2.0

## Rust Version Compatibility

Compiler support: requires `rustc` 1.56.0+

## Examples

To run the examples, clone the repository and use the following command:

```shell
cargo run --example example_name
```

Replace `example_name` with the name of the example you want to run.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## License

The project is dual-licensed under the terms of both the MIT license and the Apache License (Version 2.0).

- [Apache License, Version 2.0][02]
- [MIT license][03]

## Acknowledgments

This crate wouldn't be possible without the valuable open-source work of others, especially:

- [quick-xml](https://crates.io/crates/quick-xml) for fast XML serialization and deserialization.

[01]: https://rssgen.co "RSS Gen Website"
[02]: https://opensource.org/license/apache-2-0/ "Apache License, Version 2.0"
[03]: https://opensource.org/licenses/MIT "MIT license"
[04]: https://github.com/sebastienrousseau/rssgen/issues "Issues"
[05]: https://github.com/sebastienrousseau/rssgen/blob/main/CONTRIBUTING.md "Contributing Guidelines"
[08]: https://crates.io/crates/rss-gen "Crates.io"
[09]: https://docs.rs/rss-gen "Docs.rs"
[10]: https://lib.rs/crates/rss-gen "Lib.rs"
[14]: https://www.rust-lang.org "The Rust Programming Language"
[16]: https://github.com/sebastienrousseau/rssgen/actions?query=branch%3Amain "Build Status"

[build-badge]: https://img.shields.io/github/actions/workflow/status/sebastienrousseau/rss-gen/release.yml?branch=main&style=for-the-badge&logo=github "Build Status"
[crates-badge]: https://img.shields.io/crates/v/rss-gen.svg?style=for-the-badge 'Crates.io badge'
[docs-badge]: https://img.shields.io/docsrs/rss-gen.svg?style=for-the-badge 'Docs.rs badge'
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.1.0-orange.svg?style=for-the-badge 'Lib.rs badge'
[license-badge]: https://img.shields.io/crates/l/rss-gen.svg?style=for-the-badge 'License badge'
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust 'Made With Rust badge'
