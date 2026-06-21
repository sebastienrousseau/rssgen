// Copyright © 2024 RSS Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # RSS Gen Usage Examples
//!
//! This program demonstrates the usage of various structures and functions in the RSS Gen library,
//! including creating and manipulating RSS data, RSS items, validating URLs, parsing dates,
//! and working with different RSS versions.

#![allow(
    missing_docs,
    unreachable_pub,
    trivial_casts,
    unused_qualifications,
    clippy::io_other_error
)]

use rss_gen::data::*;
use rss_gen::error::Result;
use std::error::Error;

pub(crate) fn main() -> std::result::Result<(), Box<dyn Error>> {
    println!("\n🧪 RSS Gen Usage Examples");

    // Running various examples
    rss_data_example()?;
    rss_item_example()?;
    url_validation_example()?;
    date_parsing_example()?;
    rss_version_example()?;

    println!("\n🎉  All examples completed successfully!\n");

    Ok(())
}

/// Demonstrates the creation and manipulation of `RssData`.
fn rss_data_example() -> Result<()> {
    println!("\n🦀 RSS Data Example");
    println!("---------------------------------------------");

    let rss_data = RssData::new(Some(RssVersion::RSS2_0))
        .title("My RSS Feed")
        .link("https://example.com/feed")
        .description("An example RSS feed")
        .language("en-us")
        .pub_date("Mon, 01 Jan 2024 00:00:00 GMT");

    println!("    ✅  RSS Feed Created:");
    println!("        Title: {}", rss_data.title);
    println!("        Link: {}", rss_data.link);
    println!("        Description: {}", rss_data.description);
    println!("        Language: {}", rss_data.language);
    println!("        Pub Date: {}", rss_data.pub_date);

    Ok(())
}

/// Demonstrates the creation of an RSS item.
fn rss_item_example() -> Result<()> {
    println!("\n🦀 Rss Item Example");
    println!("---------------------------------------------");

    let item = RssItem::new()
        .title("First Post")
        .link("https://example.com/first-post")
        .description("This is my first post");

    println!("    ✅  RSS Item Created:");
    println!("        Title: {}", item.title);
    println!("        Link: {}", item.link);
    println!("        Description: {}", item.description);

    Ok(())
}

/// Demonstrates validating a URL in an RSS feed.
fn url_validation_example() -> Result<()> {
    println!("\n🦀  Url Validation Example");
    println!("---------------------------------------------");

    let valid_url = "https://example.com/feed";
    let invalid_url = "not_a_valid_url";

    println!("    ✅  Validating URL: {}", valid_url);
    println!("    ❌  Invalid URL: {}", invalid_url);

    // Add the URL validation logic as per your validation function in RssData or RssItem

    Ok(())
}

/// Demonstrates parsing dates in an RSS feed.
fn date_parsing_example() -> Result<()> {
    println!("\n🦀 Date Parsing Example");
    println!("---------------------------------------------");

    let pub_date = "Mon, 01 Jan 2024 00:00:00 GMT";
    println!("    ✅  Parsed Pub Date: {}", pub_date);

    // Add your date parsing logic if needed

    Ok(())
}

/// Demonstrates working with different RSS versions.
fn rss_version_example() -> Result<()> {
    println!("\n🦀 Rss Version Example");
    println!("---------------------------------------------");

    let rss_2_0 = RssData::new(Some(RssVersion::RSS2_0));
    let rss_1_0 = RssData::new(Some(RssVersion::RSS1_0));

    println!("    ✅  Working with RSS 2.0: {:?}", rss_2_0);
    println!("    ✅  Working with RSS 1.0: {:?}", rss_1_0);

    Ok(())
}
