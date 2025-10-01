use rquickjs::{Context as QuickContext, Ctx, Function, Object, Runtime, Value};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Direction {
    /// Left-to-Right
    Ltr,
    /// Right-to-Left
    Rtl,
}

/// Parsed article content and metadata extracted by Readability.
///
/// All fields except `title`, `content`, `text_content`, and `length` are optional
/// and depend on the input HTML having appropriate metadata.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Article {
    /// Extracted or inferred article title
    pub title: String,

    /// Clean HTML content (safe for display)
    pub content: String,

    /// Plain text with all HTML stripped
    pub text_content: String,

    /// Character count of the content
    pub length: u32,

    /// Author byline metadata
    pub byline: Option<String>,

    /// Content direction
    pub direction: Option<Direction>,

    /// Article description or short excerpt
    pub excerpt: Option<String>,

    /// Name of the website
    pub site_name: Option<String>,

    /// Content language code (BCP 47), if detectable
    pub language: Option<String>,

    /// Published time in ISO 8601 or site format, if detectable
    pub published_time: Option<String>,
}

impl<'js> TryFrom<Value<'js>> for Article {
    type Error = ReadabilityError;

    fn try_from(value: Value<'js>) -> Result<Self> {
        let obj = value.as_object().ok_or_else(|| {
            ReadabilityError::ExtractionError(
                "Expected JavaScript object, got a different type".into(),
            )
        })?;

        let title = obj
            .get::<_, String>("title")
            .map_err(|e| ReadabilityError::JsEvaluation {
                context: "failed to get title".into(),
                source: e,
            })?;

        let byline = obj
            .get::<_, Value>("byline")
            .map_err(|e| ReadabilityError::JsEvaluation {
                context: "failed to get byline".into(),
                source: e,
            })?;
        let byline = if byline.is_null() || byline.is_undefined() {
            None
        } else {
            Some(
                byline
                    .get::<String>()
                    .map_err(|e| ReadabilityError::JsEvaluation {
                        context: "failed to get byline as string".into(),
                        source: e,
                    })?,
            )
        };

        let dir = obj
            .get::<_, Value>("dir")
            .map_err(|e| ReadabilityError::JsEvaluation {
                context: "failed to get dir".into(),
                source: e,
            })?;
        let direction = if dir.is_null() || dir.is_undefined() {
            None
        } else {
            let dir_str = dir
                .get::<String>()
                .map_err(|e| ReadabilityError::JsEvaluation {
                    context: "failed to get dir as string".into(),
                    source: e,
                })?;
            match dir_str.as_str() {
                "ltr" => Some(Direction::Ltr),
                "rtl" => Some(Direction::Rtl),
                _ => None,
            }
        };

        let content =
            obj.get::<_, String>("content")
                .map_err(|e| ReadabilityError::JsEvaluation {
                    context: "failed to get content".into(),
                    source: e,
                })?;
        let text_content =
            obj.get::<_, String>("textContent")
                .map_err(|e| ReadabilityError::JsEvaluation {
                    context: "failed to get text_content".into(),
                    source: e,
                })?;
        let length = obj
            .get::<_, u32>("length")
            .map_err(|e| ReadabilityError::JsEvaluation {
                context: "failed to get length".into(),
                source: e,
            })?;

        let excerpt =
            obj.get::<_, Value>("excerpt")
                .map_err(|e| ReadabilityError::JsEvaluation {
                    context: "failed to get excerpt".into(),
                    source: e,
                })?;
        let excerpt = if excerpt.is_null() || excerpt.is_undefined() {
            None
        } else {
            Some(
                excerpt
                    .get::<String>()
                    .map_err(|e| ReadabilityError::JsEvaluation {
                        context: "failed to get excerpt as string".into(),
                        source: e,
                    })?,
            )
        };

        let site_name =
            obj.get::<_, Value>("siteName")
                .map_err(|e| ReadabilityError::JsEvaluation {
                    context: "failed to get site_name".into(),
                    source: e,
                })?;
        let site_name = if site_name.is_null() || site_name.is_undefined() {
            None
        } else {
            Some(
                site_name
                    .get::<String>()
                    .map_err(|e| ReadabilityError::JsEvaluation {
                        context: "failed to get site_name as string".into(),
                        source: e,
                    })?,
            )
        };

        let language = obj
            .get::<_, Value>("lang")
            .map_err(|e| ReadabilityError::JsEvaluation {
                context: "failed to get lang".into(),
                source: e,
            })?;
        let language = if language.is_null() || language.is_undefined() {
            None
        } else {
            Some(
                language
                    .get::<String>()
                    .map_err(|e| ReadabilityError::JsEvaluation {
                        context: "failed to get lang as string".into(),
                        source: e,
                    })?,
            )
        };

        let published_time =
            obj.get::<_, Value>("publishedTime")
                .map_err(|e| ReadabilityError::JsEvaluation {
                    context: "failed to get published_time".into(),
                    source: e,
                })?;
        let published_time =
            if published_time.is_null() || published_time.is_undefined() {
                None
            } else {
                Some(published_time.get::<String>().map_err(|e| {
                    ReadabilityError::JsEvaluation {
                        context: "failed to get published_time as string".into(),
                        source: e,
                    }
                })?)
            };

        Ok(Article {
            title,
            byline,
            direction,
            content,
            text_content,
            length,
            excerpt,
            site_name,
            language,
            published_time,
        })
    }
}

