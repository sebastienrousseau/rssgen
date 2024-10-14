// Copyright © 2024 RSS Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

// src/data.rs

//! This module contains the core data structures and functionality for RSS feeds.
//!
//! It includes definitions for RSS versions, RSS data, and RSS items, as well as
//! utility functions for URL validation and date parsing.

use crate::error::{Result, RssError};
use dtt::datetime::DateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use time::{
    format_description::well_known::Iso8601,
    format_description::well_known::Rfc2822, OffsetDateTime,
};
use url::Url;

/// Represents the different versions of RSS.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
#[non_exhaustive]
pub enum RssVersion {
    /// RSS version 0.90
    RSS0_90,
    /// RSS version 0.91
    RSS0_91,
    /// RSS version 0.92
    RSS0_92,
    /// RSS version 1.0
    RSS1_0,
    /// RSS version 2.0
    RSS2_0,
}

impl RssVersion {
    /// Returns the string representation of the RSS version.
    ///
    /// # Returns
    ///
    /// A static string slice representing the RSS version.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::RSS0_90 => "0.90",
            Self::RSS0_91 => "0.91",
            Self::RSS0_92 => "0.92",
            Self::RSS1_0 => "1.0",
            Self::RSS2_0 => "2.0",
        }
    }
}

impl Default for RssVersion {
    fn default() -> Self {
        Self::RSS2_0
    }
}

impl fmt::Display for RssVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for RssVersion {
    type Err = RssError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "0.90" => Ok(Self::RSS0_90),
            "0.91" => Ok(Self::RSS0_91),
            "0.92" => Ok(Self::RSS0_92),
            "1.0" => Ok(Self::RSS1_0),
            "2.0" => Ok(Self::RSS2_0),
            _ => Err(RssError::InvalidRssVersion(s.to_string())),
        }
    }
}

/// Represents the main structure for an RSS feed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct RssData {
    /// The Atom link of the RSS feed.
    pub atom_link: String,
    /// The author of the RSS feed.
    pub author: String,
    /// The category of the RSS feed.
    pub category: String,
    /// The copyright notice for the content of the feed.
    pub copyright: String,
    /// The description of the RSS feed.
    pub description: String,
    /// The docs link of the RSS feed.
    pub docs: String,
    /// The generator of the RSS feed.
    pub generator: String,
    /// The GUID of the RSS feed.
    pub guid: String,
    /// The image title of the RSS feed.
    pub image_title: String,
    /// The image URL of the RSS feed.
    pub image_url: String,
    /// The image link of the RSS feed.
    pub image_link: String,
    /// The language of the RSS feed.
    pub language: String,
    /// The last build date of the RSS feed.
    pub last_build_date: String,
    /// The main link to the RSS feed.
    pub link: String,
    /// The managing editor of the RSS feed.
    pub managing_editor: String,
    /// The publication date of the RSS feed.
    pub pub_date: String,
    /// The title of the RSS feed.
    pub title: String,
    /// Time To Live (TTL), the number of minutes the feed should be cached before refreshing.
    pub ttl: String,
    /// The webmaster of the RSS feed.
    pub webmaster: String,
    /// A collection of additional items in the RSS feed.
    pub items: Vec<RssItem>,
    /// The version of the RSS feed.
    pub version: RssVersion,
}

impl RssData {
    /// Creates a new `RssData` instance with default values and a specified RSS version.
    ///
    /// # Arguments
    ///
    /// * `version` - An optional `RssVersion` specifying the RSS version for the feed.
    ///
    /// # Returns
    ///
    /// A new `RssData` instance.
    #[must_use]
    pub fn new(version: Option<RssVersion>) -> Self {
        Self {
            version: version.unwrap_or_default(),
            ..Default::default()
        }
    }

    /// Sets the value of a specified field and returns the `RssData` instance for method chaining.
    ///
    /// # Arguments
    ///
    /// * `field` - The field to set.
    /// * `value` - The value to assign to the field.
    ///
    /// # Returns
    ///
    /// The updated `RssData` instance.
    pub fn set<T: Into<String>>(
        mut self,
        field: RssDataField,
        value: T,
    ) -> Self {
        let value = sanitize_input(&value.into());
        match field {
            RssDataField::AtomLink => self.atom_link = value,
            RssDataField::Author => self.author = value,
            RssDataField::Category => self.category = value,
            RssDataField::Copyright => self.copyright = value,
            RssDataField::Description => self.description = value,
            RssDataField::Docs => self.docs = value,
            RssDataField::Generator => self.generator = value,
            RssDataField::Guid => self.guid = value,
            RssDataField::ImageTitle => self.image_title = value,
            RssDataField::ImageUrl => self.image_url = value,
            RssDataField::ImageLink => self.image_link = value,
            RssDataField::Language => self.language = value,
            RssDataField::LastBuildDate => self.last_build_date = value,
            RssDataField::Link => self.link = value,
            RssDataField::ManagingEditor => {
                self.managing_editor = value
            }
            RssDataField::PubDate => self.pub_date = value,
            RssDataField::Title => self.title = value,
            RssDataField::Ttl => self.ttl = value,
            RssDataField::Webmaster => self.webmaster = value,
        }
        self
    }

