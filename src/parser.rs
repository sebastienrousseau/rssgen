// Copyright © 2024 RSS Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

// src/parser.rs

use crate::data::{RssData, RssItem, RssVersion};
use crate::error::{Result, RssError};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::str::FromStr;

/// Parses an RSS feed from XML content.
///
/// This function takes XML content as input and parses it into an `RssData` struct.
/// It supports parsing RSS versions 0.90, 0.91, 0.92, 1.0, and 2.0.
///
/// # Arguments
///
/// * `content` - A string slice containing the XML content of the RSS feed.
///
/// # Returns
///
/// * `Ok(RssData)` - The parsed RSS data if successful.
/// * `Err(RssError)` - An error if parsing fails.
///
/// # Example
///
/// ```
/// use rss_gen::parse_rss;
///
/// let xml_content = r#"
///     <?xml version="1.0" encoding="UTF-8"?>
///     <rss version="2.0">
///         <channel>
///             <title>My Blog</title>
///             <link>https://example.com</link>
///             <description>A sample blog</description>
///             <item>
///                 <title>First Post</title>
///                 <link>https://example.com/first-post</link>
///                 <description>This is my first post</description>
///             </item>
///         </channel>
///     </rss>
/// "#;
///
/// let parsed_data = parse_rss(xml_content).unwrap();
/// assert_eq!(parsed_data.title, "My Blog");
/// assert_eq!(parsed_data.items.len(), 1);
/// ```
pub fn parse_rss(content: &str) -> Result<RssData> {
    let mut reader = Reader::from_str(content);
    let mut rss_data = RssData::new(None);
    let mut buf = Vec::new();
    let mut in_channel = false;
    let mut in_item = false;
    let mut current_item = RssItem::new();
    let mut current_element = String::new();

    // Flag to determine if this is RSS 1.0 (RDF format)
    let mut is_rss_1_0 = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = e.name().0.to_vec();
                let name_str =
                    String::from_utf8_lossy(&name).into_owned();

                if name_str.is_empty() {
                    continue; // Skip if the element name is empty
                }

                // Detect RSS 1.0 (RDF-based) or RSS 2.0 based on the root element
                if name == b"rss" {
                    // RSS 2.0 feed
                    is_rss_1_0 = false;
                    if let Some(attr) = e.attributes().find(|a| {
                        a.as_ref().unwrap().key.as_ref() == b"version"
                    }) {
                        let version_str = match attr {
                            Ok(attr) => match attr.unescape_value() {
                                Ok(unescaped) => unescaped.into_owned(),
                                Err(e) => {
                                    return Err(
                                        RssError::XmlParseError(e),
                                    )
                                }
                            },
                            Err(e) => {
                                return Err(RssError::XmlParseError(
                                    quick_xml::Error::from(e),
                                ))
                            }
                        };

                        rss_data.version =
                            RssVersion::from_str(&version_str)
                                .map_err(|_| {
                                    RssError::InvalidInput(format!(
                                        "Invalid RSS version: {}",
                                        version_str
                                    ))
                                })?;
                    }
                    continue; // Skip further processing of the `rss` element
                } else if name == b"rdf:RDF" {
                    // RSS 1.0 feed (RDF format)
                    is_rss_1_0 = true;
                    continue; // Skip further processing of the `rdf:RDF` element
                }

                if name_str == "channel" {
                    in_channel = true;
                    continue; // Skip further processing for the "channel" tag itself
                } else if name_str == "item" {
                    in_item = true;
                    current_item = RssItem::new();
                }

                current_element = name_str;
            }
            Ok(Event::End(ref e)) => {
                let name = e.name().0.to_vec();
                if name == b"channel" {
                    in_channel = false;
                } else if name == b"item" {
                    in_item = false;
                    rss_data.add_item(current_item.clone());
                }

                current_element.clear();
            }
            Ok(Event::Text(ref e)) => {
                let text = e
                    .unescape()
                    .map_err(RssError::XmlParseError)?
                    .into_owned();
                if in_channel && !in_item {
                    if !current_element.is_empty() {
                        // Pass `is_rss_1_0` to handle RSS 1.0 elements
                        parse_channel_element(
                            &mut rss_data,
                            &current_element,
                            &text,
                            is_rss_1_0,
                        )?;
                    }
                } else if in_item && !current_element.is_empty() {
                    parse_item_element(
                        &mut current_item,
                        &current_element,
                        &text,
                    )?;
                }
            }
            Ok(Event::CData(ref e)) => {
                // Handle CDATA event
                let text =
                    String::from_utf8_lossy(e.as_ref()).into_owned();
                if in_channel && !in_item {
                    parse_channel_element(
                        &mut rss_data,
                        &current_element,
                        &text,
                        is_rss_1_0,
                    )?;
                } else if in_item {
                    parse_item_element(
                        &mut current_item,
                        &current_element,
                        &text,
                    )?;
                }
            }
            Ok(Event::Eof) => break Ok(rss_data),
            Err(e) => return Err(RssError::XmlParseError(e)),
            _ => (),
        }
        buf.clear();
    }
}

