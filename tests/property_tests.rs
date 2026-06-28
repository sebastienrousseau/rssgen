// Copyright © 2024 RSS Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Property-based tests for RSS generation and parsing
//! Tests serialization roundtrips, parser invariants, and mathematical properties

use proptest::prelude::*;
use rss_gen::{
    data::{parse_date, validate_url, RssData, RssItem, RssVersion},
    error::RssError,
    generate_rss, MAX_DESCRIPTION_LENGTH, MAX_GENERAL_LENGTH,
    MAX_TITLE_LENGTH,
};

/// Generate arbitrary valid URLs for testing
fn url_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("https://example.com".to_string()),
        Just("http://test.org".to_string()),
        Just("https://www.rust-lang.org".to_string()),
        prop::collection::vec(prop::char::range('a', 'z'), 1..20)
            .prop_map(|chars| format!(
                "https://{}.com",
                chars.into_iter().collect::<String>()
            ))
    ]
}

/// Generate arbitrary valid RSS versions
fn rss_version_strategy() -> impl Strategy<Value = RssVersion> {
    prop_oneof![
        Just(RssVersion::RSS0_90),
        Just(RssVersion::RSS0_91),
        Just(RssVersion::RSS0_92),
        Just(RssVersion::RSS1_0),
        Just(RssVersion::RSS2_0),
    ]
}

/// Generate arbitrary safe strings (no control chars, reasonable length)
fn safe_string_strategy(
    max_len: usize,
) -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop::char::range('\u{0020}', '\u{007E}'), // Printable ASCII
        0..=max_len,
    )
    .prop_map(|chars| chars.into_iter().collect())
}

/// Generate arbitrary safe, non-empty strings
fn safe_nonempty_string_strategy(
    max_len: usize,
) -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop::char::range('\u{0020}', '\u{007E}'),
        1..=max_len,
    )
    .prop_map(|chars| chars.into_iter().collect())
}

/// Strategy for generating valid RssData
fn rss_data_strategy() -> impl Strategy<Value = RssData> {
    (
        rss_version_strategy(),
        safe_nonempty_string_strategy(MAX_TITLE_LENGTH),
        url_strategy(),
        safe_nonempty_string_strategy(MAX_DESCRIPTION_LENGTH),
        safe_string_strategy(MAX_GENERAL_LENGTH), // author
        safe_string_strategy(MAX_GENERAL_LENGTH), // category
    )
        .prop_map(
            |(version, title, link, description, author, category)| {
                RssData::new(Some(version))
                    .title(title)
                    .link(link)
                    .description(description)
                    .author(author)
                    .category(category)
            },
        )
}

/// Strategy for generating valid RssItem
fn rss_item_strategy() -> impl Strategy<Value = RssItem> {
    (
        safe_nonempty_string_strategy(MAX_TITLE_LENGTH),
        url_strategy(),
        safe_nonempty_string_strategy(MAX_DESCRIPTION_LENGTH),
        safe_nonempty_string_strategy(100), // GUID
        safe_string_strategy(MAX_GENERAL_LENGTH), // author
    )
        .prop_map(|(title, link, description, guid, author)| {
            RssItem::new()
                .title(title)
                .link(link)
                .description(description)
                .guid(guid)
                .author(author)
        })
}

#[cfg(test)]
mod property_tests {
    use super::*;

