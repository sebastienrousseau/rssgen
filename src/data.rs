// Copyright © 2024 RSS Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

// src/data.rs

use crate::error::{Result, RssError};
use dtt::datetime::DateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
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
    pub fn as_str(&self) -> &'static str {
        match self {
            RssVersion::RSS0_90 => "0.90",
            RssVersion::RSS0_91 => "0.91",
            RssVersion::RSS0_92 => "0.92",
            RssVersion::RSS1_0 => "1.0",
            RssVersion::RSS2_0 => "2.0",
        }
    }
}

impl Default for RssVersion {
    fn default() -> Self {
        RssVersion::RSS2_0
    }
}

impl fmt::Display for RssVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for RssVersion {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "0.90" => Ok(RssVersion::RSS0_90),
            "0.91" => Ok(RssVersion::RSS0_91),
            "0.92" => Ok(RssVersion::RSS0_92),
            "1.0" => Ok(RssVersion::RSS1_0),
            "2.0" => Ok(RssVersion::RSS2_0),
            _ => Err(s.to_string()),
        }
    }
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
    DateTime::parse(date_str)
        .map_err(|_| RssError::DateParseError(date_str.to_string()))
}

/// Represents the main structure for an RSS feed.
#[derive(
    Debug, Clone, PartialEq, Serialize, Deserialize, Default, Eq, Hash,
)]
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
    /// The image URL of the RSS feed.
    pub image: String,
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

/// Represents an item in the RSS feed.
#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
#[non_exhaustive]
pub struct RssItem {
    /// The GUID of the RSS item (unique identifier).
    pub guid: String,
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
    /// The category of the RSS item (optional).
    pub category: Option<String>,
    /// The comments URL related to the RSS item (optional).
    pub comments: Option<String>,
    /// The enclosure (typically for media like podcasts) (optional).
    pub enclosure: Option<String>,
    /// The source of the RSS item (optional).
    pub source: Option<String>,
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
        RssData {
            version: version.unwrap_or(RssVersion::RSS2_0),
            atom_link: String::new(),
            author: String::new(),
            category: String::new(),
            copyright: String::new(),
            description: String::new(),
            docs: String::new(),
            generator: String::new(),
            guid: String::new(),
            image: String::new(),
            language: String::new(),
            last_build_date: String::new(),
            link: String::new(),
            managing_editor: String::new(),
            pub_date: String::new(),
            title: String::new(),
            ttl: String::new(),
            webmaster: String::new(),
            items: Vec::new(),
        }
    }

    /// Sets the value of a specified field and returns the `RssData` instance for method chaining.
    ///
    /// # Arguments
    ///
    /// * `key` - The field to set.
    /// * `value` - The value to assign to the field.
    ///
    /// # Returns
    ///
    /// The updated `RssData` instance.
    pub fn set<T: Into<String>>(mut self, key: &str, value: T) -> Self {
        let value = value.into();
        match key {
            "atom_link" => self.atom_link = value,
            "author" => self.author = value,
            "category" => self.category = value,
            "copyright" => self.copyright = value,
            "description" => self.description = value,
            "docs" => self.docs = value,
            "generator" => self.generator = value,
            "image" => self.image = value,
            "language" => self.language = value,
            "last_build_date" => self.last_build_date = value,
            "link" => self.link = value,
            "managing_editor" => self.managing_editor = value,
            "pub_date" => self.pub_date = value,
            "title" => self.title = value,
            "ttl" => self.ttl = value,
            "webmaster" => self.webmaster = value,
            _ => eprintln!(
                "Warning: Attempt to set unknown field '{}'",
                key
            ),
        }
        self
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
        }
        if self.description.is_empty() {
            errors.push("Description is missing".to_string());
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
        map.insert("image".to_string(), self.image.clone());
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
        self.set("atom_link", value)
    }

    /// Sets the author.
    #[must_use]
    pub fn author<T: Into<String>>(self, value: T) -> Self {
        self.set("author", value)
    }

    /// Sets the category.
    #[must_use]
    pub fn category<T: Into<String>>(self, value: T) -> Self {
        self.set("category", value)
    }

    /// Sets the copyright.
    #[must_use]
    pub fn copyright<T: Into<String>>(self, value: T) -> Self {
        self.set("copyright", value)
    }

    /// Sets the description.
    #[must_use]
    pub fn description<T: Into<String>>(self, value: T) -> Self {
        self.set("description", value)
    }

    /// Sets the docs link.
    #[must_use]
    pub fn docs<T: Into<String>>(self, value: T) -> Self {
        self.set("docs", value)
    }

    /// Sets the generator.
    #[must_use]
    pub fn generator<T: Into<String>>(self, value: T) -> Self {
        self.set("generator", value)
    }

    /// Sets the GUID.
    #[must_use]
    pub fn guid<T: Into<String>>(self, value: T) -> Self {
        self.set("guid", value)
    }

    /// Sets the image URL.
    #[must_use]
    pub fn image<T: Into<String>>(self, value: T) -> Self {
        self.set("image", value)
    }

    /// Sets the language.
    #[must_use]
    pub fn language<T: Into<String>>(self, value: T) -> Self {
        self.set("language", value)
    }

    /// Sets the last build date.
    #[must_use]
    pub fn last_build_date<T: Into<String>>(self, value: T) -> Self {
        self.set("last_build_date", value)
    }

    /// Sets the main link.
    #[must_use]
    pub fn link<T: Into<String>>(self, value: T) -> Self {
        self.set("link", value)
    }

    /// Sets the managing editor.
    #[must_use]
    pub fn managing_editor<T: Into<String>>(self, value: T) -> Self {
        self.set("managing_editor", value)
    }

    /// Sets the publication date.
    #[must_use]
    pub fn pub_date<T: Into<String>>(self, value: T) -> Self {
        self.set("pub_date", value)
    }

    /// Sets the title.
    #[must_use]
    pub fn title<T: Into<String>>(self, value: T) -> Self {
        self.set("title", value)
    }

    /// Sets the TTL (Time To Live).
    #[must_use]
    pub fn ttl<T: Into<String>>(self, value: T) -> Self {
        self.set("ttl", value)
    }

    /// Sets the webmaster.
    #[must_use]
    pub fn webmaster<T: Into<String>>(self, value: T) -> Self {
        self.set("webmaster", value)
    }
}

