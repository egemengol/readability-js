# readability-js

[![Crates.io](https://img.shields.io/crates/v/readability-js)](https://crates.io/crates/readability-js)
[![Documentation](https://docs.rs/readability-js/badge.svg)](https://docs.rs/readability-js)
[![License](https://img.shields.io/crates/l/readability-js)](https://github.com/egemengol/readability-js/blob/main/LICENSE)

Extract clean, readable content from web pages using Mozilla's Readability.js algorithm.

This crate provides both a Rust library and CLI tool for extracting main content from HTML documents, removing navigation, ads, and other clutter. It uses the same algorithm that powers Firefox Reader Mode.

## Installation

### CLI Tool

```bash
cargo install readability-js-cli
```

### Library

Add to your `Cargo.toml`:

```toml
[dependencies]
readability-js = "0.1"
```

## Quick Start

### CLI Usage

```bash
# Extract from URL
readable https://example.com/article > article.md

# Process local file
readable article.html > clean.md

# Use in pipelines
curl -s https://news.site/story | readable | less
```

### Library Usage

```rust
use readability_js::Readability;

// Create parser (reuse for multiple documents)
let reader = Readability::new()?;

// Extract content
let html = std::fs::read_to_string("article.html")?;
let article = reader.parse_with_url(&html, "https://example.com")?;

println!("Title: {}", article.title);
println!("Author: {}", article.byline.unwrap_or_default());
println!("Content: {}", article.content);
```

## Features

- **Production Algorithm**: Uses Mozilla's Readability.js from Firefox
- **Rich Metadata**: Extracts titles, authors, publication dates, and content
- **Multiple Formats**: HTML and plain text output
- **CLI Tool**: Converts to clean Markdown
- **High Performance**: Reusable parser instances for batch processing
- **Error Recovery**: Handles malformed HTML and edge cases

## Performance

Creating a `Readability` instance is expensive (~50-100ms) due to JavaScript engine initialization. Once created, parsing individual documents is fast (~10ms). Reuse the same instance when processing multiple documents.

## Documentation

- [API Documentation](https://docs.rs/readability-js)
- [Examples](./examples/)

## How It Works

This crate embeds Mozilla's Readability.js library using a JavaScript engine. The algorithm:

1. Analyzes page structure and content patterns
2. Identifies the main content container
3. Removes navigation, ads, and sidebar elements
4. Extracts metadata from HTML meta tags and content
5. Returns clean HTML suitable for reading

## License

Licensed under the Universal Permissive License v1.0 (UPL-1.0)
