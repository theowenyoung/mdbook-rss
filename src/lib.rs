use std::fs;
use std::path::PathBuf;

use getset::Getters;
use globset::{GlobBuilder, GlobMatcher};
use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use regex::Regex;
use rss::Item;
use url::Url;

mod feed;

/// The file name relative to SUMMARY.md where the generated RSS feed is written
const RSS_FILE_NAME: &str = "rss.xml";

/// This fallback pattern is used, when there's no date pattern configured in this preprocessor's
/// section in the book.toml.
const CONFIG_DATE_PATTERN_DEFAULT: &str = r"\d{4}-\d{2}-\d{2}";

pub struct RssProcessor;

impl RssProcessor {
    pub fn new() -> RssProcessor {
        RssProcessor
    }
}

impl Preprocessor for RssProcessor {
    /// This is what's expected to exist in the book.toml as `[preprocessor.<name>]`
    fn name(&self) -> &str {
        "rss"
    }

    fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book, Error> {
        let config = RssConfig::from_book_config(&ctx.config, self.name())?;

        let rss_items: Vec<Item> = book
            .iter()
            .filter_map(|book_item| feed::RssItem::from_book_item(&book_item, &config).ok())
            .map(|rss_item| rss_item.item())
            .collect();
        eprintln!("Collected RSS items: {}", rss_items.len());

        let rss_channel = feed::rss_channel(config, rss_items)?;

        let rss_path: PathBuf = ctx.config.book.src.join(RSS_FILE_NAME);
        eprintln!("Writing RSS feed to {:?} ...", rss_path);
        fs::write(rss_path, rss_channel.to_string())?;

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html"
    }
}

/// Struct representation of available config for this preprocessor that can be used in book.toml
#[derive(Debug, Getters)]
#[getset(get = "pub")]
struct RssConfig {
    author: String,
    files_glob: GlobMatcher,
    date_pattern: Regex,
    title: String,
    description: String,
    url_base: Url,
}

impl RssConfig {
    fn from_book_config(
        book_config: &mdbook::Config,
        preprocessor_name: &str,
    ) -> Result<RssConfig, Error> {
        let title = match &book_config.book.title {
            Some(title) => title.to_string(),
            None => anyhow::bail!("Can't find book title. Please check your book.toml"),
        };

        let description = match &book_config.book.description {
            Some(description) => description.to_string(),
            None => anyhow::bail!("Can't find book description. Please check your book.toml"),
        };

        let author = book_config.book.authors.join(", ");

        let preprocessor_config = match book_config.get_preprocessor(preprocessor_name) {
            Some(cfg) => cfg,
            None => anyhow::bail!("Can't find preprocessor config section. Please check the documentation and update your book.toml"),
        };

        let files_glob = match preprocessor_config.get("files-glob") {
            Some(files_glob) => match files_glob.as_str() {
                Some(files_glob) => GlobBuilder::new(files_glob)
                    .literal_separator(true)
                    .build()?
                    .compile_matcher(),
                None => anyhow::bail!("Expected files-glob to be a string!"),
            },
            None => anyhow::bail!(
                "Can't find files-glob in preprocessor config. Please check your book.toml"
            ),
        };

        let date_pattern = match preprocessor_config.get("date-pattern") {
            Some(date_pattern) => match date_pattern.as_str() {
                Some(date_pattern) => date_pattern,
                None => anyhow::bail!("Expected date-pattern to be a string!"),
            },
            None => CONFIG_DATE_PATTERN_DEFAULT,
        };
        let date_pattern = match Regex::new(date_pattern) {
            Ok(regex) => regex,
            Err(e) => anyhow::bail!(e),
        };

        let url_base = match preprocessor_config.get("url-base") {
            Some(url_base) => match url_base.as_str() {
                Some(url_base) => match Url::parse(url_base) {
                    Ok(url_base) => url_base,
                    Err(e) => anyhow::bail!("Expected url-base to be a valid URL! {}", e),
                },
                None => anyhow::bail!("Expected url-base to be a string!"),
            },
            None => anyhow::bail!(
                "Can't find url-base in preprocessor config. Please check your book.toml"
            ),
        };

        Ok(RssConfig {
            author,
            files_glob,
            date_pattern,
            title,
            description,
            url_base,
        })
    }
}
