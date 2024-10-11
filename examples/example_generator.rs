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
use rss_gen::{generate_rss, RssData, RssItem, RssVersion};
use std::error::Error;
use std::io::Cursor;

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
    generate_rss_0_90_example()?;
    generate_rss_0_91_example()?;
    generate_rss_0_92_example()?;
    generate_rss_1_0_example()?;
    generate_rss_2_0_example()?;

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
        .description(
            "A complex RSS feed with multiple items and custom fields",
        )
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
            message: format!(
                "Failed to generate complex RSS feed: {}",
                e
            ),
        }) as Box<dyn Error>
    })?;

    println!("    ✅  Generated complex RSS feed:\n    {}", rss_feed);

    Ok(())
}

fn generate_rss_0_90_example() -> Result<(), Box<dyn Error>> {
    println!("\n🦀 Generating RSS 0.90 Example");
    let mut rss_data = RssData::new(Some(RssVersion::RSS0_90))
        .title("Mozilla Dot Org")
        .link("http://www.mozilla.org")
        .description("the Mozilla Organization web site")
        .image_title("Mozilla")
        .image_url("http://www.mozilla.org/images/moz.gif")
        .image_link("http://www.mozilla.org");

    let items = vec![
        ("New Status Updates", "http://www.mozilla.org/status/"),
        ("Bugzilla Reorganized", "http://www.mozilla.org/bugs/"),
        ("Mozilla Party, 2.0!", "http://www.mozilla.org/party/1999/"),
        (
            "Unix Platform Parity",
            "http://www.mozilla.org/build/unix.html",
        ),
        (
            "NPL 1.0M published",
            "http://www.mozilla.org/NPL/NPL-1.0M.html",
        ),
    ];

    for (title, link) in items {
        rss_data.add_item(RssItem::new().title(title).link(link));
    }

    let rss_feed = generate_rss(&rss_data)?;
    println!("Generated RSS 0.90 feed:\n{}", rss_feed);
    Ok(())
}

fn generate_rss_0_91_example() -> Result<(), Box<dyn Error>> {
    println!("\n🦀 Generating RSS 0.91 Example");
    let mut rss_data = RssData::new(Some(RssVersion::RSS0_91))
        .title("XML.com")
        .link("http://www.xml.com/")
        .description("XML.com features a rich mix of information and services for the XML community.")
        .language("en-us");

    let items = vec![
        ("Normalizing XML, Part 2", "http://www.xml.com/pub/a/2002/12/04/normalizing.html", "In this second and final look at applying relational normalization techniques to W3C XML Schema data modeling, Will Provost discusses when not to normalize, the scope of uniqueness and the fourth and fifth normal forms."),
        ("The .NET Schema Object Model", "http://www.xml.com/pub/a/2002/12/04/som.html", "Priya Lakshminarayanan describes in detail the use of the .NET Schema Object Model for programmatic manipulation of W3C XML Schemas."),
        ("SVG's Past and Promising Future", "http://www.xml.com/pub/a/2002/12/04/svg.html", "In this month's SVG column, Antoine Quint looks back at SVG's journey through 2002 and looks forward to 2003."),
    ];

    for (title, link, description) in items {
        rss_data.add_item(
            RssItem::new()
                .title(title)
                .link(link)
                .description(description),
        );
    }

    let rss_feed = generate_rss(&rss_data)?;
    println!("Generated RSS 0.91 feed:\n{}", rss_feed);
    Ok(())
}

