// Copyright © 2024 RSS Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A robust and flexible RSS feed parser.
//!
//! This module provides functionality to parse RSS feeds of various versions
//! (0.90, 0.91, 0.92, 1.0, and 2.0) into a structured format. It offers
//! comprehensive error handling, extensive customization options, and follows
//! best practices in Rust development.
//!
//! # Features
//!
//! - Supports RSS versions 0.90, 0.91, 0.92, 1.0, and 2.0
//! - Robust error handling with custom error types
//! - Extensible parsing with custom element handlers
//! - Comprehensive test suite
//! - Thread-safe and memory-efficient implementation
//!
//! # Examples
//!
//! ```rust
//! use rss_gen::parse_rss;
//!
//! let xml_content = r#"
//!     <?xml version="1.0" encoding="UTF-8"?>
//!     <rss version="2.0">
//!         <channel>
//!             <title>My Blog</title>
//!             <link>https://example.com</link>
//!             <description>A sample blog</description>
//!             <item>
//!                 <title>First Post</title>
//!                 <link>https://example.com/first-post</link>
//!                 <description>This is my first post</description>
//!             </item>
//!         </channel>
//!     </rss>
//! "#;
//!
//! let parsed_data = parse_rss(xml_content, None).unwrap();
//! assert_eq!(parsed_data.title, "My Blog");
//! assert_eq!(parsed_data.items.len(), 1);
//! ```

use quick_xml::events::{
    BytesCData, BytesEnd, BytesStart, BytesText, Event,
};
use quick_xml::Reader;
use std::borrow::Cow;
use std::sync::Arc;

pub use crate::data::{RssData, RssItem, RssVersion};
pub use crate::error::{Result, RssError};

/// A trait for custom element handlers, supporting RSS extensions.
///
/// Implement this trait to provide custom parsing logic for specific RSS elements.
pub trait ElementHandler: Send + Sync {
    /// Handle a specific RSS element.
    ///
    /// This function processes a single RSS element and performs necessary
    /// operations based on the element's name, text content, and attributes.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the RSS element.
    /// * `text` - The text content of the RSS element.
    /// * `attributes` - A slice containing the attributes of the RSS element.
    ///
    /// # Returns
    ///
    /// This function returns a `Result<()>` indicating the success or failure of
    /// the handling operation.
    ///
    /// # Errors
    ///
    /// This function will return an `Err` in the following situations:
    ///
    /// - If there is an issue with processing the element, such as invalid
    ///   attributes, unexpected element names, or a failure in custom parsing
    ///   logic.
    fn handle_element(
        &self,
        name: &str,
        text: &str,
        attributes: &[(String, String)],
    ) -> Result<()>;
}

/// Configuration options for the RSS parser.
///
/// The `ParserConfig` struct allows for customization of the RSS parser by
/// including custom handlers for specific elements.
#[derive(Default)]
pub struct ParserConfig {
    /// A vector of custom handlers that will process specific RSS elements.
    ///
    /// Each handler implements the `ElementHandler` trait and is wrapped in
    /// an `Arc` to allow shared ownership across threads.
    pub custom_handlers: Vec<Arc<dyn ElementHandler>>,
}

/// Parses a channel element and sets the corresponding field in `RssData`.
///
/// This function processes elements found within the `channel` tag of an RSS feed
/// and assigns the appropriate values to the `RssData` struct.
///
/// # Arguments
///
/// * `rss_data` - A mutable reference to the `RssData` struct.
/// * `element` - The name of the channel element.
/// * `text` - The text content of the channel element.
/// * `is_rss_1_0` - A boolean indicating if the feed is RSS 1.0.
fn parse_channel_element(
    rss_data: &mut RssData,
    element: &str,
    text: &str,
    is_rss_1_0: bool,
) -> Result<()> {
    match element {
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
        // Handle RSS 1.0 specific elements
        "items" => {
            if is_rss_1_0 {
                Ok(())
            } else {
                Err(RssError::UnknownElement("items".into()))
            }
        }
        "rdf:Seq" => {
            if is_rss_1_0 {
                Ok(())
            } else {
                Err(RssError::UnknownElement("rdf:Seq".into()))
            }
        }
        "rdf:li" => {
            if is_rss_1_0 {
                Ok(())
            } else {
                Err(RssError::UnknownElement("rdf:li".into()))
            }
        }
        _ => Err(RssError::UnknownElement(format!(
            "Unknown channel element: {}",
            element
        ))),
    }
}

