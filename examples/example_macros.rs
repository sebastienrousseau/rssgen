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

/// Entry point for the RSS Gen macros examples.
///
/// This function demonstrates generating an RSS feed and writing XML elements using macros.
pub(crate) fn main(
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n🧪 RSS Gen Macros Usage Examples\n");

    // Running the examples
    generate_rss_macro_example()?;
    write_element_macro_example()?;

    println!("\n🎉  All examples completed successfully!\n");
    Ok(())
}

/// Demonstrates generating an RSS feed using the `macro_generate_rss!` macro.
fn generate_rss_macro_example(
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🦀  Generate Rss Feed Macro Example");
    println!("---------------------------------------------");

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let rss_data = RssData::new(Some(RssVersion::RSS2_0))
        .title("My Rust Blog")
        .link("https://myrustblog.com")
        .description("A blog about Rust programming and tutorials.");

    // Generate RSS feed using macro
    match macro_generate_rss!(&mut writer, rss_data) {
        Ok(_) => {
            let result = writer.into_inner().into_inner();
            let xml = String::from_utf8(result)?;
            println!("    ✅  Generated RSS feed:\n    {}", xml);
        }
        Err(e) => {
            let error: XmlError = e; // Explicitly casting the error to quick_xml::Error
            return Err(Box::new(error)
                as Box<dyn std::error::Error + Send + Sync>);
        }
    }

    Ok(())
}

/// Demonstrates writing an XML element using the `macro_write_element!` macro.
fn write_element_macro_example(
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n🦀 Write Xml Element Macro Example");
    println!("---------------------------------------------");

    let mut writer = Writer::new(Cursor::new(Vec::new()));

    // Writing an XML element using macro
    match macro_write_element!(&mut writer, "title", "My Rust Blog") {
        Ok(_) => {
            let result = writer.into_inner().into_inner();
            let xml = String::from_utf8(result)?;
            println!("    ✅  XML Element written:\n    {}", xml);
        }
        Err(e) => {
            let error: XmlError = e; // Explicitly casting the error to quick_xml::Error
            return Err(Box::new(error)
                as Box<dyn std::error::Error + Send + Sync>);
        }
    }

    Ok(())
}
