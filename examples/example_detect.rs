//! Demonstrates `detect_feed_format` — the v0.0.6 root-element
//! classifier that callers use to dispatch between the RSS and Atom
//! parsing/generation code paths without having to parse the whole
//! document first.
//!
//! Run with: `cargo run --example example_detect`

use rss_gen::{detect_feed_format, FeedFormat};

fn main() {
    let samples: &[(&str, &str)] = &[
        (
            "RSS 2.0",
            r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel><title>Example</title></channel>
</rss>"#,
        ),
        (
            "RSS 1.0 (RDF)",
            r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns="http://purl.org/rss/1.0/">
  <channel rdf:about="https://example.com"/>
</rdf:RDF>"#,
        ),
        (
            "Atom 1.0 (proper namespace)",
            r#"<?xml version="1.0"?>
<feed xmlns="http://www.w3.org/2005/Atom"><id/></feed>"#,
        ),
        (
            "Bare <feed> without namespace — must NOT be classified as Atom",
            r#"<?xml version="1.0"?><feed><id/></feed>"#,
        ),
        ("Garbage input", "this is not xml"),
    ];

    for (label, xml) in samples {
        let detected = detect_feed_format(xml);
        // `FeedFormat` is `#[non_exhaustive]` so callers must
        // include a wildcard arm — keeps adding new variants in
        // future releases SemVer-minor instead of -major.
        let verdict = match detected {
            FeedFormat::Rss => "RSS (2.0 / 0.9x)",
            FeedFormat::RssRdf => "RSS 1.0 (RDF)",
            FeedFormat::Atom => "Atom 1.0",
            FeedFormat::Unknown => "Unknown",
            _ => "Unhandled (newly added FeedFormat variant)",
        };
        println!("[{label}] -> {verdict}");
    }
}
