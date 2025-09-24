use clap::Parser;
use color_eyre::Result;
use color_eyre::eyre::{Context, bail};
use log::info;
use readability_js::Readability;
use std::fs::File;
use std::io::{self, Read, Write};
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
    // simple_logger::init_with_level(log::Level::Info).wrap_err("logger init")?;
    color_eyre::install()?;
    let args = Args::parse();

    let (html, urlstr) = get_html(args.input)?;

    let parser = Readability::new().wrap_err("could not create Readability")?;
    let url = urlstr.as_ref().map(|x| x.as_str());
    let article = parser.extract(&html, url, None).wrap_err("extraction")?;

    io::stdout()
        .lock()
        .write_all(article.content.as_bytes())
        .wrap_err("could not write to stdout")?;

    Ok(())
}

fn get_html(input: Option<String>) -> Result<(String, Option<String>)> {
    if input.is_none() {
        // Nothing is given, read stdin
        info!("waiting on stdin");
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
        info!("file found");
        let mut html = String::new();
        let mut file =
            File::open(&path).wrap_err_with(|| format!("could not open file {:#?}", path))?;
        file.read_to_string(&mut html)
            .wrap_err_with(|| format!("could not read file {:#?}", path))?;
        return Ok((html, None));
    }

    if let Some(url) = try_parse_url(&input) {
        info!("url: {}", url);
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

    if let Ok(url) = Url::parse(&input) {
        if is_valid_http_url(&url) {
            info!("valid url parsed: {}", url);
            return Some(url);
        }
    }

    let https_attempt = format!("https://{}", &input);
    if let Ok(url) = Url::parse(&https_attempt) {
        if is_valid_http_url(&url) {
            info!("valid url parsed with https:// prefixed: {}", url);
            return Some(url);
        }
    }
    None
}
