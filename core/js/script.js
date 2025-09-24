import { DOMParser } from "linkedom";
// const { Readability, isProbablyReaderable } = require("@mozilla/readability");

function extract(html, baseUrl, options) {
  try {
    const domParser = new DOMParser();
    let doc;

    try {
      doc = domParser.parseFromString(html, "text/html");
    } catch (e) {
      console.error("Failed to parse HTML:", e.message);
      return {
        errorType: "HtmlParseError",
        error: "Failed to parse HTML: " + e.message,
      };
    }

    // TODO maybe add a base element that holds url
    // for readability to resolve relative urls

    const reader = new Readability(doc, options || {});
    let article;

    try {
      article = reader.parse();
    } catch (e) {
      return {
        errorType: "RuntimeError",
        error: "Readability runtime error: " + e.message,
      };
    }

    if (!article) {
      return {
        errorType: "ExtractionError",
        error: "Failed to extract readable content",
      };
    }

    // Return article directly on success
    return article;
  } catch (e) {
    return {
      errorType: "RuntimeError",
      error: "Unexpected error: " + e.message,
    };
  }
}

globalThis.extract = extract;
