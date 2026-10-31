// Copyright © 2024 RSS Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

// src/generator.rs

use crate::data::{RssData, RssItem, RssVersion};
use crate::error::Result;
use oxml::{
    tree::NodeId, Document, EmptyElement, Newline, SerialiseOptions,
};

const XML_DECLARATION: &str =
    r#"<?xml version="1.0" encoding="utf-8"?>"#;

/// Strips invalid XML 1.0 control characters and unescapes pre-escaped HTML/XML entities
/// so `oxml` can escape them once without double-encoding.
pub(crate) fn clean_xml_text(content: &str) -> String {
    let filtered: String = content
        .chars()
        .filter(|&c| {
            !(c.is_control() && c != '\n' && c != '\r' && c != '\t')
        })
        .collect();

    filtered
        .replace("&#x27;", "'")
        .replace("&quot;", "\"")
        .replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&amp;", "&")
}

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
    clean_xml_text(content)
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
/// * `writer` - A mutable reference to the output writer.
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
    writer: &mut W,
    name: &str,
    content: &str,
) -> Result<()> {
    let mut doc = Document::empty();
    let elem = doc.append_element(doc.root(), None, name)?;
    doc.append_text(elem, &clean_xml_text(content))?;
    let options = SerialiseOptions {
        indent: None,
        empty_elements: EmptyElement::Expanded,
        newline: Newline::Lf,
    };
    doc.write_xml_to(writer, &options)?;
    Ok(())
}

fn append_text_element(
    doc: &mut Document,
    parent: NodeId,
    name: &str,
    content: &str,
) -> Result<()> {
    let ns = doc
        .element_name(parent)
        .and_then(|n| n.namespace.as_deref())
        .map(str::to_owned);
    let elem = doc.append_element(parent, ns.as_deref(), name)?;
    doc.append_text(elem, &clean_xml_text(content))?;
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

    let mut doc = Document::empty();

    match options.version {
        RssVersion::RSS0_90 => {
            write_rss_channel_versioned(&mut doc, options, "0.90")?;
        }
        RssVersion::RSS0_91 => {
            write_rss_channel_versioned(&mut doc, options, "0.91")?;
        }
        RssVersion::RSS0_92 => {
            write_rss_channel_versioned(&mut doc, options, "0.92")?;
        }
        RssVersion::RSS1_0 => {
            write_rss_channel_1_0(&mut doc, options)?;
        }
        RssVersion::RSS2_0 => {
            write_rss_channel_2_0(&mut doc, options)?;
        }
    }

    let serialise_opts = SerialiseOptions {
        indent: None,
        empty_elements: EmptyElement::SelfClosing,
        newline: Newline::Lf,
    };
    let mut xml = String::from(XML_DECLARATION);
    let _ = doc.write_xml_with(&mut xml, serialise_opts);
    Ok(xml)
}

/// Writes an RSS 0.9x channel element and its contents.
fn write_rss_channel_versioned(
    doc: &mut Document,
    options: &RssData,
    version: &str,
) -> Result<()> {
    let rss = doc.append_element(doc.root(), None, "rss")?;
    doc.set_attribute(rss, None, "version", version)?;
    let channel = doc.append_element(rss, None, "channel")?;
    write_channel_elements(doc, channel, options)?;
    write_items(doc, channel, options)?;
    Ok(())
}

/// Writes the RSS 1.0 channel element and its contents.
fn write_rss_channel_1_0(
    doc: &mut Document,
    options: &RssData,
) -> Result<()> {
    let rdf_ns = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
    let rss1_ns = "http://purl.org/rss/1.0/";
    let rdf = doc.append_element(doc.root(), Some(rdf_ns), "RDF")?;
    doc.declare_namespace(rdf, Some("rdf"), rdf_ns)?;
    doc.declare_namespace(rdf, None, rss1_ns)?;

    let channel = doc.append_element(rdf, Some(rss1_ns), "channel")?;
    write_channel_elements(doc, channel, options)?;
    write_items(doc, channel, options)?;
    Ok(())
}

