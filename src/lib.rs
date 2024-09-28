// Copyright © 2024 RSS Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

// src/lib.rs

//! # RSS Gen
//!
//! `rss-gen` is a comprehensive Rust library for generating, parsing, serializing, and deserializing RSS feeds.
//! It supports multiple RSS versions, providing flexibility for creating and handling feeds across different formats.
//!
//! ## Features
//!
//! - Support for RSS versions 0.90, 0.91, 0.92, 1.0, and 2.0
//! - Generation of RSS feeds from structured data
//! - Parsing of existing RSS feeds into structured data
//! - Serialization and deserialization of RSS data
//! - Extensible elements for managing standard and optional RSS fields
//! - Atom link support for modern syndication compatibility
//! - Image embedding for RSS 2.0 feeds
//!
//! ## Examples
//!
//! ### Generating an RSS 2.0 feed
//!
//! ```rust
//! use rss_gen::{RssData, RssVersion, generate_rss};
//!
//! let rss_data = RssData::new(Some(RssVersion::RSS2_0))
//!     .title("My Rust Blog")
//!     .link("https://myrustblog.com")
//!     .description("A blog about Rust programming and tutorials.");
//!
//! match generate_rss(&rss_data) {
//!     Ok(rss_feed) => println!("Generated RSS feed: {}", rss_feed),
//!     Err(e) => eprintln!("Error generating RSS feed: {}", e),
//! }
//! ```
//!
//! ### Parsing an existing RSS feed
//!
//! ```rust
//! use rss_gen::parse_rss;
//!
//! let rss_content = r#"
//!     <?xml version="1.0" encoding="UTF-8"?>
//!     <rss version="2.0">
//!         <channel>
//!             <title>My Rust Blog</title>
//!             <link>https://myrustblog.com</link>
//!             <description>A blog about Rust programming and tutorials.</description>
//!         </channel>
//!     </rss>
//! "#;
//!
//! match parse_rss(rss_content) {
//!     Ok(parsed_data) => println!("Parsed RSS data: {:?}", parsed_data),
//!     Err(e) => eprintln!("Error parsing RSS feed: {}", e),
//! }
//! ```

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

/// The current version of the rss-gen crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

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
pub fn quick_rss(
    title: &str,
    link: &str,
    description: &str,
) -> Result<String> {
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
}