/// Configuration options for content extraction.
///
/// Created with [`ReadabilityOptions::new`] and used with
/// [`Readability::parse_with_options`].
///
/// See also: [`Readability::parse`] for basic extraction without options.
/// # Examples
///
/// ```rust
/// use readability_js::ReadabilityOptions;
///
/// // Fine-tuned for news sites
/// let opts = ReadabilityOptions::new()
///     .char_threshold(500)        // Require more content
///     .nb_top_candidates(10)      // Consider more candidates
///     .keep_classes(true)         // Preserve CSS classes
///     .classes_to_preserve(vec!["highlight".into(), "code".into()]);
/// ```
#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReadabilityOptions {
    pub max_elems_to_parse: Option<usize>,
    pub nb_top_candidates: Option<usize>,
    pub char_threshold: Option<usize>,
    pub classes_to_preserve: Option<Vec<String>>,
    pub keep_classes: Option<bool>,
    pub disable_jsonld: Option<bool>,
    pub link_density_modifier: Option<f32>,
    // TODO: serializer and allowed_video_regex
}

impl ReadabilityOptions {
    /// Creates a new options builder with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum number of DOM elements to parse.
    ///
    /// Limits processing to avoid performance issues on very large documents.
    /// Default is typically around 0 (unlimited).
    ///
    /// # Arguments
    /// * `val` - Maximum elements to process (0 = unlimited)
    pub fn max_elems_to_parse(mut self, val: usize) -> Self {
        self.max_elems_to_parse = Some(val);
        self
    }

    /// Set number of top content candidates to consider.
    ///
    /// The algorithm identifies potential content containers and ranks them.
    /// Higher values may improve accuracy but reduce performance.
    /// Default is typically 5.
    ///
    /// # Arguments
    /// * `val` - Number of candidates to consider (recommended: 5-15)
    pub fn nb_top_candidates(mut self, val: usize) -> Self {
        self.nb_top_candidates = Some(val);
        self
    }

    /// Set minimum character threshold for readable content.
    ///
    /// Content with fewer characters will fail the readability check.
    /// Lower values are more permissive but may include navigation/ads.
    /// Default is typically 140 characters.
    ///
    /// # Arguments
    /// * `val` - Minimum character count (recommended: 50-500)
    pub fn char_threshold(mut self, val: usize) -> Self {
        self.char_threshold = Some(val);
        self
    }

    /// Specify CSS classes to preserve in the output.
    ///
    /// By default, most CSS classes are stripped from the cleaned HTML.
    /// Use this to preserve important styling classes.
    ///
    /// # Arguments
    /// * `val` - Vector of class names to preserve (e.g., `vec!["highlight".into()]`)
    pub fn classes_to_preserve(mut self, val: Vec<String>) -> Self {
        self.classes_to_preserve = Some(val);
        self
    }

    /// Whether to preserve CSS classes in the output.
    ///
    /// When true, CSS classes are preserved in the cleaned HTML.
    /// When false (default), most classes are stripped.
    ///
    /// # Arguments
    /// * `val` - true to preserve classes, false to strip them
    pub fn keep_classes(mut self, val: bool) -> Self {
        self.keep_classes = Some(val);
        self
    }

