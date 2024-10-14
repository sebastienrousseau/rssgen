// Copyright © 2024 RSS Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! RSS feed validator module
//!
//! This module provides functionality to validate RSS feeds, ensuring they
//! conform to the specified RSS version standards and contain valid data.

use crate::data::{RssData, RssVersion};
use crate::error::{Result, RssError, ValidationError};
use dtt::datetime::DateTime;
use url::Url;

/// Maximum allowed length for URL strings
const MAX_URL_LENGTH: usize = 2000;

/// RSS feed validator for validating the structure and content of an RSS feed.
#[derive(Debug)]
pub struct RssFeedValidator<'a> {
    rss_data: &'a RssData,
}

impl<'a> RssFeedValidator<'a> {
    /// Creates a new `RssFeedValidator` instance with the provided `RssData`.
    ///
    /// # Arguments
    ///
    /// * `rss_data` - A reference to the `RssData` to be validated.
    ///
    /// # Returns
    ///
    /// A new instance of `RssFeedValidator`.
    pub fn new(rss_data: &'a RssData) -> Self {
        RssFeedValidator { rss_data }
    }

    /// Validates the RSS feed structure and content.
    ///
    /// This method performs a comprehensive validation of the RSS feed,
    /// including structure, items, dates, and version-specific requirements.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the validation passes.
    /// * `Err(RssError::ValidationErrors)` containing a list of validation errors if any are found.
    pub fn validate(&self) -> Result<()> {
        let mut errors = Vec::new();

        self.validate_rss_data(&mut errors);
        self.validate_structure(&mut errors);
        self.validate_items(&mut errors);
        self.validate_dates(&mut errors);
        self.validate_version_specific(&mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(RssError::ValidationErrors(
                errors.into_iter().map(|e| e.to_string()).collect(),
            ))
        }
    }

    /// Validates the base RssData structure.
    fn validate_rss_data(&self, errors: &mut Vec<ValidationError>) {
        if let Err(e) = self.rss_data.validate() {
            errors.push(ValidationError {
                field: "rss_data".to_string(),
                message: e.to_string(),
            });
        }
    }

    /// Validates the overall structure of the RSS feed.
    fn validate_structure(&self, errors: &mut Vec<ValidationError>) {
        self.validate_url(&self.rss_data.link, "channel link", errors);

        for (index, item) in self.rss_data.items.iter().enumerate() {
            self.validate_url(
                &item.link,
                &format!("item[{}] link", index),
                errors,
            );
        }

        if self.rss_data.items.is_empty() {
            errors.push(ValidationError {
                field: "items".to_string(),
                message: "RSS feed must contain at least one item"
                    .to_string(),
            });
        }

        self.validate_guids(errors);
        self.validate_atom_link(errors);
    }

    /// Validates that all GUIDs in the feed are unique.
    fn validate_guids(&self, errors: &mut Vec<ValidationError>) {
        let mut guids = std::collections::HashSet::new();
        for item in &self.rss_data.items {
            if !guids.insert(&item.guid) {
                errors.push(ValidationError {
                    field: "guid".to_string(),
                    message: format!(
                        "Duplicate GUID found: {}",
                        item.guid
                    ),
                });
            }
        }
    }

    /// Validates the presence of atom:link for RSS 2.0 feeds.
    fn validate_atom_link(&self, errors: &mut Vec<ValidationError>) {
        if self.rss_data.version == RssVersion::RSS2_0
            && self.rss_data.atom_link.is_empty()
        {
            errors.push(ValidationError {
                field: "atom_link".to_string(),
                message: "atom:link is required for RSS 2.0 feeds"
                    .to_string(),
            });
        }
    }

    /// Validates individual items in the RSS feed.
    fn validate_items(&self, errors: &mut Vec<ValidationError>) {
        for (index, item) in self.rss_data.items.iter().enumerate() {
            if let Err(e) = item.validate() {
                errors.push(ValidationError {
                    field: format!("item[{}]", index),
                    message: format!("Item validation failed: {}", e),
                });
            }
        }
    }

