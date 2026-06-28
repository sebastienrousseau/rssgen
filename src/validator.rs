// Copyright © 2024 RSS Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! RSS feed validator module
//!
//! This module provides functionality to validate RSS feeds, ensuring they
//! conform to the specified RSS version standards and contain valid data.

use crate::data::{
    parse_date as parse_rss_date, validate_link_field, RssData,
    RssVersion,
};
use crate::error::{Result, RssError, ValidationError};
use time::OffsetDateTime;
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
    #[must_use]
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
    ///
    /// # Errors
    ///
    /// This function returns an `Err(RssError::ValidationErrors)` if any validation checks fail.
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
            // Preserve the structured `ValidationError { field, message }`
            // shape rather than flattening to `Vec<String>` — callers
            // (CI gates, IDE integrations, JSON error responses) can
            // now match on `e.field` instead of parsing strings.
            Err(RssError::ValidationErrors(errors))
        }
    }

    /// Validates the base `RssData` structure.
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
        Self::validate_url(&self.rss_data.link, "channel.link", errors);

        for (index, item) in self.rss_data.items.iter().enumerate() {
            // RSS 2.0 §5.7 allows item links to be relative — and an
            // <item> isn't required to have a <link> at all so long as
            // it carries a <title> or <description>. Skip the strict
            // absolute-URL check for empty links and delegate the
            // populated case to `validate_link_field` so we stay
            // aligned with `RssData::validate` / `RssItem::validate`.
            if item.link.is_empty() {
                continue;
            }
            if let Err(e) = validate_link_field(&item.link) {
                errors.push(ValidationError::new(
                    format!("item.{index}.link"),
                    format!("Invalid item.{index}.link: {e}"),
                ));
            }
        }

        if self.rss_data.items.is_empty() {
            errors.push(ValidationError::new(
                "items",
                "RSS feed must contain at least one item",
            ));
        }

        self.validate_guids(errors);
        self.validate_atom_link(errors);
    }

    /// Validates that all GUIDs in the feed are unique.
    fn validate_guids(&self, errors: &mut Vec<ValidationError>) {
        let mut guids = std::collections::HashSet::new();
        for item in &self.rss_data.items {
            if !guids.insert(&item.guid) {
                errors.push(ValidationError::new(
                    "guid",
                    format!("Duplicate GUID found: {}", item.guid),
                ));
            }
        }
    }

    /// Validates the presence of atom:link for RSS 2.0 feeds.
    fn validate_atom_link(&self, errors: &mut Vec<ValidationError>) {
        if self.rss_data.version == RssVersion::RSS2_0
            && self.rss_data.atom_link.is_empty()
        {
            errors.push(ValidationError::new(
                "atom_link",
                "atom:link is required for RSS 2.0 feeds",
            ));
        }
    }

    /// Validates individual items in the RSS feed.
    fn validate_items(&self, errors: &mut Vec<ValidationError>) {
        for (index, item) in self.rss_data.items.iter().enumerate() {
            if let Err(e) = item.validate() {
                errors.push(ValidationError::new(
                    format!("item[{index}]"),
                    format!("Item validation failed: {e}"),
                ));
            }
        }
    }

    /// Validates all dates in the RSS feed.
    fn validate_dates(&self, errors: &mut Vec<ValidationError>) {
        Self::validate_date(&self.rss_data.pub_date, "pubDate", errors);
        Self::validate_date(
            &self.rss_data.last_build_date,
            "lastBuildDate",
            errors,
        );

        for (index, item) in self.rss_data.items.iter().enumerate() {
            Self::validate_date(
                &item.pub_date,
                &format!("item[{index}].pubDate"),
                errors,
            );
        }
    }

    /// Validates a single date string.
    fn validate_date(
        date_str: &str,
        field: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        if !date_str.is_empty() {
            if let Err(e) = Self::parse_date(date_str) {
                errors.push(ValidationError::new(
                    field,
                    format!("Invalid date format: {e}"),
                ));
            }
        }
    }

    /// Parses a date string into a [`time::OffsetDateTime`].
    ///
    /// Delegates to [`crate::data::parse_date`], which accepts both
    /// RFC 2822 (the historical RSS 2.0 wire format — any timezone is
    /// accepted, including `+0000`, `+0530`, `EST`, etc.) and ISO 8601
    /// (used by Atom and Dublin Core). The previous implementation
    /// hard-required a literal `" GMT"` suffix, rejecting every
    /// spec-compliant feed produced outside of GMT.
    ///
    /// # Errors
    ///
    /// Returns [`RssError::DateParseError`] when the input matches
    /// neither RFC 2822 nor ISO 8601.
    pub fn parse_date(date_str: &str) -> Result<OffsetDateTime> {
        parse_rss_date(date_str)
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
            RssVersion::RSS1_0
                if self
                    .rss_data
                    .items
                    .iter()
                    .any(|item| item.guid.is_empty()) =>
            {
                errors.push(ValidationError {
                    field: "guid".to_string(),
                    message: "All items must have a guid in RSS 1.0"
                        .to_string(),
                });
            }
            _ => {}
        }
    }

    /// Validates a URL string.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL string to validate.
    /// * `field` - The field name for error reporting.
    /// * `errors` - A mutable vector to collect validation errors.
    fn validate_url(
        url: &str,
        field: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        if url.len() > MAX_URL_LENGTH {
            errors.push(ValidationError {
                field: field.to_string(),
                message: format!(
                    "URL exceeds maximum length of {MAX_URL_LENGTH} characters"
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
                        message: format!("Invalid URL scheme in {field}: {url}. Only HTTP and HTTPS are allowed."),
                    });
                }
            }
            Err(_) => {
                errors.push(ValidationError {
                    field: field.to_string(),
                    message: format!("Invalid URL in {field}: {url}"),
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
///
/// # Errors
///
/// This function returns an `Err(RssError::ValidationErrors)` if any validation checks fail.
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
            assert!(errors.iter().any(|e| e.field == "atom_link"
                && e.message.contains("atom:link is required")));
            assert!(errors.iter().any(|e| e.field == "items"
                && e.message.contains(
                    "RSS feed must contain at least one item"
                )));
            assert!(errors.iter().any(|e| e.field == "pubDate"
                && e.message.contains("Invalid date format")));
        } else {
            panic!("Expected ValidationErrors");
        }
    }

    #[test]
    fn test_validate_url_valid() {
        let rss_data = RssData::new(None);
        let mut errors = Vec::new();

        RssFeedValidator::validate_url(
            "https://example.com",
            "test",
            &mut errors,
        );
        RssFeedValidator::validate_url(
            "http://example.com",
            "test",
            &mut errors,
        );
        RssFeedValidator::validate_url(
            "https://sub.example.com/path?query=value",
            "test",
            &mut errors,
        );

        assert!(errors.is_empty());
        assert!(rss_data.link.is_empty());
    }

    #[test]
    fn test_validate_url_invalid() {
        let mut errors = Vec::new();

        RssFeedValidator::validate_url(
            "not a url",
            "test",
            &mut errors,
        );
        RssFeedValidator::validate_url(
            "ftp://example.com",
            "test",
            &mut errors,
        );
        RssFeedValidator::validate_url("http://", "test", &mut errors);
        RssFeedValidator::validate_url("https://", "test", &mut errors);
        RssFeedValidator::validate_url(
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
                "Unexpected errors for version {version:?}"
            );
        }
    }

    #[test]
    fn test_parse_date_valid() {
        let valid_date = "Mon, 01 Jan 2024 00:00:00 GMT";
        assert!(RssFeedValidator::parse_date(valid_date).is_ok());
    }

    #[test]
    fn test_parse_date_invalid() {
        let invalid_date = "Invalid Date";
        assert!(RssFeedValidator::parse_date(invalid_date).is_err());
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
        assert!(
            errors[0].message.contains("channel.title is missing"),
            "expected `channel.title is missing`, got: {:?}",
            errors[0].message
        );
    }

    #[test]
    fn test_validate_items_with_invalid_item() {
        let mut rss_data = RssData::new(Some(RssVersion::RSS2_0))
            .title("Test Feed")
            .link("https://example.com")
            .description("A test feed")
            .atom_link("https://example.com/feed.xml")
            .generator("Test");

        // Add an item missing required fields (title, link, description)
        rss_data.add_item(RssItem::new().guid("guid1"));

        let validator = RssFeedValidator::new(&rss_data);
        let mut errors = Vec::new();
        validator.validate_items(&mut errors);

        assert!(!errors.is_empty(), "Expected item validation errors");
        assert!(errors[0].field.contains("item[0]"));
        assert!(errors[0].message.contains("Item validation failed"));
    }

    #[test]
    fn test_validate_dates_with_invalid_item_date() {
        let mut rss_data = RssData::new(Some(RssVersion::RSS2_0))
            .title("Test Feed")
            .link("https://example.com")
            .description("A test feed")
            .atom_link("https://example.com/feed.xml")
            .pub_date("Mon, 01 Jan 2024 00:00:00 GMT")
            .generator("Test");

        rss_data.add_item(
            RssItem::new()
                .title("Item")
                .link("https://example.com/item")
                .description("Desc")
                .guid("guid1")
                .pub_date("not a valid date"),
        );

        let validator = RssFeedValidator::new(&rss_data);
        let mut errors = Vec::new();
        validator.validate_dates(&mut errors);

        assert!(!errors.is_empty(), "Expected date validation errors");
        assert!(errors.iter().any(|e| e.field == "item[0].pubDate"));
    }

    #[test]
    fn test_validate_url_exceeds_max_length() {
        let mut errors = Vec::new();
        let long_url = format!(
            "https://example.com/{}",
            "a".repeat(MAX_URL_LENGTH)
        );

        RssFeedValidator::validate_url(&long_url, "test", &mut errors);

        assert_eq!(errors.len(), 1);
        assert!(errors[0]
            .message
            .contains("URL exceeds maximum length"));
    }

    #[test]
    fn test_validate_structure_with_invalid_item_link() {
        let mut rss_data = RssData::new(Some(RssVersion::RSS2_0))
            .title("Test Feed")
            .link("https://example.com")
            .description("A test feed")
            .atom_link("https://example.com/feed.xml");

        rss_data.add_item(
            RssItem::new()
                .title("Item")
                .link("bad url with spaces")
                .description("Desc")
                .guid("guid1"),
        );

        let validator = RssFeedValidator::new(&rss_data);
        let mut errors = Vec::new();
        validator.validate_structure(&mut errors);

        assert!(errors.iter().any(|e| e.field == "item.0.link"
            && e.message.contains("Invalid item.0.link")));
    }

    #[test]
    fn test_validate_structure_allows_empty_item_link() {
        // RSS 2.0 §5.7 — an <item> with a <title> and <description>
        // does NOT require a <link>. Empty item.link must NOT fail
        // structural validation. Regression: pre-v0.0.6 the validator
        // called Url::parse("") which always errored.
        let mut rss_data = RssData::new(Some(RssVersion::RSS2_0))
            .title("Test Feed")
            .link("https://example.com")
            .description("A test feed")
            .atom_link("https://example.com/feed.xml");

        rss_data.add_item(
            RssItem::new()
                .title("Item")
                .description("Body only — no link")
                .guid("guid-no-link"),
        );

        let validator = RssFeedValidator::new(&rss_data);
        let mut errors = Vec::new();
        validator.validate_structure(&mut errors);

        // The empty item.link must NOT have produced a structural error.
        // The single item has index 0 — assert that no error fires
        // against `item.0.link` specifically.
        assert!(
            !errors.iter().any(|e| e.field == "item.0.link"),
            "empty item.link should be accepted, got: {errors:?}"
        );
    }

    #[test]
    fn test_validate_structure_allows_relative_item_link() {
        // RSS 2.0 §5.7 also allows relative URLs at the item level.
        let mut rss_data = RssData::new(Some(RssVersion::RSS2_0))
            .title("Test Feed")
            .link("https://example.com")
            .description("A test feed")
            .atom_link("https://example.com/feed.xml");

        rss_data.add_item(
            RssItem::new()
                .title("Item")
                .link("/tags/")
                .description("Tag index")
                .guid("guid-tags"),
        );

        let validator = RssFeedValidator::new(&rss_data);
        let mut errors = Vec::new();
        validator.validate_structure(&mut errors);
        assert!(
            !errors.iter().any(|e| e.field == "item.0.link"),
            "relative item.link should be accepted, got: {errors:?}"
        );
    }

    #[test]
    fn test_parse_date_accepts_numeric_timezone_offset() {
        // RFC 2822 / RFC 1123 allow numeric timezone offsets. The
        // pre-v0.0.6 implementation hard-required the literal `" GMT"`
        // suffix and rejected every offset-based date — a hard P0
        // spec violation.
        assert!(RssFeedValidator::parse_date(
            "Sun, 28 Jun 2026 00:12:20 +0000"
        )
        .is_ok());
        assert!(RssFeedValidator::parse_date(
            "Sat, 27 Jun 2026 19:12:20 -0500"
        )
        .is_ok());
    }

    #[test]
    fn test_parse_date_accepts_iso8601() {
        // Atom and Dublin Core date wire formats — must also flow
        // through the same parse helper.
        assert!(RssFeedValidator::parse_date("2026-06-28T00:12:20Z")
            .is_ok());
    }

    #[test]
    fn test_parse_date_no_longer_requires_gmt_suffix() {
        // Regression: pre-v0.0.6 this string failed with "missing GMT".
        // Today it's a perfectly compliant RFC 2822 instant and must
        // round-trip.
        assert!(RssFeedValidator::parse_date(
            "Mon, 01 Jan 2024 00:00:00 +0000"
        )
        .is_ok());
    }

    #[test]
    fn test_validate_rss_feed_convenience_function() {
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

        assert!(validate_rss_feed(&rss_data).is_ok());
    }

    #[test]
    fn test_validate_rss_feed_convenience_function_invalid() {
        let rss_data = RssData::new(Some(RssVersion::RSS2_0));
        assert!(validate_rss_feed(&rss_data).is_err());
    }
}
