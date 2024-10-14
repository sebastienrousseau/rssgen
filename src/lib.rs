// Copyright © 2024 RSS Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

// src/lib.rs

#![doc = include_str!("../README.md")]
#![doc(
    html_favicon_url = "https://kura.pro/rssgen/images/favicon.ico",
    html_logo_url = "https://kura.pro/rssgen/images/logos/rssgen.svg",
    html_root_url = "https://docs.rs/rss-gen"
)]
#![crate_name = "rss_gen"]
#![crate_type = "lib"]

/// Contains the main types and data structures used to represent RSS feeds.
pub mod data;
/// Defines error types used throughout the library.
pub mod error;
/// Implements RSS feed generation functionality.
pub mod generator;
/// Provides procedural macros for simplifying RSS operations.
pub mod macros;
/// Implements RSS feed parsing functionality.
pub mod parser;
/// Provides utilities for validating RSS feeds.
pub mod validator;

pub use data::{RssData, RssItem, RssVersion};
pub use error::{Result, RssError};
pub use generator::generate_rss;
pub use parser::parse_rss;
use url::Url;

/// The current version of the rss-gen crate, set at compile-time from Cargo.toml.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Maximum lengths for title fields in the RSS feed.
pub const MAX_TITLE_LENGTH: usize = 256;
/// Maximum lengths for link fields in the RSS feed.
pub const MAX_LINK_LENGTH: usize = 2048;
/// Maximum lengths for description fields in the RSS feed.
pub const MAX_DESCRIPTION_LENGTH: usize = 100_000;
/// Maximum lengths for general fields in the RSS feed.
pub const MAX_GENERAL_LENGTH: usize = 1024;
/// Maximum lengths for feed fields in the RSS feed.
pub const MAX_FEED_SIZE: usize = 1_048_576;