/// Parses a channel element and sets the corresponding field in `RssData`.
fn parse_channel_element(
    rss_data: &mut RssData,
    element: &str,
    text: &str,
    is_rss_1_0: bool,
) -> Result<()> {
    match element {
        // RSS 2.0 channel elements
        "title" => {
            rss_data.title = text.to_string();
            Ok(())
        }
        "link" => {
            rss_data.link = text.to_string();
            Ok(())
        }
        "description" => {
            rss_data.description = text.to_string();
            Ok(())
        }
        "language" => {
            rss_data.language = text.to_string();
            Ok(())
        }
        "copyright" => {
            rss_data.copyright = text.to_string();
            Ok(())
        }
        "managingEditor" => {
            rss_data.managing_editor = text.to_string();
            Ok(())
        }
        "webMaster" => {
            rss_data.webmaster = text.to_string();
            Ok(())
        }
        "pubDate" => {
            rss_data.pub_date = text.to_string();
            Ok(())
        }
        "lastBuildDate" => {
            rss_data.last_build_date = text.to_string();
            Ok(())
        }
        "category" => {
            rss_data.category = text.to_string();
            Ok(())
        }
        "generator" => {
            rss_data.generator = text.to_string();
            Ok(())
        }
        "docs" => {
            rss_data.docs = text.to_string();
            Ok(())
        }
        "ttl" => {
            rss_data.ttl = text.to_string();
            Ok(())
        }

        // RSS 1.0 specific elements (RDF-based format)
        "items" => {
            if is_rss_1_0 {
                // This indicates we're entering the `items` block for RSS 1.0.
                Ok(())
            } else {
                Err(RssError::UnknownElement(format!(
                    "Unexpected element: {}",
                    element
                )))
            }
        }
        "rdf:Seq" => {
            if is_rss_1_0 {
                // This block contains `rdf:li` elements, which refer to items.
                Ok(())
            } else {
                Err(RssError::UnknownElement(format!(
                    "Unexpected element: {}",
                    element
                )))
            }
        }
        "rdf:li" => {
            if is_rss_1_0 {
                // Handle RSS 1.0 item references in rdf:li
                if let Some(link) = extract_rdf_li_resource(text)? {
                    let mut item = RssItem::new();
                    item.link = link;
                    rss_data.add_item(item);
                }
                Ok(())
            } else {
                Err(RssError::UnknownElement(format!(
                    "Unexpected element: {}",
                    element
                )))
            }
        }
        _ => Err(RssError::UnknownElement(format!(
            "Unknown channel element: {}",
            element
        ))),
    }
}

/// Helper function to extract the `rdf:resource` attribute from `rdf:li` elements.
fn extract_rdf_li_resource(text: &str) -> Result<Option<String>> {
    if text.contains("rdf:resource") {
        let resource_start = text.find("rdf:resource=\"").ok_or(
            RssError::InvalidInput("Missing rdf:resource".to_string()),
        )?;
        let resource_end = text[resource_start..].find('"').ok_or(
            RssError::InvalidInput(
                "Malformed rdf:resource".to_string(),
            ),
        )?;
        let link = &text
            [(resource_start + 14)..(resource_start + resource_end)];
        Ok(Some(link.to_string()))
    } else {
        Ok(None)
    }
}

