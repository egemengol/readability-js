//! A Rust wrapper for Mozilla's Readability.js, allowing you to extract the
//! primary readable content from any HTML page.
//!
//! This crate uses an embedded JavaScript engine to run the original, battle-tested
//! Readability.js code, ensuring high-quality parsing and content extraction.
//!
//! # Example
//!
//! ```no_run
//! use readability_js::{Readability, ReadabilityOptions};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let html = "<html>...your html content...</html>";
//!     let readability = Readability::new()?;
//!     let article = readability.extract(html, Some("https://example.com"), None)?;
//!
//!     println!("Title: {}", article.title);
//!     println!("Content length: {}", article.length);
//!     Ok(())
//! }
//! ```

mod readability;
pub use readability::{Readability, ReadabilityError, ReadabilityOptions};