    proptest! {
        /// Test that RSS data serialization is deterministic
        #[test]
        fn test_rss_generation_deterministic(data in rss_data_strategy()) {
            let result1 = generate_rss(&data);
            let result2 = generate_rss(&data);

            match (result1, result2) {
                (Ok(rss1), Ok(rss2)) => prop_assert_eq!(rss1, rss2),
                (Err(_), Err(_)) => {}, // Both failed consistently
                _ => prop_assert!(false, "Non-deterministic RSS generation"),
            }
        }

        /// Test that RSS generation doesn't panic for valid data
        #[test]
        fn test_rss_generation_no_panic(data in rss_data_strategy()) {
            let _result = generate_rss(&data);
            // Should not panic regardless of result
        }

        /// Test that URL validation is consistent
        #[test]
        fn test_url_validation_consistency(url in url_strategy()) {
            let result1 = validate_url(&url);
            let result2 = validate_url(&url);
            prop_assert_eq!(result1.is_ok(), result2.is_ok());
        }

        /// Test that RSS version string conversion is bijective
        #[test]
        fn test_rss_version_string_roundtrip(version in rss_version_strategy()) {
            let version_str = version.as_str();
            let parsed_version = version_str.parse::<RssVersion>();
            prop_assert!(parsed_version.is_ok());
            if let Ok(parsed) = parsed_version {
                prop_assert_eq!(parsed, version);
            }
        }

        /// Test that adding and removing items maintains consistency
        #[test]
        fn test_item_operations_consistency(
            mut data in rss_data_strategy(),
            item in rss_item_strategy(),
        ) {
            let initial_count = data.item_count();
            let guid = item.guid.clone();

            data.add_item(item);
            prop_assert_eq!(data.item_count(), initial_count + 1);

            let removed = data.remove_item(&guid);
            prop_assert!(removed);
            prop_assert_eq!(data.item_count(), initial_count);
        }

        /// Test that RSS data validation is consistent for the same input
        #[test]
        fn test_validation_consistency(data in rss_data_strategy()) {
            let result1 = data.validate();
            let result2 = data.validate();
            prop_assert_eq!(result1.is_ok(), result2.is_ok());
        }

        /// Test mathematical property: item count after clear is always 0
        #[test]
        fn test_clear_items_mathematical_property(mut data in rss_data_strategy()) {
            data.clear_items();
            prop_assert_eq!(data.item_count(), 0);
            prop_assert!(data.items.is_empty());
        }

        /// Test that sanitize content is idempotent
        #[test]
        fn test_sanitize_content_idempotent(content in ".*") {
            use rss_gen::generator::sanitize_content;
            let sanitized_once = sanitize_content(&content);
            let sanitized_twice = sanitize_content(&sanitized_once);
            prop_assert_eq!(sanitized_once, sanitized_twice);
        }

        /// Test that validation never panics
        #[test]
        fn test_validation_no_panic(data in rss_data_strategy()) {
            let _result = data.validate();
            // Should never panic
        }

        /// Test URL validation never panics
        #[test]
        fn test_url_validation_no_panic(url in ".*") {
            let _result = validate_url(&url);
            // Should never panic
        }

        /// Test date parsing never panics
        #[test]
        fn test_date_parsing_no_panic(date in ".*") {
            let _result = parse_date(&date);
            // Should never panic
        }
    }
}

#[cfg(test)]
mod regression_tests {
    use super::*;

    /// Regression test for empty string handling in validation
    #[test]
    fn test_empty_string_validation_regression() {
        let data =
            RssData::new(None).title("").link("").description("");

        let result = data.validate();
        assert!(result.is_err());

        if let Err(RssError::ValidationErrors(errors)) = result {
            // Issue #34: errors now carry `channel.` / `item.` context prefix.
            // v0.0.6: ValidationErrors carries Vec<ValidationError> (was
            // Vec<String>); field is the dotted path and message is the
            // human-readable text — `Display` writes the bare message.
            assert!(errors.len() >= 3);
            assert!(errors.iter().any(|e| e.field == "channel.title"
                && e.message == "channel.title is missing"));
            assert!(errors.iter().any(|e| e.field == "channel.link"
                && e.message == "channel.link is missing"));
            assert!(errors
                .iter()
                .any(|e| e.field == "channel.description"
                    && e.message == "channel.description is missing"));
        } else {
            panic!("Expected ValidationErrors");
        }
    }

    /// Regression test for URL protocol validation
    #[test]
    fn test_url_protocol_validation_regression() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://example.com").is_ok());
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("file:///path/to/file").is_err());
        assert!(validate_url("example.com").is_err());
    }

    /// Regression test for maximum length validation
    #[test]
    fn test_max_length_validation_regression() {
        let long_title = "a".repeat(MAX_TITLE_LENGTH + 1);
        let data = RssData::new(None)
            .title(long_title)
            .link("https://example.com")
            .description("Valid description");

        // This should not panic and should handle the long title gracefully
        let _result = data.validate(); // May be ok or error depending on implementation
    }

    /// Regression test for special character handling in sanitization
    #[test]
    fn test_special_character_sanitization_regression() {
        use rss_gen::generator::sanitize_content;

        let dangerous_content = "<script>alert('xss')</script>";
        let sanitized = sanitize_content(dangerous_content);

        assert!(!sanitized.contains("<script>"));
        assert!(!sanitized.contains("</script>"));
        assert!(sanitized.contains("&lt;"));
        assert!(sanitized.contains("&gt;"));
    }

    /// Regression test for item removal edge cases
    #[test]
    fn test_item_removal_edge_cases_regression() {
        let mut data = RssData::new(None);

        // Try removing from empty list
        assert!(!data.remove_item("nonexistent"));

        // Add item and remove it
        let item = RssItem::new().guid("test-guid");
        data.add_item(item);
        assert_eq!(data.item_count(), 1);

        assert!(data.remove_item("test-guid"));
        assert_eq!(data.item_count(), 0);

        // Try removing the same item again
        assert!(!data.remove_item("test-guid"));
    }
}
