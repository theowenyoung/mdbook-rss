# mdbook-rss

A [preprocessor][mdbook-dev-docs] for [mdBook][mdbook-repo] that generates an RSS feed from your chapters.

The code is built on the [example][mdbook-preprocessor-example] from the documentation.

## Usage and Configuration

First, make sure to install this to your `$PATH` so `mdbook` can find and use it:

```
cargo install --git https://github.com/theowenyoung/mdbook-rss
```

### Configuration: Feed

This section describes "global" configuration for the feed itself.

`mdbook-rss` looks for a `[preprocessor.rss]` section in your book.toml:

```toml
[preprocessor.rss]
files-glob = "posts/*.md"
url-base = "https://example.com/"
```

Available configuration keys taken from the book.toml:

- `files-glob`: A file glob used to specify which files to include in the RSS feed
- `url-base`: A URL that, in combination with the files' path is used to build links to the articles.  
  Basically this is a combination of the domain where the mdbook is hosted and the book's [`site-url`](https://rust-lang.github.io/mdBook/format/config.html#html-renderer-options) option.  
  Note that due to implementation detail of [`url::Url::join`](https://docs.rs/url/2.2.1/url/struct.Url.html#method.join) this should end with a '/'.

The resulting RSS feed is written to an `rss.xml` file next to your `SUMMARY.md`, so it can be accessed via `<url-base>/rss.xml`.

### Configuration: Book Chapters and RSS Items

This section describes configuration on a per-chapter base.

In addition to global configuration, you may configure some attributes of each of your book's chapters, by defining a front matter like this:

```markdown
---
date: Wed, 03 Mar 2021 12:00:00 GMT
description: A helpful example
---

# Example

Rest of your markdown
```

Currently only `date` and `description` are supported with the description being optional.

> **NOTE:** This front matter ist removed from each chapter, so it won't be available anymore for preprocessors running after this one!

## Known Issues/Potential Improvements

- There are now actual tests yet. The only testing I did was checking the resulting feed in a feed reader.
- The RSS items' content is HTML rendered by this preprocessor using the [markdown crate](https://crates.io/crates/markdown); this means mdbook-specific features may be rendered incorrectly in the generated RSS feed.
- Front matter is removed which makes it unavailable/incompatible with other preprocessors running after this which also use front matter.  
  Keep front matter and remove it with a separate preprocessor used for removing front matter?
- This preprocessor currently assumes each RSS item is written by the same person(s) and just comma-separates the book's `authors` field for each RSS item.  
  This could be overridden in front matter?
- The titles for the RSS feed itself and each item is taken from the book and chapters.  
  This could be overridden in book.toml and front matter?

## License

The code in this repository is released under the [**Mozilla Public License Version 2.0**](LICENSE).

[mdbook-dev-docs]: https://rust-lang.github.io/mdBook/for_developers/preprocessors.html
[mdbook-preprocessor-example]: https://rust-lang.github.io/mdBook/for_developers/preprocessors.html#hooking-into-mdbook
[mdbook-repo]: https://github.com/rust-lang/mdBook
