#![allow(missing_docs)]
use criterion::{
    criterion_group, criterion_main, AxisScale, Criterion,
    PlotConfiguration,
};
use rss_gen::{generate_rss, parse_rss, RssData, RssItem, RssVersion};
use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// Memory tracking allocator to monitor allocations
#[derive(Default)]
struct TrackingAllocator {
    allocated: AtomicUsize,
    peak_allocated: AtomicUsize,
}

impl TrackingAllocator {
    fn peak_allocated(&self) -> usize {
        self.peak_allocated.load(Ordering::Relaxed)
    }

    fn reset(&self) {
        self.allocated.store(0, Ordering::Relaxed);
        self.peak_allocated.store(0, Ordering::Relaxed);
    }
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            let old_allocated = self
                .allocated
                .fetch_add(layout.size(), Ordering::Relaxed);
            let new_allocated = old_allocated + layout.size();

            // Update peak if necessary
            let mut current_peak =
                self.peak_allocated.load(Ordering::Relaxed);
            loop {
                if new_allocated <= current_peak {
                    break;
                }
                match self.peak_allocated.compare_exchange_weak(
                    current_peak,
                    new_allocated,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(x) => current_peak = x,
                }
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        self.allocated.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}

#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator {
    allocated: AtomicUsize::new(0),
    peak_allocated: AtomicUsize::new(0),
};

fn generate_stress_rss_data(
    item_count: usize,
    content_size_multiplier: usize,
) -> RssData {
    let mut rss_data = RssData::new(Some(RssVersion::RSS2_0))
        .title("Stress Test Feed - Performance Benchmark")
        .link("https://stresstestfeed.example.com")
        .description("A large RSS feed designed for performance stress testing and memory profiling")
        .language("en-US")
        .pub_date("Mon, 01 Jan 2024 00:00:00 GMT")
        .last_build_date("Mon, 01 Jan 2024 12:00:00 GMT")
        .generator("RSS Gen Stress Test Suite v0.0.4")
        .atom_link("https://stresstestfeed.example.com/feed.xml")
        .category("Technology")
        .copyright("© 2024 Stress Test Suite")
        .managing_editor("editor@stresstestfeed.example.com")
        .webmaster("webmaster@stresstestfeed.example.com")
        .ttl("60");

    let base_description = "This is a comprehensive description for stress testing RSS generation and parsing performance under heavy load. ";
    let extended_description =
        base_description.repeat(content_size_multiplier);

    for i in 0..item_count {
        let item = RssItem::new()
            .title(format!("Stress Test Item #{} - Performance Benchmark", i))
            .link(format!("https://stresstestfeed.example.com/items/item-{}", i))
            .description(format!("{} Item number: {}", extended_description, i))
            .guid(format!("unique-stress-test-id-{}-{}", i, item_count))
            .pub_date("Mon, 01 Jan 2024 00:00:00 GMT")
            .author(format!("author{}@stresstestfeed.example.com", i % 5))
            .category(format!("Category{}", i % 10))
            .comments(format!("https://stresstestfeed.example.com/items/item-{}/comments", i));

        rss_data.add_item(item);
    }

    rss_data
}

