use std::fs;
use std::path::PathBuf;

use clap::{crate_name, crate_version};
use getset::{Getters, Setters};
use globset::{GlobBuilder, GlobMatcher};
use mdbook::book::{Book, BookItem};
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use regex::Regex;
use rss::{Channel, ChannelBuilder, Item, ItemBuilder};
use url::Url;

const RSS_FILE_NAME: &str = "rss.xml";

const CONFIG_TITLE_DEFAULT: &str = "RSS Feed";
const CONFIG_DATE_PATTERN_DEFAULT: &str = r"\d{4}-\d{2}-\d{2}";
const CONFIG_DESCRIPTION_DEFAULT: &str = "RSS Feed";

pub struct RssProcessor;

impl RssProcessor {
    pub fn new() -> RssProcessor {
        RssProcessor
    }
}

impl Preprocessor for RssProcessor {
    fn name(&self) -> &str {
        "rss"
    }

    fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book, Error> {
        let book_config = ctx.config.book.clone();
        let config = RssConfig::from_book_config(&ctx.config, self.name())?;

        let rss_items: Vec<Item> = book
            .iter()
            .filter_map(|book_item| match book_item {
                BookItem::Chapter(chapter) => {
                    eprintln!("processing chapter: {}", chapter);
                    if chapter.path.is_none() {
                        eprintln!("Skipping draft chapter: {}", chapter);
                        return None;
                    }

                    match config.files_glob() {
                        Some(glob) => {
                            // unwrapping because we're skipping above if this is none
                            let chapter_path = chapter.path.clone().unwrap();
                            let chapter_name = chapter.name.clone();
                            if glob.is_match(&chapter_path) {
                                let filename = chapter_path
                                    .file_name()
                                    .expect("Chapter path does not have a filename")
                                    .to_str()
                                    .expect("Chapter path is invalid UTF-8");
                                let chapter_pub_date: String =
                                    match config.date_pattern().clone().find(filename) {
                                        Some(m) => m.as_str().to_string(),
                                        None => {
                                            eprintln!(
                                                "Chapter filename {:?} does not match the date pattern for publish dates: {} - not including this chapter in RSS.",
                                                filename,
                                                config.date_pattern()
                                            );
                                            return None;
                                        },
                                    };
                                let chapter_path_html = PathBuf::from(chapter_path).with_extension("html").to_str().expect("Chapter link is invalid UTF-8").to_string();
                                let chapter_link = config.url_base().clone().expect("Expected url-base to be configured to build RSS links").join(&chapter_path_html).expect("Generated link is invalid");
                                let chapter_content = markdown::to_html(&chapter.content);
                                let author: String = ctx.config.book.authors.join(", ");
                                match ItemBuilder::default()
                                    .title(chapter_name)
                                    .author(author)
                                    .pub_date(chapter_pub_date)
                                    .link(Some(chapter_link.into_string()))
                                    .content(Some(chapter_content))
                                    .build()
                                {
                                    Ok(item) => Some(item),
                                    Err(e) => {
                                        eprintln!(
                                            "Failed building RSS item for chapter {}: {}",
                                            chapter, e
                                        );
                                        None
                                    }
                                }
                            } else {
                                eprintln!("Chapter does not match files glob: {:?} does not match {} - not including this chapter in RSS.", chapter_path, glob.glob());
                                None
                            }
                        }
                        None => {
                            eprintln!("No pattern found against which files can be matched. Please configure the `files` key for this preprocessor if this is a mistake.");
                            None
                        },
                    }
                }
                item @ _ => {
                    eprintln!("processing item {:?}", item);
                    None
                }
            })
            .collect();
        eprintln!("Collected RSS items: {}", rss_items.len());

        let rss_channel = match ChannelBuilder::default()
            .title(config.title())
            .description(config.description())
            .items(rss_items)
            .generator(format!("{} {}", crate_name!(), crate_version!()))
            .build()
        {
            Ok(channel) => channel,
            Err(e) => anyhow::bail!(e),
        };

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
#[derive(Debug, Getters, Setters)]
#[getset(get = "pub", set = "with_prefix")]
struct RssConfig {
    files_glob: Option<GlobMatcher>,
    date_pattern: Regex,
    title: String,
    description: String,
    url_base: Option<Url>,
}

impl RssConfig {
    fn from_book_config(
        book_config: &mdbook::Config,
        preprocessor_name: &str,
    ) -> Result<RssConfig, Error> {
        let mut rss_config = RssConfig::default();

        if let Some(title) = &book_config.book.title {
            rss_config.set_title(title.to_string());
        }

        if let Some(description) = &book_config.book.description {
            rss_config.set_description(description.to_string());
        }

        if let Some(preprocessor_config) = book_config.get_preprocessor(preprocessor_name) {
            if let Some(files_glob) = preprocessor_config.get("files-glob") {
                let glob = GlobBuilder::new(
                    &files_glob
                        .as_str()
                        .expect("Expected files-glob to be a string"),
                )
                .literal_separator(true)
                .build()?
                .compile_matcher();
                rss_config.set_files_glob(Some(glob));
            }

            if let Some(date_pattern) = preprocessor_config.get("date-pattern") {
                let regex = match Regex::new(
                    &date_pattern
                        .as_str()
                        .expect("Expected date-pattern to be a string"),
                ) {
                    Ok(regex) => regex,
                    Err(e) => anyhow::bail!(e),
                };
                rss_config.set_date_pattern(regex);
            }

            if let Some(url_base) = preprocessor_config.get("url-base") {
                let url_base: Url =
                    Url::parse(&url_base.as_str().expect("Expected url-base to be a string"))
                        .expect("Expected url-base to be a valid URL");
                rss_config.set_url_base(Some(url_base));
            }
        } else {
            eprintln!(
                "Preprocessor {} does not seem to be configured in {:#?}",
                preprocessor_name, book_config
            );
        }

        Ok(rss_config)
    }
}

impl std::default::Default for RssConfig {
    fn default() -> RssConfig {
        let files_glob = None;
        let date_pattern = Regex::new(CONFIG_DATE_PATTERN_DEFAULT)
            .expect("Failed to compile the default date pattern. Please report this as a bug.");
        let title = String::from(CONFIG_TITLE_DEFAULT);
        let description = String::from(CONFIG_DESCRIPTION_DEFAULT);
        let url_base = None;

        RssConfig {
            files_glob,
            date_pattern,
            title,
            description,
            url_base,
        }
    }
}
