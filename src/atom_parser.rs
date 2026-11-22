// Copyright © 2024 RSS Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Atom 1.0 (RFC 4287) parsing into the crate's [`RssData`] model.
//!
//! [`crate::parse_rss`] dispatches here when the root element is an Atom
//! `<feed>`. Atom elements map onto the RSS-shaped fields as follows:
//!
//! | Atom                          | `RssData` / `RssItem`             |
//! | :---------------------------- | :-------------------------------- |
//! | `title`                       | `title`                           |
//! | `subtitle`                    | `description`                     |
//! | `link` (`alternate` or no rel)| `link` (first one wins)           |
//! | `link rel="self"` (feed)      | `atom_link`                       |
//! | `id`                          | `guid`                            |
//! | `updated` (feed)              | `last_build_date`                 |
//! | `published` / `updated` (entry)| `pub_date` (`published` wins)    |
//! | `author/name`                 | `author`                          |
//! | `category term=".."`          | `category`                        |
//! | `rights`, `generator`, `logo` | `copyright`, `generator`, `image_url` |
//! | `summary` / `content` (entry) | `description` (`summary` wins)    |
//!
//! Elements outside the Atom namespace (extensions) are ignored; their
//! text still counts towards an enclosing Atom text construct, so XHTML
//! `content` is read as plain text.

use oxml::stream::{Event, Reader};
use oxml::ExpandedName;

use crate::atom::ATOM_NAMESPACE;
use crate::data::{RssData, RssItem};
use crate::error::Result;
use crate::parser::ParserConfig;

/// Mutable state threaded through the event loop.
#[derive(Default)]
struct AtomState {
    /// Local names of the open Atom elements, root first.
    stack: Vec<String>,
    /// Text collected since the innermost Atom element opened.
    text: String,
    /// The entry being built, while inside `<entry>`.
    entry: Option<RssItem>,
    /// The feed being built.
    feed: RssData,
}

/// Parses an Atom 1.0 document into [`RssData`].
///
/// Custom handlers in `config` see each Atom element's local name and
/// trimmed text when the element closes.
pub(crate) fn parse_atom(
    xml_content: &str,
    config: Option<&ParserConfig>,
) -> Result<RssData> {
    let mut reader = Reader::new(xml_content.trim_start())?;
    let mut state = AtomState::default();

    while let Some(event) = reader.next_event()? {
        state.handle(event, config)?;
    }
    Ok(state.feed)
}

/// Whether an element belongs to the Atom namespace.
fn is_atom(name: &ExpandedName) -> bool {
    name.namespace.as_deref() == Some(ATOM_NAMESPACE)
}

/// Returns the value of the unprefixed attribute `key`, if present.
fn attr<'a>(
    attributes: &'a [(ExpandedName, String)],
    key: &str,
) -> Option<&'a str> {
    attributes
        .iter()
        .find(|(n, _)| n.namespace.is_none() && n.local == key)
        .map(|(_, v)| v.as_str())
}

impl AtomState {
    /// Dispatches one stream event.
    fn handle(
        &mut self,
        event: Event,
        config: Option<&ParserConfig>,
    ) -> Result<()> {
        match event {
            Event::StartElement {
                name, attributes, ..
            } if is_atom(&name) => self.open(&name.local, &attributes),
            Event::EndElement { name, .. } if is_atom(&name) => {
                return self.close(config);
            }
            Event::Text(text) => self.text.push_str(&text),
            _ => (),
        }
        Ok(())
    }

    /// Local name of the innermost open Atom element (`""` at the root).
    fn parent(&self) -> &str {
        self.stack.last().map_or("", String::as_str)
    }

    /// Whether the innermost open element is `<feed>` or `<entry>`, the
    /// only places whose children map onto the data model. Children of
    /// an entry's `<source>`, for example, are skipped.
    fn in_feed_or_entry(&self) -> bool {
        matches!(self.parent(), "feed" | "entry")
    }