/// Writes the RSS 2.0 channel element and its contents.
fn write_rss_channel_2_0(
    doc: &mut Document,
    options: &RssData,
) -> Result<()> {
    let atom_ns = "http://www.w3.org/2005/Atom";
    let rss = doc.append_element(doc.root(), None, "rss")?;
    doc.set_attribute(rss, None, "version", "2.0")?;
    doc.declare_namespace(rss, Some("atom"), atom_ns)?;

    let channel = doc.append_element(rss, None, "channel")?;
    write_channel_elements(doc, channel, options)?;
    write_image_element(doc, channel, options)?;
    write_atom_link_element(doc, channel, options)?;
    write_items(doc, channel, options)?;
    Ok(())
}

/// Writes the channel elements to the document.
fn write_channel_elements(
    doc: &mut Document,
    channel: NodeId,
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
            append_text_element(doc, channel, name, content)?;
        }
    }

    Ok(())
}

/// Writes the image element to the channel.
fn write_image_element(
    doc: &mut Document,
    channel: NodeId,
    options: &RssData,
) -> Result<()> {
    if !options.image_url.is_empty() {
        let image = doc.append_element(channel, None, "image")?;
        append_text_element(doc, image, "url", &options.image_url)?;
        append_text_element(doc, image, "title", &options.title)?;
        append_text_element(doc, image, "link", &options.link)?;
    }
    Ok(())
}

/// Writes the item elements to the RSS feed.
fn write_items(
    doc: &mut Document,
    channel: NodeId,
    options: &RssData,
) -> Result<()> {
    for item in &options.items {
        write_item(doc, channel, item)?;
    }
    Ok(())
}

/// Writes a single item element to the RSS feed.
fn write_item(
    doc: &mut Document,
    channel: NodeId,
    item: &RssItem,
) -> Result<()> {
    let ns = doc
        .element_name(channel)
        .and_then(|n| n.namespace.as_deref())
        .map(str::to_owned);
    let item_node =
        doc.append_element(channel, ns.as_deref(), "item")?;

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
            append_text_element(doc, item_node, name, content)?;
        }
    }

    Ok(())
}