fn generate_rss_0_92_example() -> Result<(), Box<dyn Error>> {
    println!("\n🦀 Generating RSS 0.92 Example");
    let mut rss_data = RssData::new(Some(RssVersion::RSS0_92))
        .title("My Website")
        .link("http://www.example.com/")
        .description("News and updates from my website.")
        .language("en-us")
        .last_build_date("Mon, 11 Oct 2024 21:57:00 GMT")
        .image_title("My Website Image")
        .image_url("http://www.example.com/image.jpg")
        .image_link("http://www.example.com/");

    let items = vec![
        (
            "First article title",
            "http://www.example.com/article1",
            "Short description of the article.",
            "Mon, 11 Oct 2024 12:00:00 GMT",
        ),
        (
            "Second article title",
            "http://www.example.com/article2",
            "Short description of the article.",
            "Sun, 10 Oct 2024 12:00:00 GMT",
        ),
    ];

    for (title, link, description, pub_date) in items {
        rss_data.add_item(
            RssItem::new()
                .title(title)
                .link(link)
                .description(description)
                .pub_date(pub_date),
        );
    }

    let rss_feed = generate_rss(&rss_data)?;
    println!("Generated RSS 0.92 feed:\n{}", rss_feed);
    Ok(())
}

fn generate_rss_1_0_example() -> Result<(), Box<dyn Error>> {
    println!("\n🦀 Generating RSS 1.0 Example");
    let mut rss_data = RssData::new(Some(RssVersion::RSS1_0))
        .title("XML.com")
        .link("http://www.xml.com/")
        .description("XML.com features a rich mix of information and services for the XML community.")
        .language("en-us");

    let items = vec![
        ("Normalizing XML, Part 2", "http://www.xml.com/pub/a/2002/12/04/normalizing.html", "In this second and final look at applying relational normalization techniques to W3C XML Schema data modeling, Will Provost discusses when not to normalize, the scope of uniqueness and the fourth and fifth normal forms.", "Will Provost", "2002-12-04"),
        ("The .NET Schema Object Model", "http://www.xml.com/pub/a/2002/12/04/som.html", "Priya Lakshminarayanan describes in detail the use of the .NET Schema Object Model for programmatic manipulation of W3C XML Schemas.", "Priya Lakshminarayanan", "2002-12-04"),
        ("SVG's Past and Promising Future", "http://www.xml.com/pub/a/2002/12/04/svg.html", "In this month's SVG column, Antoine Quint looks back at SVG's journey through 2002 and looks forward to 2003.", "Antoine Quint", "2002-12-04"),
    ];

    for (title, link, description, author, date) in items {
        rss_data.add_item(
            RssItem::new()
                .title(title)
                .link(link)
                .description(description)
                .author(author)
                .pub_date(date),
        );
    }

    let rss_feed = generate_rss(&rss_data)?;
    println!("Generated RSS 1.0 feed:\n{}", rss_feed);
    Ok(())
}

fn generate_rss_2_0_example() -> Result<(), Box<dyn Error>> {
    println!("\n🦀 Generating RSS 2.0 Example");
    let mut rss_data = RssData::new(Some(RssVersion::RSS2_0))
        .title("XML.com")
        .link("http://www.xml.com/")
        .description("XML.com features a rich mix of information and services for the XML community.")
        .language("en-us");

    let items = vec![
        ("Normalizing XML, Part 2", "http://www.xml.com/pub/a/2002/12/04/normalizing.html", "In this second and final look at applying relational normalization techniques to W3C XML Schema data modeling, Will Provost discusses when not to normalize, the scope of uniqueness and the fourth and fifth normal forms.", "Will Provost", "2002-12-04"),
        ("The .NET Schema Object Model", "http://www.xml.com/pub/a/2002/12/04/som.html", "Priya Lakshminarayanan describes in detail the use of the .NET Schema Object Model for programmatic manipulation of W3C XML Schemas.", "Priya Lakshminarayanan", "2002-12-04"),
        ("SVG's Past and Promising Future", "http://www.xml.com/pub/a/2002/12/04/svg.html", "In this month's SVG column, Antoine Quint looks back at SVG's journey through 2002 and looks forward to 2003.", "Antoine Quint", "2002-12-04"),
    ];

    for (title, link, description, author, date) in items {
        rss_data.add_item(
            RssItem::new()
                .title(title)
                .link(link)
                .description(description)
                .author(author)
                .pub_date(date),
        );
    }

    let rss_feed = generate_rss(&rss_data)?;
    println!("Generated RSS 2.0 feed:\n{}", rss_feed);
    Ok(())
}
