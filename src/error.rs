// Copyright © 2024 RSS Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

// src/error.rs

use log;
use quick_xml;
use std::string::FromUtf8Error;
use thiserror::Error;

/// Errors that can occur when generating or parsing RSS feeds.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RssError {
    /// Error occurred while writing XML.
    #[error("XML error occurred: {0}")]
    XmlWriteError(#[from] quick_xml::Error),

    /// Error occurred during XML parsing.
    #[error("XML parse error occurred: {0}")]
    XmlParseError(quick_xml::Error),

    /// Error occurred during UTF-8 conversion.
    #[error("UTF-8 conversion error occurred: {0}")]
    Utf8Error(#[from] FromUtf8Error),

    /// Error indicating a required field is missing.
    #[error("A required field is missing: {0}")]
    MissingField(String),

    /// Error indicating a date parsing failure.
    #[error("Date parse error: {0}")]
    DateParseError(String),

    /// General I/O error.
    #[error("I/O error occurred: {0}")]
    IoError(#[from] std::io::Error),

    /// Error for invalid input data.
    #[error("Invalid input data provided: {0}")]
    InvalidInput(String),

    /// Error for invalid URL provided.
    #[error("Invalid URL provided: {0}")]
    InvalidUrl(String),

    /// Error for unknown XML elements encountered during parsing.
    #[error("Unknown XML element found: {0}")]
    UnknownElement(String),

    /// Error for validation errors.
    ///
    /// Carries a `Vec<ValidationError>` rather than `Vec<String>` so callers
    /// can programmatically inspect the offending `field` rather than parse
    /// human-readable strings. The wrapped `ValidationError::Display` impl
    /// formats as the bare `message` (e.g. `channel.title is missing`),
    /// matching the string format used in earlier releases.
    #[error("Validation errors: {0:?}")]
    ValidationErrors(Vec<ValidationError>),

    /// Error for date sort errors.
    #[error("Date sort error: {0:?}")]
    DateSortError(Vec<DateSortError>),

    /// Error for item validation errors.
    #[error("Item validation error: {0}")]
    ItemValidationError(String),

    /// Error for unknown field encountered during parsing.
    #[error("Unknown field encountered: {0}")]
    UnknownField(String),

    /// Custom error for unforeseen scenarios.
    #[error("Custom error: {0}")]
    Custom(String),

    /// Error for invalid RSS version.
    #[error("Invalid RSS version: {0}")]
    InvalidRssVersion(String),
    // #[error("Unknown RSS element: {0}")]
    // UnknownElement(String),

    // #[error("XML parsing error: {0}")]
    // XmlParseError(#[from] quick_xml::Error),

    // #[error("IO error: {0}")]
    // IoError(#[from] std::io::Error),
}

/// Represents a specific validation error.
///
/// `field` identifies the offending element using a dotted path
/// (`channel.title`, `item.0.link`, `feed.id`, `entry.2.updated`, …) so
/// downstream tooling can dispatch on the field without parsing strings.
/// `message` is the full human-readable error text and is what
/// [`std::fmt::Display`] writes — preserving the bare-string format that previous
/// releases of this crate emitted as the [`RssError::ValidationErrors`]
/// payload.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
#[error("{message}")]
pub struct ValidationError {
    /// The dotted path identifying the field that failed validation.
    pub field: String,
    /// The full human-readable error text. Read via `Display` /
    /// `to_string()`; matches the string format used in
    /// pre-v0.0.6 releases.
    pub message: String,
}

impl ValidationError {
    /// Constructs a [`ValidationError`].
    ///
    /// `field` should be the dotted path (e.g. `"channel.title"`);
    /// `message` is the full human-readable error text and is what
    /// [`std::fmt::Display`] writes back out — keeping the bare-string format used
    /// by earlier releases of this crate.
    #[must_use]
    pub fn new<F: Into<String>, M: Into<String>>(
        field: F,
        message: M,
    ) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

/// Represents a specific date sorting error.
#[derive(Debug, Error)]
#[non_exhaustive]
#[error("Date sort error: {message}")]
pub struct DateSortError {
    /// The index of the item with the date sort error.
    pub index: usize,
    /// The error message.
    pub message: String,
}

/// Result type for RSS operations.
///
/// This type alias provides a convenient way to return results from RSS operations,
/// where the error type is always `RssError`.
pub type Result<T> = std::result::Result<T, RssError>;

impl RssError {
    /// Creates a new `RssError::MissingField` error.
    ///
    /// # Arguments
    ///
    /// * `field_name` - The name of the missing field.
    ///
    /// # Returns
    ///
    /// Returns a new `RssError::MissingField` instance.
    pub fn missing_field<S: Into<String>>(field_name: S) -> Self {
        RssError::MissingField(field_name.into())
    }

    /// Creates a new `DateSortError`.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the item with the date sort error.
    /// * `message` - The error message.
    ///
    /// # Returns
    ///
    /// Returns a new `DateSortError` instance.
    pub fn date_sort_error<S: Into<String>>(
        index: usize,
        message: S,
    ) -> DateSortError {
        DateSortError {
            index,
            message: message.into(),
        }
    }

    /// Creates a new `RssError::InvalidInput` error.
    ///
    /// # Arguments
    ///
    /// * `message` - A description of why the input is invalid.
    ///
    /// # Returns
    ///
    /// Returns a new `RssError::InvalidInput` instance.
    pub fn invalid_input<S: Into<String>>(message: S) -> Self {
        RssError::InvalidInput(message.into())
    }

    /// Creates a new `RssError::Custom` error.
    ///
    /// # Arguments
    ///
    /// * `message` - A custom error message.
    ///
    /// # Returns
    ///
    /// Returns a new `RssError::Custom` instance.
    pub fn custom<S: Into<String>>(message: S) -> Self {
        RssError::Custom(message.into())
    }

    /// Logs the error using the `log` crate.
    ///
    /// This method logs the error at the error level. It uses the `log` crate,
    /// so the application using this library should configure a logger.
    pub fn log(&self) {
        log::error!("RSS Error occurred: {self}");
    }

    /// Converts the `RssError` into an appropriate HTTP status code.
    ///
    /// This method is useful when the library is used in web services.
    ///
    /// # Returns
    ///
    /// Returns a `u16` representing an HTTP status code.
    #[must_use]
    pub fn to_http_status(&self) -> u16 {
        match self {
            // Combine all cases that map to 500
            RssError::XmlWriteError(_)
            | RssError::XmlParseError(_)
            | RssError::Utf8Error(_)
            | RssError::IoError(_)
            | RssError::UnknownElement(_)
            | RssError::DateSortError(_)
            | RssError::UnknownField(_)
            | RssError::Custom(_) => 500,

            // Combine all cases that map to 400
            RssError::MissingField(_)
            | RssError::InvalidInput(_)
            | RssError::DateParseError(_)
            | RssError::InvalidUrl(_)
            | RssError::ValidationErrors(_)
            | RssError::ItemValidationError(_)
            | RssError::InvalidRssVersion(_) => 400,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::io;

    #[test]
    fn test_rss_error_display() {
        let error = RssError::missing_field("title");
        assert_eq!(
            error.to_string(),
            "A required field is missing: title"
        );
    }

    #[test]
    fn test_xml_write_error() {
        let xml_error = quick_xml::Error::Io(std::sync::Arc::new(
            io::Error::other("XML error"),
        ));
        let error = RssError::XmlWriteError(xml_error);
        assert_eq!(
            error.to_string(),
            "XML error occurred: I/O error: XML error"
        );
    }

    #[test]
    fn test_utf8_error() {
        let utf8_error =
            String::from_utf8(vec![0, 159, 146, 150]).unwrap_err();
        let error = RssError::Utf8Error(utf8_error);
        assert_eq!(error.to_string(), "UTF-8 conversion error occurred: invalid utf-8 sequence of 1 bytes from index 1");
    }

    #[test]
    fn test_io_error() {
        let io_error =
            io::Error::new(io::ErrorKind::NotFound, "File not found");
        let error: RssError = io_error.into();
        assert_eq!(
            error.to_string(),
            "I/O error occurred: File not found"
        );
    }

    #[test]
    fn test_error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RssError>();
    }

    #[test]
    fn test_error_source() {
        let xml_error = quick_xml::Error::Io(std::sync::Arc::new(
            io::Error::new(io::ErrorKind::NotFound, "File not found"),
        ));
        let error = RssError::XmlWriteError(xml_error);
        assert!(error.source().is_some());

        let io_error: RssError =
            io::Error::new(io::ErrorKind::NotFound, "File not found")
                .into();
        assert!(io_error.source().is_some());
    }

    #[test]
    fn test_missing_field_with_string() {
        let error = RssError::missing_field(String::from("author"));
        assert_eq!(
            error.to_string(),
            "A required field is missing: author"
        );
    }

    #[test]
    fn test_missing_field_with_str() {
        let error = RssError::missing_field("description");
        assert_eq!(
            error.to_string(),
            "A required field is missing: description"
        );
    }

    #[test]
    fn test_error_downcast() {
        let error: Box<dyn Error> =
            Box::new(RssError::missing_field("category"));
        let downcast_result = error.downcast::<RssError>();
        assert!(downcast_result.is_ok());
    }

    #[test]
    fn test_invalid_input_error() {
        let error = RssError::invalid_input("Invalid date format");
        assert_eq!(
            error.to_string(),
            "Invalid input data provided: Invalid date format"
        );
    }

    #[test]
    fn test_custom_error() {
        let error = RssError::custom("Unforeseen error occurred");
        assert_eq!(
            error.to_string(),
            "Custom error: Unforeseen error occurred"
        );
    }

    #[test]
    fn test_to_http_status() {
        assert_eq!(
            RssError::missing_field("title").to_http_status(),
            400
        );
        assert_eq!(
            RssError::XmlWriteError(quick_xml::Error::Io(
                std::sync::Arc::new(io::Error::other("XML error"))
            ))
            .to_http_status(),
            500
        );
        assert_eq!(
            RssError::InvalidInput("Bad input".to_string())
                .to_http_status(),
            400
        );
    }

    #[test]
    fn test_validation_error() {
        // v0.0.6: Display writes the bare `message`, not the previous
        // "Validation error: …" prefix — callers that compared the
        // string against `e.to_string()` now see the same strings the
        // crate produced as `Vec<String>` entries pre-v0.0.6, which is
        // what the property tests in tests/property_tests.rs assert.
        let error = ValidationError::new(
            "channel.title",
            "channel.title is missing",
        );
        assert_eq!(error.to_string(), "channel.title is missing");
        assert_eq!(error.field, "channel.title");
        assert_eq!(error.message, "channel.title is missing");
    }

    #[test]
    fn test_date_sort_error() {
        let error = DateSortError {
            index: 0,
            message: "Invalid date".to_string(),
        };
        assert_eq!(error.to_string(), "Date sort error: Invalid date");
    }

    #[test]
    fn test_missing_field_error() {
        let rss_error = RssError::MissingField("title".to_string());

        assert_eq!(
            format!("{rss_error}"),
            "A required field is missing: title"
        );
    }

    #[test]
    fn test_date_parse_error() {
        let rss_error =
            RssError::DateParseError("Invalid date format".to_string());

        assert_eq!(
            format!("{rss_error}"),
            "Date parse error: Invalid date format"
        );
    }

    #[test]
    fn test_invalid_url_error() {
        let rss_error =
            RssError::InvalidUrl("https://invalid-url".to_string());

        assert_eq!(
            format!("{rss_error}"),
            "Invalid URL provided: https://invalid-url"
        );
    }

    #[test]
    fn test_unknown_element_error() {
        let rss_error =
            RssError::UnknownElement("unknown-element".to_string());

        assert_eq!(
            format!("{rss_error}"),
            "Unknown XML element found: unknown-element"
        );
    }

    #[test]
    fn test_validation_errors() {
        let validation_errors = vec![
            ValidationError::new(
                "channel.title",
                "channel.title is missing",
            ),
            ValidationError::new(
                "channel.pub_date",
                "Invalid channel.pub_date: 2026/06/28",
            ),
        ];
        let rss_error =
            RssError::ValidationErrors(validation_errors.clone());

        // Display of ValidationError is the bare `message` — matches the
        // string format pre-v0.0.6 callers were used to.
        assert_eq!(
            validation_errors[0].to_string(),
            "channel.title is missing"
        );
        // The wrapping RssError formats the Vec via its Debug impl so
        // both the field and the message are surfaced.
        let rendered = format!("{rss_error}");
        assert!(rendered.contains("channel.title"));
        assert!(rendered.contains("Invalid channel.pub_date"));
    }

    #[test]
    fn test_validation_error_field_is_accessible() {
        let err = ValidationError::new(
            "item.0.link",
            "item.0.link is missing",
        );
        assert_eq!(err.field, "item.0.link");
        assert_eq!(err.to_string(), "item.0.link is missing");
    }

    #[test]
    fn test_error_log() {
        let error = RssError::missing_field("title");
        // log() writes to the log crate; just verify it doesn't panic
        error.log();

        let error = RssError::custom("something went wrong");
        error.log();
    }

    #[test]
    fn test_custom_error_http_status() {
        assert_eq!(
            RssError::Custom("err".to_string()).to_http_status(),
            500
        );
    }

    #[test]
    fn test_date_sort_error_constructor() {
        let error = RssError::date_sort_error(3, "dates out of order");
        let DateSortError { index, message } = error;
        assert_eq!(index, 3);
        assert_eq!(message, "dates out of order");
    }
}