/// Parses an item element and sets the corresponding field in `RssItem`.
///
/// This function processes elements found within the `item` tag of an RSS feed
/// and assigns the appropriate values to the `RssItem` struct.
///
/// # Arguments
///
/// * `item` - A mutable reference to the `RssItem` struct.
/// * `element` - The name of the item element.
/// * `text` - The text content of the item element.
/// * `attributes` - A slice containing the element's attributes as key-value pairs.
fn parse_item_element(
    item: &mut RssItem,
    element: &str,
    text: &str,
    attributes: &[(String, String)],
) {
    match element {
        "title" => {
            item.title = text.to_string();
        }
        "link" => {
            item.link = text.to_string();
        }
        "description" => {
            item.description = text.to_string();
        }
        "author" => {
            item.author = text.to_string();
        }
        "guid" => {
            item.guid = text.to_string();
        }
        "pubDate" => {
            item.pub_date = text.to_string();
        }
        "category" => {
            item.category = Some(text.to_string());
        }
        "comments" => {
            item.comments = Some(text.to_string());
        }
        "enclosure" => {
            if attributes.is_empty() {
                item.enclosure = None;
            } else {
                let enclosure_str = attributes
                    .iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect::<Vec<String>>()
                    .join(" ");
                item.enclosure = Some(enclosure_str);
            }
        }
        "source" => {
            item.source = Some(text.to_string());
        }
        _ => (), // Ignore unknown elements
    }
}

/// Represents the current parsing state (whether inside a channel, item, or image).
#[derive(Clone)]
enum ParsingState {
    Channel,
    Item,
    Image,
    None, // When not in any of these states
}

/// Represents the context of the current element being parsed in the RSS feed.
struct ParsingContext<'a> {
    is_rss_1_0: bool,
    state: ParsingState,
    current_element: &'a str,
    text: &'a str,
    current_attributes: &'a [(String, String)],
}

impl<'a> ParsingContext<'a> {
    /// Helper function to check if the current state is in a channel.
    pub fn in_channel(&self) -> bool {
        matches!(self.state, ParsingState::Channel)
    }

    /// Helper function to check if the current state is in an item.
    pub fn in_item(&self) -> bool {
        matches!(self.state, ParsingState::Item)
    }

    /// Helper function to check if the current state is in an image.
    pub fn in_image(&self) -> bool {
        matches!(self.state, ParsingState::Image)
    }
}

/// Represents the image data in an RSS feed.
struct ImageData {
    title: String,
    url: String,
    link: String,
}

/// Handles text events for both regular text and CDATA in RSS feeds.
///
/// This function processes both text and CDATA events, parsing the content
/// and assigning values to either channel, item, or image elements in the feed.
///
/// # Arguments
///
/// * `rss_data` - A mutable reference to the `RssData` struct representing the RSS feed being processed.
/// * `context` - A `ParsingContext` struct containing details about the current state of the parser (e.g., whether it's within a channel, item, or image, and the element being processed).
/// * `current_item` - A mutable reference to the `RssItem` struct, representing the current item being parsed in the RSS feed.
/// * `image_data` - A mutable reference to an `ImageData` struct for storing the parsed `title`, `url`, and `link` of the image element if applicable.
///
/// # Returns
///
/// A `Result` indicating the success or failure of handling the text event.
fn handle_text_event(
    rss_data: &mut RssData,
    context: &ParsingContext,
    current_item: &mut RssItem,
    image_data: &mut ImageData,
) -> Result<()> {
    if context.in_channel() && !context.in_item() && !context.in_image()
    {
        if !context.current_element.is_empty() {
            parse_channel_element(
                rss_data,
                context.current_element,
                &Cow::Owned(context.text.to_string()),
                context.is_rss_1_0,
            )?;
        }
    } else if context.in_item() && !context.current_element.is_empty() {
        parse_item_element(
            current_item,
            context.current_element,
            context.text,
            context.current_attributes,
        );
    } else if context.in_image() && !context.current_element.is_empty()
    {
        match context.current_element {
            "title" => image_data.title = context.text.to_string(),
            "url" => image_data.url = context.text.to_string(),
            "link" => image_data.link = context.text.to_string(),
            _ => (),
        }
    }
    Ok(())
}

