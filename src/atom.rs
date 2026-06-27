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

use crate::error::{Result, RssError};
use crate::generator::sanitize_content;
use quick_xml::events::{
    BytesDecl, BytesEnd, BytesStart, BytesText, Event,
};
use quick_xml::Writer;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

/// Atom 1.0 namespace literal used on the root `<feed>` element.
pub const ATOM_NAMESPACE: &str = "http://www.w3.org/2005/Atom";

const XML_VERSION: &str = "1.0";
const XML_ENCODING: &str = "utf-8";

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
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(start) | Event::Empty(start)) => {
                let name = start.name();
                let local = name.as_ref();
                if local == b"rss" {
                    return FeedFormat::Rss;
                }
                if local == b"rdf:RDF" || local == b"RDF" {
                    return FeedFormat::RssRdf;
                }
                if local == b"feed" {
                    // Only classify as Atom if the namespace matches —
                    // some hand-rolled feeds use <feed> with no xmlns.
                    let has_atom_ns =
                        start.attributes().flatten().any(|a| {
                            a.key.as_ref() == b"xmlns"
                                && a.value.as_ref()
                                    == ATOM_NAMESPACE.as_bytes()
                        });
                    return if has_atom_ns {
                        FeedFormat::Atom
                    } else {
                        FeedFormat::Unknown
                    };
                }
                return FeedFormat::Unknown;
            }
            Ok(Event::Eof) | Err(_) => return FeedFormat::Unknown,
            Ok(_) => buf.clear(),
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
        let mut errors = Vec::new();

        if self.id.is_empty() {
            errors.push("feed.id is missing".to_string());
        }
        if self.title.is_empty() {
            errors.push("feed.title is missing".to_string());
        }
        if self.updated.is_empty() {
            errors.push("feed.updated is missing".to_string());
        } else if !is_rfc3339(&self.updated) {
            errors.push(format!(
                "feed.updated is not a valid RFC 3339 timestamp: {}",
                self.updated
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
                errors.push(format!(
                    "entry.{idx}.author is missing (and feed has no \
                     feed-level author)"
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
        let mut errors = Vec::new();
        push_entry_errors(self, "entry.", &mut errors);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(RssError::ValidationErrors(errors))
        }
    }

    fn validate_with_index(&self, idx: usize) -> Result<()> {
        let prefix = format!("entry.{idx}.");
        let mut errors = Vec::new();
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
    errors: &mut Vec<String>,
) {
    if entry.id.is_empty() {
        errors.push(format!("{prefix}id is missing"));
    }
    if entry.title.is_empty() {
        errors.push(format!("{prefix}title is missing"));
    }
    if entry.updated.is_empty() {
        errors.push(format!("{prefix}updated is missing"));
    } else if !is_rfc3339(&entry.updated) {
        errors.push(format!(
            "{prefix}updated is not a valid RFC 3339 timestamp: {}",
            entry.updated
        ));
    }
    if !entry.published.is_empty() && !is_rfc3339(&entry.published) {
        errors.push(format!(
            "{prefix}published is not a valid RFC 3339 timestamp: {}",
            entry.published
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

    let mut writer = Writer::new(Cursor::new(Vec::new()));

    writer.write_event(Event::Decl(BytesDecl::new(
        XML_VERSION,
        Some(XML_ENCODING),
        None,
    )))?;

    let mut feed_start = BytesStart::new("feed");
    feed_start.push_attribute(("xmlns", ATOM_NAMESPACE));
    if !feed.language.is_empty() {
        feed_start.push_attribute(("xml:lang", feed.language.as_str()));
    }
    writer.write_event(Event::Start(feed_start))?;

    write_text_element(&mut writer, "id", &feed.id)?;
    write_text_element(&mut writer, "title", &feed.title)?;
    write_text_element(&mut writer, "updated", &feed.updated)?;
    if !feed.subtitle.is_empty() {
        write_text_element(&mut writer, "subtitle", &feed.subtitle)?;
    }
    if !feed.rights.is_empty() {
        write_text_element(&mut writer, "rights", &feed.rights)?;
    }
    if !feed.icon.is_empty() {
        write_text_element(&mut writer, "icon", &feed.icon)?;
    }
    if !feed.logo.is_empty() {
        write_text_element(&mut writer, "logo", &feed.logo)?;
    }
    if !feed.generator.is_empty() {
        write_text_element(&mut writer, "generator", &feed.generator)?;
    }

    for person in &feed.authors {
        write_person(&mut writer, "author", person)?;
    }
    for person in &feed.contributors {
        write_person(&mut writer, "contributor", person)?;
    }
    for link in &feed.links {
        write_link(&mut writer, link)?;
    }
    for category in &feed.categories {
        write_category(&mut writer, category)?;
    }

    for entry in &feed.entries {
        write_entry(&mut writer, entry)?;
    }

    writer.write_event(Event::End(BytesEnd::new("feed")))?;

    let xml = writer.into_inner().into_inner();
    String::from_utf8(xml).map_err(RssError::from)
}

fn write_text_element<W: std::io::Write>(
    writer: &mut Writer<W>,
    name: &str,
    content: &str,
) -> Result<()> {
    let escaped = sanitize_content(content);
    writer.write_event(Event::Start(BytesStart::new(name)))?;
    writer
        .write_event(Event::Text(BytesText::from_escaped(escaped)))?;
    writer.write_event(Event::End(BytesEnd::new(name)))?;
    Ok(())
}

fn write_typed_text<W: std::io::Write>(
    writer: &mut Writer<W>,
    name: &str,
    content: &str,
    text_type: AtomTextType,
) -> Result<()> {
    let escaped = sanitize_content(content);
    let mut start = BytesStart::new(name);
    start.push_attribute(("type", text_type.as_attr()));
    writer.write_event(Event::Start(start))?;
    writer
        .write_event(Event::Text(BytesText::from_escaped(escaped)))?;
    writer.write_event(Event::End(BytesEnd::new(name)))?;
    Ok(())
}

fn write_person<W: std::io::Write>(
    writer: &mut Writer<W>,
    element: &str,
    person: &AtomPerson,
) -> Result<()> {
    writer.write_event(Event::Start(BytesStart::new(element)))?;
    write_text_element(writer, "name", &person.name)?;
    if !person.email.is_empty() {
        write_text_element(writer, "email", &person.email)?;
    }
    if !person.uri.is_empty() {
        write_text_element(writer, "uri", &person.uri)?;
    }
    writer.write_event(Event::End(BytesEnd::new(element)))?;
    Ok(())
}

fn write_link<W: std::io::Write>(
    writer: &mut Writer<W>,
    link: &AtomLink,
) -> Result<()> {
    let mut start = BytesStart::new("link");
    start.push_attribute(("href", link.href.as_str()));
    if !link.rel.is_empty() {
        start.push_attribute(("rel", link.rel.as_str()));
    }
    if !link.mime_type.is_empty() {
        start.push_attribute(("type", link.mime_type.as_str()));
    }
    if !link.length.is_empty() {
        start.push_attribute(("length", link.length.as_str()));
    }
    if !link.title.is_empty() {
        start.push_attribute(("title", link.title.as_str()));
    }
    writer.write_event(Event::Empty(start))?;
    Ok(())
}

fn write_category<W: std::io::Write>(
    writer: &mut Writer<W>,
    term: &str,
) -> Result<()> {
    let mut start = BytesStart::new("category");
    start.push_attribute(("term", term));
    writer.write_event(Event::Empty(start))?;
    Ok(())
}

fn write_entry<W: std::io::Write>(
    writer: &mut Writer<W>,
    entry: &AtomEntry,
) -> Result<()> {
    writer.write_event(Event::Start(BytesStart::new("entry")))?;

    write_text_element(writer, "id", &entry.id)?;
    write_text_element(writer, "title", &entry.title)?;
    write_text_element(writer, "updated", &entry.updated)?;
    if !entry.published.is_empty() {
        write_text_element(writer, "published", &entry.published)?;
    }
    if !entry.summary.is_empty() {
        write_typed_text(
            writer,
            "summary",
            &entry.summary,
            entry.summary_type,
        )?;
    }
    if !entry.content.is_empty() {
        write_typed_text(
            writer,
            "content",
            &entry.content,
            entry.content_type,
        )?;
    }
    if !entry.rights.is_empty() {
        write_text_element(writer, "rights", &entry.rights)?;
    }

    for person in &entry.authors {
        write_person(writer, "author", person)?;
    }
    for person in &entry.contributors {
        write_person(writer, "contributor", person)?;
    }
    for link in &entry.links {
        write_link(writer, link)?;
    }
    for category in &entry.categories {
        write_category(writer, category)?;
    }

    writer.write_event(Event::End(BytesEnd::new("entry")))?;
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
        let RssError::ValidationErrors(msgs) = err else {
            panic!("expected ValidationErrors");
        };
        assert!(msgs.iter().any(|m| m == "feed.id is missing"));
        assert!(msgs.iter().any(|m| m == "feed.title is missing"));
        assert!(msgs.iter().any(|m| m == "feed.updated is missing"));
    }

    #[test]
    fn validate_rejects_non_rfc3339_updated() {
        let feed = AtomFeed::new()
            .id("urn:example:feed")
            .title("Example")
            .updated("yesterday afternoon")
            .author_name("Tester");
        let err = feed.validate().unwrap_err();
        let RssError::ValidationErrors(msgs) = err else {
            panic!("expected ValidationErrors");
        };
        assert!(msgs.iter().any(|m| m.starts_with(
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
        let RssError::ValidationErrors(msgs) = err else {
            panic!("expected ValidationErrors");
        };
        assert!(msgs.iter().any(|m| m.contains("entry.0.author")));
    }

    #[test]
    fn entry_validate_uses_unindexed_prefix() {
        let entry = AtomEntry::new();
        let err = entry.validate().unwrap_err();
        let RssError::ValidationErrors(msgs) = err else {
            panic!("expected ValidationErrors");
        };
        assert!(msgs.iter().any(|m| m == "entry.id is missing"));
        assert!(msgs.iter().any(|m| m == "entry.title is missing"));
        assert!(msgs.iter().any(|m| m == "entry.updated is missing"));
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