    /// Disable JSON-LD metadata extraction.
    ///
    /// JSON-LD structured data can provide additional article metadata
    /// (author, publish date, etc.). Disable this if you don't need
    /// metadata or if it causes issues.
    ///
    /// # Arguments
    /// * `val` - true to disable JSON-LD parsing, false to enable it
    pub fn disable_jsonld(mut self, val: bool) -> Self {
        self.disable_jsonld = Some(val);
        self
    }

    /// Modify the link density calculation.
    ///
    /// Content with high link density is often navigation rather than article
    /// content. This modifier adjusts how strictly link density is evaluated.
    /// Values > 1.0 are more permissive, < 1.0 are stricter.
    ///
    /// # Arguments
    /// * `val` - Link density modifier (recommended: 0.5-2.0, default: 1.0)
    pub fn link_density_modifier(mut self, val: f32) -> Self {
        self.link_density_modifier = Some(val);
        self
    }

    fn build<'js>(self, ctx: Ctx<'js>) -> Result<Object<'js>> {
        let obj = Object::new(ctx).map_err(|e| ReadabilityError::JsEvaluation {
            context: "failed to create options object".into(),
            source: e,
        })?;

        if let Some(val) = self.max_elems_to_parse {
            obj.set("maxElemsToParse", val)
                .map_err(|e| ReadabilityError::JsEvaluation {
                    context: "failed to set maxElemsToParse option".into(),
                    source: e,
                })?;
        }
        if let Some(val) = self.nb_top_candidates {
            obj.set("nbTopCandidates", val)
                .map_err(|e| ReadabilityError::JsEvaluation {
                    context: "failed to set nbTopCandidates option".into(),
                    source: e,
                })?;
        }
        if let Some(val) = self.char_threshold {
            obj.set("charThreshold", val)
                .map_err(|e| ReadabilityError::JsEvaluation {
                    context: "failed to set charThreshold option".to_string(),
                    source: e,
                })?;
        }
        if let Some(ref val) = self.classes_to_preserve {
            obj.set("classesToPreserve", val.clone()).map_err(|e| {
                ReadabilityError::JsEvaluation {
                    context: "failed to set classesToPreserve option".to_string(),
                    source: e,
                }
            })?;
        }
        if let Some(val) = self.keep_classes {
            obj.set("keepClasses", val)
                .map_err(|e| ReadabilityError::JsEvaluation {
                    context: "failed to set keepClasses option".to_string(),
                    source: e,
                })?;
        }
        if let Some(val) = self.disable_jsonld {
            obj.set("disableJSONLD", val)
                .map_err(|e| ReadabilityError::JsEvaluation {
                    context: "failed to set disableJSONLD option".to_string(),
                    source: e,
                })?;
        }
        if let Some(val) = self.link_density_modifier {
            obj.set("linkDensityModifier", val)
                .map_err(|e| ReadabilityError::JsEvaluation {
                    context: "failed to set linkDensityModifier option".to_string(),
                    source: e,
                })?;
        }
        Ok(obj)
    }
}

// #[derive(Default, Debug, Clone)]
// struct ReadabilityCheckOptions {
//     pub min_content_length: Option<usize>, // default 140
//     pub min_score: Option<usize>,          // default 20
//                                            // TODO visibility checker
// }

// impl ReadabilityCheckOptions {
//     pub fn new() -> Self {
//         Self::default()
//     }
//     pub fn min_content_length(mut self, val: usize) -> Self {
//         self.min_content_length = Some(val);
//         self
//     }
//     pub fn min_score(mut self, val: usize) -> Self {
//         self.min_score = Some(val);
//         self
//     }

//     fn build<'js>(self, ctx: Ctx<'js>) -> Result<Object<'js>> {
//         let obj = Object::new(ctx).map_err(|e| ReadabilityError::JsEvaluation {
//             context: "failed to create check options object".to_string(),
//             source: e,
//         })?;

//         if let Some(val) = self.min_content_length {
//             obj.set("minContentLength", val)
//                 .map_err(|e| ReadabilityError::JsEvaluation {
//                     context: "failed to set minContentLength option".to_string(),
//                     source: e,
//                 })?
//         }
//         if let Some(val) = self.min_score {
//             obj.set("minScore", val)
//                 .map_err(|e| ReadabilityError::JsEvaluation {
//                     context: "failed to set minScore option".to_string(),
//                     source: e,
//                 })?;
//         }
//         Ok(obj)
//     }
// }
//
/// Errors that can occur during content extraction.
#[derive(Error, Debug)]
pub enum ReadabilityError {
    /// HTML could not be parsed (malformed, empty, etc.)
    ///
    /// This typically occurs when:
    /// - HTML is severely malformed or incomplete
    /// - Empty or whitespace-only input
    /// - Input contains non-HTML content
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use readability_js::Readability;
    /// let reader = Readability::new()?;
    /// // This will likely fail with HtmlParseError
    /// let result = reader.parse("<not valid html>");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[error("Failed to parse HTML: {0}")]
    HtmlParseError(String),