    /// Sets the value of a specified item field for the most recent RSS item.
    ///
    /// # Arguments
    ///
    /// * `field` - The field to set.
    /// * `value` - The value to assign to the field.
    ///
    pub fn set_item_field<T: Into<String>>(
        &mut self,
        field: RssItemField,
        value: T,
    ) {
        let value = sanitize_input(&value.into());
        if self.items.is_empty() {
            self.items.push(RssItem::new());
        }
        let item = self.items.last_mut().unwrap();
        match field {
            RssItemField::Guid => item.guid = value,
            RssItemField::Category => item.category = Some(value),
            RssItemField::Description => item.description = value,
            RssItemField::Link => item.link = value,
            RssItemField::PubDate => item.pub_date = value,
            RssItemField::Title => item.title = value,
            RssItemField::Author => item.author = value,
            RssItemField::Comments => item.comments = Some(value),
            RssItemField::Enclosure => item.enclosure = Some(value),
            RssItemField::Source => item.source = Some(value),
        }
    }

    /// Sets the image for the RSS feed.
    ///
    /// # Arguments
    ///
    /// * `title` - The title of the image.
    /// * `url` - The URL of the image.
    /// * `link` - The link associated with the image.
    pub fn set_image(
        &mut self,
        title: String,
        url: String,
        link: String,
    ) {
        self.image_title = sanitize_input(&title);
        self.image_url = sanitize_input(&url);
        self.image_link = sanitize_input(&link);
    }

    /// Adds an item to the RSS feed.
    ///
    /// This method appends the given `RssItem` to the `items` vector of the `RssData` struct.
    ///
    /// # Arguments
    ///
    /// * `item` - The `RssItem` to be added to the feed.
    pub fn add_item(&mut self, item: RssItem) {
        self.items.push(item);
    }

    /// Removes an item from the RSS feed by its GUID.
    ///
    /// # Arguments
    ///
    /// * `guid` - The GUID of the item to remove.
    ///
    /// # Returns
    ///
    /// `true` if an item was removed, `false` otherwise.
    pub fn remove_item(&mut self, guid: &str) -> bool {
        let initial_len = self.items.len();
        self.items.retain(|item| item.guid != guid);
        self.items.len() < initial_len
    }

    /// Returns the number of items in the RSS feed.
    #[must_use]
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Clears all items from the RSS feed.
    pub fn clear_items(&mut self) {
        self.items.clear();
    }

    /// Validates the `RssData` to ensure that all required fields are set and valid.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the `RssData` is valid.
    /// * `Err(RssError)` if any validation errors are found.
    pub fn validate(&self) -> Result<()> {
        let mut errors = Vec::new();

        if self.title.is_empty() {
            errors.push("Title is missing".to_string());
        }

        if self.link.is_empty() {
            errors.push("Link is missing".to_string());
        } else if let Err(e) = validate_url(&self.link) {
            errors.push(format!("Invalid link: {}", e));
        }

        if self.description.is_empty() {
            errors.push("Description is missing".to_string());
        }

        if !self.pub_date.is_empty() {
            if let Err(e) = parse_date(&self.pub_date) {
                errors.push(format!("Invalid publication date: {}", e));
            }
        }

        if !errors.is_empty() {
            return Err(RssError::ValidationErrors(errors));
        }

        Ok(())
    }