    /// Handles an Atom start tag.
    fn open(
        &mut self,
        local: &str,
        attributes: &[(ExpandedName, String)],
    ) {
        self.text.clear();
        if self.in_feed_or_entry() {
            match local {
                "entry" if self.parent() == "feed" => {
                    self.entry = Some(RssItem::new());
                }
                "link" => self.apply_link(
                    attr(attributes, "rel").unwrap_or("alternate"),
                    attr(attributes, "href").unwrap_or_default(),
                ),
                "category" => self.apply_category(
                    attr(attributes, "term").unwrap_or_default(),
                ),
                _ => (),
            }
        }
        self.stack.push(local.to_string());
    }

    /// Handles an Atom end tag: assigns the collected text to a field.
    fn close(&mut self, config: Option<&ParserConfig>) -> Result<()> {
        let Some(local) = self.stack.pop() else {
            return Ok(());
        };
        let text = std::mem::take(&mut self.text);
        let text = text.trim();
        self.assign(&local, text);

        if let Some(cfg) = config {
            for handler in &cfg.custom_handlers {
                handler.handle_element(&local, text, &[])?;
            }
        }
        Ok(())
    }

    /// Routes a closed element's text to the entry or feed field it
    /// maps to. `self.stack` no longer holds `local`.
    fn assign(&mut self, local: &str, text: &str) {
        if local == "entry" && self.parent() == "feed" {
            if let Some(entry) = self.entry.take() {
                self.feed.add_item(entry);
            }
            return;
        }
        let author_name = self.parent() == "author"
            && local == "name"
            && matches!(
                self.stack.iter().rev().nth(1).map(String::as_str),
                Some("feed" | "entry")
            );
        if author_name {
            self.apply_author(text);
            return;
        }
        if !self.in_feed_or_entry() {
            return;
        }
        match self.entry.as_mut() {
            Some(entry) => apply_entry_text(entry, local, text),
            None => apply_feed_text(&mut self.feed, local, text),
        }
    }

    /// Records a `<link>` on the current entry or the feed.
    fn apply_link(&mut self, rel: &str, href: &str) {
        match (self.entry.as_mut(), rel) {
            (Some(entry), "alternate") if entry.link.is_empty() => {
                entry.link = href.to_string();
            }
            (None, "alternate") if self.feed.link.is_empty() => {
                self.feed.link = href.to_string();
            }
            (None, "self") => self.feed.atom_link = href.to_string(),
            _ => (),
        }
    }

    /// Records a `<category term="..">` on the current entry or the feed.
    fn apply_category(&mut self, term: &str) {
        match self.entry.as_mut() {
            Some(entry) => entry.category = Some(term.to_string()),
            None => self.feed.category = term.to_string(),
        }
    }

    /// Records an `<author><name>` on the current entry or the feed.
    fn apply_author(&mut self, text: &str) {
        match self.entry.as_mut() {
            Some(entry) => entry.author = text.to_string(),
            None => self.feed.author = text.to_string(),
        }
    }
}

/// Assigns the text of a closed feed-level element.
fn apply_feed_text(feed: &mut RssData, local: &str, text: &str) {
    let field = match local {
        "title" => &mut feed.title,
        "subtitle" => &mut feed.description,
        "id" => &mut feed.guid,
        "updated" => &mut feed.last_build_date,
        "rights" => &mut feed.copyright,
        "generator" => &mut feed.generator,
        "logo" => &mut feed.image_url,
        _ => return,
    };
    *field = text.to_string();
}

/// Assigns the text of a closed entry-level element.
///
/// `published` takes precedence over `updated`, and `summary` over
/// `content`, whichever order they appear in.
fn apply_entry_text(entry: &mut RssItem, local: &str, text: &str) {
    match local {
        "title" => entry.title = text.to_string(),
        "id" => entry.guid = text.to_string(),
        "published" => entry.pub_date = text.to_string(),
        "summary" => entry.description = text.to_string(),
        "updated" if entry.pub_date.is_empty() => {
            entry.pub_date = text.to_string();
        }
        "content" if entry.description.is_empty() => {
            entry.description = text.to_string();
        }
        _ => (),
    }
}
