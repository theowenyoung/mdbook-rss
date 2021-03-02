use std::path::PathBuf;

use clap::{crate_name, crate_version};
use mdbook::book::BookItem;
use rss::{Channel, ChannelBuilder, Item, ItemBuilder};

use super::RssConfig;

/// Construct an [`rss::Channel`] from an [`RssConfig`] and [`rss::Item`]s, taking information
/// about the resulting RSS feed from the config.
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
        Ok(channel) => Ok(channel),
        Err(e) => anyhow::bail!(e),
    }
}

/// Wrapper struct around [`rss::Item`] to encapsulate "parsing" a [`BookItem`] to an `rss::Item`.
///
/// When the returned `Result` is an `Err` variant, either the chapter did not match specific
/// criteria like the book's configured file glob or date pattern, or the available configuration
/// was somehow not usable along the way.
pub(super) struct RssItem(Item);

impl RssItem {
    pub(super) fn from_book_item(
        book_item: &BookItem,
        config: &RssConfig,
    ) -> Result<Self, anyhow::Error> {
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

                let chapter_name = chapter.name.to_owned();
                let filename = match chapter_path.file_name() {
                    Some(filename) => match filename.to_str() {
                        Some(filename) => filename,
                        None => anyhow::bail!("Chapter path is invalid UTF-8"),
                    },
                    None => anyhow::bail!("Chapter does not have a file name"),
                };
                let chapter_pub_date = match config.date_pattern().find(filename) {
                    Some(date_match) => date_match.as_str().to_string(),
                    None => anyhow::bail!(
                        "Chapter filename {:?} does not match date pattern: {} - skipping",
                        filename,
                        config.date_pattern()
                    ),
                };
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
                let chapter_content_html = markdown::to_html(&chapter.content);
                let author = config.author().to_owned();

                match ItemBuilder::default()
                    .title(chapter_name)
                    .author(Some(author))
                    .pub_date(chapter_pub_date)
                    .link(Some(chapter_link.into_string()))
                    .content(Some(chapter_content_html))
                    .build()
                {
                    Ok(item) => Ok(Self(item)),
                    Err(e) => anyhow::bail!(e),
                }
            }
            item @ _ => anyhow::bail!("{:?} is not a chapter", item),
        }
    }

    pub(super) fn item(self) -> Item {
        self.0
    }
}
