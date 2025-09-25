use clap::Parser;
use color_eyre::Result;
use color_eyre::eyre::{Context, bail};
use readability_js::Readability;
use std::fs::File;
use std::io::Write;
use std::io::{self, Read};
use std::path::PathBuf;
use url::Url;

#[derive(Parser)]
#[command()]
struct Args {
    #[arg(
           help = "Input html file or URL (reads from stdin if not provided)",
           value_hint = clap::ValueHint::AnyPath
       )]
    input: Option<String>,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let (html, urlstr) = get_html(args.input)?;

    let parser = Readability::new().wrap_err("could not create Readability")?;
    let article = parser
        .extract(&html, urlstr.as_deref(), None)
        .wrap_err("extraction")?;

    let convert_to_markdown = true;
    if convert_to_markdown {
        let markdown = html2md::parse_html(&article.content);
        io::stdout().lock().write_all(markdown.as_bytes())?;
    } else {
        io::stdout().lock().write_all(article.content.as_bytes())?;
    };

    Ok(())
}

fn get_html(input: Option<String>) -> Result<(String, Option<String>)> {
    if input.is_none() {
        // Nothing is given, read stdin
        let mut html = String::new();
        io::stdin()
            .lock()
            .read_to_string(&mut html)
            .wrap_err("could not read stdin")?;
        return Ok((html, None));
    }
    let input = input.unwrap();

    let path = PathBuf::from(&input);

    // First try if the file exists
    if let Ok(true) = path.try_exists()
        && path.is_file()
    {
        let mut html = String::new();
        let mut file =
            File::open(&path).wrap_err_with(|| format!("could not open file {:#?}", path))?;
        file.read_to_string(&mut html)
            .wrap_err_with(|| format!("could not read file {:#?}", path))?;
        return Ok((html, None));
    }

    if let Some(url) = try_parse_url(&input) {
        let body: String = ureq::get(url.as_str())
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36")
            .call()
            .wrap_err("requesting url")?
            .body_mut()
            .read_to_string()
            .wrap_err("reading response")?;
        return Ok((body, Some(url.to_string())));
    }

    // error out with file not found
    bail!("file not found: {}", &input);
}

fn try_parse_url(input: &str) -> Option<Url> {
    // Helper function to validate URL
    let is_valid_http_url = |url: &Url| -> bool {
        (url.scheme() == "http" || url.scheme() == "https")
            && url.host_str().is_some()
            && !url.host_str().unwrap().is_empty()
            && !url.host_str().unwrap().contains("..")
            && url.host_str().unwrap().contains('.')
    };

    if let Ok(url) = Url::parse(input)
        && is_valid_http_url(&url)
    {
        return Some(url);
    }

    let https_attempt = format!("https://{}", &input);
    if let Ok(url) = Url::parse(&https_attempt)
        && is_valid_http_url(&url)
    {
        return Some(url);
    }
    None
}