    /// Validates all dates in the RSS feed.
    fn validate_dates(&self, errors: &mut Vec<ValidationError>) {
        self.validate_date(&self.rss_data.pub_date, "pubDate", errors);
        self.validate_date(
            &self.rss_data.last_build_date,
            "lastBuildDate",
            errors,
        );

        for (index, item) in self.rss_data.items.iter().enumerate() {
            self.validate_date(
                &item.pub_date,
                &format!("item[{}].pubDate", index),
                errors,
            );
        }
    }

    /// Validates a single date string.
    fn validate_date(
        &self,
        date_str: &str,
        field: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        if !date_str.is_empty() {
            if let Err(e) = self.parse_date(date_str) {
                errors.push(ValidationError {
                    field: field.to_string(),
                    message: format!("Invalid date format: {}", e),
                });
            }
        }
    }

    /// Parses a date string into a DateTime object.
    fn parse_date(&self, date_str: &str) -> Result<DateTime> {
        // Define the custom RSS date format without the fixed "GMT"
        let rss_date_format = "[weekday repr:short], [day] [month repr:short] [year] [hour]:[minute]:[second]";

        // Use strip_suffix to handle " GMT"
        let date_without_gmt =
            date_str.strip_suffix(" GMT").ok_or_else(|| {
                RssError::DateParseError(format!(
                    "Invalid date format (missing GMT): {}",
                    date_str
                ))
            })?;

        let mut date = DateTime::parse_custom_format(
            date_without_gmt,
            rss_date_format,
        )
        .map_err(|_| {
            RssError::DateParseError(format!(
                "Failed to parse date: {}",
                date_str
            ))
        })?;

        // Manually set the UTC offset to "GMT"
        date.offset = time::UtcOffset::UTC;
        Ok(date)
    }

    /// Validates version-specific requirements of the RSS feed.
    fn validate_version_specific(
        &self,
        errors: &mut Vec<ValidationError>,
    ) {
        match self.rss_data.version {
            RssVersion::RSS2_0 => {
                if self.rss_data.generator.is_empty() {
                    errors.push(ValidationError {
                        field: "generator".to_string(),
                        message:
                            "generator is recommended for RSS 2.0 feeds"
                                .to_string(),
                    });
                }
                if self.rss_data.atom_link.is_empty() {
                    errors.push(ValidationError {
                        field: "atom_link".to_string(),
                        message:
                            "atom:link is required for RSS 2.0 feeds"
                                .to_string(),
                    });
                }
            }
            RssVersion::RSS1_0 => {
                if self
                    .rss_data
                    .items
                    .iter()
                    .any(|item| item.guid.is_empty())
                {
                    errors.push(ValidationError {
                        field: "guid".to_string(),
                        message:
                            "All items must have a guid in RSS 1.0"
                                .to_string(),
                    });
                }
            }
            RssVersion::RSS0_92
            | RssVersion::RSS0_91
            | RssVersion::RSS0_90 => {
                // Add specific checks for older RSS versions if needed
            }
        }
    }

    /// Validates a URL string.
    fn validate_url(
        &self,
        url: &str,
        field: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        if url.len() > MAX_URL_LENGTH {
            errors.push(ValidationError {
                field: field.to_string(),
                message: format!(
                    "URL exceeds maximum length of {} characters",
                    MAX_URL_LENGTH
                ),
            });
            return;
        }

        match Url::parse(url) {
            Ok(parsed_url) => {
                if parsed_url.scheme() != "http"
                    && parsed_url.scheme() != "https"
                {
                    errors.push(ValidationError {
                        field: field.to_string(),
                        message: format!("Invalid URL scheme in {}: {}. Only HTTP and HTTPS are allowed.", field, url),
                    });
                }
            }
            Err(_) => {
                errors.push(ValidationError {
                    field: field.to_string(),
                    message: format!(
                        "Invalid URL in {}: {}",
                        field, url
                    ),
                });
            }
        }
    }
}