/// A convenience function to generate a minimal valid RSS 2.0 feed.
///
/// This function creates an RSS 2.0 feed with the provided title, link, and description,
/// and includes one example item.
///
/// # Arguments
///
/// * `title` - The title of the RSS feed.
/// * `link` - The link to the website associated with the RSS feed.
/// * `description` - A brief description of the RSS feed.
///
/// # Returns
///
/// A `Result` containing the generated RSS feed as a `String` if successful,
/// or an `RssError` if generation fails.
///
/// # Examples
///
/// ```rust
/// use rss_gen::quick_rss;
///
/// let rss = quick_rss(
///     "My Rust Blog",
///     "https://myrustblog.com",
///     "A blog about Rust programming"
/// );
///
/// match rss {
///     Ok(feed) => println!("Generated RSS feed: {}", feed),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
///
/// # Security
///
/// This function performs basic input validation, but it's recommended to sanitize
/// the input parameters before passing them to this function, especially if they
/// come from untrusted sources.
#[must_use = "This function returns a Result that should be handled"]
pub fn quick_rss(
    title: &str,
    link: &str,
    description: &str,
) -> Result<String> {
    // Validate input
    if title.is_empty() || link.is_empty() || description.is_empty() {
        return Err(RssError::InvalidInput(
            "Title, link, and description must not be empty"
                .to_string(),
        ));
    }

    if title.len() > MAX_TITLE_LENGTH {
        return Err(RssError::InvalidInput(format!(
            "Title exceeds maximum allowed length of {} characters",
            MAX_TITLE_LENGTH
        )));
    }

    if link.len() > MAX_LINK_LENGTH {
        return Err(RssError::InvalidInput(format!(
            "Link exceeds maximum allowed length of {} characters",
            MAX_LINK_LENGTH
        )));
    }

    if description.len() > MAX_DESCRIPTION_LENGTH {
        return Err(RssError::InvalidInput(
            format!("Description exceeds maximum allowed length of {} characters", MAX_DESCRIPTION_LENGTH)
        ));
    }

    if Url::parse(link).is_err() {
        return Err(RssError::InvalidInput(
            "Invalid URL format".to_string(),
        ));
    }

    let mut rss_data = RssData::new(Some(RssVersion::RSS2_0))
        .title(title)
        .link(link)
        .description(description);

    // Add an example item
    rss_data.add_item(
        RssItem::new()
            .title("Example Item")
            .link(format!("{}/example-item", link))
            .description("This is an example item in the RSS feed"),
    );

    generate_rss(&rss_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quick_rss() {
        let result = quick_rss(
            "Test Feed",
            "https://example.com",
            "A test RSS feed",
        );
        assert!(result.is_ok());
        let feed = result.unwrap();
        assert!(feed.contains("<title>Test Feed</title>"));
        assert!(feed.contains("<link>https://example.com</link>"));
        assert!(
            feed.contains("<description>A test RSS feed</description>")
        );
        assert!(feed.contains("<item>"));
        assert!(feed.contains("<title>Example Item</title>"));
        assert!(feed
            .contains("<link>https://example.com/example-item</link>"));
        assert!(feed.contains("<description>This is an example item in the RSS feed</description>"));
    }

    #[test]
    fn test_quick_rss_invalid_input() {
        let result =
            quick_rss("", "https://example.com", "Description");
        assert!(result.is_err());
        assert!(matches!(result, Err(RssError::InvalidInput(_))));

        let result = quick_rss("Title", "not-a-url", "Description");
        assert!(result.is_err());
        assert!(matches!(result, Err(RssError::InvalidInput(_))));
    }

    #[test]
    fn test_version_constant() {
        assert!(VERSION.starts_with(char::is_numeric));
        assert!(VERSION.split('.').count() >= 2);
    }

    #[test]
    fn test_quick_rss_max_title_length() {
        let long_title = "a".repeat(MAX_TITLE_LENGTH + 1);
        let result = quick_rss(
            &long_title,
            "https://example.com",
            "Description",
        );
        assert!(result.is_err());
        assert!(matches!(result, Err(RssError::InvalidInput(_))));

        let max_title = "a".repeat(MAX_TITLE_LENGTH);
        let result =
            quick_rss(&max_title, "https://example.com", "Description");
        assert!(result.is_ok());
    }

    #[test]
    fn test_quick_rss_max_link_length() {
        let long_link = format!(
            "https://example.com/{}",
            "a".repeat(MAX_LINK_LENGTH - 19)
        );
        let result = quick_rss("Title", &long_link, "Description");
        assert!(result.is_err());
        assert!(matches!(result, Err(RssError::InvalidInput(_))));

        let max_link = format!(
            "https://example.com/{}",
            "a".repeat(MAX_LINK_LENGTH - 20)
        );
        let result = quick_rss("Title", &max_link, "Description");
        assert!(result.is_ok());
    }

    #[test]
    fn test_quick_rss_max_description_length() {
        let long_description = "a".repeat(MAX_DESCRIPTION_LENGTH + 1);
        let result = quick_rss(
            "Title",
            "https://example.com",
            &long_description,
        );
        assert!(result.is_err());
        assert!(matches!(result, Err(RssError::InvalidInput(_))));

        let max_description = "a".repeat(MAX_DESCRIPTION_LENGTH);
        let result =
            quick_rss("Title", "https://example.com", &max_description);
        assert!(result.is_ok());
    }

    #[test]
    fn test_quick_rss_https() {
        let result = quick_rss(
            "Test Feed",
            "https://example.com",
            "A test RSS feed",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_quick_rss_http() {
        let result = quick_rss(
            "Test Feed",
            "http://example.com",
            "A test RSS feed",
        );
        assert!(result.is_ok());
    }

    // Note: The following tests depend on the implementation of RssData and its methods,
    // which are not shown in the provided code. You may need to adjust these tests
    // based on your actual implementation.

    #[test]
    fn test_rss_data_validate_size() {
        let mut rss_data = RssData::new(Some(RssVersion::RSS2_0))
            .title("Test Feed")
            .link("https://example.com")
            .description("A test RSS feed");

        // Add items until we exceed MAX_FEED_SIZE
        let item_content = "a".repeat(10000); // Increased content size
        for _ in 0..100 {
            // Reduced number of iterations
            rss_data.add_item(
                RssItem::new()
                    .title(&item_content)
                    .link("https://example.com/item")
                    .description(&item_content),
            );
        }

        assert!(rss_data.validate_size().is_err());
    }

    #[test]
    fn test_max_general_length() {
        let mut rss_data = RssData::new(Some(RssVersion::RSS2_0))
            .title("Test Feed")
            .link("https://example.com")
            .description("A test RSS feed");

        let long_general_field = "a".repeat(MAX_GENERAL_LENGTH + 1);
        rss_data.category = long_general_field.clone();

        assert!(rss_data.validate().is_err());

        rss_data.category = "a".repeat(MAX_GENERAL_LENGTH);
        assert!(rss_data.validate().is_ok());
    }
}
