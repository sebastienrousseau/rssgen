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
    #[error("Validation errors: {0:?}")]
    ValidationErrors(Vec<String>),

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
}

/// Represents a specific validation error.
#[derive(Debug, Error)]
#[non_exhaustive]
#[error("Validation error: {message}")]
pub struct ValidationError {
    /// The field that failed validation.
    pub field: String,
    /// The error message.
    pub message: String,
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
        log::error!("RSS Error occurred: {}", self);
    }

    /// Converts the `RssError` into an appropriate HTTP status code.
    ///
    /// This method is useful when the library is used in web services.
    ///
    /// # Returns
    ///
    /// Returns a `u16` representing an HTTP status code.
    pub fn to_http_status(&self) -> u16 {
        match self {
            RssError::XmlWriteError(_) | RssError::XmlParseError(_) => {
                500
            }
            RssError::Utf8Error(_) => 500,
            RssError::MissingField(_) | RssError::InvalidInput(_) => {
                400
            }
            RssError::DateParseError(_) => 400,
            RssError::IoError(_) => 500,
            RssError::InvalidUrl(_) => 400,
            RssError::UnknownElement(_) => 500,
            RssError::ValidationErrors(_) => 400,
            RssError::DateSortError(_) => 500,
            RssError::ItemValidationError(_) => 400,
            RssError::UnknownField(_) => 500,
            RssError::Custom(_) => 500,
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
            io::Error::new(io::ErrorKind::Other, "XML error"),
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
                std::sync::Arc::new(io::Error::new(
                    io::ErrorKind::Other,
                    "XML error"
                ))
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
        let error = ValidationError {
            field: "some_field".to_string(),
            message: "Invalid field".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Validation error: Invalid field"
        );
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
            format!("{}", rss_error),
            "A required field is missing: title"
        );
    }

    #[test]
    fn test_date_parse_error() {
        let rss_error =
            RssError::DateParseError("Invalid date format".to_string());

        assert_eq!(
            format!("{}", rss_error),
            "Date parse error: Invalid date format"
        );
    }

    #[test]
    fn test_invalid_url_error() {
        let rss_error =
            RssError::InvalidUrl("https://invalid-url".to_string());

        assert_eq!(
            format!("{}", rss_error),
            "Invalid URL provided: https://invalid-url"
        );
    }

    #[test]
    fn test_unknown_element_error() {
        let rss_error =
            RssError::UnknownElement("unknown-element".to_string());

        assert_eq!(
            format!("{}", rss_error),
            "Unknown XML element found: unknown-element"
        );
    }

    #[test]
    fn test_validation_errors() {
        let validation_errors = vec![
            "Title is missing".to_string(),
            "Invalid pub date".to_string(),
        ];
        let rss_error =
            RssError::ValidationErrors(validation_errors.clone());

        assert_eq!(
            format!("{}", rss_error),
            format!("Validation errors: {:?}", validation_errors)
        );
    }
}