impl RssItem {
    /// Creates a new `RssItem` with default values.
    #[must_use]
    pub fn new() -> Self {
        RssItem::default()
    }

    /// Sets the value of a field and returns the `RssItem` instance for method chaining.
    ///
    /// # Arguments
    ///
    /// * `key` - The field to set.
    /// * `value` - The value to assign to the field.
    pub fn set<T: Into<String>>(mut self, key: &str, value: T) -> Self {
        let value = value.into();
        match key {
            "guid" => self.guid = value,
            "description" => self.description = value,
            "link" => self.link = value,
            "pub_date" => self.pub_date = value,
            "title" => self.title = value,
            "author" => self.author = value,
            _ => eprintln!(
                "Warning: Attempt to set unknown field '{}'",
                key
            ),
        }
        self
    }

    /// Validates the `RssItem` to ensure all required fields are set and valid.
    pub fn validate(&self) -> Result<()> {
        let mut validation_errors = Vec::new();

        if self.title.is_empty() {
            validation_errors.push("Title is missing".to_string());
        }
        if self.link.is_empty() {
            validation_errors.push("Link is missing".to_string());
        }
        if self.guid.is_empty() {
            validation_errors.push("GUID is missing".to_string());
        }

        if !validation_errors.is_empty() {
            return Err(RssError::ValidationErrors(validation_errors));
        }

        Ok(())
    }

    // Field setter methods

    /// Sets the GUID.
    pub fn guid<T: Into<String>>(self, value: T) -> Self {
        self.set("guid", value)
    }

    /// Sets the description.
    pub fn description<T: Into<String>>(self, value: T) -> Self {
        self.set("description", value)
    }

    /// Sets the link.
    pub fn link<T: Into<String>>(self, value: T) -> Self {
        self.set("link", value)
    }

    /// Sets the publication date.
    pub fn pub_date<T: Into<String>>(self, value: T) -> Self {
        self.set("pub_date", value)
    }

    /// Sets the title.
    pub fn title<T: Into<String>>(self, value: T) -> Self {
        self.set("title", value)
    }

    /// Sets the author.
    pub fn author<T: Into<String>>(self, value: T) -> Self {
        self.set("author", value)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rss_data_new_and_set() {
        let rss_data = RssData::new(None)
            .title("Test RSS Feed")
            .link("https://example.com")
            .description("A test RSS feed")
            .version(RssVersion::RSS2_0);

        assert_eq!(rss_data.title, "Test RSS Feed");
        assert_eq!(rss_data.link, "https://example.com");
        assert_eq!(rss_data.description, "A test RSS feed");
        assert_eq!(rss_data.version, RssVersion::RSS2_0);
    }

    #[test]
    fn test_rss_data_validate() {
        let valid_rss_data = RssData::new(None)
            .title("Test RSS Feed")
            .link("https://example.com")
            .description("A test RSS feed");

        assert!(valid_rss_data.validate().is_ok());

        let invalid_rss_data = RssData::new(None)
            .title("Test RSS Feed")
            .description("A test RSS feed");

        assert!(invalid_rss_data.validate().is_err());
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
    fn test_rss_version() {
        assert_eq!(RssVersion::RSS2_0.as_str(), "2.0");
        assert_eq!(RssVersion::default(), RssVersion::RSS2_0);
        assert_eq!(RssVersion::RSS1_0.to_string(), "1.0");
        assert_eq!("2.0".parse::<RssVersion>(), Ok(RssVersion::RSS2_0));
        assert!("3.0".parse::<RssVersion>().is_err());
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("not a url").is_err());
    }

    #[test]
    fn test_parse_date() {
        assert!(parse_date("2024-03-21T12:00:00Z").is_ok());
        assert!(parse_date("invalid date").is_err());
    }
}