    /// Converts the `RssData` into a `HashMap<String, String>` for easier manipulation.
    ///
    /// # Returns
    ///
    /// A `HashMap<String, String>` containing the RSS feed data.
    #[must_use]
    pub fn to_hash_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("atom_link".to_string(), self.atom_link.clone());
        map.insert("author".to_string(), self.author.clone());
        map.insert("category".to_string(), self.category.clone());
        map.insert("copyright".to_string(), self.copyright.clone());
        map.insert("description".to_string(), self.description.clone());
        map.insert("docs".to_string(), self.docs.clone());
        map.insert("generator".to_string(), self.generator.clone());
        map.insert("guid".to_string(), self.guid.clone());
        map.insert("image_title".to_string(), self.image_title.clone());
        map.insert("image_url".to_string(), self.image_url.clone());
        map.insert("image_link".to_string(), self.image_link.clone());
        map.insert("language".to_string(), self.language.clone());
        map.insert(
            "last_build_date".to_string(),
            self.last_build_date.clone(),
        );
        map.insert("link".to_string(), self.link.clone());
        map.insert(
            "managing_editor".to_string(),
            self.managing_editor.clone(),
        );
        map.insert("pub_date".to_string(), self.pub_date.clone());
        map.insert("title".to_string(), self.title.clone());
        map.insert("ttl".to_string(), self.ttl.clone());
        map.insert("webmaster".to_string(), self.webmaster.clone());
        map
    }

    // Field setter methods

    /// Sets the RSS version.
    #[must_use]
    pub fn version(mut self, version: RssVersion) -> Self {
        self.version = version;
        self
    }

    /// Sets the Atom link.
    #[must_use]
    pub fn atom_link<T: Into<String>>(self, value: T) -> Self {
        self.set(RssDataField::AtomLink, value)
    }

    /// Sets the author.
    #[must_use]
    pub fn author<T: Into<String>>(self, value: T) -> Self {
        self.set(RssDataField::Author, value)
    }

    /// Sets the category.
    #[must_use]
    pub fn category<T: Into<String>>(self, value: T) -> Self {
        self.set(RssDataField::Category, value)
    }

    /// Sets the copyright.
    #[must_use]
    pub fn copyright<T: Into<String>>(self, value: T) -> Self {
        self.set(RssDataField::Copyright, value)
    }

    /// Sets the description.
    #[must_use]
    pub fn description<T: Into<String>>(self, value: T) -> Self {
        self.set(RssDataField::Description, value)
    }

    /// Sets the docs link.
    #[must_use]
    pub fn docs<T: Into<String>>(self, value: T) -> Self {
        self.set(RssDataField::Docs, value)
    }

    /// Sets the generator.
    #[must_use]
    pub fn generator<T: Into<String>>(self, value: T) -> Self {
        self.set(RssDataField::Generator, value)
    }

    /// Sets the GUID.
    #[must_use]
    pub fn guid<T: Into<String>>(self, value: T) -> Self {
        self.set(RssDataField::Guid, value)
    }

    /// Sets the image title.
    #[must_use]
    pub fn image_title<T: Into<String>>(self, value: T) -> Self {
        self.set(RssDataField::ImageTitle, value)
    }

    /// Sets the image URL.
    #[must_use]
    pub fn image_url<T: Into<String>>(self, value: T) -> Self {
        self.set(RssDataField::ImageUrl, value)
    }

    /// Sets the image link.
    #[must_use]
    pub fn image_link<T: Into<String>>(self, value: T) -> Self {
        self.set(RssDataField::ImageLink, value)
    }

    /// Sets the language.
    #[must_use]
    pub fn language<T: Into<String>>(self, value: T) -> Self {
        self.set(RssDataField::Language, value)
    }

    /// Sets the last build date.
    #[must_use]
    pub fn last_build_date<T: Into<String>>(self, value: T) -> Self {
        self.set(RssDataField::LastBuildDate, value)
    }

    /// Sets the main link.
    #[must_use]
    pub fn link<T: Into<String>>(self, value: T) -> Self {
        self.set(RssDataField::Link, value)
    }

    /// Sets the managing editor.
    #[must_use]
    pub fn managing_editor<T: Into<String>>(self, value: T) -> Self {
        self.set(RssDataField::ManagingEditor, value)
    }

    /// Sets the publication date.
    #[must_use]
    pub fn pub_date<T: Into<String>>(self, value: T) -> Self {
        self.set(RssDataField::PubDate, value)
    }

    /// Sets the title.
    #[must_use]
    pub fn title<T: Into<String>>(self, value: T) -> Self {
        self.set(RssDataField::Title, value)
    }

    /// Sets the TTL (Time To Live).
    #[must_use]
    pub fn ttl<T: Into<String>>(self, value: T) -> Self {
        self.set(RssDataField::Ttl, value)
    }

    /// Sets the webmaster.
    #[must_use]
    pub fn webmaster<T: Into<String>>(self, value: T) -> Self {
        self.set(RssDataField::Webmaster, value)
    }
}

/// Represents the fields of an RSS data structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RssDataField {
    /// The Atom link of the RSS feed.
    AtomLink,
    /// The author of the RSS feed.
    Author,
    /// The category of the RSS feed.
    Category,
    /// The copyright notice.
    Copyright,
    /// The description of the RSS feed.
    Description,
    /// The docs link of the RSS feed.
    Docs,
    /// The generator of the RSS feed.
    Generator,
    /// The GUID of the RSS feed.
    Guid,
    /// The image title of the RSS feed.
    ImageTitle,
    /// The image URL of the RSS feed.
    ImageUrl,
    /// The image link of the RSS feed.
    ImageLink,
    /// The language of the RSS feed.
    Language,
    /// The last build date of the RSS feed.
    LastBuildDate,
    /// The main link to the RSS feed.
    Link,
    /// The managing editor of the RSS feed.
    ManagingEditor,
    /// The publication date of the RSS feed.
    PubDate,
    /// The title of the RSS feed.
    Title,
    /// Time To Live (TTL), the number of minutes the feed should be cached before refreshing.
    Ttl,
    /// The webmaster of the RSS feed.
    Webmaster,
}

/// Represents an item in the RSS feed.
#[derive(
    Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize,
)]
#[non_exhaustive]
pub struct RssItem {
    /// The GUID of the RSS item (unique identifier).
    pub guid: String,
    /// The category of the RSS item.
    pub category: Option<String>,
    /// The description of the RSS item.
    pub description: String,
    /// The link to the RSS item.
    pub link: String,
    /// The publication date of the RSS item.
    pub pub_date: String,
    /// The title of the RSS item.
    pub title: String,
    /// The author of the RSS item.
    pub author: String,
    /// The comments URL related to the RSS item (optional).
    pub comments: Option<String>,
    /// The enclosure (typically for media like podcasts) (optional).
    pub enclosure: Option<String>,
    /// The source of the RSS item (optional).
    pub source: Option<String>,
}

