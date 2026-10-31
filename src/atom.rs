// Copyright © 2026 RSS Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

// src/atom.rs

//! Atom 1.0 feed generation, validation, and format detection.
//!
//! This module implements the subset of [RFC 4287](https://www.rfc-editor.org/rfc/rfc4287)
//! needed to author syndication feeds in the Atom 1.0 format alongside the
//! existing RSS support. It is fully independent of the RSS code path: callers
//! that want Atom output construct an [`AtomFeed`], add [`AtomEntry`] values
//! via the builder API, and serialize through [`generate_atom`].
//!
//! It also exposes [`FeedFormat`] and [`detect_feed_format`] for callers that
//! need to dispatch between RSS and Atom inputs without parsing the whole
//! document.
//!
//! # Required elements (RFC 4287 §4.1.1 & §4.1.2)
//!
//! * `<feed>`: `id`, `title`, `updated` are required.
//! * `<entry>`: `id`, `title`, `updated` are required.
//!
//! Validation in this module enforces those required elements and prefixes
//! reported errors with `feed.` / `entry.<idx>.` to stay consistent with the
//! contextual validation errors introduced for the RSS path in issue #34.
//!
//! # Example
//!
//! ```rust
//! use rss_gen::atom::{generate_atom, AtomEntry, AtomFeed};
//!
//! let feed = AtomFeed::new()
//!     .id("https://example.com/feed")
//!     .title("Example Feed")
//!     .updated("2026-06-27T00:00:00Z")
//!     .author_name("Jane Doe")
//!     .self_link("https://example.com/atom.xml")
//!     .add_entry(
//!         AtomEntry::new()
//!             .id("https://example.com/post-1")
//!             .title("First Post")
//!             .updated("2026-06-27T00:00:00Z")
//!             .summary("Hello, Atom"),
//!     );
//!
//! let xml = generate_atom(&feed).unwrap();
//! assert!(xml.contains(r#"<feed xmlns="http://www.w3.org/2005/Atom">"#));
//! ```

use crate::error::{Result, RssError, ValidationError};
use crate::generator::clean_xml_text;
use oxml::{
    tree::NodeId, Document, EmptyElement, Newline, SerialiseOptions,
};
use serde::{Deserialize, Serialize};

/// Atom 1.0 namespace literal used on the root `<feed>` element.
pub const ATOM_NAMESPACE: &str = "http://www.w3.org/2005/Atom";

const XML_DECLARATION: &str =
    r#"<?xml version="1.0" encoding="utf-8"?>"#;

/// Discriminates between supported feed formats.
///
/// Returned by [`detect_feed_format`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FeedFormat {
    /// RSS 0.9x / 2.0 (`<rss>` root element).
    Rss,
    /// RSS 1.0 (`<rdf:RDF>` root element).
    RssRdf,
    /// Atom 1.0 (`<feed xmlns="http://www.w3.org/2005/Atom">` root).
    Atom,
    /// The document could not be classified.
    Unknown,
}

/// Peeks the root element of an XML document and returns the detected
/// [`FeedFormat`].
///
/// This is a lightweight heuristic intended for dispatching between
/// [`crate::generator::generate_rss`] / [`crate::parser::parse_rss`] and the
/// Atom code path. It reads only as far as the first start element and does
/// not validate the document.
///
/// # Examples
///
/// ```rust
/// use rss_gen::atom::{detect_feed_format, FeedFormat};
///
/// let rss = r#"<?xml version="1.0"?><rss version="2.0"><channel/></rss>"#;
/// assert_eq!(detect_feed_format(rss), FeedFormat::Rss);
///
/// let atom = r#"<?xml version="1.0"?><feed xmlns="http://www.w3.org/2005/Atom"/>"#;
/// assert_eq!(detect_feed_format(atom), FeedFormat::Atom);
/// ```
#[must_use]
pub fn detect_feed_format(xml: &str) -> FeedFormat {
    use oxml::stream::{Event, Reader};

    let Ok(mut reader) = Reader::new(xml.trim_start()) else {
        return FeedFormat::Unknown;
    };
    loop {
        match reader.next_event() {
            Ok(Some(Event::StartElement { name, .. })) => {
                return match name.local.as_str() {
                    "rss" => FeedFormat::Rss,
                    "RDF" => FeedFormat::RssRdf,
                    "feed" => {
                        // Only classify as Atom if the namespace matches —
                        // some hand-rolled feeds use <feed> with no xmlns.
                        if name.namespace.as_deref()
                            == Some(ATOM_NAMESPACE)
                        {
                            FeedFormat::Atom
                        } else {
                            FeedFormat::Unknown
                        }
                    }
                    _ => FeedFormat::Unknown,
                };
            }
            Ok(None) | Err(_) => return FeedFormat::Unknown,
            Ok(Some(_)) => {}
        }
    }
}

