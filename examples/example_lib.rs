// Copyright © 2024 RSS Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

//! # RSS Gen Library Usage Examples
//!
//! This example demonstrates the usage of various components of the RSS Gen library,
//! including RSS feed generation, parsing, serialization, and deserialization.

#![allow(missing_docs)]

use rss_gen::{generate_rss, parse_rss, RssData, RssVersion};

/// Entry point for the RSS Gen library examples.
///
/// This function demonstrates feed generation, feed parsing,
/// and how to work with different versions of RSS feeds.
pub(crate) fn main() {
    println!("\n🧪 RSS Gen Library Usage Examples\n");

    // Run the examples
    generate_rss_example();
    parse_rss_example();

    println!("\n🎉  All examples completed successfully!\n");
}

/// Demonstrates generating an RSS feed with version 2.0.
fn generate_rss_example() {
    println!("🦀  Generate Rss Feed Example");
    println!("---------------------------------------------");

    let rss_data = RssData::new(Some(RssVersion::RSS2_0))
        .title("My Rust Blog")
        .link("https://myrustblog.com")
        .description("A blog about Rust programming and tutorials.");

    // Generate the RSS feed
    match generate_rss(&rss_data) {
        Ok(rss_feed) => {
            println!("    ✅  Generated RSS feed:\n    {}", rss_feed)
        }
        Err(e) => println!("    ❌  Error generating RSS feed: {}", e),
    }
}

/// Demonstrates parsing an existing RSS feed.
fn parse_rss_example() {
    println!("\n🦀 Parse Rss Feed Example");
    println!("---------------------------------------------");

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

    // Parse the RSS content
    match parse_rss(rss_content) {
        Ok(rss_data) => {
            println!("    ✅  Parsed RSS feed: {:#?}", rss_data); // Pretty-print the parsed data
        }
        Err(e) => println!("    ❌  Error parsing RSS feed: {}", e),
    }
}
