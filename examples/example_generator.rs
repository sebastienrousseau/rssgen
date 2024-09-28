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

/// Entry point for the RSS Generator library examples.
///
/// This function demonstrates content sanitization, XML element writing,
/// and basic RSS generation.
pub(crate) fn main() {
    println!("\n🧪 RSS Generator Library Usage Examples\n");

    // Running the examples
    sanitize_content_example();
    write_element_example();

    println!("\n🎉  All examples completed successfully!\n");
}

/// Demonstrates content sanitization by removing invalid XML characters and escaping special characters.
fn sanitize_content_example() {
    println!("🦀 Content Sanitization Example");
    println!("---------------------------------------------");

    let content = "This is a test <content> with invalid & characters";
    let sanitized = sanitize_content(content);

    println!("    ✅  Original content: {}", content);
    println!("    ✅  Sanitized content: {}", sanitized);
}

/// Demonstrates writing an XML element using the `quick_xml` crate.
///
/// This function writes an XML element and prints its representation.
fn write_element_example() {
    println!("\n🦀 Write Xml Element Example");
    println!("---------------------------------------------");

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let element_name = "example";
    let content = "This is an example content";

    // Writing the XML element
    match write_element(&mut writer, element_name, content) {
        Ok(_) => {
            let result = writer.into_inner().into_inner();
            let xml = String::from_utf8(result).unwrap();
            println!("    ✅  XML Element written:\n    {}", xml);
        }
        Err(e) => println!("    ❌  Failed to write element: {}", e),
    }
}
