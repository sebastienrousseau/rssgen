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
        .generator("RSS Gen v0.0.3"); // Adding generator

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
fn validate_invalid_rss_example() -> Result<(), Box<dyn Error>> {
    println!("\n🧪 RSS Gen Invalid Feed Validator Example");
    let invalid_rss_data = RssData::new(Some(RssVersion::RSS2_0))
        .title("") // Invalid: empty title
        .link("not-a-valid-url") // Invalid: incorrect URL format
        .description("An invalid RSS feed");

    let validator = RssFeedValidator::new(&invalid_rss_data);
    match validator.validate() {
        Ok(_) => println!(
            "    ❌ Validation unexpectedly passed for invalid feed"
        ),
        Err(e) => println!("    ✅ Validation correctly failed: {}", e),
    }
    Ok(())
}

fn validate_rss_0_90_example() -> Result<(), Box<dyn Error>> {
    println!("\n🦀 Validating RSS 0.90 Example");
    let mut rss_data = RssData::new(Some(RssVersion::RSS0_90))
        .title("Mozilla Dot Org")
        .link("http://www.mozilla.org")
        .description("the Mozilla Organization web site");

    rss_data.add_item(
        RssItem::new()
            .title("New Status Updates")
            .link("http://www.mozilla.org/status/")
            .guid("http://www.mozilla.org/status/"),
    );

    let validator = RssFeedValidator::new(&rss_data);
    match validator.validate() {
        Ok(_) => println!("    ✅ RSS 0.90 feed is valid"),
        Err(e) => {
            println!("    ❌ RSS 0.90 feed validation failed: {}", e)
        }
    }
    Ok(())
}

fn validate_rss_0_91_example() -> Result<(), Box<dyn Error>> {
    println!("\n🦀 Validating RSS 0.91 Example");
    let mut rss_data = RssData::new(Some(RssVersion::RSS0_91))
        .title("XML.com")
        .link("http://www.xml.com/")
        .description("XML.com features a rich mix of information and services for the XML community.")
        .language("en-us");

    rss_data.add_item(RssItem::new()
        .title("Normalizing XML, Part 2")
        .link("http://www.xml.com/pub/a/2002/12/04/normalizing.html")
        .description("In this second and final look at applying relational normalization techniques to W3C XML Schema data modeling, Will Provost discusses when not to normalize, the scope of uniqueness and the fourth and fifth normal forms.")
        .guid("http://www.xml.com/pub/a/2002/12/04/normalizing.html"));

    let validator = RssFeedValidator::new(&rss_data);
    match validator.validate() {
        Ok(_) => println!("    ✅ RSS 0.91 feed is valid"),
        Err(e) => {
            println!("    ❌ RSS 0.91 feed validation failed: {}", e)
        }
    }
    Ok(())
}

fn validate_rss_0_92_example() -> Result<(), Box<dyn Error>> {
    println!("\n🦀 Validating RSS 0.92 Example");
    let mut rss_data = RssData::new(Some(RssVersion::RSS0_92))
        .title("My Website")
        .link("http://www.example.com/")
        .description("News and updates from my website.")
        .language("en-us")
        .last_build_date("Mon, 11 Oct 2024 21:57:00 GMT");

    rss_data.add_item(
        RssItem::new()
            .title("First article title")
            .link("http://www.example.com/article1")
            .description("Short description of the article.")
            .pub_date("Mon, 11 Oct 2024 12:00:00 GMT")
            .guid("http://www.example.com/article1"),
    );

    let validator = RssFeedValidator::new(&rss_data);
    match validator.validate() {
        Ok(_) => println!("    ✅ RSS 0.92 feed is valid"),
        Err(e) => {
            println!("    ❌ RSS 0.92 feed validation failed: {}", e)
        }
    }
    Ok(())
}

fn validate_rss_1_0_example() -> Result<(), Box<dyn Error>> {
    println!("\n🦀 Validating RSS 1.0 Example");
    let mut rss_data = RssData::new(Some(RssVersion::RSS1_0))
        .title("XML.com")
        .link("http://www.xml.com/")
        .description("XML.com features a rich mix of information and services for the XML community.")
        .language("en-us");

    rss_data.add_item(RssItem::new()
        .title("Normalizing XML, Part 2")
        .link("http://www.xml.com/pub/a/2002/12/04/normalizing.html")
        .description("In this second and final look at applying relational normalization techniques to W3C XML Schema data modeling, Will Provost discusses when not to normalize, the scope of uniqueness and the fourth and fifth normal forms.")
        .pub_date("Wed, 04 Dec 2002 00:00:00 GMT")
        .guid("http://www.xml.com/pub/a/2002/12/04/normalizing.html"));

    let validator = RssFeedValidator::new(&rss_data);
    match validator.validate() {
        Ok(_) => println!("    ✅ RSS 1.0 feed is valid"),
        Err(e) => {
            println!("    ❌ RSS 1.0 feed validation failed: {}", e)
        }
    }
    Ok(())
}

fn validate_rss_2_0_example() -> Result<(), Box<dyn Error>> {
    println!("\n🦀 Validating RSS 2.0 Example");
    let mut rss_data = RssData::new(Some(RssVersion::RSS2_0))
        .title("XML.com")
        .link("http://www.xml.com/")
        .description("XML.com features a rich mix of information and services for the XML community.")
        .language("en-us")
        .atom_link("http://www.xml.com/rss.xml")
        .generator("RSS Gen v0.1.0");

    rss_data.add_item(RssItem::new()
        .title("Normalizing XML, Part 2")
        .link("http://www.xml.com/pub/a/2002/12/04/normalizing.html")
        .description("In this second and final look at applying relational normalization techniques to W3C XML Schema data modeling, Will Provost discusses when not to normalize, the scope of uniqueness and the fourth and fifth normal forms.")
        .pub_date("Wed, 04 Dec 2002 00:00:00 GMT")
        .guid("http://www.xml.com/pub/a/2002/12/04/normalizing.html"));

    let validator = RssFeedValidator::new(&rss_data);
    match validator.validate() {
        Ok(_) => println!("    ✅ RSS 2.0 feed is valid"),
        Err(e) => {
            println!("    ❌ RSS 2.0 feed validation failed: {}", e)
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
    validate_invalid_rss_example()?;
    validate_rss_0_90_example()?;
    validate_rss_0_91_example()?;
    validate_rss_0_92_example()?;
    validate_rss_1_0_example()?;
    validate_rss_2_0_example()?;

    Ok(())
}
