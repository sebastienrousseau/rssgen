// Copyright © 2024 RSS Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

// src/generator.rs

use crate::data::{RssData, RssItem, RssVersion};
use crate::error::{Result, RssError};
use quick_xml::events::{
    BytesDecl, BytesEnd, BytesStart, BytesText, Event,
};
use quick_xml::Writer;
use std::io::Cursor;

const XML_VERSION: &str = "1.0";
const XML_ENCODING: &str = "utf-8";

/// Sanitizes the content by removing invalid XML characters and escaping special characters.
///
/// # Arguments
///
/// * `content` - A string slice containing the content to be sanitized.
///
/// # Returns
///
/// A `String` with invalid XML characters removed and special characters escaped.
#[must_use]
pub fn sanitize_content(content: &str) -> String {
    content
        .chars()
        .filter(|&c| {
            !(c.is_control() && c != '\n' && c != '\r' && c != '\t') // Keep valid control characters like newlines and tabs
        })
        .collect::<String>()
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Writes an XML element with the given name and content.
///
/// # Arguments
///
/// * `writer` - A mutable reference to the XML writer.
/// * `name` - The name of the XML element.
/// * `content` - The content of the XML element.
///
/// # Returns
///
/// A `Result` indicating success or failure of the write operation.
///
/// # Errors
///
/// This function returns an `Err` if there is an issue with writing XML content.
pub fn write_element<W: std::io::Write>(
    writer: &mut Writer<W>,
    name: &str,
    content: &str,
) -> Result<()> {
    writer.write_event(Event::Start(BytesStart::new(name)))?;
    writer.write_event(Event::Text(BytesText::new(content)))?;
    writer.write_event(Event::End(BytesEnd::new(name)))?;
    Ok(())
}

/// Generates an RSS feed from the given `RssData` struct.
///
/// This function creates a complete RSS feed in XML format based on the data contained in the provided `RssData`.
/// It generates the feed according to the RSS version set in the `RssData`.
///
/// # Arguments
///
/// * `options` - A reference to a `RssData` struct containing the RSS feed data.
///
/// # Returns
///
/// * `Ok(String)` - The generated RSS feed as a string if successful.
/// * `Err(RssError)` - An error if RSS generation fails.
///
/// # Errors
///
/// This function returns an error if there are issues in validating the RSS data or writing the RSS feed.
///
/// # Example
///
/// ```
/// use rss_gen::{RssData, generate_rss, RssVersion};
///
/// let rss_data = RssData::new(Some(RssVersion::RSS2_0))
///     .title("My Blog")
///     .link("https://myblog.com")
///     .description("A blog about Rust programming");
///
/// match generate_rss(&rss_data) {
///     Ok(rss_feed) => println!("{}", rss_feed),
///     Err(e) => eprintln!("Error generating RSS: {}", e),
/// }
/// ```
pub fn generate_rss(options: &RssData) -> Result<String> {
    options.validate()?;

    let mut writer = Writer::new(Cursor::new(Vec::new()));

    write_xml_declaration(&mut writer)?;

    match options.version {
        RssVersion::RSS0_90 => {
            write_rss_channel_0_90(&mut writer, options)?;
        }
        RssVersion::RSS0_91 => {
            write_rss_channel_0_91(&mut writer, options)?;
        }
        RssVersion::RSS0_92 => {
            write_rss_channel_0_92(&mut writer, options)?;
        }
        RssVersion::RSS1_0 => {
            write_rss_channel_1_0(&mut writer, options)?;
        }
        RssVersion::RSS2_0 => {
            write_rss_channel_2_0(&mut writer, options)?;
        }
    }

    let xml = writer.into_inner().into_inner();
    String::from_utf8(xml).map_err(RssError::from)
}

/// Writes the XML declaration to the writer.
fn write_xml_declaration<W: std::io::Write>(
    writer: &mut Writer<W>,
) -> Result<()> {
    Ok(writer.write_event(Event::Decl(BytesDecl::new(
        XML_VERSION,
        Some(XML_ENCODING),
        None,
    )))?)
}

/// Writes the RSS 0.90 channel element and its contents.
fn write_rss_channel_0_90<W: std::io::Write>(
    writer: &mut Writer<W>,
    options: &RssData,
) -> Result<()> {
    let mut rss_start = BytesStart::new("rss");
    rss_start.push_attribute(("version", "0.90"));
    writer.write_event(Event::Start(rss_start))?;

    writer.write_event(Event::Start(BytesStart::new("channel")))?;

    write_channel_elements(writer, options)?;
    write_items(writer, options)?;

    writer.write_event(Event::End(BytesEnd::new("channel")))?;
    writer.write_event(Event::End(BytesEnd::new("rss")))?;

    Ok(())
}

/// Writes the RSS 0.91 channel element and its contents.
fn write_rss_channel_0_91<W: std::io::Write>(
    writer: &mut Writer<W>,
    options: &RssData,
) -> Result<()> {
    let mut rss_start = BytesStart::new("rss");
    rss_start.push_attribute(("version", "0.91"));
    writer.write_event(Event::Start(rss_start))?;

    writer.write_event(Event::Start(BytesStart::new("channel")))?;

    write_channel_elements(writer, options)?;
    write_items(writer, options)?;

    writer.write_event(Event::End(BytesEnd::new("channel")))?;
    writer.write_event(Event::End(BytesEnd::new("rss")))?;

    Ok(())
}

/// Writes the RSS 0.92 channel element and its contents.
fn write_rss_channel_0_92<W: std::io::Write>(
    writer: &mut Writer<W>,
    options: &RssData,
) -> Result<()> {
    let mut rss_start = BytesStart::new("rss");
    rss_start.push_attribute(("version", "0.92"));
    writer.write_event(Event::Start(rss_start))?;

    writer.write_event(Event::Start(BytesStart::new("channel")))?;

    write_channel_elements(writer, options)?;
    write_items(writer, options)?;

    writer.write_event(Event::End(BytesEnd::new("channel")))?;
    writer.write_event(Event::End(BytesEnd::new("rss")))?;

    Ok(())
}

/// Writes the RSS 1.0 channel element and its contents.
fn write_rss_channel_1_0<W: std::io::Write>(
    writer: &mut Writer<W>,
    options: &RssData,
) -> Result<()> {
    let mut rdf_start = BytesStart::new("rdf:RDF");
    rdf_start.push_attribute((
        "xmlns:rdf",
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
    ));
    rdf_start.push_attribute(("xmlns", "http://purl.org/rss/1.0/"));
    writer.write_event(Event::Start(rdf_start))?;

    writer.write_event(Event::Start(BytesStart::new("channel")))?;

    write_channel_elements(writer, options)?;
    write_items(writer, options)?;

    writer.write_event(Event::End(BytesEnd::new("channel")))?;
    writer.write_event(Event::End(BytesEnd::new("rdf:RDF")))?;

    Ok(())
}

/// Writes the RSS 2.0 channel element and its contents.
fn write_rss_channel_2_0<W: std::io::Write>(
    writer: &mut Writer<W>,
    options: &RssData,
) -> Result<()> {
    let mut rss_start = BytesStart::new("rss");
    rss_start.push_attribute(("version", "2.0"));
    rss_start
        .push_attribute(("xmlns:atom", "http://www.w3.org/2005/Atom"));
    writer.write_event(Event::Start(rss_start))?;

    writer.write_event(Event::Start(BytesStart::new("channel")))?;

    write_channel_elements(writer, options)?;
    write_image_element(writer, options)?;
    write_atom_link_element(writer, options)?;
    write_items(writer, options)?;

    writer.write_event(Event::End(BytesEnd::new("channel")))?;
    writer.write_event(Event::End(BytesEnd::new("rss")))?;

    Ok(())
}

/// Writes the channel elements to the writer.
fn write_channel_elements<W: std::io::Write>(
    writer: &mut Writer<W>,
    options: &RssData,
) -> Result<()> {
    let elements = [
        ("title", &options.title),
        ("link", &options.link),
        ("description", &options.description),
        ("language", &options.language),
        ("pubDate", &options.pub_date),
        ("lastBuildDate", &options.last_build_date),
        ("docs", &options.docs),
        ("generator", &options.generator),
        ("managingEditor", &options.managing_editor),
        ("webMaster", &options.webmaster),
        ("category", &options.category),
        ("ttl", &options.ttl),
    ];

    for (name, content) in &elements {
        if !content.is_empty() {
            write_element(writer, name, content)?;
        }
    }

    Ok(())
}

/// Writes the image element to the writer.
fn write_image_element<W: std::io::Write>(
    writer: &mut Writer<W>,
    options: &RssData,
) -> Result<()> {
    if !options.image_url.is_empty() {
        writer.write_event(Event::Start(BytesStart::new("image")))?;
        write_element(writer, "url", &options.image_url)?;
        write_element(writer, "title", &options.title)?;
        write_element(writer, "link", &options.link)?;
        writer.write_event(Event::End(BytesEnd::new("image")))?;
    }
    Ok(())
}

/// Writes the item elements to the RSS feed.
fn write_items<W: std::io::Write>(
    writer: &mut Writer<W>,
    options: &RssData,
) -> Result<()> {
    for item in &options.items {
        write_item(writer, item)?;
    }
    Ok(())
}

/// Writes a single item element to the RSS feed.
fn write_item<W: std::io::Write>(
    writer: &mut Writer<W>,
    item: &RssItem,
) -> Result<()> {
    writer.write_event(Event::Start(BytesStart::new("item")))?;

    let item_elements = [
        ("title", &item.title),
        ("link", &item.link),
        ("description", &item.description),
        ("guid", &item.guid),
        ("pubDate", &item.pub_date),
        ("author", &item.author),
    ];

    for (name, content) in &item_elements {
        if !content.is_empty() {
            write_element(writer, name, content)?;
        }
    }

    writer.write_event(Event::End(BytesEnd::new("item")))?;
    Ok(())
}

/// Writes the Atom link element to the writer.
fn write_atom_link_element<W: std::io::Write>(
    writer: &mut Writer<W>,
    options: &RssData,
) -> Result<()> {
    if !options.atom_link.is_empty() {
        let mut atom_link_start = BytesStart::new("atom:link");
        atom_link_start
            .push_attribute(("href", options.atom_link.as_str()));
        atom_link_start.push_attribute(("rel", "self"));
        atom_link_start.push_attribute(("type", "application/rss+xml"));
        writer.write_event(Event::Empty(atom_link_start))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::events::Event;
    use quick_xml::Reader;

    fn assert_xml_element(xml: &str, element: &str, expected: &str) {
        let mut reader = Reader::from_str(xml);
        let mut found = false;

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e))
                    if e.name().as_ref() == element.as_bytes() =>
                {
                    match reader.read_event() {
                        Ok(Event::Text(e)) => {
                            let unescaped = e.unescape().unwrap();
                            assert_eq!(unescaped, expected);
                            found = true;
                            break;
                        }
                        _ => continue,
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => panic!(
                    "Error at position {}: {:?}",
                    reader.buffer_position(),
                    e
                ),
                _ => (),
            }
        }
        assert!(
            found,
            "Element '{}' not found or doesn't match expected content",
            element
        );
    }

    #[test]
    fn test_generate_rss_minimal() {
        let rss_data = RssData::new(None)
            .title("Minimal Feed")
            .link("https://example.com")
            .description("A minimal RSS feed");

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert!(rss_feed.contains(r#"<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">"#));
        assert_xml_element(&rss_feed, "title", "Minimal Feed");
        assert_xml_element(&rss_feed, "link", "https://example.com");
        assert_xml_element(
            &rss_feed,
            "description",
            "A minimal RSS feed",
        );
    }

    #[test]
    fn test_generate_rss_full() {
        let mut rss_data = RssData::new(None)
            .title("Full Feed")
            .link("https://example.com")
            .description("A full RSS feed")
            .language("en-US")
            .pub_date("Mon, 01 Jan 2023 00:00:00 GMT")
            .last_build_date("Mon, 01 Jan 2023 00:00:00 GMT")
            .docs("https://example.com/rss/docs")
            .generator("rss-gen")
            .managing_editor("editor@example.com")
            .webmaster("webmaster@example.com")
            .category("Technology")
            .ttl("60")
            .image_url("https://example.com/image.png")
            .atom_link("https://example.com/feed.xml");

        rss_data.add_item(
            RssItem::new()
                .title("Test Item")
                .link("https://example.com/item1")
                .description("A test item")
                .guid("https://example.com/item1")
                .pub_date("Mon, 01 Jan 2023 00:00:00 GMT")
                .author("John Doe"),
        );

        let result = generate_rss(&rss_data);

        // Add this to print the error if the result is not ok
        if let Err(ref e) = result {
            eprintln!("Error generating RSS: {:?}", e);
        }

        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert!(rss_feed.contains(r#"<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">"#));
        assert_xml_element(&rss_feed, "title", "Full Feed");
        assert_xml_element(&rss_feed, "link", "https://example.com");
        assert_xml_element(&rss_feed, "description", "A full RSS feed");
        assert_xml_element(&rss_feed, "language", "en-US");
        assert_xml_element(
            &rss_feed,
            "pubDate",
            "Mon, 01 Jan 2023 00:00:00 GMT",
        );
        assert!(rss_feed.contains("<item>"));
        assert_xml_element(&rss_feed, "author", "John Doe");
        assert_xml_element(
            &rss_feed,
            "guid",
            "https://example.com/item1",
        );
    }

    #[test]
    fn test_generate_rss_empty_fields() {
        let rss_data = RssData::new(None)
            .title("Empty Fields Feed")
            .link("https://example.com")
            .description("An RSS feed with some empty fields")
            .language("")
            .pub_date("")
            .last_build_date("");

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert_xml_element(&rss_feed, "title", "Empty Fields Feed");
        assert_xml_element(&rss_feed, "link", "https://example.com");
        assert_xml_element(
            &rss_feed,
            "description",
            "An RSS feed with some empty fields",
        );
        assert!(!rss_feed.contains("<language>"));
        assert!(!rss_feed.contains("<pubDate>"));
        assert!(!rss_feed.contains("<lastBuildDate>"));
    }

    #[test]
    fn test_generate_rss_special_characters() {
        let rss_data = RssData::new(None)
            .title("Special & Characters")
            .link("https://example.com/special?param=value")
            .description("Feed with <special> & \"characters\"");

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert_xml_element(
            &rss_feed,
            "title",
            "Special &amp; Characters",
        );
        assert_xml_element(
            &rss_feed,
            "link",
            "https://example.com/special?param=value",
        );
        assert_xml_element(
            &rss_feed,
            "description",
            "Feed with &lt;special&gt; &amp; &quot;characters&quot;",
        );
    }

    #[test]
    fn test_generate_rss_multiple_items() {
        let mut rss_data = RssData::new(None)
            .title("Multiple Items Feed")
            .link("https://example.com")
            .description("An RSS feed with multiple items");

        for i in 1..=3 {
            rss_data.add_item(
                RssItem::new()
                    .title(format!("Item {}", i))
                    .link(format!("https://example.com/item{}", i))
                    .description(format!("Description for item {}", i))
                    .guid(format!("https://example.com/item{}", i))
                    .pub_date(format!(
                        "Mon, 0{} Jan 2023 00:00:00 GMT",
                        i
                    )),
            );
        }

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert_xml_element(&rss_feed, "title", "Multiple Items Feed");

        for i in 1..=3 {
            assert!(rss_feed
                .contains(&format!("<title>Item {}</title>", i)));
            assert!(rss_feed.contains(&format!(
                "<link>https://example.com/item{}</link>",
                i
            )));
            assert!(rss_feed.contains(&format!(
                "<description>Description for item {}</description>",
                i
            )));
            assert!(rss_feed.contains(&format!(
                "<guid>https://example.com/item{}</guid>",
                i
            )));
            assert!(rss_feed.contains(&format!(
                "<pubDate>Mon, 0{} Jan 2023 00:00:00 GMT</pubDate>",
                i
            )));
        }
    }

    #[test]
    fn test_generate_rss_invalid_xml_characters() {
        let rss_data = RssData::new(None)
            .title(sanitize_content("Invalid XML \u{0000} Characters"))
            .link("https://example.com")
            .description(sanitize_content(
                "Description with invalid \u{0000} characters",
            ));

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert!(!rss_feed.contains('\u{0000}')); // Ensure \u{0000} is not present in the feed
    }

    #[test]
    fn test_generate_rss_long_content() {
        let long_description = "a".repeat(10000);
        let rss_data = RssData::new(None)
            .title("Long Content Feed")
            .link("https://example.com")
            .description(&long_description);

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert_xml_element(&rss_feed, "title", "Long Content Feed");
        assert_xml_element(&rss_feed, "description", &long_description);
    }

    #[test]
    fn test_sanitize_content() {
        let input =
            "Text with \u{0000}null\u{0001} and \u{0008}backspace";
        let sanitized = sanitize_content(input);
        assert_eq!(sanitized, "Text with null and backspace");

        let input_with_newlines = "Text with \nnewlines\r\nand\ttabs";
        let sanitized_newlines = sanitize_content(input_with_newlines);
        assert_eq!(sanitized_newlines, input_with_newlines);
    }

    #[test]
    fn test_generate_rss_with_author() {
        let mut rss_data = RssData::new(None)
            .title("Feed with Author")
            .link("https://example.com")
            .description("An RSS feed with author information");

        rss_data.add_item(
            RssItem::new()
                .title("Authored Item")
                .link("https://example.com/item")
                .description("An item with an author")
                .author("John Doe"),
        );

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert!(rss_feed.contains("<author>John Doe</author>"));
    }

    #[test]
    fn test_generate_rss_different_versions() {
        let versions = vec![
            RssVersion::RSS0_90,
            RssVersion::RSS0_91,
            RssVersion::RSS0_92,
            RssVersion::RSS1_0,
            RssVersion::RSS2_0,
        ];

        for version in versions {
            let rss_data = RssData::new(Some(version))
                .title(format!("RSS {} Feed", version))
                .link("https://example.com")
                .description(format!(
                    "RSS {} feed description",
                    version
                ));

            let result = generate_rss(&rss_data);
            assert!(result.is_ok());

            let rss_feed = result.unwrap();
            match version {
                RssVersion::RSS0_90 => assert!(rss_feed.contains(r#"<rss version="0.90">"#)),
                RssVersion::RSS0_91 => assert!(rss_feed.contains(r#"<rss version="0.91">"#)),
                RssVersion::RSS0_92 => assert!(rss_feed.contains(r#"<rss version="0.92">"#)),
                RssVersion::RSS1_0 => assert!(rss_feed.contains(r#"<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns="http://purl.org/rss/1.0/">"#)),
                RssVersion::RSS2_0 => assert!(rss_feed.contains(r#"<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">"#)),
            }
            assert_xml_element(
                &rss_feed,
                "title",
                &format!("RSS {} Feed", version),
            );
            assert_xml_element(
                &rss_feed,
                "link",
                "https://example.com",
            );
            assert_xml_element(
                &rss_feed,
                "description",
                &format!("RSS {} feed description", version),
            );
        }
    }
}