    /// Content failed internal readability checks
    ///
    /// This usually means:
    /// - Page has too little readable content (< 140 characters by default)
    /// - Content couldn't be reliably distinguished from navigation/ads
    /// - Page is mostly navigation, ads, or other non-content elements
    /// - Content has too high link density (likely navigation)
    ///
    /// # What to do
    ///
    /// Try lowering the `char_threshold` in [`ReadabilityOptions`] or check
    /// if the HTML actually contains substantial article content:
    ///
    /// ```rust
    /// # use readability_js::{Readability, ReadabilityOptions};
    /// let options = ReadabilityOptions::new().char_threshold(50);
    /// let reader = Readability::new()?;
    /// let article = reader.parse_with_options(&html, None, Some(options))?;
    /// # Ok::<(), readability_js::ReadabilityError>(())
    /// ```
    #[error("Content failed readability check")]
    ReadabilityCheckFailed,

    /// Content extraction failed for other reasons
    ///
    /// This is a catch-all error for unexpected extraction failures that don't
    /// fit into other categories. Often indicates issues with the JavaScript
    /// execution environment or unexpected content structures.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use readability_js::{Readability, ReadabilityError};
    /// let reader = Readability::new()?;
    /// match reader.parse(&html) {
    ///     Err(ReadabilityError::ExtractionError(msg)) => {
    ///         eprintln!("Extraction failed: {}", msg);
    ///         // Maybe try with different options or fallback processing
    ///     }
    ///     Ok(article) => println!("Success: {}", article.title),
    ///     Err(e) => eprintln!("Other error: {}", e),
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[error("Failed to extract readable content: {0}")]
    ExtractionError(String),

    /// JavaScript engine evaluation error
    ///
    /// Occurs when the embedded JavaScript engine fails to execute Readability.js
    /// code. This could indicate:
    /// - Memory constraints
    /// - JavaScript syntax errors in the bundled code
    /// - Runtime exceptions in the JavaScript environment
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use readability_js::{Readability, ReadabilityError};
    /// let reader = Readability::new()?;
    /// match reader.parse(&html) {
    ///     Err(ReadabilityError::JsEvaluation { context, source }) => {
    ///         eprintln!("JavaScript error in {}: {}", context, source);
    ///         // This usually indicates a bug - please report it!
    ///     }
    ///     Ok(article) => println!("Success: {}", article.title),
    ///     Err(e) => eprintln!("Other error: {}", e),
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[error("Failed to evaluate JavaScript: {context}")]
    JsEvaluation {
        context: String,
        #[source]
        source: rquickjs::Error,
    },

    /// Invalid input parameters (usually base URL)
    ///
    /// This error occurs when:
    /// - Base URL has invalid format or unsupported scheme
    /// - URL uses dangerous schemes like `javascript:` or `data:`
    /// - URL is not HTTP(S) when validation is enabled
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use readability_js::{Readability, ReadabilityError};
    /// let reader = Readability::new()?;
    /// // This will fail with InvalidOptions
    /// let result = reader.parse_with_url(&html, "javascript:alert('xss')");
    /// assert!(matches!(result, Err(ReadabilityError::InvalidOptions(_))));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[error("Invalid options: {0}")]
    InvalidOptions(String),
}

trait JsResultExt<T> {
    fn js_context(self, context: &str) -> Result<T>;
}

impl<T> JsResultExt<T> for std::result::Result<T, rquickjs::Error> {
    fn js_context(self, context: &str) -> Result<T> {
        self.map_err(|source| ReadabilityError::JsEvaluation {
            context: context.into(),
            source,
        })
    }
}

type Result<T> = std::result::Result<T, ReadabilityError>;