impl RssItem {
    /// Creates a new `RssItem` with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the value of a field and returns the `RssItem` instance for method chaining.
    ///
    /// # Arguments
    ///
    /// * `field` - The field to set.
    /// * `value` - The value to assign to the field.
    ///
    /// # Returns
    ///
    /// The updated `RssItem` instance.
    pub fn set<T: Into<String>>(
        mut self,
        field: RssItemField,
        value: T,
    ) -> Self {
        let value = sanitize_input(&value.into());
        match field {
            RssItemField::Guid => self.guid = value,
            RssItemField::Category => self.category = Some(value),
            RssItemField::Description => self.description = value,
            RssItemField::Link => self.link = value,
            RssItemField::PubDate => self.pub_date = value,
            RssItemField::Title => self.title = value,
            RssItemField::Author => self.author = value,
            RssItemField::Comments => self.comments = Some(value),
            RssItemField::Enclosure => self.enclosure = Some(value),
            RssItemField::Source => self.source = Some(value),
        }
        self
    }

    /// Validates the `RssItem` to ensure all required fields are set and valid.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the `RssItem` is valid.
    /// * `Err(RssError)` if any validation errors are found.
    pub fn validate(&self) -> Result<()> {
        let mut validation_errors = Vec::new();

        if self.title.is_empty() {
            validation_errors.push("Title is missing".to_string());
        }

        if self.link.is_empty() {
            validation_errors.push("Link is missing".to_string());
        } else if let Err(e) = validate_url(&self.link) {
            validation_errors.push(format!("Invalid link: {}", e));
        }

        if self.guid.is_empty() {
            validation_errors.push("GUID is missing".to_string());
        }

        if !self.pub_date.is_empty() {
            if let Err(e) = parse_date(&self.pub_date) {
                validation_errors
                    .push(format!("Invalid publication date: {}", e));
            }
        }

        if !validation_errors.is_empty() {
            return Err(RssError::ValidationErrors(validation_errors));
        }

        Ok(())
    }

    /// Parses the `pub_date` string into a `DateTime` object.
    ///
    /// # Returns
    ///
    /// * `Ok(DateTime)` if the date is valid and successfully parsed.
    /// * `Err(RssError)` if the date is invalid or cannot be parsed.
    pub fn pub_date_parsed(&self) -> Result<DateTime> {
        parse_date(&self.pub_date)
    }

    // Field setter methods

    /// Sets the GUID.
    #[must_use]
    pub fn guid<T: Into<String>>(self, value: T) -> Self {
        self.set(RssItemField::Guid, value)
    }

    /// Sets the category.
    #[must_use]
    pub fn category<T: Into<String>>(self, value: T) -> Self {
        self.set(RssItemField::Category, value)
    }

    /// Sets the description.
    #[must_use]
    pub fn description<T: Into<String>>(self, value: T) -> Self {
        self.set(RssItemField::Description, value)
    }

    /// Sets the link.
    #[must_use]
    pub fn link<T: Into<String>>(self, value: T) -> Self {
        self.set(RssItemField::Link, value)
    }

    /// Sets the publication date.
    #[must_use]
    pub fn pub_date<T: Into<String>>(self, value: T) -> Self {
        self.set(RssItemField::PubDate, value)
    }

    /// Sets the title.
    #[must_use]
    pub fn title<T: Into<String>>(self, value: T) -> Self {
        self.set(RssItemField::Title, value)
    }

    /// Sets the author.
    #[must_use]
    pub fn author<T: Into<String>>(self, value: T) -> Self {
        self.set(RssItemField::Author, value)
    }

    /// Sets the comments URL.
    #[must_use]
    pub fn comments<T: Into<String>>(self, value: T) -> Self {
        self.set(RssItemField::Comments, value)
    }

    /// Sets the enclosure.
    #[must_use]
    pub fn enclosure<T: Into<String>>(self, value: T) -> Self {
        self.set(RssItemField::Enclosure, value)
    }

    /// Sets the source.
    #[must_use]
    pub fn source<T: Into<String>>(self, value: T) -> Self {
        self.set(RssItemField::Source, value)
    }
}

/// Represents the fields of an RSS item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RssItemField {
    /// The GUID of the RSS item.
    Guid,
    /// The category of the RSS item.
    Category,
    /// The description of the RSS item.
    Description,
    /// The link to the RSS item.
    Link,
    /// The publication date of the RSS item.
    PubDate,
    /// The title of the RSS item.
    Title,
    /// The author of the RSS item.
    Author,
    /// The comments URL related to the RSS item.
    Comments,
    /// The enclosure (typically for media like podcasts).
    Enclosure,
    /// The source of the RSS item.
    Source,
}

