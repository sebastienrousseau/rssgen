#![allow(missing_docs)]
use criterion::{criterion_group, criterion_main, Criterion};
use rss_gen::{
    detect_feed_format, generate_atom, generate_rss, parse_rss,
    AtomEntry, AtomFeed, RssData, RssItem, RssVersion,
};
use std::hint::black_box;
use std::time::Duration;

fn generate_large_rss_data(item_count: usize) -> RssData {
    let mut rss_data = RssData::new(Some(RssVersion::RSS2_0))
        .title("Benchmark Feed")
        .link("https://example.com")
        .description("A large RSS feed for benchmarking")
        .language("en-US")
        .pub_date("Mon, 01 Jan 2024 00:00:00 GMT")
        .last_build_date("Mon, 01 Jan 2024 00:00:00 GMT")
        .generator("RSS Gen Benchmark")
        .atom_link("https://example.com/feed.xml");

    for i in 0..item_count {
        rss_data.add_item(
            RssItem::new()
                .title(format!("Item {}", i))
                .link(format!("https://example.com/item{}", i))
                .description(format!(
                    "This is the description for item {}",
                    i
                ))
                .guid(format!("unique-id-{}", i))
                .pub_date("Mon, 01 Jan 2024 00:00:00 GMT"),
        );
    }

    rss_data
}

fn generate_large_atom_feed(entry_count: usize) -> AtomFeed {
    let mut feed = AtomFeed::new()
        .id("https://example.com/feed")
        .title("Benchmark Atom Feed")
        .updated("2026-06-28T00:00:00Z")
        .author_name("rss-gen bench harness")
        .self_link("https://example.com/atom.xml");

    for i in 0..entry_count {
        feed = feed.add_entry(
            AtomEntry::new()
                .id(format!("https://example.com/post-{i}"))
                .title(format!("Entry {i}"))
                .updated("2026-06-28T00:00:00Z")
                .summary(format!("Summary for entry {i}")),
        );
    }
    feed
}

use std::sync::LazyLock;

static SMALL_DATA: LazyLock<RssData> =
    LazyLock::new(|| generate_large_rss_data(10));
static MEDIUM_DATA: LazyLock<RssData> =
    LazyLock::new(|| generate_large_rss_data(100));
static LARGE_DATA: LazyLock<RssData> =
    LazyLock::new(|| generate_large_rss_data(1000));
static SMALL_XML: LazyLock<String> =
    LazyLock::new(|| generate_rss(&SMALL_DATA).unwrap());
static MEDIUM_XML: LazyLock<String> =
    LazyLock::new(|| generate_rss(&MEDIUM_DATA).unwrap());
static LARGE_XML: LazyLock<String> =
    LazyLock::new(|| generate_rss(&LARGE_DATA).unwrap());
static SMALL_ATOM: LazyLock<AtomFeed> =
    LazyLock::new(|| generate_large_atom_feed(10));
static MEDIUM_ATOM: LazyLock<AtomFeed> =
    LazyLock::new(|| generate_large_atom_feed(100));
static LARGE_ATOM: LazyLock<AtomFeed> =
    LazyLock::new(|| generate_large_atom_feed(1000));
static SMALL_ATOM_XML: LazyLock<String> =
    LazyLock::new(|| generate_atom(&SMALL_ATOM).unwrap());
static LARGE_ATOM_XML: LazyLock<String> =
    LazyLock::new(|| generate_atom(&LARGE_ATOM).unwrap());

fn benchmark_generate_rss(c: &mut Criterion) {
    let mut group = c.benchmark_group("Generate RSS");
    group
        .sample_size(100)
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(8));
    group.bench_function("Small", |b| {
        b.iter(|| generate_rss(black_box(&*SMALL_DATA)))
    });
    group.bench_function("Medium", |b| {
        b.iter(|| generate_rss(black_box(&*MEDIUM_DATA)))
    });
    group.bench_function("Large", |b| {
        b.iter(|| generate_rss(black_box(&*LARGE_DATA)))
    });
    group.finish();
}

fn benchmark_parse_rss(c: &mut Criterion) {
    let mut group = c.benchmark_group("Parse RSS");
    group
        .sample_size(100)
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(8));
    group.bench_function("Small", |b| {
        b.iter(|| parse_rss(black_box(&*SMALL_XML), None))
    });
    group.bench_function("Medium", |b| {
        b.iter(|| parse_rss(black_box(&*MEDIUM_XML), None))
    });
    group.bench_function("Large", |b| {
        b.iter(|| parse_rss(black_box(&*LARGE_XML), None))
    });
    group.finish();
}

fn benchmark_generate_atom(c: &mut Criterion) {
    let mut group = c.benchmark_group("Generate Atom 1.0");
    group
        .sample_size(100)
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(8));
    group.bench_function("Small (10)", |b| {
        b.iter(|| generate_atom(black_box(&*SMALL_ATOM)))
    });
    group.bench_function("Medium (100)", |b| {
        b.iter(|| generate_atom(black_box(&*MEDIUM_ATOM)))
    });
    group.bench_function("Large (1000)", |b| {
        b.iter(|| generate_atom(black_box(&*LARGE_ATOM)))
    });
    group.finish();
}

fn benchmark_detect_feed_format(c: &mut Criterion) {
    let mut group = c.benchmark_group("Detect feed format");
    group
        .sample_size(200)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3));
    group.bench_function("RSS (small)", |b| {
        b.iter(|| detect_feed_format(black_box(&SMALL_XML)))
    });
    group.bench_function("RSS (large)", |b| {
        b.iter(|| detect_feed_format(black_box(&LARGE_XML)))
    });
    group.bench_function("Atom (small)", |b| {
        b.iter(|| detect_feed_format(black_box(&SMALL_ATOM_XML)))
    });
    group.bench_function("Atom (large)", |b| {
        b.iter(|| detect_feed_format(black_box(&LARGE_ATOM_XML)))
    });
    group.finish();
}

fn benchmark_validate(c: &mut Criterion) {
    let mut group = c.benchmark_group("Validate");
    group
        .sample_size(100)
        .warm_up_time(Duration::from_secs(2))
        .measurement_time(Duration::from_secs(4));
    group.bench_function("RSS validate (1000 items)", |b| {
        b.iter(|| {
            let _ = black_box(LARGE_DATA.validate());
        })
    });
    group.bench_function("Atom validate (1000 entries)", |b| {
        b.iter(|| {
            let _ = black_box(LARGE_ATOM.validate());
        })
    });
    group.finish();
}

criterion_group!(
    benches,
    benchmark_generate_rss,
    benchmark_parse_rss,
    benchmark_generate_atom,
    benchmark_detect_feed_format,
    benchmark_validate,
);
criterion_main!(benches);
