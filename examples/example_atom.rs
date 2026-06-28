//! End-to-end Atom 1.0 example.
//!
//! Builds an [`AtomFeed`] covering the v0.0.6 capability surface —
//! multi-author + contributor lists, a `rel="self"` link, an
//! `<entry>` carrying both `<summary>` and HTML `<content>`, a media
//! `<link rel="enclosure">` (RFC 4287 §4.2.7.2), and per-entry
//! categories — then serialises it through [`generate_atom`] and
//! prints the document.
//!
//! Run with: `cargo run --example example_atom`

use rss_gen::{
    generate_atom, AtomEntry, AtomFeed, AtomLink, AtomPerson,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let feed = AtomFeed::new()
        .id("urn:example:atom-demo")
        .title("rss-gen Atom demo")
        .subtitle("Showcases the surface added in v0.0.6")
        .updated("2026-06-28T00:00:00Z")
        .language("en-GB")
        .generator("rss-gen example_atom")
        // Multiple feed-level authors plus a contributor.
        .add_author(
            AtomPerson::new("Jane Doe")
                .email("jane@example.com")
                .uri("https://example.com/~jane"),
        )
        .add_author(AtomPerson::new("Sam Roe"))
        .add_contributor(AtomPerson::new("Ada Reviewer"))
        // Self link advertises the canonical feed location.
        .self_link("https://example.com/atom.xml")
        .alternate_link("https://example.com/")
        .add_category("rust")
        .add_category("syndication")
        // First entry — text summary + HTML content.
        .add_entry(
            AtomEntry::new()
                .id("https://example.com/post-1")
                .title("Hello, Atom")
                .updated("2026-06-28T00:00:00Z")
                .published("2026-06-27T12:00:00Z")
                .summary("Plain-text summary of the post.")
                .content_html(
                    "<p>HTML content body — type=\"html\".</p>",
                )
                .alternate_link("https://example.com/post-1")
                .add_category("intro"),
        )
        // Second entry — media enclosure for a podcast episode.
        .add_entry(
            AtomEntry::new()
                .id("https://example.com/ep-1")
                .title("Episode 1")
                .updated("2026-06-28T00:00:00Z")
                .summary("Pilot episode")
                .add_enclosure(
                    "https://example.com/ep-1.mp3",
                    "audio/mpeg",
                    12_345_678,
                )
                .add_link(
                    AtomLink::alternate("https://example.com/ep-1")
                        .title("Show notes"),
                ),
        );

    let xml = generate_atom(&feed)?;
    println!("{xml}");
    Ok(())
}