/// The main readability parser that extracts clean content from HTML.
///
/// Uses Mozilla's Readability.js algorithm running in an embedded JavaScript engine.
/// Create once and reuse for multiple extractions - the JS context initialization
/// is expensive.
///
/// # Examples
///
/// ```rust
/// use readability_js::{Readability, ReadabilityOptions};
///
/// // Create parser (expensive - reuse this!)
/// let reader = Readability::new()?;
///
/// // Basic extraction
/// let article = reader.extract(html, Some("https://example.com"), None)?;
///
/// // With custom options
/// let options = ReadabilityOptions::new()
///     .char_threshold(500);
/// let article = reader.extract(html, Some("https://example.com"), Some(options))?;
/// # Ok::<(), readability_js::ReadabilityError>(())
/// ```
///
/// # Thread Safety
///
/// `Readability` instances are **not** thread-safe (`!Send + !Sync`). Each instance
/// contains an embedded JavaScript engine that cannot be moved between threads or
/// shared between threads.
pub struct Readability {
    context: QuickContext,
}
impl Readability {
    /// Creates a new readability parser.
    ///
    /// # Performance
    ///
    /// This operation is expensive (50-100ms) as it initializes a JavaScript engine
    /// and loads the Readability.js library. Create one instance and reuse it for
    /// multiple extractions.
    ///
    /// # JavaScript Engine
    ///
    /// This method initializes an embedded QuickJS runtime. The JavaScript code
    /// executed is Mozilla's Readability.js library and is considered safe for
    /// processing untrusted HTML input.
    pub fn new() -> Result<Self> {
        let runtime = Runtime::new().js_context("Failed to create runtime")?;
        let context = QuickContext::full(&runtime).js_context("Failed to create context")?;

        context.with(|ctx| {
            let readability_code = include_str!("../vendor/readability/Readability.js");
            ctx.eval::<(), _>(readability_code)
                .js_context("Failed to load Readability")?;

            let bundle = include_str!("../js/bundled.js");
            ctx.eval::<(), _>(bundle)
                .js_context("Failed to load bundle")?;

            Ok(())
        })?;

        Ok(Self { context })
    }

    fn validate_base_url(url: &str) -> Result<String> {
        if url.starts_with("javascript:") || url.starts_with("data:") {
            return Err(ReadabilityError::InvalidOptions(
                "Invalid base URL scheme".into(),
            ));
        }

        // Optional: Parse with url crate for stricter validation
        match url::Url::parse(url) {
            Ok(parsed) if matches!(parsed.scheme(), "http" | "https") => Ok(url.to_string()),
            _ => Err(ReadabilityError::InvalidOptions(
                "Base URL must be HTTP(S)".into(),
            )),
        }
    }

    /// Extract readable content from HTML.
    ///
    /// This is the main extraction method. It processes the HTML to remove
    /// ads, navigation, sidebars and other clutter, leaving just the main article content.
    ///
    /// # Arguments
    ///
    /// * `html` - The HTML content to process. Should be a complete HTML document.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use readability_js::Readability;
    ///
    /// let html = r#"
    ///   <html>
    ///     <body>
    ///       <article>
    ///         <h1>Breaking News</h1>
    ///         <p>Important news content here...</p>
    ///       </article>
    ///       <nav>Navigation menu</nav>
    ///       <aside>Advertisement</aside>
    ///     </body>
    ///   </html>
    /// "#;
    ///
    /// let reader = Readability::new()?;
    /// let article = reader.parse(html)?;
    ///
    /// assert_eq!(article.title, "Breaking News");
    /// assert!(article.content.contains("Important news content"));
    /// // Navigation and ads are removed from the output
    /// # Ok::<(), readability_js::ReadabilityError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`ReadabilityError`] if:
    /// * The HTML is malformed or empty (`HtmlParseError`)
    /// * The page fails readability checks (`ReadabilityCheckFailed`)
    /// * JavaScript evaluation fails (`JsEvaluation`)
    ///
    /// # Performance
    ///
    /// This method is fast (typically <10ms) once the [`Readability`] instance
    /// is created. The expensive operation is [`Readability::new()`] which should
    /// be called once and reused.
    pub fn parse(&self, html: &str) -> Result<Article> {
        self.extract(html, None, None)
    }

