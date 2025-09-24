use rquickjs::{Context as QuickContext, Ctx, Function, Object, Runtime, Value};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Direction {
    /// Left-to-Right
    Ltr,
    /// Right-to-Left
    Rtl,
}

/// Represents a parsed article from Readability.js
#[derive(Debug, Clone, PartialEq)]
pub struct Article {
    /// Title of the article (parsed or inferred from document)
    pub title: String,

    /// Author byline metadata
    pub byline: Option<String>,

    /// Content direction
    pub direction: Option<Direction>,

    /// HTML content of the processed article
    pub content: String,

    /// Plain-text content with all HTML tags removed
    pub text_content: String,

    /// Length of article content in characters
    pub length: u32,

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

#[derive(Default, Debug, Clone)]
pub struct ReadabilityOptions {
    pub debug: Option<bool>,
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
    pub fn new() -> Self {
        Self::default()
    }
    pub fn debug(mut self, val: bool) -> Self {
        self.debug = Some(val);
        self
    }
    pub fn max_elems_to_parse(mut self, val: usize) -> Self {
        self.max_elems_to_parse = Some(val);
        self
    }
    pub fn nb_top_candidates(mut self, val: usize) -> Self {
        self.nb_top_candidates = Some(val);
        self
    }
    pub fn char_threshold(mut self, val: usize) -> Self {
        self.char_threshold = Some(val);
        self
    }
    pub fn classes_to_preserve(mut self, val: Vec<String>) -> Self {
        self.classes_to_preserve = Some(val);
        self
    }
    pub fn keep_classes(mut self, val: bool) -> Self {
        self.keep_classes = Some(val);
        self
    }
    pub fn disable_jsonld(mut self, val: bool) -> Self {
        self.disable_jsonld = Some(val);
        self
    }
    pub fn link_density_modifier(mut self, val: f32) -> Self {
        self.link_density_modifier = Some(val);
        self
    }

    fn build<'js>(self, ctx: Ctx<'js>) -> Result<Object<'js>> {
        let obj = Object::new(ctx).map_err(|e| ReadabilityError::JsEvaluation {
            context: "failed to create options object".into(),
            source: e,
        })?;

        if let Some(val) = self.debug {
            obj.set("debug", val)
                .map_err(|e| ReadabilityError::JsEvaluation {
                    context: "failed to set debug option".into(),
                    source: e,
                })?;
        }
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

#[derive(Error, Debug)]
pub enum ReadabilityError {
    #[error("Failed to parse HTML: {0}")]
    HtmlParseError(String),

    #[error("Content failed readability check")]
    ReadabilityCheckFailed,

    #[error("Failed to extract readable content: {0}")]
    ExtractionError(String),

    #[error("Failed to evaluate JavaScript: {context}")]
    JsEvaluation {
        context: String,
        #[source] // This attribute is key!
        source: rquickjs::Error,
    },

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

pub struct Readability {
    context: QuickContext,
}
impl Readability {
    pub fn new() -> Result<Self> {
        let runtime = Runtime::new().js_context("Failed to create runtime")?;
        let context = QuickContext::full(&runtime).js_context("Failed to create context")?;

        // context.with(|ctx| {
        //     // Load JSDOMParser
        //     let jsdom_parser_code = include_str!("../vendor/linkedom/worker.js");
        //     ctx.eval::<(), _>(jsdom_parser_code)
        //         .js_context("Failed to load linkedom")?;

        //     // Load Readability
        //     let readability_code = include_str!("../vendor/readability/Readability.js");
        //     ctx.eval::<(), _>(readability_code)
        //         .js_context("Failed to load Readability")?;

        //     // Load our functions
        //     let script = include_str!("./script.js");
        //     ctx.eval::<(), _>(script)
        //         .js_context("Failed to load script")?;

        //     Ok(())
        // })?;

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

    /// Extract readable content unconditionally
    pub fn extract(
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

            println!("{}", &html[..100]);
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