/// Validates a URL string.
///
/// # Arguments
///
/// * `url` - A string slice that holds the URL to validate.
///
/// # Returns
///
/// * `Ok(())` if the URL is valid.
/// * `Err(RssError)` if the URL is invalid.
pub fn validate_url(url: &str) -> Result<()> {
    let parsed_url = Url::parse(url)
        .map_err(|_| RssError::InvalidUrl(url.to_string()))?;

    if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
        return Err(RssError::InvalidUrl(
            "URL must use http or https protocol".to_string(),
        ));
    }

    Ok(())
}

/// Parses a date string into a `DateTime`.
///
/// # Arguments
///
/// * `date_str` - A string slice that holds the date to parse.
///
/// # Returns
///
/// * `Ok(DateTime)` if the date is valid and successfully parsed.
/// * `Err(RssError)` if the date is invalid or cannot be parsed.
pub fn parse_date(date_str: &str) -> Result<DateTime> {
    // Try parsing as RFC 2822
    if OffsetDateTime::parse(date_str, &Rfc2822).is_ok() {
        return Ok(
            DateTime::new_with_tz("UTC").expect("UTC is always valid")
        );
    }

    // Try parsing as ISO 8601
    if OffsetDateTime::parse(date_str, &Iso8601::DEFAULT).is_ok() {
        return Ok(
            DateTime::new_with_tz("UTC").expect("UTC is always valid")
        );
    }

    // If the date format is not RFC 2822 or ISO 8601, fall back to manual parsing
    let components: Vec<&str> = date_str.split_whitespace().collect();

    if components.len() == 6 {
        let _day: u8 = components[1].parse().map_err(|_| {
            RssError::DateParseError(date_str.to_string())
        })?;
        let _month = parse_month(components[2])?;
        let _year: i32 = components[3].parse().map_err(|_| {
            RssError::DateParseError(date_str.to_string())
        })?;
        let time_components: Vec<&str> =
            components[4].split(':').collect();
        let hours: i8 = time_components[0].parse().map_err(|_| {
            RssError::DateParseError(date_str.to_string())
        })?;
        let minutes: i8 = time_components[1].parse().map_err(|_| {
            RssError::DateParseError(date_str.to_string())
        })?;
        let _seconds: i8 =
            time_components[2].parse().map_err(|_| {
                RssError::DateParseError(date_str.to_string())
            })?;

        // Create a new DateTime with custom hours and minutes offset
        return DateTime::new_with_custom_offset(hours, minutes)
            .map_err(|e| RssError::DateParseError(e.to_string()));
    }

    // If the format doesn't match any of the above, return an error
    Err(RssError::DateParseError(date_str.to_string()))
}

/// Parses a month string into its numerical representation.
///
/// # Arguments
///
/// * `month` - A string slice representing the month.
///
/// # Returns
///
/// * `Ok(u8)` if the month is valid and successfully parsed.
/// * `Err(RssError)` if the month is invalid or cannot be parsed.
fn parse_month(month: &str) -> Result<u8> {
    match month {
        "Jan" => Ok(1),
        "Feb" => Ok(2),
        "Mar" => Ok(3),
        "Apr" => Ok(4),
        "May" => Ok(5),
        "Jun" => Ok(6),
        "Jul" => Ok(7),
        "Aug" => Ok(8),
        "Sep" => Ok(9),
        "Oct" => Ok(10),
        "Nov" => Ok(11),
        "Dec" => Ok(12),
        _ => Err(RssError::DateParseError(month.to_string())),
    }
}