/// Writes the Atom link element to the channel.
fn write_atom_link_element(
    doc: &mut Document,
    channel: NodeId,
    options: &RssData,
) -> Result<()> {
    if !options.atom_link.is_empty() {
        let atom_ns = "http://www.w3.org/2005/Atom";
        let atom_link =
            doc.append_element(channel, Some(atom_ns), "link")?;
        doc.set_attribute(
            atom_link,
            None,
            "href",
            &clean_xml_text(&options.atom_link),
        )?;
        doc.set_attribute(atom_link, None, "rel", "self")?;
        doc.set_attribute(
            atom_link,
            None,
            "type",
            "application/rss+xml",
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_xml_element(xml: &str, element: &str, expected: &str) {
        let open_tag = format!("<{element}>");
        let close_tag = format!("</{element}>");

        if let Some(start) = xml.find(&open_tag) {
            let content_start = start + open_tag.len();
            if let Some(end) = xml[content_start..].find(&close_tag) {
                let content = &xml[content_start..content_start + end];
                assert_eq!(
                    content, expected,
                    "Element '{element}' content mismatch"
                );
            } else {
                panic!("Closing tag '{close_tag}' not found in XML");
            }
        } else {
            panic!("Element '{element}' not found in XML");
        }
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
        assert!(rss_feed.contains(r#"<rss xmlns:atom="http://www.w3.org/2005/Atom" version="2.0">"#));
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
            eprintln!("Error generating RSS: {e:?}");
        }

        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert!(rss_feed.contains(r#"<rss xmlns:atom="http://www.w3.org/2005/Atom" version="2.0">"#));
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
            "Feed with &lt;special&gt; &amp; \"characters\"",
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
                    .title(format!("Item {i}"))
                    .link(format!("https://example.com/item{i}"))
                    .description(format!("Description for item {i}"))
                    .guid(format!("https://example.com/item{i}"))
                    .pub_date(format!(
                        "Mon, 0{i} Jan 2023 00:00:00 GMT"
                    )),
            );
        }

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert_xml_element(&rss_feed, "title", "Multiple Items Feed");

        for i in 1..=3 {
            assert!(
                rss_feed.contains(&format!("<title>Item {i}</title>"))
            );
            assert!(rss_feed.contains(&format!(
                "<link>https://example.com/item{i}</link>"
            )));
            assert!(rss_feed.contains(&format!(
                "<description>Description for item {i}</description>"
            )));
            assert!(rss_feed.contains(&format!(
                "<guid>https://example.com/item{i}</guid>"
            )));
            assert!(rss_feed.contains(&format!(
                "<pubDate>Mon, 0{i} Jan 2023 00:00:00 GMT</pubDate>"
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
    fn test_sanitize_content_idempotent() {
        // Test that pre-escaped entities are handled correctly (unescape then re-escape)
        let input = "&amp; &lt; &gt; &quot; &#x27;";
        let result = sanitize_content(input);
        assert_eq!(result, "&amp; &lt; &gt; &quot; &#x27;");

        // Applying sanitize_content again should produce the same result
        let result2 = sanitize_content(&result);
        assert_eq!(result2, result);
    }

    #[test]
    fn test_sanitize_content_mixed_escaped_and_raw() {
        // Mix of already-escaped and raw special chars
        let input = "Hello &amp; <world> &quot;test&quot;";
        let result = sanitize_content(input);
        assert_eq!(
            result,
            "Hello &amp; &lt;world&gt; &quot;test&quot;"
        );
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
                .title(format!("RSS {version} Feed"))
                .link("https://example.com")
                .description(format!("RSS {version} feed description"));

            let result = generate_rss(&rss_data);
            assert!(result.is_ok());

            let rss_feed = result.unwrap();
            match version {
                RssVersion::RSS0_90 => assert!(rss_feed.contains(r#"<rss version="0.90">"#)),
                RssVersion::RSS0_91 => assert!(rss_feed.contains(r#"<rss version="0.91">"#)),
                RssVersion::RSS0_92 => assert!(rss_feed.contains(r#"<rss version="0.92">"#)),
                RssVersion::RSS1_0 => assert!(rss_feed.contains(r#"<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns="http://purl.org/rss/1.0/">"#)),
                RssVersion::RSS2_0 => assert!(rss_feed.contains(r#"<rss xmlns:atom="http://www.w3.org/2005/Atom" version="2.0">"#)),
            }
            assert_xml_element(
                &rss_feed,
                "title",
                &format!("RSS {version} Feed"),
            );
            assert_xml_element(
                &rss_feed,
                "link",
                "https://example.com",
            );
            assert_xml_element(
                &rss_feed,
                "description",
                &format!("RSS {version} feed description"),
            );
        }
    }

    #[test]
    fn test_generate_rss_2_0_without_image() {
        let rss_data = RssData::new(Some(RssVersion::RSS2_0))
            .title("No Image Feed")
            .link("https://example.com")
            .description("Feed without image");

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());
        let rss_feed = result.unwrap();
        assert!(!rss_feed.contains("<image>"));
    }

    #[test]
    fn test_generate_rss_2_0_without_atom_link() {
        let rss_data = RssData::new(Some(RssVersion::RSS2_0))
            .title("No Atom Link Feed")
            .link("https://example.com")
            .description("Feed without atom link");

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());
        let rss_feed = result.unwrap();
        assert!(!rss_feed.contains("atom:link"));
    }

    #[test]
    fn test_generate_rss_2_0_with_image() {
        let mut rss_data = RssData::new(Some(RssVersion::RSS2_0))
            .title("Feed With Image")
            .link("https://example.com")
            .description("Feed with image");
        rss_data.set_image(
            "Image Title",
            "https://example.com/image.png",
            "https://example.com",
        );

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());
        let rss_feed = result.unwrap();
        assert!(rss_feed.contains("<image>"));
        assert_xml_element(
            &rss_feed,
            "url",
            "https://example.com/image.png",
        );
    }

    #[test]
    fn test_generate_rss_2_0_with_atom_link() {
        let rss_data = RssData::new(Some(RssVersion::RSS2_0))
            .title("Feed With Atom Link")
            .link("https://example.com")
            .description("Feed with atom link")
            .atom_link("https://example.com/feed.xml");

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());
        let rss_feed = result.unwrap();
        assert!(rss_feed.contains("atom:link"));
        assert!(rss_feed.contains("https://example.com/feed.xml"));
    }

    #[test]
    fn test_sanitize_content_control_chars() {
        let input = "Hello\x00World\x01Foo\nBar\tBaz";
        let result = sanitize_content(input);
        assert!(!result.contains('\x00'));
        assert!(!result.contains('\x01'));
        assert!(result.contains('\n'));
        assert!(result.contains('\t'));
    }
}
