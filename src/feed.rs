//! The functions in this module do the actual work for generating the RSS feed from the book.
use std::path::PathBuf;

use clap::{crate_name, crate_version};
use getset::Getters;
use gray_matter::matter::Matter;
use mdbook::book::BookItem;
use rss::{Channel, ChannelBuilder, Item, ItemBuilder};
use serde::Deserialize;

use super::RssConfig;

/// [Front matter](https://crates.io/crates/gray_matter) metadata for a chapter, that's used for
/// the generated RSS item, if available.
#[derive(Debug, Deserialize, Getters)]
#[getset(get = "pub")]
struct FrontMatter {
    date: String,
    description: Option<String>,
}

/// Construct an [`rss::Channel`] from an [`RssConfig`] and [`rss::Item`]s, taking information
/// about the resulting RSS feed from the config.
///
/// The resulting Channel is the RSS feed.
pub(super) fn rss_channel(
    mdbook_rss_config: RssConfig,
    rss_items: Vec<Item>,
) -> Result<Channel, anyhow::Error> {
    match ChannelBuilder::default()
        .title(mdbook_rss_config.title())
        .description(mdbook_rss_config.description())
        .items(rss_items)
        .generator(format!("{} {}", crate_name!(), crate_version!()))
        .build()
    {
        channel => Ok(channel),
    }
}

/// "Parse" an [`rss::Item`] from a [`BookItem`], potentially mutating the `BookItem` to
/// remove preprocessor-specific information that's not to be included in the final rendered book.
///
/// Information used for generating the `rss::Item` is a combination of data from the book itself
/// (e.g. chapter title) and front matter, if available.
/// See [`FrontMatter`] for fields that are used.
///
/// When the returned `Result` is an `Err` variant, either the chapter did not match specific
/// criteria like the book's configured file glob or date pattern, or the available configuration
/// was somehow not usable along the way.
pub(super) fn item(book_item: &mut BookItem, config: &RssConfig) -> Result<Item, anyhow::Error> {
    match book_item {
        BookItem::Chapter(chapter) => {
            anyhow::ensure!(
                chapter.path.is_some(),
                "Skipping draft chapter: {}",
                chapter
            );
            // unwrapping because we're skipping above if this is none
            let chapter_path = chapter.path.clone().unwrap();
            anyhow::ensure!(
                config.files_glob().is_match(&chapter_path),
                "Chapter {:?} does not match files glob {} - skipping",
                chapter_path,
                config.files_glob().glob()
            );

            let front_matter: Matter<gray_matter::engine::yaml::YAML> = Matter::new();
            let original_markdown = chapter.content.to_owned();
            let front_matter = front_matter.matter(original_markdown);

            // we need the markdown without front matter for the chapter now and later for the
            // html, so we need to get it before deserializing to `FrontMatter` reusing the name.
            //
            // NOTE: this may remove front matter used by other preprocessors running after this
            // one! :(
            let markdown_without_front_matter = front_matter.content;
            chapter.content = markdown_without_front_matter.clone();

            let front_matter: FrontMatter = front_matter.data.deserialize()?;

            let chapter_name = chapter.name.to_owned();
            let pub_date: String = front_matter.date().to_owned();
            let description: Option<String> = front_matter
                .description()
                .as_ref()
                .map(|description| description.to_owned());

            // unwrapping because invalid UTF-8 would have already exited above
            let chapter_path_html = PathBuf::from(chapter_path)
                .with_extension("html")
                .to_str()
                .unwrap()
                .to_string();
            let chapter_link = match config.url_base().join(&chapter_path_html) {
                Ok(url) => url,
                Err(e) => anyhow::bail!(e),
            };
            let chapter_content_html = markdown::to_html(&markdown_without_front_matter);
            let author = config.author().to_owned();
            let rss_item = ItemBuilder::default()
                .title(chapter_name)
                .description(description)
                .author(Some(author))
                .pub_date(Some(pub_date))
                .link(Some(chapter_link.to_string()))
                .content(Some(chapter_content_html))
                .build();

            Ok(rss_item)
        }
        item @ _ => anyhow::bail!("{:?} is not a chapter", item),
    }
}
