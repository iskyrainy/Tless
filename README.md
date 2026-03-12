# Tless

Tless is a Rust-powered blog engine that combines a local authoring workflow with a small Actix Web server, Tera templates, Markdown rendering, and file watching. It can scaffold a site, create draft posts and pages, publish drafts into posts, render Markdown into HTML, and hot-reload configuration, templates, helpers, and source content while the server is running.

## Current scope

Implemented today:

- Site bootstrap with a default `tless.toml`, source folders, theme folders, and starter templates
- Draft, post, and page content management through the CLI
- Markdown rendering for posts into `public/`
- Live reload of config, source files, templates, and custom helper scripts
- Tera helper functions for URLs, lists, pagination, metadata, TOC generation, partials, and more
- Basic private-post access control through an access key and allowlist

Present in the CLI but not implemented yet:

- `tless site --generate`
- `tless site --deploy`
- `tless site --backup`

## Build

```bash
cargo build
```

Run the CLI help:

```bash
cargo run -- --help
```

## Quick start

1. Initialize a new site structure:

```bash
cargo run -- site --init
```

This creates a `blog/` directory in the current working directory.

2. Enter the generated site:

```bash
cd blog
```

3. Create a draft post and a page:

```bash
../target/debug/tless blog add hello-tless
../target/debug/tless page add about
```

4. Publish the draft into a post:

```bash
../target/debug/tless blog publish hello-tless
```

5. Start the development server:

```bash
../target/debug/tless server --run --port 8917
```

## Generated project layout

After `tless site --init`, the generated site looks like this:

```text
blog/
├── tless.toml
├── source/
│   ├── draft/
│   ├── post/
│   └── page/
├── theme/
│   └── base/
│       ├── layout/
│       └── resource/
├── public/
├── plugin/
├── helper/
└── statistic/
```

Key directories:

- `source/draft/`: unpublished Markdown posts
- `source/post/`: published Markdown posts
- `source/page/`: standalone pages rendered through Tera templates
- `theme/<name>/layout/`: Tera HTML templates
- `theme/<name>/resource/`: theme assets
- `public/`: rendered post HTML output plus `.post_hash.json`
- `helper/`: Rhai helper scripts loaded at runtime

## CLI commands

### Site

```bash
tless site --init
```

Creates the site skeleton and starter theme.

### Blog

Create a draft:

```bash
tless blog add hello-tless
```

Remove a draft or published post:

```bash
tless blog remove hello-tless
tless blog remove --class post hello-tless
```

Publish a draft into `source/post/`:

```bash
tless blog publish hello-tless
tless blog publish --prva hello-tless
```

`--prva` marks the post as private.

### Page

Create or remove a page:

```bash
tless page add about
tless page remove about
```

## Content format

Posts and pages are Markdown files with front matter.

Draft post created by `tless blog add <name>`:

```md
---
date: 2026-03-13 12:00:00
tags:
categories:
---

# New Blog
Write your content here.
```

Published posts carry more metadata:

```md
---
title: hello-tless
date: 2026-03-13 12:00:00
tags: [rust, blog]
categories: [general]
prva: false
---

# Hello
```

Pages default to a template layout:

```md
---
title: about
date: 2026-03-13 12:00:00
layout: index.html
---
```

Recognized metadata fields:

- `title`
- `date`
- `layout`
- `tags`
- `categories`
- `prva`

## Configuration

Tless reads `tless.toml` from the site root. The generated default config is:

```toml
[site]
title = "My Tless Site"
description = "A fast blog powered by Tless."
author = "Your Name"
url = "http://127.0.0.1:8917"
zone = "UTC"
theme = "base"
favicon = ""
menu = [
    { name = "Home", link = "/" },
    { name = "Example Post", link = "/archives/hello-tless" },
    { name = "Rust Tag", link = "/tags/rust" },
    { name = "General Category", link = "/categories/general" }
]

[auth]
ak = ""
allows = [
    "127.0.0.1"
]
```

Important fields:

- `site.url`: base URL used by helper functions
- `site.zone`: timezone used when generating timestamps for blog commands
- `site.theme`: folder name under `theme/`
- `auth.ak`: access key for `/login`
- `auth.allows`: initial IP allowlist for private post access

## Runtime behavior

When you run `tless server --run`, Tless:

1. Loads `tless.toml`
2. Parses `source/post/` and `source/page/`
3. Builds in-memory tag and category indexes
4. Loads Tera templates from `theme/<theme>/layout/*.html`
5. Renders posts into `public/`
6. Starts file watchers for config, content, templates, and helpers
7. Serves HTTP requests with Actix Web

Watched changes trigger reloads without restarting the process.

## Routes

Current server routes:

- `GET /`: returns a placeholder `Hello world!`
- `POST /login`: adds the caller IP to the allowlist when the submitted access key matches
- `GET /archives/{post}`: serves rendered post HTML from `public/`
- `GET /categories/{category}`: renders category listing
- `GET /tags/{tag}`: renders tag listing

There is also a `get_page` handler in the codebase for page rendering, but it is not currently registered on the Actix app.

## Templates and helpers

The default theme writes three templates:

- `index.html`
- `archive.html`
- `category.html`

Tless registers a substantial helper set into Tera, including helpers for:

- date formatting
- relative and full URL generation
- gravatar URLs
- CSS, JS, image, mail, favicon, feed, and generic link tags
- partial template rendering
- post, page, category, and tag listing
- pagination
- number formatting
- open graph metadata
- table-of-contents generation

Custom helpers can also be loaded from the `helper/` directory through Rhai scripts.

## Rendering model

Posts are rendered from Markdown with `pulldown-cmark` and wrapped with the `archive.html` template. Tless stores a SHA-256 hash per source file in `public/.post_hash.json` and skips re-rendering unchanged posts.

## Private posts

Private posts are published with:

```bash
tless blog publish --prva secret-note
```

Access flow:

- the post metadata carries `prva: true`
- `GET /archives/{post}` checks whether the request IP is in the allowlist
- `POST /login` accepts a JSON string matching `auth.ak`
- on success, the caller IP is appended to the in-memory allowlist

Note that the current implementation stores newly allowed IPs only in process memory. They are not written back to `tless.toml`.

## Development notes

- The executable expects `tless.toml` to exist in the current working directory when running the server or blog/page commands.
- `site --init` creates a nested `blog/` directory rather than initializing in place.
- Template reload, helper reload, config reload, and source reload are implemented with `notify` and `notify-debouncer-full`.
- Logging is enabled through `env_logger` with `RUST_LOG=debug`.

## Repository structure

The Rust project itself is organized as:

- `src/main.rs`: program entry point
- `src/cmd.rs`: CLI parsing and command dispatch
- `src/file/`: draft, post, and page file operations plus front matter parsing
- `src/site.rs`: site bootstrap and default theme/config generation
- `src/server/`: config loading, site indexing, helpers, rendering, watchers, and HTTP server

## License

This project is licensed under the terms of the [LICENSE](LICENSE) file.
