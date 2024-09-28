// Copyright © 2024 RSS Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

// src/validator.rs

use crate::data::{RssData, RssVersion};
use crate::error::{Result, RssError, ValidationError};
use dtt::datetime::DateTime;

/// RSS feed validator for validating the structure and content of an RSS feed.
pub struct RssFeedValidator<'a> {
    rss_data: &'a RssData,
}

impl<'a> RssFeedValidator<'a> {
    /// Creates a new `RssFeedValidator` instance with the provided `RssData`.
    pub fn new(rss_data: &'a RssData) -> Self {
        RssFeedValidator { rss_data }
    }
    /// Validates the RSS feed structure and content.
    pub fn validate(&self) -> Result<()> {
        let mut errors = Vec::new();

        // Validate RssData
        if let Err(e) = self.rss_data.validate() {
            println!("RssData validation error: {:?}", e);
            errors.push(ValidationError {
                field: "rss_data".to_string(),
                message: e.to_string(),
            });
        }

        // Validate overall structure
        println!("    ✅  Validating structure...");
        self.validate_structure(&mut errors);

        // Validate items
        println!("    ✅  Validating items...");
        self.validate_items(&mut errors);

        // Validate dates
        println!("    ✅  Validating dates...");
        self.validate_dates(&mut errors);

        // Validate version-specific requirements
        println!("    ✅  Validating version-specific...");
        self.validate_version_specific(&mut errors);

        if errors.is_empty() {
            println!("    ✅  Validation passed!");
            Ok(())
        } else {
            println!("Validation failed with errors: {:?}", errors);
            Err(RssError::ValidationErrors(
                errors.into_iter().map(|e| e.to_string()).collect(),
            ))
        }
    }

    fn validate_structure(&self, errors: &mut Vec<ValidationError>) {
        if self.rss_data.items.is_empty() {
            errors.push(ValidationError {
                field: "items".to_string(),
                message: "RSS feed must contain at least one item"
                    .to_string(),
            });
        }

        // Check for duplicate GUIDs
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

        // Validate that atom:link is present for RSS 2.0
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

    fn validate_dates(&self, errors: &mut Vec<ValidationError>) {
        if let Err(e) =
            self.parse_date(&self.rss_data.pub_date, "pubDate")
        {
            errors.push(e);
        }
        if let Err(e) = self
            .parse_date(&self.rss_data.last_build_date, "lastBuildDate")
        {
            errors.push(e);
        }

        for (index, item) in self.rss_data.items.iter().enumerate() {
            if let Err(e) = self.parse_date(
                &item.pub_date,
                &format!("item[{}].pubDate", index),
            ) {
                errors.push(e);
            }
        }
    }

    fn parse_date(
        &self,
        date_str: &str,
        field: &str,
    ) -> std::result::Result<DateTime, ValidationError> {
        if !date_str.is_empty() {
            // Define the custom RSS date format without the fixed "GMT"
            let rss_date_format = "[weekday repr:short], [day] [month repr:short] [year] [hour]:[minute]:[second]";

            // Use strip_suffix to handle " GMT"
            if let Some(date_without_gmt) =
                date_str.strip_suffix(" GMT")
            {
                match DateTime::parse_custom_format(
                    date_without_gmt,
                    rss_date_format,
                ) {
                    Ok(mut date) => {
                        // Manually set the UTC offset to "GMT"
                        date.offset = time::UtcOffset::UTC;
                        Ok(date)
                    }
                    Err(_) => {
                        println!(
                            "Failed to parse date for field: {}",
                            field
                        );
                        Err(ValidationError {
                            field: field.to_string(),
                            message: format!(
                                "Invalid date format: {}",
                                date_str
                            ),
                        })
                    }
                }
            } else {
                Err(ValidationError {
                    field: field.to_string(),
                    message: format!(
                        "Invalid date format (missing GMT): {}",
                        date_str
                    ),
                })
            }
        } else {
            Ok(DateTime::now()) // Return current date if empty
        }
    }

    fn validate_version_specific(
        &self,
        errors: &mut Vec<ValidationError>,
    ) {
        match self.rss_data.version {
            RssVersion::RSS2_0 => {
                // RSS 2.0 specific validations
                if self.rss_data.generator.is_empty() {
                    errors.push(ValidationError {
                        field: "generator".to_string(),
                        message:
                            "generator is recommended for RSS 2.0 feeds"
                                .to_string(),
                    });
                }
            }
            RssVersion::RSS1_0 => {
                // RSS 1.0 specific validations
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
            _ => {
                // Other version specific validations can be added here
            }
        }
    }
}

/// Validates the provided `RssData` and returns a `Result` indicating success or failure.
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

    // Add more tests as needed
}
