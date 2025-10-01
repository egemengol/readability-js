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

## Why `readability-js`?

This crate uses Mozilla's actual Readability.js library implementation - the same code that powers Firefox Reader Mode. Creating a `Readability` instance takes ~30ms while processing a document takes ~10ms which is good enough for most applications, negligible compared to the accuracy benefits.

## How It Works

This crate embeds Mozilla's Readability.js library using the QuickJS JavaScript engine. The JavaScript bundle combines:

- **Mozilla Readability.js**: The core algorithm from Firefox Reader Mode
- **linkedom**: A fast DOM implementation for server-side JavaScript

The extraction process:

1. Rust loads the HTML and calls into the embedded JavaScript context
2. linkedom parses the HTML into a DOM tree (fast, server-optimized parsing)
3. Mozilla's Readability.js analyzes the DOM structure and content patterns
4. The algorithm identifies the main content container and removes clutter
5. Clean HTML and extracted data are returned to Rust

This approach ensures we use the exact same algorithm as Firefox while maintaining excellent performance through the lightweight QuickJS engine.

## License

Licensed under the Universal Permissive License v1.0 (UPL-1.0)

### Dependencies

This crate bundles the following JavaScript libraries:

- **Mozilla Readability.js** - Licensed under Apache License 2.0 (using unmodified files is permitted)
- **linkedom** - Licensed under ISC License (highly permissive, no restrictions on use)

Both dependencies use permissive licenses that fully allow bundling unmodified source code. No modifications were made to either library, ensuring full license compliance.
