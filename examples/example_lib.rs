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
use std::error::Error;

/// Custom error type for example execution
#[derive(Debug)]
struct ExampleError {
    message: String,
}

impl std::fmt::Display for ExampleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Example Error: {}", self.message)
    }
}

impl Error for ExampleError {}

/// Entry point for the RSS Gen library examples.
///
/// This function demonstrates feed generation, feed parsing,
/// and how to work with different versions of RSS feeds.
pub fn main() -> Result<(), Box<dyn Error>> {
    println!("\n🧪 RSS Gen Library Usage Examples\n");

    // Run the examples
    generate_rss_example()?;
    parse_rss_example()?;
    quick_rss_example()?;

    println!("\n🎉  All examples completed successfully!\n");
    Ok(())
}

/// Demonstrates generating an RSS feed with version 2.0.
fn generate_rss_example() -> Result<(), Box<dyn Error>> {
    println!("🦀  Generate Rss Feed Example");
    println!("---------------------------------------------");

    let rss_data = RssData::new(Some(RssVersion::RSS2_0))
        .title("My Rust Blog")
        .link("https://myrustblog.com")
        .description("A blog about Rust programming and tutorials.");

    // Generate the RSS feed
    let rss_feed = generate_rss(&rss_data).map_err(|e| {
        Box::new(ExampleError {
            message: format!("Failed to generate RSS feed: {}", e),
        }) as Box<dyn Error>
    })?;

    println!("    ✅  Generated RSS feed:\n    {}", rss_feed);

    Ok(())
}

/// Demonstrates parsing an existing RSS feed.
fn parse_rss_example() -> Result<(), Box<dyn Error>> {
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
    let rss_data = parse_rss(rss_content).map_err(|e| {
        Box::new(ExampleError {
            message: format!("Failed to parse RSS feed: {}", e),
        }) as Box<dyn Error>
    })?;

    println!("    ✅  Parsed RSS feed: {:#?}", rss_data); // Pretty-print the parsed data

    Ok(())
}

/// Demonstrates using the quick_rss function.
fn quick_rss_example() -> Result<(), Box<dyn Error>> {
    println!("\n🦀 Quick RSS Generation Example");
    println!("---------------------------------------------");

    let rss_feed = rss_gen::quick_rss(
        "Quick RSS Feed",
        "https://example.com/quick",
        "A quickly generated RSS feed",
    )
    .map_err(|e| {
        Box::new(ExampleError {
            message: format!(
                "Failed to generate quick RSS feed: {}",
                e
            ),
        }) as Box<dyn Error>
    })?;

    println!("    ✅  Generated quick RSS feed:\n    {}", rss_feed);

    Ok(())
}
