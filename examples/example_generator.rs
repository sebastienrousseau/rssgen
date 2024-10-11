// Copyright © 2024 RSS Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

//! # RSS Generator Library Usage Examples
//!
//! This example demonstrates the usage of various components of the RSS Generator library,
//! including content sanitization, XML element writing, and RSS generation functionality.

#![allow(missing_docs)]

use quick_xml::Writer;
use rss_gen::generator::{sanitize_content, write_element};
use std::io::Cursor;
use std::error::Error;
use rss_gen::{generate_rss, RssData, RssItem, RssVersion};

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

/// Entry point for the RSS Generator library examples.
///
/// This function demonstrates content sanitization, XML element writing,
/// and basic RSS generation.
pub fn main() -> Result<(), Box<dyn Error>> {
    println!("\n🧪 RSS Generator Library Usage Examples\n");

    // Running the examples
    sanitize_content_example()?;
    write_element_example()?;
    generate_complex_rss_example()?;

    println!("\n🎉  All examples completed successfully!\n");
    Ok(())
}

/// Demonstrates content sanitization by removing invalid XML characters and escaping special characters.
fn sanitize_content_example() -> Result<(), Box<dyn Error>> {
    println!("🦀 Content Sanitization Example");
    println!("---------------------------------------------");

    let content = "This is a test <content> with invalid & characters";
    let sanitized = sanitize_content(content);

    println!("    ✅  Original content: {}", content);
    println!("    ✅  Sanitized content: {}", sanitized);

    Ok(())
}

/// Demonstrates writing an XML element using the `quick_xml` crate.
///
/// This function writes an XML element and prints its representation.
fn write_element_example() -> Result<(), Box<dyn Error>> {
    println!("\n🦀 Write Xml Element Example");
    println!("---------------------------------------------");

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let element_name = "example";
    let content = "This is an example content";

    // Writing the XML element
    write_element(&mut writer, element_name, content).map_err(|e| {
        Box::new(ExampleError {
            message: format!("Failed to write XML element: {}", e),
        }) as Box<dyn Error>
    })?;

    let result = writer.into_inner().into_inner();
    let xml = String::from_utf8(result).map_err(|e| {
        Box::new(ExampleError {
            message: format!("Failed to convert XML to UTF-8: {}", e),
        }) as Box<dyn Error>
    })?;

    println!("    ✅  XML Element written:\n    {}", xml);

    Ok(())
}

/// Demonstrates generating an RSS feed with multiple items and custom fields.
fn generate_complex_rss_example() -> Result<(), Box<dyn Error>> {
    println!("\n🦀  Generate Complex Rss Feed Example");
    println!("---------------------------------------------");

    let mut rss_data = RssData::new(Some(RssVersion::RSS2_0))
        .title("Complex RSS Feed")
        .link("https://example.com/complex")
        .description("A complex RSS feed with multiple items and custom fields")
        .language("en-US")
        .pub_date("Mon, 01 Jan 2024 00:00:00 GMT")
        .last_build_date("Mon, 01 Jan 2024 12:00:00 GMT")
        .ttl("60")
        .image_url("https://example.com/image.jpg");

    // Add multiple items
    for i in 1..=3 {
        let item = RssItem::new()
            .title(format!("Item {}", i))
            .link(format!("https://example.com/item{}", i))
            .description(format!("Description for item {}", i))
            .pub_date(format!("Mon, 0{} Jan 2024 00:00:00 GMT", i))
            .guid(format!("unique-id-{}", i));
        rss_data.add_item(item);
    }

    let rss_feed = generate_rss(&rss_data).map_err(|e| {
        Box::new(ExampleError {
            message: format!("Failed to generate complex RSS feed: {}", e),
        }) as Box<dyn Error>
    })?;

    println!("    ✅  Generated complex RSS feed:\n    {}", rss_feed);

    Ok(())
}