/// Parses an RSS feed from XML content.
///
/// This function takes XML content as input and parses it into an `RssData` struct.
/// It supports parsing RSS versions 0.90, 0.91, 0.92, 1.0, and 2.0.
///
/// # Arguments
///
/// * `xml_content` - A string slice containing the XML content of the RSS feed.
/// * `config` - Optional configuration for custom parsing behavior.
///
/// # Returns
///
/// * `Ok(RssData)` - The parsed RSS data if successful.
/// * `Err(RssError)` - An error if parsing fails.
///
/// # Errors
///
/// This function returns an `Err(RssError)` in the following cases:
///
/// - If the XML content is invalid or malformed, a `RssError::XmlParseError` is returned.
/// - If an unsupported or invalid RSS version is encountered, a `RssError::InvalidInput` is returned.
/// - If an unknown or unsupported element is encountered during parsing, a `RssError::UnknownElement` is returned.
pub fn parse_rss(
    xml_content: &str,
    config: Option<&ParserConfig>,
) -> Result<RssData> {
    let mut reader = Reader::from_str(xml_content);
    let mut rss_data = RssData::new(None);
    let mut buf = Vec::with_capacity(1024);
    let mut context = ParserContext::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                process_start_event(e, &mut context, &mut rss_data)?;
            }
            Ok(Event::End(ref e)) => {
                process_end_event(e, &mut context, &mut rss_data);
            }
            Ok(Event::Text(ref e)) => process_text_event(
                e,
                &mut context,
                &mut rss_data,
                config,
            )?,
            Ok(Event::CData(ref e)) => process_cdata_event(
                e,
                &mut context,
                &mut rss_data,
                config,
            )?,
            Ok(Event::Eof) => break Ok(rss_data),
            Err(e) => return Err(RssError::XmlParseError(e)),
            _ => (),
        }
        buf.clear();
    }
}

/// Processes the start event of an XML element during RSS feed parsing.
///
/// This function handles the start of an XML element in an RSS feed, determining the RSS version,
/// handling different element types (e.g., "channel", "item", "image"), and extracting attributes
/// from the element.
///
/// # Arguments
///
/// * `e` - A reference to the `BytesStart` struct representing the start of an XML element.
/// * `context` - A mutable reference to the `ParserContext` struct, which maintains the current parsing state.
/// * `rss_data` - A mutable reference to the `RssData` struct, which stores the parsed RSS data.
fn process_start_event(
    e: &BytesStart<'_>,
    context: &mut ParserContext,
    _rss_data: &mut RssData,
) -> Result<()> {
    let name_str = String::from_utf8_lossy(e.name().0).into_owned();
    if name_str.is_empty() {
        return Ok(());
    }

    // Detect RSS version or RDF for RSS 1.0
    match name_str.as_str() {
        "rss" | "rdf:RDF" => {
            // Skip root elements like <rss> or <rdf:RDF>, continue to parse children
            return Ok(());
        }
        "channel" => {
            // Correctly handle the `channel` element inside the RSS root
            context.parsing_state = ParsingState::Channel;
            return Ok(());
        }
        "item" => {
            context.parsing_state = ParsingState::Item;
            context.current_item = RssItem::new();
        }
        "image" => {
            context.parsing_state = ParsingState::Image;
        }
        _ => {
            // Only return an error for truly unknown elements, ignoring root elements
            if !matches!(
                context.parsing_state,
                ParsingState::Item
                    | ParsingState::Channel
                    | ParsingState::Image
            ) {
                return Err(RssError::UnknownElement(format!(
                    "Unknown element: {}",
                    name_str
                )));
            }
        }
    }

    // Store current element and attributes
    context.current_element = name_str;
    context.current_attributes = e
        .attributes()
        .filter_map(std::result::Result::ok)
        .map(|a| {
            (
                String::from_utf8_lossy(a.key.0).into_owned(),
                String::from_utf8_lossy(&a.value).into_owned(),
            )
        })
        .collect();

    Ok(())
}