/// A person reference (`<author>` / `<contributor>`) per RFC 4287 §3.2.
#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize,
)]
#[non_exhaustive]
pub struct AtomPerson {
    /// Human-readable name (required by the spec).
    pub name: String,
    /// Optional email address.
    pub email: String,
    /// Optional homepage URI.
    pub uri: String,
}

impl AtomPerson {
    /// Creates a new `AtomPerson` with the given name and empty contact
    /// fields.
    #[must_use]
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: sanitize_input(&name.into()),
            ..Self::default()
        }
    }

    /// Sets the email address.
    #[must_use]
    pub fn email<S: Into<String>>(mut self, value: S) -> Self {
        self.email = sanitize_input(&value.into());
        self
    }

    /// Sets the homepage URI.
    #[must_use]
    pub fn uri<S: Into<String>>(mut self, value: S) -> Self {
        self.uri = sanitize_input(&value.into());
        self
    }
}

/// An Atom `<link>` element per RFC 4287 §4.2.7.
///
/// Atom links are typed pointers; the canonical alternate link to the
/// resource is `rel="alternate"` and self-references use `rel="self"`. Media
/// enclosures (e.g. podcasts) use `rel="enclosure"` together with the `type`
/// and `length` attributes.
#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize,
)]
#[non_exhaustive]
pub struct AtomLink {
    /// Target URI of the link.
    pub href: String,
    /// Link relation (`alternate`, `self`, `enclosure`, …). Empty means
    /// `alternate`, per RFC 4287 §4.2.7.2.
    pub rel: String,
    /// MIME type of the linked resource.
    pub mime_type: String,
    /// Length in bytes (relevant for `rel="enclosure"`).
    pub length: String,
    /// Optional human-readable title.
    pub title: String,
}

impl AtomLink {
    /// Constructs an `AtomLink` with the given href, defaulting to
    /// `rel="alternate"`.
    #[must_use]
    pub fn alternate<S: Into<String>>(href: S) -> Self {
        Self {
            href: sanitize_input(&href.into()),
            rel: "alternate".to_string(),
            ..Self::default()
        }
    }

    /// Constructs a `rel="self"` link pointing at the canonical location of
    /// the feed.
    #[must_use]
    pub fn self_ref<S: Into<String>>(href: S) -> Self {
        Self {
            href: sanitize_input(&href.into()),
            rel: "self".to_string(),
            ..Self::default()
        }
    }

    /// Constructs a `rel="enclosure"` link for a media attachment.
    #[must_use]
    pub fn enclosure<S, T>(href: S, mime_type: T, length: u64) -> Self
    where
        S: Into<String>,
        T: Into<String>,
    {
        Self {
            href: sanitize_input(&href.into()),
            rel: "enclosure".to_string(),
            mime_type: sanitize_input(&mime_type.into()),
            length: length.to_string(),
            ..Self::default()
        }
    }

    /// Sets the optional title attribute.
    #[must_use]
    pub fn title<S: Into<String>>(mut self, value: S) -> Self {
        self.title = sanitize_input(&value.into());
        self
    }
}

/// Text content for `<title>`, `<summary>`, `<content>` per RFC 4287 §3.1.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
)]
#[non_exhaustive]
pub enum AtomTextType {
    /// Plain text. Special characters are XML-escaped.
    #[default]
    Text,
    /// HTML payload. Treated as opaque text by this module and emitted
    /// verbatim after XML-escaping; consumers are responsible for ensuring
    /// the payload is safe.
    Html,
}

impl AtomTextType {
    fn as_attr(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Html => "html",
        }
    }
}

