//! Stress / smoke benchmark — large RSS + Atom emit and format detect.
//!
//! Builds a 50 000-item RSS 2.0 feed and a 50 000-entry Atom 1.0 feed,
//! serialises both, measures wall-clock per stage, and runs the
//! lightweight `detect_feed_format` classifier across the resulting
//! megabyte-scale documents to confirm it stays sub-millisecond.
//!
//! Run with: `cargo run --release --example stress_huge_feed`

use rss_gen::{
    detect_feed_format, generate_atom, generate_rss, AtomEntry,
    AtomFeed, RssData, RssItem, RssVersion,
};
use std::time::Instant;

const ITEMS: usize = 50_000;
const BUDGET_MS: u128 = 1000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("== rss-gen stress: {ITEMS} items / entries ==\n");

    // ------------------------------------------------------------- RSS
    let t = Instant::now();
    let mut rss = RssData::new(Some(RssVersion::RSS2_0))
        .title("Stress feed")
        .link("https://example.com")
        .description("Synthetic large feed used for perf sanity checks")
        .language("en-GB")
        .pub_date("Sun, 28 Jun 2026 00:00:00 +0000")
        .last_build_date("Sun, 28 Jun 2026 00:00:00 +0000")
        .generator("rss-gen stress_huge_feed")
        .atom_link("https://example.com/feed.xml");
    for i in 0..ITEMS {
        rss.add_item(
            RssItem::new()
                .title(format!("Item {i}"))
                .link(format!("https://example.com/item-{i}"))
                .description(format!(
                    "Body for item {i} — Lorem ipsum dolor sit amet."
                ))
                .guid(format!("https://example.com/item-{i}"))
                .pub_date("Sun, 28 Jun 2026 00:00:00 +0000")
                .author("editor@example.com"),
        );
    }
    let build_ms = t.elapsed().as_millis();

    let t = Instant::now();
    let rss_xml = generate_rss(&rss)?;
    let rss_ms = t.elapsed().as_millis();

    report("RSS build", build_ms);
    report("RSS generate_rss", rss_ms);
    println!("    bytes emitted: {}", rss_xml.len());

    // ------------------------------------------------------------ Atom
    let t = Instant::now();
    let mut atom = AtomFeed::new()
        .id("urn:example:stress")
        .title("Stress feed")
        .updated("2026-06-28T00:00:00Z")
        .author_name("editor@example.com")
        .self_link("https://example.com/atom.xml");
    for i in 0..ITEMS {
        atom = atom.add_entry(
            AtomEntry::new()
                .id(format!("https://example.com/entry-{i}"))
                .title(format!("Entry {i}"))
                .updated("2026-06-28T00:00:00Z")
                .summary(format!("Summary for entry {i}")),
        );
    }
    let atom_build_ms = t.elapsed().as_millis();

    let t = Instant::now();
    let atom_xml = generate_atom(&atom)?;
    let atom_ms = t.elapsed().as_millis();

    report("Atom build", atom_build_ms);
    report("Atom generate_atom", atom_ms);
    println!("    bytes emitted: {}", atom_xml.len());

    // ---------------------------------------------------- Format detect
    let t = Instant::now();
    let rss_kind = detect_feed_format(&rss_xml);
    let atom_kind = detect_feed_format(&atom_xml);
    let detect_us = t.elapsed().as_micros();
    println!(
        "Format detect on 2 docs ({} bytes total): {} µs total",
        rss_xml.len() + atom_xml.len(),
        detect_us
    );
    println!("    RSS classified as : {rss_kind:?}");
    println!("    Atom classified as: {atom_kind:?}");

    // -------------------------------------------- Budget enforcement
    println!("\nBudget: < {BUDGET_MS} ms per generate stage.");
    let mut over_budget = Vec::<&'static str>::new();
    if rss_ms > BUDGET_MS {
        over_budget.push("RSS generate_rss");
    }
    if atom_ms > BUDGET_MS {
        over_budget.push("Atom generate_atom");
    }
    if over_budget.is_empty() {
        println!("All stages within budget.");
    } else {
        for stage in over_budget {
            eprintln!("OVER BUDGET: {stage}");
        }
        std::process::exit(1);
    }
    Ok(())
}

fn report(label: &str, ms: u128) {
    println!("  {label:<24} {ms:>6} ms");
}
