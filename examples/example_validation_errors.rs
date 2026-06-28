//! Shows how v0.0.6's structured `ValidationError` lets callers
//! dispatch programmatically on `field` rather than parsing strings.
//!
//! Demonstrates both the RSS path (`channel.` / `item.0.` prefixes)
//! and the Atom path (`feed.` / `entry.0.` prefixes), plus the fact
//! that the validator now accepts RFC 2822 dates with any timezone
//! (the GMT-only requirement was a P0 spec violation removed in
//! v0.0.6).
//!
//! Run with: `cargo run --example example_validation_errors`

use rss_gen::{
    generate_atom, generate_rss, AtomEntry, AtomFeed, RssData,
    RssError, RssItem,
};

fn main() {
    print_rss_validation_errors();
    print_atom_validation_errors();
    print_date_acceptance();
}

fn print_rss_validation_errors() {
    println!("== RSS channel + item validation ==");

    // Channel without title/link/description, plus one item missing
    // its description — multiple violations at multiple contexts.
    let mut rss = RssData::new(None);
    rss.add_item(
        RssItem::new()
            .title("Item missing description")
            .link("https://example.com/post"),
    );

    match generate_rss(&rss) {
        Ok(_) => println!("  (no errors)"),
        Err(RssError::ValidationErrors(errs)) => {
            for e in &errs {
                // `field` is the dotted path; `message` is the
                // human-readable text. `Display` writes the message.
                println!("  field = {:<24}  message = {}", e.field, e);
            }
        }
        Err(other) => println!("  unexpected error: {other}"),
    }
}

fn print_atom_validation_errors() {
    println!("\n== Atom feed + entry validation ==");

    // Feed missing id/title/updated AND missing feed-level author,
    // with an entry that lacks its own author — triggers the
    // RFC 4287 §4.1.1 inheritance rule.
    let feed = AtomFeed::new().add_entry(
        AtomEntry::new()
            .id("urn:example:1")
            .title("Entry one")
            .updated("2026-06-28T00:00:00Z"),
    );

    match generate_atom(&feed) {
        Ok(_) => println!("  (no errors)"),
        Err(RssError::ValidationErrors(errs)) => {
            for e in &errs {
                println!("  field = {:<24}  message = {}", e.field, e);
            }
        }
        Err(other) => println!("  unexpected error: {other}"),
    }
}

fn print_date_acceptance() {
    use rss_gen::data::parse_date;

    println!("\n== Date parser accepts RFC 2822 + ISO 8601 ==");
    let inputs = [
        "Mon, 01 Jan 2024 00:00:00 GMT",
        "Sun, 28 Jun 2026 00:12:20 +0000",
        "Sat, 27 Jun 2026 19:12:20 -0500",
        "2026-06-28T00:12:20Z",
    ];
    for input in inputs {
        match parse_date(input) {
            Ok(_) => println!("  OK   {input}"),
            Err(e) => println!("  FAIL {input} ({e})"),
        }
    }
}