    /// Extract readable content from HTML with URL context.
    ///
    /// The URL helps with better link resolution and metadata extraction.
    ///
    /// # Arguments
    ///
    /// * `html` - The HTML content to extract from
    /// * `base_url` - The original URL of the page for link resolution
    ///
    /// # Examples
    /// ```rust
    /// use readability_js::Readability;
    ///
    /// let reader = Readability::new()?;
    /// let article = reader.parse_with_url(html, "https://example.com/article")?;
    /// // Links in the article will be properly resolved
    /// # Ok::<(), readability_js::ReadabilityError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The HTML is malformed or cannot be parsed ([`ReadabilityError::HtmlParseError`])
    /// * The base URL is invalid ([`ReadabilityError::InvalidOptions`])
    /// * The content fails internal readability checks ([`ReadabilityError::ReadabilityCheckFailed`])
    /// * JavaScript evaluation fails ([`ReadabilityError::JsEvaluation`])
    pub fn parse_with_url(&self, html: &str, base_url: &str) -> Result<Article> {
        self.extract(html, Some(base_url), None)
    }

    /// Extract readable content with custom parsing options.
    ///
    /// # Arguments
    ///
    /// * `html` - The HTML content to extract from
    /// * `base_url` - Optional URL for link resolution
    /// * `options` - Custom parsing options
    ///
    /// # Examples
    /// ```rust
    /// use readability_js::{Readability, ReadabilityOptions};
    ///
    /// let options = ReadabilityOptions::new()
    ///     .char_threshold(500);
    ///
    /// let reader = Readability::new()?;
    /// let article = reader.parse_with_options(html, Some("https://example.com"), Some(options))?;
    /// # Ok::<(), readability_js::ReadabilityError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The HTML is malformed or cannot be parsed ([`ReadabilityError::HtmlParseError`])
    /// * The base URL is invalid ([`ReadabilityError::InvalidOptions`])
    /// * The content fails internal readability checks ([`ReadabilityError::ReadabilityCheckFailed`])
    /// * JavaScript evaluation fails ([`ReadabilityError::JsEvaluation`])
    pub fn parse_with_options(
        &self,
        html: &str,
        base_url: Option<&str>,
        options: Option<ReadabilityOptions>,
    ) -> Result<Article> {
        self.extract(html, base_url, options)
    }

    fn extract(
        &self,
        html: &str,
        base_url: Option<&str>,
        options: Option<ReadabilityOptions>,
    ) -> Result<Article> {
        let clean_base_url = match base_url {
            None => None,
            Some(url) => Some(Self::validate_base_url(url)?),
        };
        self.context.with(|ctx| {
            let extract_fn: Function = ctx
                .globals()
                .get("extract")
                .js_context("extract function not found")?;
            let options_obj = match options {
                None => None,
                Some(options) => Some(options.build(ctx.clone())?),
            };

            let result: Value = extract_fn
                .call((html, clean_base_url, options_obj))
                .js_context("Failed to call extract")?;

            // Check if result is an error object
            if let Some(obj) = result.as_object()
                && let Ok(error_type) = obj.get::<_, String>("errorType")
            {
                let error_msg = obj
                    .get::<_, String>("error")
                    .unwrap_or_else(|_| "Unknown error".to_string());

                return Err(match error_type.as_str() {
                    "HtmlParseError" => ReadabilityError::HtmlParseError(error_msg),
                    "ExtractionError" => ReadabilityError::ExtractionError(error_msg),
                    "RuntimeError" => ReadabilityError::JsEvaluation {
                        context: format!("JavaScript runtime error: {}", error_msg),
                        source: rquickjs::Error::Unknown,
                    },
                    _ => ReadabilityError::ExtractionError(format!(
                        "Unknown error type '{}': {}",
                        error_type, error_msg
                    )),
                });
            }

            // If not an error object, try to parse as Article
            Article::try_from(result)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_extraction() {
        let html = r#"
            <html>
            <head><title>Test Article Title</title></head>
            <body>
                <h1>This is a test article</h1>
                <p>This is the first paragraph with some content that should be long enough to be considered readable content by the readability algorithm.</p>
                <p>This is another paragraph with more content. It has enough text to make the article substantial and worth reading.</p>
                <p>And here's a third paragraph to make sure we have enough content for the readability parser to work with.</p>
            </body>
            </html>
        "#;

        let readability = Readability::new().unwrap();
        let article = readability
            .extract(html, Some("https://example.com"), None)
            .unwrap();

        assert_eq!(article.title, "Test Article Title");
        assert!(article.content.contains("first paragraph"));
        assert!(article.content.contains("another paragraph"));
        assert!(article.content.contains("third paragraph"));
        assert!(article.content.contains("<p>"));
        assert!(article.text_content.contains("This is a test article"));
        assert!(!article.text_content.contains("<"));
        assert!(article.length > 0);
    }
}
