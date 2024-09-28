#![allow(missing_docs)]
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rss_gen::{generate_rss, parse_rss, RssData, RssItem, RssVersion};
use lazy_static::lazy_static;
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
                .description(format!("This is the description for item {}", i))
                .guid(format!("unique-id-{}", i))
                .pub_date("Mon, 01 Jan 2024 00:00:00 GMT"),
        );
    }

    rss_data
}

lazy_static! {
    static ref SMALL_DATA: RssData = generate_large_rss_data(10);
    static ref MEDIUM_DATA: RssData = generate_large_rss_data(100);
    static ref LARGE_DATA: RssData = generate_large_rss_data(1000);

    static ref SMALL_XML: String = generate_rss(&SMALL_DATA).unwrap();
    static ref MEDIUM_XML: String = generate_rss(&MEDIUM_DATA).unwrap();
    static ref LARGE_XML: String = generate_rss(&LARGE_DATA).unwrap();
}

fn benchmark_generate_rss(c: &mut Criterion) {
    let mut group = c.benchmark_group("Generate RSS");
    group.sample_size(100)
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
    group.sample_size(100)
         .warm_up_time(Duration::from_secs(3))
         .measurement_time(Duration::from_secs(8));
    group.bench_function("Small", |b| {
        b.iter(|| parse_rss(black_box(&*SMALL_XML)))
    });
    group.bench_function("Medium", |b| {
        b.iter(|| parse_rss(black_box(&*MEDIUM_XML)))
    });
    group.bench_function("Large", |b| {
        b.iter(|| parse_rss(black_box(&*LARGE_XML)))
    });
    group.finish();
}

criterion_group!(benches, benchmark_generate_rss, benchmark_parse_rss);
criterion_main!(benches);
