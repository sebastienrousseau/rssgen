// Copyright © 2024 RSS Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

//! # RSS Gen Macros Usage Examples
//!
//! This example demonstrates the usage of various macros provided by the RSS Gen library,
//! including generating RSS feeds and setting RSS fields.

#![allow(missing_docs)]

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Error as XmlError;
use quick_xml::Writer;
use rss_gen::{
    macro_generate_rss, macro_write_element, RssData, RssVersion,
};
use std::io::Cursor;
use std::error::Error;
use rss_gen::macro_set_rss_data_fields;

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

impl std::error::Error for ExampleError {}

/// Entry point for the RSS Gen macros examples.
///
/// This function demonstrates generating an RSS feed and writing XML elements using macros.
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 RSS Gen Macros Usage Examples\n");

    // Running the examples
    generate_rss_macro_example()?;
    write_element_macro_example()?;
    set_rss_data_fields_macro_example()?;

    println!("\n🎉  All examples completed successfully!\n");
    Ok(())
}

/// Demonstrates generating an RSS feed using the `macro_generate_rss!` macro.
fn generate_rss_macro_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀  Generate Rss Feed Macro Example");
    println!("---------------------------------------------");

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let rss_data = RssData::new(Some(RssVersion::RSS2_0))
        .title("My Rust Blog")
        .link("https://myrustblog.com")
        .description("A blog about Rust programming and tutorials.");

    // Generate RSS feed using macro
    macro_generate_rss!(&mut writer, rss_data).map_err(|e: XmlError| {
        Box::new(ExampleError {
            message: format!("Failed to generate RSS: {}", e),
        }) as Box<dyn std::error::Error>
    })?;

    let result = writer.into_inner().into_inner();
    let xml = String::from_utf8(result).map_err(|e| {
        Box::new(ExampleError {
            message: format!("Failed to convert RSS to UTF-8: {}", e),
        }) as Box<dyn std::error::Error>
    })?;

    println!("    ✅  Generated RSS feed:\n    {}", xml);

    Ok(())
}

/// Demonstrates writing an XML element using the `macro_write_element!` macro.
fn write_element_macro_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🦀 Write Xml Element Macro Example");
    println!("---------------------------------------------");

    let mut writer = Writer::new(Cursor::new(Vec::new()));

    // Writing an XML element using macro
    macro_write_element!(&mut writer, "title", "My Rust Blog").map_err(|e: XmlError| {
        Box::new(ExampleError {
            message: format!("Failed to write XML element: {}", e),
        }) as Box<dyn std::error::Error>
    })?;

    let result = writer.into_inner().into_inner();
    let xml = String::from_utf8(result).map_err(|e| {
        Box::new(ExampleError {
            message: format!("Failed to convert XML to UTF-8: {}", e),
        }) as Box<dyn std::error::Error>
    })?;

    println!("    ✅  XML Element written:\n    {}", xml);

    Ok(())
}

/// Demonstrates using the macro_set_rss_data_fields! macro.
fn set_rss_data_fields_macro_example() -> Result<(), Box<dyn Error>> {
    println!("\n🦀 Set Rss Data Fields Macro Example");
    println!("---------------------------------------------");

    let mut rss_data = RssData::new(Some(RssVersion::RSS2_0));

    macro_set_rss_data_fields!(rss_data,
        Title = "Macro-generated Feed",
        Link = "https://example.com/macro",
        Description = "This feed was created using the macro_set_rss_data_fields! macro",
        Language = "en-US",
        PubDate = "Mon, 01 Jan 2024 00:00:00 GMT"
    );

    println!("    ✅  RSS Data set using macro:");
    println!("       Title: {}", rss_data.title);
    println!("       Link: {}", rss_data.link);
    println!("       Description: {}", rss_data.description);
    println!("       Language: {}", rss_data.language);
    println!("       PubDate: {}", rss_data.pub_date);

    Ok(())
}