/// An Atom 1.0 feed (`<feed>`) document.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AtomFeed {
    /// Permanent, universally unique identifier of the feed (RFC 4287 §4.2.6).
    pub id: String,
    /// Human-readable title of the feed.
    pub title: String,
    /// Optional human-readable subtitle.
    pub subtitle: String,
    /// RFC 3339 timestamp of the most recent significant modification.
    pub updated: String,
    /// Optional rights / copyright string.
    pub rights: String,
    /// Generator identification (e.g. `"rss-gen"`).
    pub generator: String,
    /// Optional URI of a small icon representing the feed.
    pub icon: String,
    /// Optional URI of a larger logo representing the feed.
    pub logo: String,
    /// Language tag (e.g. `"en-US"`). Emitted as `xml:lang` on `<feed>`.
    pub language: String,
    /// Authors of the feed.
    pub authors: Vec<AtomPerson>,
    /// Contributors to the feed.
    pub contributors: Vec<AtomPerson>,
    /// Links advertised at feed level (`self`, `alternate`, …).
    pub links: Vec<AtomLink>,
    /// Category tags applied to the feed (term values only).
    pub categories: Vec<String>,
    /// Entries contained in the feed.
    pub entries: Vec<AtomEntry>,
}

/// An Atom 1.0 entry (`<entry>`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AtomEntry {
    /// Permanent, universally unique identifier of the entry.
    pub id: String,
    /// Human-readable title.
    pub title: String,
    /// RFC 3339 timestamp of the most recent significant modification.
    pub updated: String,
    /// Optional RFC 3339 publication timestamp.
    pub published: String,
    /// Short human-readable description.
    pub summary: String,
    /// Type of the `summary` payload.
    pub summary_type: AtomTextType,
    /// Full content payload (may be empty).
    pub content: String,
    /// Type of the `content` payload.
    pub content_type: AtomTextType,
    /// Optional rights string.
    pub rights: String,
    /// Authors of the entry.
    pub authors: Vec<AtomPerson>,
    /// Contributors to the entry.
    pub contributors: Vec<AtomPerson>,
    /// Links attached to the entry, including media enclosures.
    pub links: Vec<AtomLink>,
    /// Category tags applied to the entry (term values only).
    pub categories: Vec<String>,
}

impl AtomFeed {
    /// Creates an empty `AtomFeed`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the feed's permanent identifier.
    #[must_use]
    pub fn id<S: Into<String>>(mut self, value: S) -> Self {
        self.id = sanitize_input(&value.into());
        self
    }

    /// Sets the feed title.
    #[must_use]
    pub fn title<S: Into<String>>(mut self, value: S) -> Self {
        self.title = sanitize_input(&value.into());
        self
    }

    /// Sets the feed subtitle.
    #[must_use]
    pub fn subtitle<S: Into<String>>(mut self, value: S) -> Self {
        self.subtitle = sanitize_input(&value.into());
        self
    }

    /// Sets the feed-level `updated` timestamp (RFC 3339).
    #[must_use]
    pub fn updated<S: Into<String>>(mut self, value: S) -> Self {
        self.updated = sanitize_input(&value.into());
        self
    }

    /// Sets the rights string.
    #[must_use]
    pub fn rights<S: Into<String>>(mut self, value: S) -> Self {
        self.rights = sanitize_input(&value.into());
        self
    }

    /// Sets the generator string.
    #[must_use]
    pub fn generator<S: Into<String>>(mut self, value: S) -> Self {
        self.generator = sanitize_input(&value.into());
        self
    }

    /// Sets the icon URI.
    #[must_use]
    pub fn icon<S: Into<String>>(mut self, value: S) -> Self {
        self.icon = sanitize_input(&value.into());
        self
    }

    /// Sets the logo URI.
    #[must_use]
    pub fn logo<S: Into<String>>(mut self, value: S) -> Self {
        self.logo = sanitize_input(&value.into());
        self
    }

    /// Sets the `xml:lang` value.
    #[must_use]
    pub fn language<S: Into<String>>(mut self, value: S) -> Self {
        self.language = sanitize_input(&value.into());
        self
    }

    /// Adds an author with the given name.
    #[must_use]
    pub fn author_name<S: Into<String>>(mut self, name: S) -> Self {
        self.authors.push(AtomPerson::new(name));
        self
    }

    /// Adds an [`AtomPerson`] to the author list.
    #[must_use]
    pub fn add_author(mut self, author: AtomPerson) -> Self {
        self.authors.push(author);
        self
    }

    /// Adds an [`AtomPerson`] to the contributor list.
    #[must_use]
    pub fn add_contributor(mut self, contributor: AtomPerson) -> Self {
        self.contributors.push(contributor);
        self
    }

    /// Adds an [`AtomLink`] to the feed.
    #[must_use]
    pub fn add_link(mut self, link: AtomLink) -> Self {
        self.links.push(link);
        self
    }

    /// Convenience: adds a `rel="self"` link to the canonical feed URL.
    #[must_use]
    pub fn self_link<S: Into<String>>(self, href: S) -> Self {
        self.add_link(AtomLink::self_ref(href))
    }