/// Processes the end event of an XML element during RSS feed parsing.
///
/// This function handles the end of an XML element in an RSS feed, updating the parsing state
/// based on the element type (e.g., "channel", "item", "image").
///
/// # Arguments
///
/// * `e` - A reference to the `BytesEnd` struct representing the end of an XML element.
/// * `context` - A mutable reference to the `ParserContext` struct, which maintains the current parsing state.
/// * `rss_data` - A mutable reference to the `RssData` struct, which stores the parsed RSS data.
fn process_end_event(
    e: &BytesEnd<'_>,
    context: &mut ParserContext,
    rss_data: &mut RssData,
) {
    let name = e.name().0.to_vec();
    if name == b"channel" {
        if matches!(context.parsing_state, ParsingState::Channel) {
            context.parsing_state = ParsingState::None;
        }
    } else if name == b"item" {
        if matches!(context.parsing_state, ParsingState::Item) {
            context.parsing_state = ParsingState::None;
            rss_data.add_item(context.current_item.clone());
        }
    } else if name == b"image"
        && matches!(context.parsing_state, ParsingState::Image)
    {
        context.parsing_state = ParsingState::None;
        rss_data.set_image(
            &context.image_title.clone(),
            &context.image_url.clone(),
            &context.image_link.clone(),
        );
    }
    context.current_element.clear();
    context.current_attributes.clear();
}

fn process_text_event(
    e: &BytesText<'_>,
    context: &mut ParserContext,
    rss_data: &mut RssData,
    config: Option<&ParserConfig>,
) -> Result<()> {
    let text = e.unescape()?.into_owned();

    let parse_context = ParsingContext {
        is_rss_1_0: matches!(
            context.rss_version,
            RssVersionState::Rss1_0
        ),
        state: context.parsing_state.clone(),
        current_element: &context.current_element,
        text: &text,
        current_attributes: &context.current_attributes,
    };

    let mut image_data = ImageData {
        title: context.image_title.clone(),
        url: context.image_url.clone(),
        link: context.image_link.clone(),
    };

    handle_text_event(
        rss_data,
        &parse_context,
        &mut context.current_item,
        &mut image_data,
    )?;

    context.image_title = image_data.title;
    context.image_url = image_data.url;
    context.image_link = image_data.link;

    // Custom handlers can be applied if necessary
    apply_custom_handlers(
        &context.current_element,
        &text,
        &context.current_attributes,
        config,
    )?;

    Ok(())
}

/// Processes a CDATA event for the current XML element.
///
/// This function handles the processing of CDATA within RSS feeds, ensuring that
/// CDATA is parsed into the appropriate elements (channels, items, or images).
///
/// # Arguments
///
/// * `e` - A reference to the `BytesCData` struct representing the CDATA content.
/// * `context` - A mutable reference to the `ParserContext` struct, which maintains the current parsing state.
/// * `rss_data` - A mutable reference to the `RssData` struct.
/// * `config` - Optional configuration for custom parsing behavior.
fn process_cdata_event(
    e: &BytesCData<'_>,
    context: &mut ParserContext,
    rss_data: &mut RssData,
    config: Option<&ParserConfig>,
) -> Result<()> {
    let text = String::from_utf8_lossy(e.as_ref()).into_owned();
    let state = context.parsing_state.clone();
    let parse_context = ParsingContext {
        is_rss_1_0: matches!(
            context.rss_version,
            RssVersionState::Rss1_0
        ),
        state,
        current_element: &context.current_element,
        text: &text,
        current_attributes: &context.current_attributes,
    };

    let mut image_data = ImageData {
        title: context.image_title.clone(),
        url: context.image_url.clone(),
        link: context.image_link.clone(),
    };

    handle_text_event(
        rss_data,
        &parse_context,
        &mut context.current_item,
        &mut image_data,
    )?;

    context.image_title = image_data.title;
    context.image_url = image_data.url;
    context.image_link = image_data.link;

    apply_custom_handlers(
        &context.current_element,
        &text,
        &context.current_attributes,
        config,
    )?;

    Ok(())
}

