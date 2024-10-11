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

/// Demonstrates validating an RSS feed using the `RssFeedValidator`.
fn validate_rss_feed_example() -> Result<(), Box<dyn Error>> {
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
    validator.validate().map_err(|e| {
        Box::new(ExampleError {
            message: format!("RSS feed validation failed: {}", e),
        }) as Box<dyn Error>
    })?;

    println!("    ✅  RSS feed is valid!");

    Ok(())
}

/// Demonstrates validating an invalid RSS feed.
fn validate_invalid_rss_feed_example() -> Result<(), Box<dyn Error>> {
    println!("\n🧪 RSS Gen Invalid Feed Validator Example \n");

    let invalid_rss_data = RssData::new(Some(RssVersion::RSS2_0))
        .title("") // Invalid: empty title
        .link("not-a-valid-url") // Invalid: incorrect URL format
        .description("An invalid RSS feed");

    let validator = RssFeedValidator::new(&invalid_rss_data);

    match validator.validate() {
        Ok(_) => {
            return Err(Box::new(ExampleError {
                message: "Validation unexpectedly passed for invalid feed".to_string(),
            }));
        }
        Err(e) => {
            println!("    ✅  Validation correctly failed: {}", e);
        }
    }

    Ok(())
}

/// Entry point for the RSS Gen Validator examples.
pub fn main() -> Result<(), Box<dyn Error>> {
    println!("🦀 Rss Gen Validator Usage Examples");
    println!("---------------------------------------------");

    // Run the RSS feed validation example
    validate_rss_feed_example()?;
    validate_invalid_rss_feed_example()?;

    Ok(())
}