/// Parses an item element and sets the corresponding field in `RssItem`.
fn parse_item_element(
    item: &mut RssItem,
    element: &str,
    text: &str,
) -> Result<()> {
    match element {
        "title" => {
            item.title = text.to_string();
            Ok(())
        }
        "link" => {
            item.link = text.to_string();
            Ok(())
        }
        "description" => {
            item.description = text.to_string();
            Ok(())
        }
        "author" => {
            item.author = text.to_string();
            Ok(())
        }
        "guid" => {
            item.guid = text.to_string();
            Ok(())
        }
        "pubDate" => {
            item.pub_date = text.to_string();
            Ok(())
        }
        "category" => {
            item.category = Some(text.to_string());
            Ok(())
        }
        "comments" => {
            item.comments = Some(text.to_string());
            Ok(())
        }
        "enclosure" => {
            item.enclosure = Some(text.to_string());
            Ok(())
        }
        "source" => {
            item.source = Some(text.to_string());
            Ok(())
        }
        // If the item has an unknown or unexpected element, we can just skip it to avoid erroring out.
        _ => {
            // Log or handle any unrecognized elements without returning an error.
            // We can choose to either log or ignore unrecognized fields.
            Ok(()) // Continue parsing without raising an error for unknown elements
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rss_2_0() {
        let xml_content = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0">
          <channel>
            <title>My RSS Feed</title>
            <link>https://example.com</link>
            <description>A sample RSS feed</description>
            <language>en-us</language>
            <item>
              <title>First Post</title>
              <link>https://example.com/first-post</link>
              <description>This is my first post</description>
              <guid>https://example.com/first-post</guid>
              <pubDate>Mon, 01 Jan 2023 00:00:00 GMT</pubDate>
            </item>
          </channel>
        </rss>
        "#;

        let rss_data = parse_rss(xml_content).unwrap();
        assert_eq!(rss_data.version, RssVersion::RSS2_0);
        assert_eq!(rss_data.title, "My RSS Feed");
        assert_eq!(rss_data.link, "https://example.com");
        assert_eq!(rss_data.description, "A sample RSS feed");
        assert_eq!(rss_data.language, "en-us");
        assert_eq!(rss_data.items.len(), 1);

        let item = &rss_data.items[0];
        assert_eq!(item.title, "First Post");
        assert_eq!(item.link, "https://example.com/first-post");
        assert_eq!(item.description, "This is my first post");
        assert_eq!(item.guid, "https://example.com/first-post");
        assert_eq!(item.pub_date, "Mon, 01 Jan 2023 00:00:00 GMT");
    }

    #[test]
    fn test_parse_rss_1_0() {
        let xml_content = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns="http://purl.org/rss/1.0/">
          <channel rdf:about="https://example.com/rss">
            <title>My RSS 1.0 Feed</title>
            <link>https://example.com</link>
            <description>A sample RSS 1.0 feed</description>
            <items>
              <rdf:Seq>
                <rdf:li rdf:resource="https://example.com/first-post" />
              </rdf:Seq>
            </items>
          </channel>
          <item rdf:about="https://example.com/first-post">
            <title>First Post</title>
            <link>https://example.com/first-post</link>
            <description>This is my first post in RSS 1.0</description>
          </item>
        </rdf:RDF>
        "#;

        let rss_data = parse_rss(xml_content).unwrap();
        assert_eq!(rss_data.title, "My RSS 1.0 Feed");
        assert_eq!(rss_data.link, "https://example.com");
        assert_eq!(rss_data.description, "A sample RSS 1.0 feed");
        assert_eq!(rss_data.items.len(), 1);

        let item = &rss_data.items[0];
        assert_eq!(item.title, "First Post");
        assert_eq!(item.link, "https://example.com/first-post");
        assert_eq!(
            item.description,
            "This is my first post in RSS 1.0"
        );
    }

    #[test]
    fn test_parse_invalid_xml() {
        let invalid_xml = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0">
          <channel>
            <title>Invalid Feed</title>
            <link>https://example.com</link>
            <description>This XML is invalid
          </channel>
        </rss>
        "#;

        assert!(parse_rss(invalid_xml).is_err());
    }

    #[test]
    fn test_parse_unknown_version() {
        let unknown_version_xml = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <rss version="3.0">
          <channel>
            <title>Unknown Version Feed</title>
            <link>https://example.com</link>
            <description>This feed has an unknown version</description>
          </channel>
        </rss>
        "#;

        assert!(parse_rss(unknown_version_xml).is_err());
    }

    #[test]
    fn test_parse_rss_with_multiple_items() {
        let xml_content = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0">
          <channel>
            <title>Multiple Items Feed</title>
            <link>https://example.com</link>
            <description>A feed with multiple items</description>
            <item>
              <title>First Post</title>
              <link>https://example.com/first-post</link>
              <description>This is my first post</description>
            </item>
            <item>
              <title>Second Post</title>
              <link>https://example.com/second-post</link>
              <description>This is my second post</description>
            </item>
          </channel>
        </rss>
        "#;

        let rss_data = parse_rss(xml_content).unwrap();
        assert_eq!(rss_data.title, "Multiple Items Feed");
        assert_eq!(rss_data.items.len(), 2);
        assert_eq!(rss_data.items[0].title, "First Post");
        assert_eq!(rss_data.items[1].title, "Second Post");
    }

    #[test]
    fn test_parse_rss_with_cdata() {
        let xml_content = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0">
          <channel>
            <title>CDATA Feed</title>
            <link>https://example.com</link>
            <description><![CDATA[A feed with <strong>CDATA</strong> content]]></description>
            <item>
              <title>CDATA Post</title>
              <link>https://example.com/cdata-post</link>
              <description><![CDATA[This post contains <em>emphasized</em> text]]></description>
            </item>
          </channel>
        </rss>
        "#;

        let rss_data = parse_rss(xml_content).unwrap();
        assert_eq!(rss_data.title, "CDATA Feed");
        assert_eq!(
            rss_data.description,
            "A feed with <strong>CDATA</strong> content"
        );
        assert_eq!(
            rss_data.items[0].description,
            "This post contains <em>emphasized</em> text"
        );
    }
}