/// Validates the provided `RssData` and returns a `Result` indicating success or failure.
///
/// # Arguments
///
/// * `rss_data` - A reference to the `RssData` to be validated.
///
/// # Returns
///
/// * `Ok(())` if the validation passes.
/// * `Err(RssError::ValidationErrors)` containing a list of validation errors if any are found.
pub fn validate_rss_feed(rss_data: &RssData) -> Result<()> {
    let validator = RssFeedValidator::new(rss_data);
    validator.validate()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::RssItem;

    #[test]
    fn test_valid_rss_feed() {
        let mut rss_data = RssData::new(Some(RssVersion::RSS2_0))
            .title("Test Feed")
            .link("https://example.com")
            .description("A test feed")
            .atom_link("https://example.com/feed.xml")
            .pub_date("Mon, 01 Jan 2024 00:00:00 GMT")
            .generator("RSS Gen Test");

        rss_data.add_item(
            RssItem::new()
                .title("Test Item")
                .link("https://example.com/item1")
                .description("A test item")
                .guid("unique-id-1")
                .pub_date("Mon, 01 Jan 2024 00:00:00 GMT"),
        );

        let validator = RssFeedValidator::new(&rss_data);
        assert!(validator.validate().is_ok());
    }

    #[test]
    fn test_invalid_rss_feed() {
        let rss_data = RssData::new(Some(RssVersion::RSS2_0))
            .title("Test Feed")
            .link("https://example.com")
            .description("A test feed")
            .pub_date("Invalid Date");

        let validator = RssFeedValidator::new(&rss_data);
        let result = validator.validate();
        assert!(result.is_err());
        if let Err(RssError::ValidationErrors(errors)) = result {
            assert!(errors
                .iter()
                .any(|e| e.contains("atom:link is required")));
            assert!(errors.iter().any(|e| e
                .contains("RSS feed must contain at least one item")));
            assert!(errors
                .iter()
                .any(|e| e.contains("Invalid date format")));
        } else {
            panic!("Expected ValidationErrors");
        }
    }

    #[test]
    fn test_validate_url_valid() {
        let rss_data = RssData::new(None);
        let validator = RssFeedValidator::new(&rss_data);
        let mut errors = Vec::new();

        validator.validate_url(
            "https://example.com",
            "test",
            &mut errors,
        );
        validator.validate_url(
            "http://example.com",
            "test",
            &mut errors,
        );
        validator.validate_url(
            "https://sub.example.com/path?query=value",
            "test",
            &mut errors,
        );

        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_url_invalid() {
        let rss_data = RssData::new(None);
        let validator = RssFeedValidator::new(&rss_data);
        let mut errors = Vec::new();

        validator.validate_url("not a url", "test", &mut errors);
        validator.validate_url(
            "ftp://example.com",
            "test",
            &mut errors,
        );
        validator.validate_url("http://", "test", &mut errors);
        validator.validate_url("https://", "test", &mut errors);
        validator.validate_url(
            "file:///path/to/file",
            "test",
            &mut errors,
        );

        assert_eq!(errors.len(), 5);
    }

    #[test]
    fn test_validate_structure_with_urls() {
        let mut rss_data = RssData::new(Some(RssVersion::RSS2_0))
            .title("Test Feed")
            .link("https://example.com")
            .description("A test feed")
            .atom_link("https://example.com/feed.xml");

        rss_data.add_item(
            RssItem::new()
                .title("Test Item")
                .link("https://example.com/item1")
                .description("A test item")
                .guid("unique-id-1"),
        );

        let validator = RssFeedValidator::new(&rss_data);
        let mut errors = Vec::new();
        validator.validate_structure(&mut errors);
        assert!(errors.is_empty());

        // Test with invalid URL
        rss_data.link = "not a url".to_string();
        let validator = RssFeedValidator::new(&rss_data);
        let mut errors = Vec::new();
        validator.validate_structure(&mut errors);
        assert!(errors
            .iter()
            .any(|e| e.message.contains("Invalid URL")));
    }

    #[test]
    fn test_validate_version_specific_rss2_0() {
        let mut rss_data = RssData::new(Some(RssVersion::RSS2_0))
            .title("Test Feed")
            .link("https://example.com")
            .description("A test feed")
            .atom_link("https://example.com/feed.xml")
            .generator("RSS Gen Test");

        let validator = RssFeedValidator::new(&rss_data);
        let mut errors = Vec::new();
        validator.validate_version_specific(&mut errors);
        assert!(errors.is_empty());

        // Test without generator
        rss_data.generator = String::new();
        let validator = RssFeedValidator::new(&rss_data);
        let mut errors = Vec::new();
        validator.validate_version_specific(&mut errors);
        assert!(errors
            .iter()
            .any(|e| e.message.contains("generator is recommended")));

        // Test without atom:link
        rss_data.atom_link = String::new();
        let validator = RssFeedValidator::new(&rss_data);
        let mut errors = Vec::new();
        validator.validate_version_specific(&mut errors);
        assert!(errors
            .iter()
            .any(|e| e.message.contains("atom:link is required")));
    }

    #[test]
    fn test_validate_version_specific_rss1_0() {
        let mut rss_data = RssData::new(Some(RssVersion::RSS1_0))
            .title("Test Feed")
            .link("https://example.com")
            .description("A test feed");

        rss_data.add_item(
            RssItem::new()
                .title("Test Item")
                .link("https://example.com/item1")
                .description("A test item")
                .guid("unique-id-1"),
        );

        let validator = RssFeedValidator::new(&rss_data);
        let mut errors = Vec::new();
        validator.validate_version_specific(&mut errors);
        assert!(errors.is_empty());

        // Test without guid
        rss_data.items[0].guid = String::new();
        let validator = RssFeedValidator::new(&rss_data);
        let mut errors = Vec::new();
        validator.validate_version_specific(&mut errors);
        assert!(errors.iter().any(|e| e
            .message
            .contains("All items must have a guid in RSS 1.0")));
    }

    #[test]
    fn test_validate_version_specific_older_versions() {
        for version in &[
            RssVersion::RSS0_90,
            RssVersion::RSS0_91,
            RssVersion::RSS0_92,
        ] {
            let rss_data = RssData::new(Some(*version))
                .title("Test Feed")
                .link("https://example.com")
                .description("A test feed");

            let validator = RssFeedValidator::new(&rss_data);
            let mut errors = Vec::new();
            validator.validate_version_specific(&mut errors);
            assert!(
                errors.is_empty(),
                "Unexpected errors for version {:?}",
                version
            );
        }
    }

    #[test]
    fn test_parse_date_valid() {
        let rss_data = RssData::new(None);
        let validator = RssFeedValidator::new(&rss_data);

        let valid_date = "Mon, 01 Jan 2024 00:00:00 GMT";
        assert!(validator.parse_date(valid_date).is_ok());
    }

    #[test]
    fn test_parse_date_invalid() {
        let rss_data = RssData::new(None);
        let validator = RssFeedValidator::new(&rss_data);

        let invalid_date = "Invalid Date";
        assert!(validator.parse_date(invalid_date).is_err());
    }

    #[test]
    fn test_validate_guids() {
        let mut rss_data = RssData::new(Some(RssVersion::RSS2_0))
            .title("Test Feed")
            .link("https://example.com")
            .description("A test feed");

        rss_data.add_item(RssItem::new().guid("guid1"));
        rss_data.add_item(RssItem::new().guid("guid2"));
        rss_data.add_item(RssItem::new().guid("guid1")); // Duplicate

        let validator = RssFeedValidator::new(&rss_data);
        let mut errors = Vec::new();
        validator.validate_guids(&mut errors);

        assert_eq!(errors.len(), 1);
        assert!(errors[0]
            .message
            .contains("Duplicate GUID found: guid1"));
    }

    #[test]
    fn test_validate_atom_link() {
        let rss_data = RssData::new(Some(RssVersion::RSS2_0))
            .title("Test Feed")
            .link("https://example.com")
            .description("A test feed");

        let validator = RssFeedValidator::new(&rss_data);
        let mut errors = Vec::new();
        validator.validate_atom_link(&mut errors);

        assert_eq!(errors.len(), 1);
        assert!(errors[0]
            .message
            .contains("atom:link is required for RSS 2.0 feeds"));

        let rss_data_with_atom =
            rss_data.atom_link("https://example.com/feed.xml");
        let validator = RssFeedValidator::new(&rss_data_with_atom);
        let mut errors = Vec::new();
        validator.validate_atom_link(&mut errors);

        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_rss_data() {
        let invalid_rss_data = RssData::new(Some(RssVersion::RSS2_0)); // Missing required fields

        let validator = RssFeedValidator::new(&invalid_rss_data);
        let mut errors = Vec::new();
        validator.validate_rss_data(&mut errors);

        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("Title is missing"));
    }
}
