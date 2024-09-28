// Copyright © 2024 RSS Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # RSS Gen Error Handling Examples
//!
//! This program demonstrates the usage of various error types and functions
//! in the RSS Gen library's error module, including creating different types of errors,
//! and converting errors to HTTP status codes.

#![allow(missing_docs)]

use quick_xml::Error as XmlError;
use rss_gen::error::{Result, RssError};
use std::io;
use std::sync::Arc;

pub fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 RSS Gen Error Handling Examples\n");

    // Run various error handling examples
    missing_field_example()?;
    invalid_input_example()?;
    xml_error_example()?;
    utf8_error_example()?;
    io_error_example()?;
    validation_errors_example()?;
    date_sort_error_example()?;
    custom_error_example()?;
    http_status_example()?;

    println!(
        "\n🎉  All error handling examples completed successfully!"
    );

    Ok(())
}

/// Demonstrates creating and handling a MissingField error.
fn missing_field_example() -> Result<()> {
    println!("🦀  Missing Field Error Example");
    println!("---------------------------------------------");

    let error = RssError::missing_field("title");
    println!("    ❌  Missing Field Error: {}", error);

    match validate_rss_data() {
        Ok(_) => println!("    ✅  RSS data is valid"),
        Err(e) => println!("    ❌  Validation failed: {}", e),
    }

    Ok(())
}

/// Demonstrates creating and handling an InvalidInput error.
fn invalid_input_example() -> Result<()> {
    println!("\n🦀 Invalid Input Error Example");
    println!("---------------------------------------------");

    let error = RssError::invalid_input("Invalid date format");
    println!("    ❌  Invalid Input Error: {}", error);

    Ok(())
}

/// Demonstrates handling XML-related errors.
fn xml_error_example() -> Result<()> {
    println!("\n🦀 Xml Error Example");
    println!("---------------------------------------------");

    let xml_error = XmlError::Io(Arc::new(io::Error::new(
        io::ErrorKind::Other,
        "XML parsing failed",
    )));
    let error = RssError::XmlWriteError(xml_error);
    println!("    ❌  XML Error: {}", error);

    Ok(())
}

/// Demonstrates handling UTF-8 conversion errors.
fn utf8_error_example() -> Result<()> {
    println!("\n🦀 UTF-8 Error Example");
    println!("---------------------------------------------");

    let invalid_utf8 = vec![0, 159, 146, 150];
    let utf8_result = String::from_utf8(invalid_utf8);

    match utf8_result {
        Ok(_) => println!("    ✅  UTF-8 conversion successful"),
        Err(e) => {
            let error = RssError::Utf8Error(e);
            println!("    ❌  UTF-8 Error: {}", error);
        }
    }

    Ok(())
}

/// Demonstrates handling I/O errors.
fn io_error_example() -> Result<()> {
    println!("\n🦀  I/O Error Example");
    println!("---------------------------------------------");

    let io_error =
        io::Error::new(io::ErrorKind::NotFound, "File not found");
    let error: RssError = io_error.into();
    println!("    ❌  I/O Error: {}", error);

    Ok(())
}

/// Demonstrates handling validation errors.
fn validation_errors_example() -> Result<()> {
    println!("\n🦀 Validation Errors Example");
    println!("---------------------------------------------");

    let errors = vec![
        "Title is missing".to_string(),
        "Invalid publication date".to_string(),
    ];
    let error = RssError::ValidationErrors(errors);
    println!("    ❌  Validation Errors: {}", error);

    Ok(())
}

/// Demonstrates handling date sort errors.
fn date_sort_error_example() -> Result<()> {
    println!("\n🦀 Date Sort Error Example");
    println!("---------------------------------------------");

    let date_errors = vec![
        RssError::date_sort_error(0, "Invalid date format"),
        RssError::date_sort_error(2, "Date out of range"),
    ];
    let error = RssError::DateSortError(date_errors);
    println!("    ❌  Date Sort Error: {}", error);

    Ok(())
}

/// Demonstrates creating and handling a custom error.
fn custom_error_example() -> Result<()> {
    println!("\n🦀 Custom Error Example");
    println!("---------------------------------------------");

    let error = RssError::custom("An unexpected error occurred");
    println!("    ❌  Custom Error: {}", error);

    Ok(())
}

/// Demonstrates converting errors to HTTP status codes.
fn http_status_example() -> Result<()> {
    println!("\n🦀 Http Status Code Example");
    println!("---------------------------------------------");

    let missing_field_error = RssError::missing_field("author");
    let xml_error = RssError::XmlWriteError(XmlError::Io(Arc::new(
        io::Error::new(io::ErrorKind::Other, "XML error"),
    )));

    println!(
        "    ❌  Missing Field Error HTTP Status: {}",
        missing_field_error.to_http_status()
    );
    println!(
        "    ❌  XML Error HTTP Status: {}",
        xml_error.to_http_status()
    );

    Ok(())
}

/// Helper function to simulate RSS data validation.
fn validate_rss_data() -> Result<()> {
    Err(RssError::missing_field("title"))
}
