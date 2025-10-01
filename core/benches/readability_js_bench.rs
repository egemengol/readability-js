use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use readability_js::Readability;

// Sample HTML content for testing
const SIMPLE_HTML: &str = include_str!("./simple.html");
const COMPLEX_HTML: &str = include_str!("./complex.html");

fn bench_readability_new(c: &mut Criterion) {
    c.bench_function("readability_new", |b| {
        b.iter(|| {
            let reader = Readability::new().expect("Failed to create Readability instance");
            std::hint::black_box(reader)
        })
    });
}

fn bench_parse_with_url(c: &mut Criterion) {
    // Create reader outside the benchmark to avoid initialization overhead
    let reader =
        Readability::new().expect("Failed to create Readability instance for bench_parse_with_url");

    let mut group = c.benchmark_group("parse_with_url");

    // Set sample size to reduce benchmark time if needed
    group.sample_size(10);

    group.bench_with_input(
        BenchmarkId::new("simple_html", SIMPLE_HTML.len()),
        &SIMPLE_HTML,
        |b, html| {
            b.iter(|| {
                let result = reader
                    .parse_with_url(std::hint::black_box(html), "https://example.com/article")
                    .expect("Failed to parse simple HTML");
                std::hint::black_box(result)
            })
        },
    );

    // Only benchmark complex HTML if it has content
    if !COMPLEX_HTML.is_empty() && COMPLEX_HTML.len() > 100 {
        group.bench_with_input(
            BenchmarkId::new("complex_html", COMPLEX_HTML.len()),
            &COMPLEX_HTML,
            |b, html| {
                b.iter(|| {
                    let result = reader
                        .parse_with_url(std::hint::black_box(html), "https://example.com/article")
                        .expect("Failed to parse complex HTML");
                    std::hint::black_box(result)
                })
            },
        );
    }

    group.finish();
}

fn bench_parse_without_url(c: &mut Criterion) {
    let reader = Readability::new()
        .expect("Failed to create Readability instance for bench_parse_without_url");

    c.bench_function("parse_without_url", |b| {
        b.iter(|| {
            let result = reader
                .parse(std::hint::black_box(SIMPLE_HTML))
                .expect("Failed to parse HTML without URL");
            std::hint::black_box(result)
        })
    });
}

criterion_group!(
    benches,
    bench_readability_new,
    bench_parse_with_url,
    bench_parse_without_url
);
criterion_main!(benches);
