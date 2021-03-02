# mdbook-rss
A [preprocessor][mdbook-dev-docs] for [mdBook][mdbook-repo] that generates an RSS feed from your chapters.

The code is built on the [example][mdbook-preprocessor-example] from the documentation.

## Usage
First, make sure to install this to your `$PATH` so `mdbook` can find and use it:
```
cargo install --git <url to this repository>
```

Then configure your mdbook to use it as a preprocessor:
```toml
[preprocessor.rss]
files-glob = "posts/*.md"
url-base = "https://example.com/"
date-pattern = "\\d{4}-\\d{2}-\\d{2}"
```

Available configuration keys taken from the book.toml:
- `files-glob`: A file glob used to specify which files to include in the RSS feed
- `url-base`: A URL that, in combination with the files' path is used to build links to the articles.  
  Basically this is a combination of the domain where the mdbook is hosted and the book's [`site-url`](https://rust-lang.github.io/mdBook/format/config.html#html-renderer-options) option.  
  Note that due to implementation detail of [`url::Url::join`](https://docs.rs/url/2.2.1/url/struct.Url.html#method.join) this should end with a '/'.
- `date-pattern`: A regex to be used for extracting the articles' publication date from the filename.  
  This is optional; the default is shown in the example above.

The resulting RSS feed is written to an `rss.xml` file next to your `SUMMARY.md`, so it can be accessed via `<url-base>/rss.xml`.


## License
The code in this repository is released under the [**Mozilla Public License Version 2.0**](LICENSE).


[mdbook-dev-docs]: https://rust-lang.github.io/mdBook/for_developers/preprocessors.html
[mdbook-preprocessor-example]: https://rust-lang.github.io/mdBook/for_developers/preprocessors.html#hooking-into-mdbook
[mdbook-repo]: https://github.com/rust-lang/mdBook