    /// Convenience: adds a `rel="alternate"` link to the human page.
    #[must_use]
    pub fn alternate_link<S: Into<String>>(self, href: S) -> Self {
        self.add_link(AtomLink::alternate(href))
    }

    /// Adds a category tag.
    #[must_use]
    pub fn add_category<S: Into<String>>(mut self, term: S) -> Self {
        self.categories.push(sanitize_input(&term.into()));
        self
    }

    /// Appends an entry to the feed.
    #[must_use]
    pub fn add_entry(mut self, entry: AtomEntry) -> Self {
        self.entries.push(entry);
        self
    }

    /// Number of entries in the feed.
    #[must_use]
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Validates the feed against the required RFC 4287 elements.
    ///
    /// Error messages are prefixed with `feed.` for top-level violations
    /// and `entry.<idx>.` for per-entry violations, matching the contextual
    /// validation introduced in issue #34 for the RSS code path.
    ///
    /// # Errors
    ///
    /// Returns [`RssError::ValidationErrors`] containing every missing or
    /// invalid required element discovered.
    pub fn validate(&self) -> Result<()> {
        let mut errors: Vec<ValidationError> = Vec::new();

        if self.id.is_empty() {
            errors.push(ValidationError::new(
                "feed.id",
                "feed.id is missing",
            ));
        }
        if self.title.is_empty() {
            errors.push(ValidationError::new(
                "feed.title",
                "feed.title is missing",
            ));
        }
        if self.updated.is_empty() {
            errors.push(ValidationError::new(
                "feed.updated",
                "feed.updated is missing",
            ));
        } else if !is_rfc3339(&self.updated) {
            errors.push(ValidationError::new(
                "feed.updated",
                format!(
                    "feed.updated is not a valid RFC 3339 timestamp: {}",
                    self.updated
                ),
            ));
        }

        // RFC 4287 §4.1.1: a feed without a feed-level author MUST have
        // an author on every entry. Surface that explicitly so callers
        // don't silently emit a non-conformant document.
        let feed_has_author = !self.authors.is_empty();
        for (idx, entry) in self.entries.iter().enumerate() {
            if let Err(RssError::ValidationErrors(mut entry_errors)) =
                entry.validate_with_index(idx)
            {
                errors.append(&mut entry_errors);
            }
            if !feed_has_author && entry.authors.is_empty() {
                errors.push(ValidationError::new(
                    format!("entry.{idx}.author"),
                    format!(
                        "entry.{idx}.author is missing (and feed has \
                         no feed-level author)"
                    ),
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(RssError::ValidationErrors(errors))
        }
    }
}

impl AtomEntry {
    /// Creates an empty `AtomEntry`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the entry's permanent identifier.
    #[must_use]
    pub fn id<S: Into<String>>(mut self, value: S) -> Self {
        self.id = sanitize_input(&value.into());
        self
    }

    /// Sets the entry title.
    #[must_use]
    pub fn title<S: Into<String>>(mut self, value: S) -> Self {
        self.title = sanitize_input(&value.into());
        self
    }

    /// Sets the entry-level `updated` timestamp (RFC 3339).
    #[must_use]
    pub fn updated<S: Into<String>>(mut self, value: S) -> Self {
        self.updated = sanitize_input(&value.into());
        self
    }

    /// Sets the entry-level `published` timestamp (RFC 3339).
    #[must_use]
    pub fn published<S: Into<String>>(mut self, value: S) -> Self {
        self.published = sanitize_input(&value.into());
        self
    }

    /// Sets the plain-text summary.
    #[must_use]
    pub fn summary<S: Into<String>>(mut self, value: S) -> Self {
        self.summary = sanitize_input(&value.into());
        self.summary_type = AtomTextType::Text;
        self
    }

    /// Sets the HTML summary payload.
    #[must_use]
    pub fn summary_html<S: Into<String>>(mut self, value: S) -> Self {
        self.summary = sanitize_input(&value.into());
        self.summary_type = AtomTextType::Html;
        self
    }

    /// Sets the plain-text content payload.
    #[must_use]
    pub fn content<S: Into<String>>(mut self, value: S) -> Self {
        self.content = sanitize_input(&value.into());
        self.content_type = AtomTextType::Text;
        self
    }

    /// Sets the HTML content payload.
    #[must_use]
    pub fn content_html<S: Into<String>>(mut self, value: S) -> Self {
        self.content = sanitize_input(&value.into());
        self.content_type = AtomTextType::Html;
        self
    }

    /// Sets the rights string.
    #[must_use]
    pub fn rights<S: Into<String>>(mut self, value: S) -> Self {
        self.rights = sanitize_input(&value.into());
        self
    }

    /// Adds an author with the given name.
    #[must_use]
    pub fn author_name<S: Into<String>>(mut self, name: S) -> Self {
        self.authors.push(AtomPerson::new(name));
        self
    }

    /// Adds an [`AtomPerson`] to the entry's author list.
    #[must_use]
    pub fn add_author(mut self, author: AtomPerson) -> Self {
        self.authors.push(author);
        self
    }

    /// Adds an [`AtomLink`] to the entry.
    #[must_use]
    pub fn add_link(mut self, link: AtomLink) -> Self {
        self.links.push(link);
        self
    }

    /// Convenience: adds a `rel="alternate"` link to the entry's resource.
    #[must_use]
    pub fn alternate_link<S: Into<String>>(self, href: S) -> Self {
        self.add_link(AtomLink::alternate(href))
    }

    /// Convenience: attaches a media enclosure (RFC 4287 §4.2.7.2).
    #[must_use]
    pub fn add_enclosure<S, T>(
        self,
        href: S,
        mime_type: T,
        length: u64,
    ) -> Self
    where
        S: Into<String>,
        T: Into<String>,
    {
        self.add_link(AtomLink::enclosure(href, mime_type, length))
    }

    /// Adds a category tag.
    #[must_use]
    pub fn add_category<S: Into<String>>(mut self, term: S) -> Self {
        self.categories.push(sanitize_input(&term.into()));
        self
    }

    /// Validates the entry against RFC 4287 §4.1.2 required elements.
    ///
    /// Error messages are prefixed with `entry.` (no index).
    ///
    /// # Errors
    ///
    /// Returns [`RssError::ValidationErrors`] containing every missing or
    /// invalid required element discovered.
    pub fn validate(&self) -> Result<()> {
        let mut errors: Vec<ValidationError> = Vec::new();
        push_entry_errors(self, "entry.", &mut errors);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(RssError::ValidationErrors(errors))
        }
    }

    fn validate_with_index(&self, idx: usize) -> Result<()> {
        let prefix = format!("entry.{idx}.");
        let mut errors: Vec<ValidationError> = Vec::new();
        push_entry_errors(self, &prefix, &mut errors);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(RssError::ValidationErrors(errors))
        }
    }
}

fn push_entry_errors(
    entry: &AtomEntry,
    prefix: &str,
    errors: &mut Vec<ValidationError>,
) {
    let field_path = |suffix: &str| format!("{prefix}{suffix}");

    if entry.id.is_empty() {
        errors.push(ValidationError::new(
            field_path("id"),
            format!("{prefix}id is missing"),
        ));
    }
    if entry.title.is_empty() {
        errors.push(ValidationError::new(
            field_path("title"),
            format!("{prefix}title is missing"),
        ));
    }
    if entry.updated.is_empty() {
        errors.push(ValidationError::new(
            field_path("updated"),
            format!("{prefix}updated is missing"),
        ));
    } else if !is_rfc3339(&entry.updated) {
        errors.push(ValidationError::new(
            field_path("updated"),
            format!(
                "{prefix}updated is not a valid RFC 3339 timestamp: {}",
                entry.updated
            ),
        ));
    }
    if !entry.published.is_empty() && !is_rfc3339(&entry.published) {
        errors.push(ValidationError::new(
            field_path("published"),
            format!(
                "{prefix}published is not a valid RFC 3339 timestamp: {}",
                entry.published
            ),
        ));
    }
}

fn is_rfc3339(value: &str) -> bool {
    use time::format_description::well_known::Rfc3339;
    use time::OffsetDateTime;
    OffsetDateTime::parse(value, &Rfc3339).is_ok()
}

fn sanitize_input(value: &str) -> String {
    value
        .chars()
        .filter(|c| !c.is_control() || matches!(*c, '\n' | '\r' | '\t'))
        .collect()
}

/// Serializes an [`AtomFeed`] into an Atom 1.0 XML string.
///
/// Validation runs first via [`AtomFeed::validate`]; invalid feeds return
/// the validation errors unchanged. On success the produced string is a
/// stand-alone Atom 1.0 document with `xmlns="http://www.w3.org/2005/Atom"`
/// on the root element.
///
/// # Errors
///
/// Returns [`RssError::ValidationErrors`] when required elements are missing
/// or malformed, or [`RssError::XmlWriteError`] when the underlying XML
/// writer fails.
pub fn generate_atom(feed: &AtomFeed) -> Result<String> {
    feed.validate()?;

    let mut doc = Document::empty();
    let feed_elem =
        doc.append_element(doc.root(), Some(ATOM_NAMESPACE), "feed")?;
    doc.declare_namespace(feed_elem, None, ATOM_NAMESPACE)?;
    if !feed.language.is_empty() {
        doc.set_attribute(
            feed_elem,
            Some("http://www.w3.org/XML/1998/namespace"),
            "lang",
            &clean_xml_text(&feed.language),
        )?;
    }

    write_text_element(&mut doc, feed_elem, "id", &feed.id)?;
    write_text_element(&mut doc, feed_elem, "title", &feed.title)?;
    write_text_element(&mut doc, feed_elem, "updated", &feed.updated)?;
    if !feed.subtitle.is_empty() {
        write_text_element(
            &mut doc,
            feed_elem,
            "subtitle",
            &feed.subtitle,
        )?;
    }
    if !feed.rights.is_empty() {
        write_text_element(
            &mut doc,
            feed_elem,
            "rights",
            &feed.rights,
        )?;
    }
    if !feed.icon.is_empty() {
        write_text_element(&mut doc, feed_elem, "icon", &feed.icon)?;
    }
    if !feed.logo.is_empty() {
        write_text_element(&mut doc, feed_elem, "logo", &feed.logo)?;
    }
    if !feed.generator.is_empty() {
        write_text_element(
            &mut doc,
            feed_elem,
            "generator",
            &feed.generator,
        )?;
    }

    for person in &feed.authors {
        write_person(&mut doc, feed_elem, "author", person)?;
    }
    for person in &feed.contributors {
        write_person(&mut doc, feed_elem, "contributor", person)?;
    }
    for link in &feed.links {
        write_link(&mut doc, feed_elem, link)?;
    }
    for category in &feed.categories {
        write_category(&mut doc, feed_elem, category)?;
    }

    for entry in &feed.entries {
        write_entry(&mut doc, feed_elem, entry)?;
    }

    let serialise_opts = SerialiseOptions {
        indent: None,
        empty_elements: EmptyElement::SelfClosing,
        newline: Newline::Lf,
    };
    let mut xml = String::from(XML_DECLARATION);
    let _ = doc.write_xml_with(&mut xml, serialise_opts);
    Ok(xml)
}

fn write_text_element(
    doc: &mut Document,
    parent: NodeId,
    name: &str,
    content: &str,
) -> Result<()> {
    let elem =
        doc.append_element(parent, Some(ATOM_NAMESPACE), name)?;
    doc.append_text(elem, &clean_xml_text(content))?;
    Ok(())
}

fn write_typed_text(
    doc: &mut Document,
    parent: NodeId,
    name: &str,
    content: &str,
    text_type: AtomTextType,
) -> Result<()> {
    let elem =
        doc.append_element(parent, Some(ATOM_NAMESPACE), name)?;
    doc.set_attribute(elem, None, "type", text_type.as_attr())?;
    doc.append_text(elem, &clean_xml_text(content))?;
    Ok(())
}

fn write_person(
    doc: &mut Document,
    parent: NodeId,
    element: &str,
    person: &AtomPerson,
) -> Result<()> {
    let person_elem =
        doc.append_element(parent, Some(ATOM_NAMESPACE), element)?;
    write_text_element(doc, person_elem, "name", &person.name)?;
    if !person.email.is_empty() {
        write_text_element(doc, person_elem, "email", &person.email)?;
    }
    if !person.uri.is_empty() {
        write_text_element(doc, person_elem, "uri", &person.uri)?;
    }
    Ok(())
}

fn write_link(
    doc: &mut Document,
    parent: NodeId,
    link: &AtomLink,
) -> Result<()> {
    let link_elem =
        doc.append_element(parent, Some(ATOM_NAMESPACE), "link")?;
    doc.set_attribute(
        link_elem,
        None,
        "href",
        &clean_xml_text(&link.href),
    )?;
    if !link.rel.is_empty() {
        doc.set_attribute(
            link_elem,
            None,
            "rel",
            &clean_xml_text(&link.rel),
        )?;
    }
    if !link.mime_type.is_empty() {
        doc.set_attribute(
            link_elem,
            None,
            "type",
            &clean_xml_text(&link.mime_type),
        )?;
    }
    if !link.length.is_empty() {
        doc.set_attribute(
            link_elem,
            None,
            "length",
            &clean_xml_text(&link.length),
        )?;
    }
    if !link.title.is_empty() {
        doc.set_attribute(
            link_elem,
            None,
            "title",
            &clean_xml_text(&link.title),
        )?;
    }
    Ok(())
}

fn write_category(
    doc: &mut Document,
    parent: NodeId,
    term: &str,
) -> Result<()> {
    let cat =
        doc.append_element(parent, Some(ATOM_NAMESPACE), "category")?;
    doc.set_attribute(cat, None, "term", &clean_xml_text(term))?;
    Ok(())
}

fn write_entry(
    doc: &mut Document,
    parent: NodeId,
    entry: &AtomEntry,
) -> Result<()> {
    let entry_elem =
        doc.append_element(parent, Some(ATOM_NAMESPACE), "entry")?;

    write_text_element(doc, entry_elem, "id", &entry.id)?;
    write_text_element(doc, entry_elem, "title", &entry.title)?;
    write_text_element(doc, entry_elem, "updated", &entry.updated)?;
    if !entry.published.is_empty() {
        write_text_element(
            doc,
            entry_elem,
            "published",
            &entry.published,
        )?;
    }
    if !entry.summary.is_empty() {
        write_typed_text(
            doc,
            entry_elem,
            "summary",
            &entry.summary,
            entry.summary_type,
        )?;
    }
    if !entry.content.is_empty() {
        write_typed_text(
            doc,
            entry_elem,
            "content",
            &entry.content,
            entry.content_type,
        )?;
    }
    if !entry.rights.is_empty() {
        write_text_element(doc, entry_elem, "rights", &entry.rights)?;
    }

    for person in &entry.authors {
        write_person(doc, entry_elem, "author", person)?;
    }
    for person in &entry.contributors {
        write_person(doc, entry_elem, "contributor", person)?;
    }
    for link in &entry.links {
        write_link(doc, entry_elem, link)?;
    }
    for category in &entry.categories {
        write_category(doc, entry_elem, category)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_feed() -> AtomFeed {
        AtomFeed::new()
            .id("urn:example:feed")
            .title("Example")
            .updated("2026-06-27T00:00:00Z")
            .author_name("Tester")
    }

    #[test]
    fn validate_rejects_missing_required_fields() {
        let feed = AtomFeed::new();
        let err = feed.validate().unwrap_err();
        let RssError::ValidationErrors(errs) = err else {
            panic!("expected ValidationErrors");
        };
        assert!(errs.iter().any(|e| e.field == "feed.id"
            && e.message == "feed.id is missing"));
        assert!(errs.iter().any(|e| e.field == "feed.title"
            && e.message == "feed.title is missing"));
        assert!(errs.iter().any(|e| e.field == "feed.updated"
            && e.message == "feed.updated is missing"));
    }

    #[test]
    fn validate_rejects_non_rfc3339_updated() {
        let feed = AtomFeed::new()
            .id("urn:example:feed")
            .title("Example")
            .updated("yesterday afternoon")
            .author_name("Tester");
        let err = feed.validate().unwrap_err();
        let RssError::ValidationErrors(errs) = err else {
            panic!("expected ValidationErrors");
        };
        assert!(errs.iter().any(|e| e.field == "feed.updated"
            && e.message.starts_with(
                "feed.updated is not a valid RFC 3339 timestamp"
            )));
    }

    #[test]
    fn entry_inherits_feed_author_requirement() {
        let feed = AtomFeed::new()
            .id("urn:example:feed")
            .title("Example")
            .updated("2026-06-27T00:00:00Z")
            .add_entry(
                AtomEntry::new()
                    .id("urn:example:entry-1")
                    .title("Entry 1")
                    .updated("2026-06-27T00:00:00Z"),
            );
        let err = feed.validate().unwrap_err();
        let RssError::ValidationErrors(errs) = err else {
            panic!("expected ValidationErrors");
        };
        assert!(errs.iter().any(|e| e.field == "entry.0.author"));
    }

    #[test]
    fn entry_validate_uses_unindexed_prefix() {
        let entry = AtomEntry::new();
        let err = entry.validate().unwrap_err();
        let RssError::ValidationErrors(errs) = err else {
            panic!("expected ValidationErrors");
        };
        assert!(errs.iter().any(|e| e.field == "entry.id"
            && e.message == "entry.id is missing"));
        assert!(errs.iter().any(|e| e.field == "entry.title"
            && e.message == "entry.title is missing"));
        assert!(errs.iter().any(|e| e.field == "entry.updated"
            && e.message == "entry.updated is missing"));
    }

    #[test]
    fn generate_minimal_feed_emits_required_elements() {
        let xml = generate_atom(&minimal_feed()).unwrap();
        assert!(xml
            .contains(r#"<feed xmlns="http://www.w3.org/2005/Atom">"#));
        assert!(xml.contains("<id>urn:example:feed</id>"));
        assert!(xml.contains("<title>Example</title>"));
        assert!(xml.contains("<updated>2026-06-27T00:00:00Z</updated>"));
        assert!(xml.contains("<author>"));
        assert!(xml.contains("<name>Tester</name>"));
    }

    #[test]
    fn generate_feed_with_language_sets_xml_lang() {
        let feed = minimal_feed().language("en-US");
        let xml = generate_atom(&feed).unwrap();
        assert!(xml.contains(r#"xml:lang="en-US""#));
    }

    #[test]
    fn generate_feed_with_self_link_emits_rel_self() {
        let feed =
            minimal_feed().self_link("https://example.com/atom.xml");
        let xml = generate_atom(&feed).unwrap();
        assert!(xml.contains(
            r#"<link href="https://example.com/atom.xml" rel="self"/>"#
        ));
    }

    #[test]
    fn generate_entry_with_enclosure_emits_rel_enclosure() {
        let feed = minimal_feed().add_entry(
            AtomEntry::new()
                .id("urn:example:ep-1")
                .title("Episode 1")
                .updated("2026-06-27T00:00:00Z")
                .summary("Pilot episode")
                .add_enclosure(
                    "https://example.com/ep-1.mp3",
                    "audio/mpeg",
                    12_345_678,
                ),
        );
        let xml = generate_atom(&feed).unwrap();
        assert!(xml.contains(r#"rel="enclosure""#));
        assert!(xml.contains(r#"type="audio/mpeg""#));
        assert!(xml.contains(r#"length="12345678""#));
    }

    #[test]
    fn generate_entry_with_html_content_sets_type_html() {
        let feed = minimal_feed().add_entry(
            AtomEntry::new()
                .id("urn:example:post-1")
                .title("Post 1")
                .updated("2026-06-27T00:00:00Z")
                .content_html("<p>Hello</p>"),
        );
        let xml = generate_atom(&feed).unwrap();
        assert!(xml.contains(r#"<content type="html">"#));
        // The HTML payload must be escaped — the angle brackets in <p>
        // become entities so the Atom document stays well-formed.
        assert!(xml.contains("&lt;p&gt;Hello&lt;/p&gt;"));
    }

    #[test]
    fn detect_feed_format_classifies_correctly() {
        let rss = r#"<?xml version="1.0"?><rss version="2.0"><channel/></rss>"#;
        let atom = r#"<?xml version="1.0"?><feed xmlns="http://www.w3.org/2005/Atom"><id/></feed>"#;
        let rdf = r#"<?xml version="1.0"?><rdf:RDF xmlns:rdf="..."><channel/></rdf:RDF>"#;
        let other = r#"<?xml version="1.0"?><html><body/></html>"#;
        let unparseable = "not xml at all";
        assert_eq!(detect_feed_format(rss), FeedFormat::Rss);
        assert_eq!(detect_feed_format(atom), FeedFormat::Atom);
        assert_eq!(detect_feed_format(rdf), FeedFormat::RssRdf);
        assert_eq!(detect_feed_format(other), FeedFormat::Unknown);
        assert_eq!(
            detect_feed_format(unparseable),
            FeedFormat::Unknown
        );
    }

    #[test]
    fn detect_treats_feed_without_atom_namespace_as_unknown() {
        let no_ns = r#"<?xml version="1.0"?><feed><id/></feed>"#;
        assert_eq!(detect_feed_format(no_ns), FeedFormat::Unknown);
    }

    #[test]
    fn round_trip_detect_after_generate() {
        let xml = generate_atom(&minimal_feed()).unwrap();
        assert_eq!(detect_feed_format(&xml), FeedFormat::Atom);
    }

    #[test]
    fn special_characters_are_escaped_in_text_payloads() {
        let feed = AtomFeed::new()
            .id("urn:example:feed")
            .title("A & B < C > D")
            .updated("2026-06-27T00:00:00Z")
            .author_name("Tester");
        let xml = generate_atom(&feed).unwrap();
        assert!(xml.contains("<title>A &amp; B &lt; C &gt; D</title>"));
    }
}