fn benchmark_stress_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Stress Test - RSS Generation");
    group
        .sample_size(20)
        .warm_up_time(Duration::from_secs(5))
        .measurement_time(Duration::from_secs(15))
        .plot_config(
            PlotConfiguration::default()
                .summary_scale(AxisScale::Logarithmic),
        );

    // 10x stress test - 10K items
    let stress_10x_data = generate_stress_rss_data(10_000, 1);
    group.bench_function("10K Items (10x stress)", |b| {
        b.iter_custom(|iters| {
            ALLOCATOR.reset();
            let start = std::time::Instant::now();
            for _ in 0..iters {
                let _ = generate_rss(black_box(&stress_10x_data));
            }
            let elapsed = start.elapsed();
            println!(
                "10K Items - Peak memory: {} KB",
                ALLOCATOR.peak_allocated() / 1024
            );
            elapsed
        })
    });

    // 100x stress test - 100K items
    let stress_100x_data = generate_stress_rss_data(100_000, 1);
    group.bench_function("100K Items (100x stress)", |b| {
        b.iter_custom(|iters| {
            ALLOCATOR.reset();
            let start = std::time::Instant::now();
            for _ in 0..iters {
                let _ = generate_rss(black_box(&stress_100x_data));
            }
            let elapsed = start.elapsed();
            println!(
                "100K Items - Peak memory: {} MB",
                ALLOCATOR.peak_allocated() / (1024 * 1024)
            );
            elapsed
        })
    });

    // Content size scaling test
    let content_heavy_data = generate_stress_rss_data(1_000, 10);
    group.bench_function("1K Items Heavy Content", |b| {
        b.iter_custom(|iters| {
            ALLOCATOR.reset();
            let start = std::time::Instant::now();
            for _ in 0..iters {
                let _ = generate_rss(black_box(&content_heavy_data));
            }
            let elapsed = start.elapsed();
            println!(
                "Heavy Content - Peak memory: {} KB",
                ALLOCATOR.peak_allocated() / 1024
            );
            elapsed
        })
    });

    group.finish();
}

fn benchmark_stress_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("Stress Test - RSS Parsing");
    group
        .sample_size(20)
        .warm_up_time(Duration::from_secs(5))
        .measurement_time(Duration::from_secs(15))
        .plot_config(
            PlotConfiguration::default()
                .summary_scale(AxisScale::Logarithmic),
        );

    // Generate XML for parsing tests
    let stress_10x_data = generate_stress_rss_data(10_000, 1);
    let stress_10x_xml = generate_rss(&stress_10x_data)
        .expect("Failed to generate 10K XML");

    let stress_100x_data = generate_stress_rss_data(100_000, 1);
    let stress_100x_xml = generate_rss(&stress_100x_data)
        .expect("Failed to generate 100K XML");

    group.bench_function("Parse 10K Items (10x stress)", |b| {
        b.iter_custom(|iters| {
            ALLOCATOR.reset();
            let start = std::time::Instant::now();
            for _ in 0..iters {
                let _ = parse_rss(black_box(&stress_10x_xml), None);
            }
            let elapsed = start.elapsed();
            println!(
                "Parse 10K - Peak memory: {} KB",
                ALLOCATOR.peak_allocated() / 1024
            );
            elapsed
        })
    });

    group.bench_function("Parse 100K Items (100x stress)", |b| {
        b.iter_custom(|iters| {
            ALLOCATOR.reset();
            let start = std::time::Instant::now();
            for _ in 0..iters {
                let _ = parse_rss(black_box(&stress_100x_xml), None);
            }
            let elapsed = start.elapsed();
            println!(
                "Parse 100K - Peak memory: {} MB",
                ALLOCATOR.peak_allocated() / (1024 * 1024)
            );
            elapsed
        })
    });

    group.finish();
}

fn benchmark_sustained_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("Sustained Load Test");
    group
        .sample_size(10)
        .warm_up_time(Duration::from_secs(2))
        .measurement_time(Duration::from_secs(30)); // Long measurement for sustained load

    let medium_data = generate_stress_rss_data(1_000, 1);

    group.bench_function("Sustained Generation (30s)", |b| {
        b.iter_custom(|_iters| {
            ALLOCATOR.reset();
            let start = std::time::Instant::now();
            let end_time = start + Duration::from_secs(30);
            let mut iterations = 0u64;

            while std::time::Instant::now() < end_time {
                let _ = generate_rss(black_box(&medium_data));
                iterations += 1;
            }

            let elapsed = start.elapsed();
            let throughput = iterations as f64 / elapsed.as_secs_f64();
            println!(
                "Sustained load: {} ops/sec, Peak memory: {} KB",
                throughput,
                ALLOCATOR.peak_allocated() / 1024
            );

            elapsed
        })
    });

    group.finish();
}

criterion_group!(
    stress_benches,
    benchmark_stress_generation,
    benchmark_stress_parsing,
    benchmark_sustained_load
);
criterion_main!(stress_benches);
