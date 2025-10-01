//! Extract clean, readable content from web pages using Mozilla's Readability.js algorithm.
//!
//! This crate provides both a Rust library and CLI tool for extracting the main content
//! from HTML documents, removing navigation, ads, and other clutter. It uses the same
//! algorithm as Firefox Reader Mode.
//!
//! # Quick Start
//!
//! ```rust
//! use readability_js::Readability;
//!
//! let html = r#"<html><body><h1>Article Title</h1><p>Main content...</p></body></html>"#;
//! let reader = Readability::new()?;
//! let article = reader.parse(&html)?;
//!
//! println!("Title: {}", article.title);
//! println!("Content: {}", article.content);
//! # Ok::<(), readability_js::ReadabilityError>(())
//! ```
//!
//! # Parsing with URL Context
//!
//! Providing a URL improves link resolution and metadata extraction:
//!
//! ```rust
//! use readability_js::Readability;
//!
//! let reader = Readability::new()?;
//! let article = reader.parse_with_url(&html, "https://example.com/article")?;
//! # Ok::<(), readability_js::ReadabilityError>(())
//! ```
//!
//! # Custom Options
//!
//! Configure the parsing behavior with [`ReadabilityOptions`]:
//!
//! ```rust
//! use readability_js::{Readability, ReadabilityOptions};
//!
//! let options = ReadabilityOptions::new()
//!     .char_threshold(500)
//!     .keep_classes(true);
//!
//! let reader = Readability::new()?;
//! let article = reader.parse_with_options(&html, Some("https://example.com"), Some(options))?;
//! # Ok::<(), readability_js::ReadabilityError>(())
//! ```
//!
//! # Performance Considerations
//!
//! Creating a [`Readability`] instance is expensive (~50-100ms) as it initializes a JavaScript
//! engine. Once created, parsing individual documents is fast (~10ms). Reuse the same instance
//! when processing multiple documents:
//!
//! ```rust
//! use readability_js::Readability;
//!
//! let reader = Readability::new()?;
//! for html in documents {
//!     let article = reader.parse(&html)?;
//!     process_article(article);
//! }
//! # Ok::<(), readability_js::ReadabilityError>(())
//! ```
//!
//! # Error Handling
//!
//! The most common error is [`ReadabilityError::ReadabilityCheckFailed`], which occurs
//! when the algorithm cannot extract sufficient readable content:
//!
//! ```rust
//! use readability_js::{Readability, ReadabilityError, ReadabilityOptions};
//!
//! let reader = Readability::new()?;
//! match reader.parse(&html) {
//!     Ok(article) => println!("Extracted: {}", article.title),
//!     Err(ReadabilityError::ReadabilityCheckFailed) => {
//!         // Try with lower threshold
//!         let options = ReadabilityOptions::new().char_threshold(100);
//!         let article = reader.parse_with_options(&html, None, Some(options))?;
//!         println!("Extracted with relaxed settings: {}", article.title);
//!     }
//!     Err(e) => return Err(e),
//! }
//! # Ok::<(), readability_js::ReadabilityError>(())
//! ```
//!
//! # CLI Usage
//!
//! The CLI tool extracts content and converts it to clean Markdown:
//!
//! ```bash
//! # Install the CLI tool
//! cargo install readability-js-cli
//!
//! # Process local files
//! readable article.html > article.md
//!
//! # Fetch and process URLs
//! readable https://example.com/news > news.md
//!
//! # Process from stdin (great for pipelines)
//! curl -s https://site.com/article | readable > clean.md
//!
//! # View directly in terminal
//! readable https://news.site/story | less
//! ```
//!
//! The CLI automatically:
//! - Detects whether input is a file path or URL
//! - Fetches web content with proper headers
//! - Converts the clean HTML to Markdown
//! - Handles errors gracefully
//!
//! # Troubleshooting
//!
//! ## "Content failed readability check"
//!
//! This happens when the page doesn't contain enough readable content or
//! the algorithm can't distinguish content from navigation. Try:
//!
//! ```rust
//! use readability_js::{Readability, ReadabilityOptions};
//!
//! let options = ReadabilityOptions::new()
//!     .char_threshold(100)         // Lower threshold (default: ~140)
//!     .nb_top_candidates(10)       // Consider more candidates
//!     .link_density_modifier(2.0); // More permissive with links
//!
//! let reader = Readability::new()?;
//! let article = reader.parse_with_options(&html, None, Some(options))?;
//! # Ok::<(), readability_js::ReadabilityError>(())
//! ```
//!
//! ## Poor extraction quality
//!
//! If the extracted content is incomplete or includes unwanted elements:
//!
//! ```rust
//! use readability_js::{Readability, ReadabilityOptions};
//!
//! // Better link resolution and metadata extraction
//! let reader = Readability::new()?;
//! let article = reader.parse_with_url(&html, "https://example.com/article")?;
//!
//! // Or preserve important CSS classes
//! let options = ReadabilityOptions::new()
//!     .keep_classes(true)
//!     .classes_to_preserve(vec!["highlight".into(), "code".into(), "caption".into()]);
//! let article = reader.parse_with_options(&html, None, Some(options))?;
//! # Ok::<(), readability_js::ReadabilityError>(())
//! ```
//!
//! ## Memory or performance issues
//!
//! For very large documents or resource-constrained environments:
//!
//! ```rust
//! use readability_js::{Readability, ReadabilityOptions};
//!
//! let options = ReadabilityOptions::new()
//!     .max_elems_to_parse(1000)   // Limit processing
//!     .nb_top_candidates(3);      // Fewer candidates = faster
//!
//! let reader = Readability::new()?;
//! let article = reader.parse_with_options(&html, None, Some(options))?;
//! # Ok::<(), readability_js::ReadabilityError>(())
//! ```

mod readability;
pub use readability::{Article, Direction, Readability, ReadabilityError, ReadabilityOptions};
