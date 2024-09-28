// Copyright © 2024 RSS Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

//! # RSS Gen Validator Usage Examples
//!
//! This example demonstrates how to use the `RssFeedValidator` to validate the structure
//! and content of RSS feeds using the RSS Gen library.

#![allow(missing_docs)]

use rss_gen::validator::RssFeedValidator;
use rss_gen::{RssData, RssItem, RssVersion};

/// Demonstrates validating an RSS feed using the `RssFeedValidator`.
fn validate_rss_feed_example() {
    println!("\n🧪 RSS Gen Validator Usage Examples \n");

    let mut rss_data = RssData::new(Some(RssVersion::RSS2_0))
        .title("My Rust Blog")
        .link("https://myrustblog.com")
        .description("A blog about Rust programming and tutorials.")
        .atom_link("https://myrustblog.com/rss.xml") // Adding atom:link
        .generator("RSS Gen v0.0.1"); // Adding generator

    // Adding at least one item to the feed
    let item = RssItem::new()
        .title("First Post")
        .link("https://myrustblog.com/first-post")
        .description("This is my first post")
        .guid("123");
    rss_data.add_item(item);

    // Initialize the RSS feed validator
    let validator = RssFeedValidator::new(&rss_data);

    // Perform validation
    match validator.validate() {
        Ok(_) => println!("    ✅  RSS feed is valid!"),
        Err(e) => println!("    ❌  RSS feed validation failed: {}", e),
    }
}

/// Entry point for the RSS Gen Validator examples.
pub fn main() {
    println!("🦀 Rss Gen Validator Usage Examples");
    println!("---------------------------------------------");

    // Run the RSS feed validation example
    validate_rss_feed_example();
}