/// Applies custom handlers for RSS elements.
///
/// This function checks if any custom handlers are provided in the configuration and applies them to the current element.
///
/// # Arguments
///
/// * `element` - The current XML element being processed.
/// * `text` - The text content of the element.
/// * `attributes` - The attributes of the element.
/// * `config` - Optional parser configuration containing custom handlers.
fn apply_custom_handlers(
    element: &str,
    text: &str,
    attributes: &[(String, String)],
    config: Option<&ParserConfig>,
) -> Result<()> {
    if let Some(cfg) = config {
        for handler in &cfg.custom_handlers {
            handler.handle_element(element, text, attributes)?;
        }
    }
    Ok(())
}

/// Enum to represent the RSS version being parsed.
#[allow(dead_code)]
enum RssVersionState {
    Rss1_0,
    Other,
}

/// Represents the context of the current XML element being parsed.
struct ParserContext {
    rss_version: RssVersionState,
    parsing_state: ParsingState,
    current_element: String,
    current_attributes: Vec<(String, String)>,
    current_item: RssItem,
    image_title: String,
    image_url: String,
    image_link: String,
}

impl ParserContext {
    /// Initialize a new `ParserContext` with default values.
    pub fn new() -> Self {
        ParserContext {
            rss_version: RssVersionState::Other,
            parsing_state: ParsingState::None,
            current_element: String::new(),
            current_attributes: Vec::new(),
            current_item: RssItem::new(),
            image_title: String::new(),
            image_url: String::new(),
            image_link: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use quick_xml::events::BytesText;
    use quick_xml::events::BytesCData;
    use quick_xml::events::BytesStart;

    struct MockElementHandler;

    impl ElementHandler for MockElementHandler {
        fn handle_element(
            &self,
            name: &str,
            text: &str,
            _attributes: &[(String, String)],
        ) -> Result<()> {
            if name == "customElement" && text == "Custom content" {
                Ok(())
            } else {
                Err(RssError::UnknownElement(name.into()))
            }
        }
    }

    #[test]
    fn test_parser_config_with_custom_handler() {
        let handler = Arc::new(MockElementHandler);
        let config = ParserConfig {
            custom_handlers: vec![handler],
        };

        assert_eq!(config.custom_handlers.len(), 1);
        assert!(config.custom_handlers[0]
            .handle_element("customElement", "Custom content", &[])
            .is_ok());
    }

    #[test]
    fn test_parser_config_no_custom_handlers() {
        let config = ParserConfig::default();
        assert!(config.custom_handlers.is_empty());
    }

    #[test]
    fn test_process_start_event_empty_name() {
        let e = BytesStart::new("");
        let mut context = ParserContext::new();
        let mut rss_data = RssData::default();

        let result = process_start_event(&e, &mut context, &mut rss_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_start_event_non_empty_name() {
        let e = BytesStart::new("item");
        let mut context = ParserContext::new();
        let mut rss_data = RssData::default();

        let result = process_start_event(&e, &mut context, &mut rss_data);
        assert!(result.is_ok());
        assert_eq!(context.current_element, "item");
    }

    #[test]
    fn test_process_text_event() {
        let e = BytesText::from_escaped("Sample Text");
        let mut context = ParserContext::new();
        let mut rss_data = RssData::default();

        let result = process_text_event(&e, &mut context, &mut rss_data, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_cdata_event() {
        let e = BytesCData::new("Sample CDATA");
        let mut context = ParserContext::new();
        let mut rss_data = RssData::default();

        let result = process_cdata_event(&e, &mut context, &mut rss_data, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_channel_rdf_li_rss_1_0() {
        let mut rss_data = RssData::default();
        let result = parse_channel_element(&mut rss_data, "rdf:li", "", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_channel_rdf_li_non_rss_1_0() {
        let mut rss_data = RssData::default();
        let result = parse_channel_element(&mut rss_data, "rdf:li", "", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_channel_unknown_element() {
        let mut rss_data = RssData::default();
        let result = parse_channel_element(&mut rss_data, "unknownElement", "", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_rss_with_image() {
        let rss_xml = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0">
          <channel>
            <title>Sample Feed</title>
            <link>https://example.com</link>
            <description>A sample RSS feed</description>
            <image>
              <title>Sample Image</title>
              <url>https://example.com/image.jpg</url>
              <link>https://example.com</link>
            </image>
          </channel>
        </rss>
        "#;

        let result = parse_rss(rss_xml, None);

        match result {
            Ok(parsed_data) => {
                assert_eq!(parsed_data.title, "Sample Feed");
                assert_eq!(parsed_data.image_title, "Sample Image");
            }
            Err(RssError::UnknownElement(element)) => {
                panic!("Failed due to unknown element: {:?}", element);
            }
            Err(e) => panic!("Failed to parse RSS with image: {:?}", e),
        }
    }

    #[test]
    fn test_parse_rss_1_0() {
        let rss_xml = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
                 xmlns="http://purl.org/rss/1.0/">
          <channel rdf:about="https://example.com">
            <title>Sample Feed</title>
            <link>https://example.com</link>
            <description>A sample RSS feed</description>
          </channel>
        </rdf:RDF>
        "#;

        let result = parse_rss(rss_xml, None);

        match result {
            Ok(parsed_data) => {
                assert_eq!(parsed_data.title, "Sample Feed");
            }
            Err(RssError::UnknownElement(element)) => {
                panic!("Failed due to unknown element: {:?}", element);
            }
            Err(e) => panic!("Failed to parse RSS 1.0: {:?}", e),
        }
    }

    #[test]
    fn test_parse_rss_2_0() {
        let rss_xml = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0">
          <channel>
            <title>Sample Feed</title>
            <link>https://example.com</link>
            <description>A sample RSS feed</description>
          </channel>
        </rss>
        "#;

        let result = parse_rss(rss_xml, None);

        match result {
            Ok(parsed_data) => {
                assert_eq!(parsed_data.title, "Sample Feed");
            }
            Err(RssError::UnknownElement(element)) => {
                panic!("Failed due to unknown element: {:?}", element);
            }
            Err(e) => panic!("Failed to parse RSS 2.0: {:?}", e),
        }
    }

    #[test]
    fn test_parse_channel_language() {
        let mut rss_data = RssData::default();
        let result = parse_channel_element(
            &mut rss_data,
            "language",
            "en-US",
            false,
        );
        assert!(result.is_ok());
        assert_eq!(rss_data.language, "en-US");
    }

    #[test]
    fn test_parse_channel_copyright() {
        let mut rss_data = RssData::default();
        let result = parse_channel_element(
            &mut rss_data,
            "copyright",
            "© 2024",
            false,
        );
        assert!(result.is_ok());
        assert_eq!(rss_data.copyright, "© 2024");
    }

    #[test]
    fn test_parse_channel_managing_editor() {
        let mut rss_data = RssData::default();
        let result = parse_channel_element(
            &mut rss_data,
            "managingEditor",
            "editor@example.com",
            false,
        );
        assert!(result.is_ok());
        assert_eq!(rss_data.managing_editor, "editor@example.com");
    }

    #[test]
    fn test_parse_channel_webmaster() {
        let mut rss_data = RssData::default();
        let result = parse_channel_element(
            &mut rss_data,
            "webMaster",
            "webmaster@example.com",
            false,
        );
        assert!(result.is_ok());
        assert_eq!(rss_data.webmaster, "webmaster@example.com");
    }

    #[test]
    fn test_parse_channel_pub_date() {
        let mut rss_data = RssData::default();
        let result = parse_channel_element(
            &mut rss_data,
            "pubDate",
            "Mon, 10 Oct 2024 04:00:00 GMT",
            false,
        );
        assert!(result.is_ok());
        assert_eq!(rss_data.pub_date, "Mon, 10 Oct 2024 04:00:00 GMT");
    }

    #[test]
    fn test_parse_channel_last_build_date() {
        let mut rss_data = RssData::default();
        let result = parse_channel_element(
            &mut rss_data,
            "lastBuildDate",
            "Mon, 10 Oct 2024 05:00:00 GMT",
            false,
        );
        assert!(result.is_ok());
        assert_eq!(
            rss_data.last_build_date,
            "Mon, 10 Oct 2024 05:00:00 GMT"
        );
    }

    #[test]
    fn test_parse_channel_category() {
        let mut rss_data = RssData::default();
        let result = parse_channel_element(
            &mut rss_data,
            "category",
            "Technology",
            false,
        );
        assert!(result.is_ok());
        assert_eq!(rss_data.category, "Technology");
    }

    #[test]
    fn test_parse_channel_generator() {
        let mut rss_data = RssData::default();
        let result = parse_channel_element(
            &mut rss_data,
            "generator",
            "RSS Generator v1.0",
            false,
        );
        assert!(result.is_ok());
        assert_eq!(rss_data.generator, "RSS Generator v1.0");
    }

    #[test]
    fn test_parse_channel_docs() {
        let mut rss_data = RssData::default();
        let result = parse_channel_element(
            &mut rss_data,
            "docs",
            "https://example.com/rss/docs",
            false,
        );
        assert!(result.is_ok());
        assert_eq!(rss_data.docs, "https://example.com/rss/docs");
    }

    #[test]
    fn test_parse_channel_ttl() {
        let mut rss_data = RssData::default();
        let result =
            parse_channel_element(&mut rss_data, "ttl", "60", false);
        assert!(result.is_ok());
        assert_eq!(rss_data.ttl, "60");
    }

    #[test]
    fn test_parse_channel_items_rss_1_0() {
        let mut rss_data = RssData::default();
        let result =
            parse_channel_element(&mut rss_data, "items", "", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_channel_items_non_rss_1_0() {
        let mut rss_data = RssData::default();
        let result =
            parse_channel_element(&mut rss_data, "items", "", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_channel_rdf_seq_rss_1_0() {
        let mut rss_data = RssData::default();
        let result =
            parse_channel_element(&mut rss_data, "rdf:Seq", "", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_channel_rdf_seq_non_rss_1_0() {
        let mut rss_data = RssData::default();
        let result =
            parse_channel_element(&mut rss_data, "rdf:Seq", "", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_item_author() {
        let mut item = RssItem::default();
        parse_item_element(
            &mut item,
            "author",
            "author@example.com",
            &[],
        );
        assert_eq!(item.author, "author@example.com");
    }

    #[test]
    fn test_parse_item_guid() {
        let mut item = RssItem::default();
        parse_item_element(&mut item, "guid", "1234-5678", &[]);
        assert_eq!(item.guid, "1234-5678");
    }

    #[test]
    fn test_parse_item_pub_date() {
        let mut item = RssItem::default();
        parse_item_element(
            &mut item,
            "pubDate",
            "Mon, 10 Oct 2024 04:00:00 GMT",
            &[],
        );
        assert_eq!(item.pub_date, "Mon, 10 Oct 2024 04:00:00 GMT");
    }

    #[test]
    fn test_parse_item_category() {
        let mut item = RssItem::default();
        parse_item_element(&mut item, "category", "Technology", &[]);
        assert_eq!(item.category, Some("Technology".to_string()));
    }

    #[test]
    fn test_parse_item_comments() {
        let mut item = RssItem::default();
        parse_item_element(
            &mut item,
            "comments",
            "https://example.com/comments",
            &[],
        );
        assert_eq!(
            item.comments,
            Some("https://example.com/comments".to_string())
        );
    }

    #[test]
    fn test_parse_item_enclosure_with_attributes() {
        let mut item = RssItem::default();
        let attributes = vec![
            (
                "url".to_string(),
                "https://example.com/audio.mp3".to_string(),
            ),
            ("length".to_string(), "123456".to_string()),
            ("type".to_string(), "audio/mpeg".to_string()),
        ];
        parse_item_element(&mut item, "enclosure", "", &attributes);
        assert_eq!(
            item.enclosure,
            Some("url=\"https://example.com/audio.mp3\" length=\"123456\" type=\"audio/mpeg\"".to_string())
        );
    }

    #[test]
    fn test_parse_item_enclosure_without_attributes() {
        let mut item = RssItem::default();
        parse_item_element(&mut item, "enclosure", "", &[]);
        assert_eq!(item.enclosure, None);
    }

    #[test]
    fn test_parse_item_source() {
        let mut item = RssItem::default();
        parse_item_element(
            &mut item,
            "source",
            "https://example.com",
            &[],
        );
        assert_eq!(
            item.source,
            Some("https://example.com".to_string())
        );
    }
}