/// Sanitizes input by escaping HTML special characters.
///
/// # Arguments
///
/// * `input` - A string slice containing the input to sanitize.
///
/// # Returns
///
/// A `String` with HTML special characters escaped.
fn sanitize_input(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::de::from_str;

    #[test]
    fn test_rss_version() {
        assert_eq!(RssVersion::RSS2_0.as_str(), "2.0");
        assert_eq!(RssVersion::default(), RssVersion::RSS2_0);
        assert_eq!(RssVersion::RSS1_0.to_string(), "1.0");
        assert!(matches!(
            "2.0".parse::<RssVersion>(),
            Ok(RssVersion::RSS2_0)
        ));
        assert!("3.0".parse::<RssVersion>().is_err());
    }

    #[test]
    fn test_rss_data_new() {
        let rss_data = RssData::new(Some(RssVersion::RSS2_0));
        assert_eq!(rss_data.version, RssVersion::RSS2_0);
    }

    #[test]
    fn test_rss_data_setters() {
        let rss_data = RssData::new(None)
            .title("Test Feed")
            .link("https://example.com")
            .description("A test feed")
            .generator("RSS Gen")
            .guid("unique-guid")
            .pub_date("2024-03-21T12:00:00Z")
            .language("en");

        assert_eq!(rss_data.title, "Test Feed");
        assert_eq!(rss_data.link, "https://example.com");
        assert_eq!(rss_data.description, "A test feed");
        assert_eq!(rss_data.generator, "RSS Gen");
        assert_eq!(rss_data.guid, "unique-guid");
        assert_eq!(rss_data.pub_date, "2024-03-21T12:00:00Z");
        assert_eq!(rss_data.language, "en");
    }

    #[test]
    fn test_rss_data_validate() {
        let valid_rss_data = RssData::new(None)
            .title("Valid Feed")
            .link("https://example.com")
            .description("A valid RSS feed");

        assert!(valid_rss_data.validate().is_ok());

        let invalid_rss_data = RssData::new(None)
            .title("Invalid Feed")
            .link("not a valid url")
            .description("An invalid RSS feed");

        let result = invalid_rss_data.validate();
        assert!(result.is_err());
        if let Err(RssError::ValidationErrors(errors)) = result {
            assert!(errors.iter().any(|e| e.contains("Invalid link")),
                "Expected an error containing 'Invalid link', but got: {:?}", errors);
        } else {
            panic!("Expected ValidationErrors");
        }
    }

    #[test]
    fn test_add_item() {
        let mut rss_data = RssData::new(None)
            .title("Test RSS Feed")
            .link("https://example.com")
            .description("A test RSS feed");

        let item = RssItem::new()
            .title("Test Item")
            .link("https://example.com/item")
            .description("A test item")
            .guid("unique-id-1")
            .pub_date("2024-03-21");

        rss_data.add_item(item);

        assert_eq!(rss_data.items.len(), 1);
        assert_eq!(rss_data.items[0].title, "Test Item");
        assert_eq!(rss_data.items[0].link, "https://example.com/item");
        assert_eq!(rss_data.items[0].description, "A test item");
        assert_eq!(rss_data.items[0].guid, "unique-id-1");
        assert_eq!(rss_data.items[0].pub_date, "2024-03-21");
    }

    #[test]
    fn test_remove_item() {
        let mut rss_data = RssData::new(None)
            .title("Test RSS Feed")
            .link("https://example.com")
            .description("A test RSS feed");

        let item1 = RssItem::new()
            .title("Item 1")
            .link("https://example.com/item1")
            .description("First item")
            .guid("guid1");

        let item2 = RssItem::new()
            .title("Item 2")
            .link("https://example.com/item2")
            .description("Second item")
            .guid("guid2");

        rss_data.add_item(item1);
        rss_data.add_item(item2);

        assert_eq!(rss_data.item_count(), 2);

        assert!(rss_data.remove_item("guid1"));
        assert_eq!(rss_data.item_count(), 1);
        assert_eq!(rss_data.items[0].title, "Item 2");

        assert!(!rss_data.remove_item("non-existent-guid"));
        assert_eq!(rss_data.item_count(), 1);
    }

    #[test]
    fn test_clear_items() {
        let mut rss_data = RssData::new(None)
            .title("Test RSS Feed")
            .link("https://example.com")
            .description("A test RSS feed");

        rss_data.add_item(RssItem::new().title("Item 1").guid("guid1"));
        rss_data.add_item(RssItem::new().title("Item 2").guid("guid2"));

        assert_eq!(rss_data.item_count(), 2);

        rss_data.clear_items();

        assert_eq!(rss_data.item_count(), 0);
    }

    #[test]
    fn test_rss_item_validate() {
        let valid_item = RssItem::new()
            .title("Valid Item")
            .link("https://example.com/valid")
            .description("A valid item")
            .guid("valid-guid");

        assert!(valid_item.validate().is_ok());

        let invalid_item = RssItem::new()
            .title("Invalid Item")
            .description("An invalid item");

        let result = invalid_item.validate();
        assert!(result.is_err());

        if let Err(RssError::ValidationErrors(errors)) = result {
            assert_eq!(errors.len(), 2);
            assert!(errors.contains(&"Link is missing".to_string()));
            assert!(errors.contains(&"GUID is missing".to_string()));
        } else {
            panic!("Expected ValidationErrors");
        }
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("not a url").is_err());
    }

    #[test]
    fn test_parse_date() {
        assert!(parse_date("Mon, 01 Jan 2024 00:00:00 GMT").is_ok());
        assert!(parse_date("2024-03-21T12:00:00Z").is_ok());
        assert!(parse_date("invalid date").is_err());
    }

    #[test]
    fn test_sanitize_input() {
        let input = "Test <script>alert('XSS')</script>";
        let sanitized = sanitize_input(input);
        assert_eq!(
            sanitized,
            "Test &lt;script&gt;alert(&#x27;XSS&#x27;)&lt;/script&gt;"
        );
    }

    #[test]
    fn test_rss_data_set_with_enum() {
        let rss_data = RssData::new(None)
            .set(RssDataField::Title, "Test Title")
            .set(RssDataField::Link, "https://example.com")
            .set(RssDataField::Description, "Test Description");

        assert_eq!(rss_data.title, "Test Title");
        assert_eq!(rss_data.link, "https://example.com");
        assert_eq!(rss_data.description, "Test Description");
    }

    #[test]
    fn test_rss_item_set_with_enum() {
        let item = RssItem::new()
            .set(RssItemField::Title, "Test Item")
            .set(RssItemField::Link, "https://example.com/item")
            .set(RssItemField::Guid, "unique-id");

        assert_eq!(item.title, "Test Item");
        assert_eq!(item.link, "https://example.com/item");
        assert_eq!(item.guid, "unique-id");
    }

    #[test]
    fn test_to_hash_map() {
        let rss_data = RssData::new(None)
            .title("Test Title")
            .link("https://example.com/rss")
            .description("A test RSS feed")
            .atom_link("https://example.com/atom")
            .language("en")
            .managing_editor("editor@example.com")
            .webmaster("webmaster@example.com")
            .last_build_date("2024-03-21T12:00:00Z")
            .pub_date("2024-03-21T12:00:00Z")
            .ttl("60")
            .generator("RSS Gen")
            .guid("unique-guid")
            .image_title("Image Title".to_string())
            .docs("https://docs.example.com");

        let map = rss_data.to_hash_map();

        assert_eq!(map.get("title").unwrap(), "Test Title");
        assert_eq!(map.get("link").unwrap(), "https://example.com/rss");
        assert_eq!(
            map.get("atom_link").unwrap(),
            "https://example.com/atom"
        );
        assert_eq!(map.get("language").unwrap(), "en");
        assert_eq!(
            map.get("managing_editor").unwrap(),
            "editor@example.com"
        );
        assert_eq!(
            map.get("webmaster").unwrap(),
            "webmaster@example.com"
        );
        assert_eq!(
            map.get("last_build_date").unwrap(),
            "2024-03-21T12:00:00Z"
        );
        assert_eq!(
            map.get("pub_date").unwrap(),
            "2024-03-21T12:00:00Z"
        );
        assert_eq!(map.get("ttl").unwrap(), "60");
        assert_eq!(map.get("generator").unwrap(), "RSS Gen");
        assert_eq!(map.get("guid").unwrap(), "unique-guid");
        assert_eq!(map.get("image_title").unwrap(), "Image Title");
        assert_eq!(
            map.get("docs").unwrap(),
            "https://docs.example.com"
        );
    }

    #[test]
    fn test_set_image() {
        let mut rss_data = RssData::new(None);
        rss_data.set_image(
            "Test Image Title".to_string(),
            "https://example.com/image.jpg".to_string(),
            "https://example.com".to_string(),
        );
        rss_data.title = "RSS Feed Title".to_string();

        assert_eq!(rss_data.image_title, "Test Image Title");
        assert_eq!(rss_data.image_url, "https://example.com/image.jpg");
        assert_eq!(rss_data.image_link, "https://example.com");
        assert_eq!(rss_data.title, "RSS Feed Title");
    }

    #[test]
    fn test_rss_feed_parsing() {
        let rss_xml = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0" xmlns:content="http://purl.org/rss/1.0/modules/content/"
             xmlns:dc="http://purl.org/dc/elements/1.1/"
             xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
             xmlns:taxo="http://purl.org/rss/1.0/modules/taxonomy/">
          <channel>
            <title>GETS Open Tenders or Quotes</title>
            <link>https://www.gets.govt.nz//ExternalIndex.htm</link>
            <description>This feed lists the current open tenders or requests for quote listed on the GETS.</description>
            <image>
              <title>Open tenders or Requests for Quote from GETS</title>
              <url>https://www.gets.govt.nz//ext/default/img/getsLogo.jpg</url>
              <link>https://www.gets.govt.nz//ExternalIndex.htm</link>
            </image>
          </channel>
        </rss>
        "#;

        #[derive(Debug, Deserialize, PartialEq)]
        struct Image {
            title: String,
            url: String,
            link: String,
        }

        #[derive(Debug, Deserialize, PartialEq)]
        struct Channel {
            title: String,
            link: String,
            description: String,
            image: Image,
        }

        #[derive(Debug, Deserialize, PartialEq)]
        struct Rss {
            #[serde(rename = "channel")]
            channel: Channel,
        }

        let parsed: Rss =
            from_str(rss_xml).expect("Failed to parse RSS XML");

        assert_eq!(parsed.channel.title, "GETS Open Tenders or Quotes");
        assert_eq!(
            parsed.channel.link,
            "https://www.gets.govt.nz//ExternalIndex.htm"
        );
        assert_eq!(parsed.channel.description, "This feed lists the current open tenders or requests for quote listed on the GETS.");
        assert_eq!(
            parsed.channel.image.title,
            "Open tenders or Requests for Quote from GETS"
        );
        assert_eq!(
            parsed.channel.image.url,
            "https://www.gets.govt.nz//ext/default/img/getsLogo.jpg"
        );
        assert_eq!(
            parsed.channel.image.link,
            "https://www.gets.govt.nz//ExternalIndex.htm"
        );
    }

    #[test]
    fn test_rss_version_from_str() {
        assert_eq!(
            RssVersion::from_str("0.90").unwrap(),
            RssVersion::RSS0_90
        );
        assert_eq!(
            RssVersion::from_str("0.91").unwrap(),
            RssVersion::RSS0_91
        );
        assert_eq!(
            RssVersion::from_str("0.92").unwrap(),
            RssVersion::RSS0_92
        );
        assert_eq!(
            RssVersion::from_str("1.0").unwrap(),
            RssVersion::RSS1_0
        );
        assert_eq!(
            RssVersion::from_str("2.0").unwrap(),
            RssVersion::RSS2_0
        );
        assert!(RssVersion::from_str("3.0").is_err());
    }

    #[test]
    fn test_rss_version_display() {
        assert_eq!(format!("{}", RssVersion::RSS0_90), "0.90");
        assert_eq!(format!("{}", RssVersion::RSS0_91), "0.91");
        assert_eq!(format!("{}", RssVersion::RSS0_92), "0.92");
        assert_eq!(format!("{}", RssVersion::RSS1_0), "1.0");
        assert_eq!(format!("{}", RssVersion::RSS2_0), "2.0");
    }

    #[test]
    fn test_rss_data_set_methods() {
        let rss_data = RssData::new(None)
            .atom_link("https://example.com/atom")
            .author("John Doe")
            .category("Technology")
            .copyright("© 2024 Example Inc.")
            .description("A sample RSS feed")
            .docs("https://example.com/rss-docs")
            .generator("RSS Gen v1.0")
            .guid("unique-guid-123")
            .image_title("Feed Image")
            .image_url("https://example.com/image.jpg")
            .image_link("https://example.com")
            .language("en-US")
            .last_build_date("2024-03-21T12:00:00Z")
            .link("https://example.com")
            .managing_editor("editor@example.com")
            .pub_date("2024-03-21T00:00:00Z")
            .title("Sample Feed")
            .ttl("60")
            .webmaster("webmaster@example.com");

        assert_eq!(rss_data.atom_link, "https://example.com/atom");
        assert_eq!(rss_data.author, "John Doe");
        assert_eq!(rss_data.category, "Technology");
        assert_eq!(rss_data.copyright, "© 2024 Example Inc.");
        assert_eq!(rss_data.description, "A sample RSS feed");
        assert_eq!(rss_data.docs, "https://example.com/rss-docs");
        assert_eq!(rss_data.generator, "RSS Gen v1.0");
        assert_eq!(rss_data.guid, "unique-guid-123");
        assert_eq!(rss_data.image_title, "Feed Image");
        assert_eq!(rss_data.image_url, "https://example.com/image.jpg");
        assert_eq!(rss_data.image_link, "https://example.com");
        assert_eq!(rss_data.language, "en-US");
        assert_eq!(rss_data.last_build_date, "2024-03-21T12:00:00Z");
        assert_eq!(rss_data.link, "https://example.com");
        assert_eq!(rss_data.managing_editor, "editor@example.com");
        assert_eq!(rss_data.pub_date, "2024-03-21T00:00:00Z");
        assert_eq!(rss_data.title, "Sample Feed");
        assert_eq!(rss_data.ttl, "60");
        assert_eq!(rss_data.webmaster, "webmaster@example.com");
    }

    #[test]
    fn test_rss_data_empty() {
        let rss_data = RssData::new(None);
        assert!(rss_data.title.is_empty());
        assert!(rss_data.link.is_empty());
        assert!(rss_data.description.is_empty());
        assert_eq!(rss_data.items.len(), 0);
    }

    #[test]
    fn test_rss_item_empty() {
        let item = RssItem::new();
        assert!(item.title.is_empty());
        assert!(item.link.is_empty());
        assert!(item.guid.is_empty());
        assert!(item.description.is_empty());
    }

    #[test]
    fn test_rss_data_to_hash_map() {
        let rss_data = RssData::new(None)
            .title("Test Feed")
            .link("https://example.com")
            .description("A test feed");

        let hash_map = rss_data.to_hash_map();
        assert_eq!(hash_map.get("title").unwrap(), "Test Feed");
        assert_eq!(
            hash_map.get("link").unwrap(),
            "https://example.com"
        );
        assert_eq!(hash_map.get("description").unwrap(), "A test feed");
    }

    #[test]
    fn test_rss_data_version_setter() {
        let rss_data = RssData::new(None).version(RssVersion::RSS1_0);
        assert_eq!(rss_data.version, RssVersion::RSS1_0);
    }

    #[test]
    fn test_remove_item_not_found() {
        let mut rss_data = RssData::new(None);
        let item = RssItem::new().guid("existing-guid");
        rss_data.add_item(item);

        // Try removing an item with a non-existent GUID
        let removed = rss_data.remove_item("non-existent-guid");
        assert!(!removed);
        assert_eq!(rss_data.items.len(), 1);
    }

    #[test]
    fn test_set_item_field_empty_items() {
        let mut rss_data = RssData::new(None);
        rss_data.set_item_field(RssItemField::Title, "Test Item Title");

        assert_eq!(rss_data.items.len(), 1);
        assert_eq!(rss_data.items[0].title, "Test Item Title");
    }

    #[test]
    fn test_set_image_empty() {
        let mut rss_data = RssData::new(None);
        rss_data.set_image(
            "".to_string(),
            "".to_string(),
            "".to_string(),
        );

        assert!(rss_data.image_title.is_empty());
        assert!(rss_data.image_url.is_empty());
        assert!(rss_data.image_link.is_empty());
    }

    #[test]
    fn test_rss_item_set_empty_field() {
        let item = RssItem::new().set(RssItemField::Title, "");
        assert!(item.title.is_empty());
    }
}
