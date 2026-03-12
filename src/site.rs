use std::{env, error::Error, fs, path::Path};

/// Initialize the site structure in the current directory.
pub fn init() -> Result<(), Box<dyn Error>> {
    let current_dir = match env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return Err("Can not get current dir".into()),
    };
    let current_dir = current_dir.join("blog");
    if current_dir.exists() {
        return Err("Site directory already exists.".into());
    }
    fs::create_dir(&current_dir)?;

    let conf_path = current_dir.join("tless.toml");
    fs::write(&conf_path, base_config_text())?;

    let dirs = vec!["source", "theme", "public", "plugin", "helper", "statistic"];
    for dir in dirs {
        fs::create_dir(current_dir.join(dir))?;
    }
    let blog_dirs = vec!["draft", "post", "page"];
    for dir in blog_dirs {
        fs::create_dir(current_dir.join("source").join(dir))?;
    }

    let theme_dir = current_dir.join("theme").join("base");
    fs::create_dir(&theme_dir)?;
    let layout_dir = theme_dir.join("layout");
    fs::create_dir(&layout_dir)?;
    let resource_dir = theme_dir.join("resource");
    fs::create_dir(&resource_dir)?;
    write_base_theme(&layout_dir)?;
    Ok(())
}

fn write_base_theme(layout_dir: &Path) -> Result<(), Box<dyn Error>> {
    fs::write(layout_dir.join("index.html"), base_index_theme_text())?;
    fs::write(layout_dir.join("archive.html"), base_archive_theme_text())?;
    fs::write(layout_dir.join("category.html"), base_category_theme_text())?;
    Ok(())
}

/// Generate a base configuration file content.
fn base_config_text() -> String {
    String::from(
        r#"# Tless configuration
# Update these values for your own site before publishing.
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
"#,
    )
}

fn base_style_text() -> &'static str {
    r#"
        :root {
            --bg: #f4efe7;
            --surface: rgba(255, 251, 245, 0.88);
            --surface-strong: #fffaf2;
            --line: rgba(34, 28, 20, 0.14);
            --text: #1f1a14;
            --muted: #6f6252;
            --accent: #0f766e;
            --accent-strong: #115e59;
            --shadow: 0 18px 48px rgba(49, 38, 27, 0.09);
            --radius: 22px;
            --max: 1024px;
        }

        * {
            box-sizing: border-box;
        }

        body {
            margin: 0;
            color: var(--text);
            font-family: "Iowan Old Style", "Palatino Linotype", "Book Antiqua", Georgia, serif;
            line-height: 1.65;
            background:
                radial-gradient(circle at top left, rgba(15, 118, 110, 0.12), transparent 28rem),
                linear-gradient(180deg, #fcf8f1 0%, var(--bg) 100%);
        }

        a {
            color: var(--accent-strong);
            text-decoration: none;
        }

        a:hover {
            text-decoration: underline;
        }

        .shell {
            max-width: var(--max);
            margin: 0 auto;
            padding: 32px 20px 64px;
        }

        .card {
            background: var(--surface);
            border: 1px solid var(--line);
            border-radius: var(--radius);
            box-shadow: var(--shadow);
            backdrop-filter: blur(10px);
        }

        .hero {
            padding: 32px;
            margin-bottom: 24px;
        }

        .section {
            padding: 28px;
        }

        .eyebrow {
            display: inline-block;
            margin-bottom: 12px;
            padding: 6px 10px;
            border-radius: 999px;
            background: #efe4d4;
            color: var(--muted);
            letter-spacing: 0.04em;
            text-transform: uppercase;
            font-size: 0.82rem;
        }

        h1, h2, h3 {
            margin: 0 0 12px;
            line-height: 1.12;
        }

        h1 {
            font-size: clamp(2.4rem, 5vw, 4.6rem);
        }

        h2 {
            font-size: clamp(1.35rem, 2.5vw, 1.9rem);
        }

        p {
            margin: 0 0 1rem;
        }

        .lede,
        .muted {
            color: var(--muted);
        }

        .stack > * + * {
            margin-top: 16px;
        }

        .grid {
            display: grid;
            grid-template-columns: 2fr 1fr;
            gap: 24px;
        }

        .menu,
        .links {
            display: flex;
            flex-wrap: wrap;
            gap: 10px;
            list-style: none;
            padding: 0;
            margin: 0;
        }

        .menu a,
        .pill {
            display: inline-flex;
            align-items: center;
            padding: 9px 14px;
            border-radius: 999px;
            border: 1px solid var(--line);
            background: var(--surface-strong);
        }

        .post-list,
        .pagination-list {
            list-style: none;
            padding: 0;
            margin: 0;
        }

        .post-list li + li {
            border-top: 1px solid var(--line);
        }

        .prose {
            font-size: 1.06rem;
        }

        .prose img {
            max-width: 100%;
            border-radius: 16px;
        }

        .prose pre,
        .prose code {
            font-family: "IBM Plex Mono", "SFMono-Regular", Consolas, monospace;
        }

        .prose pre {
            overflow-x: auto;
            padding: 16px;
            border-radius: 16px;
            background: #1f2937;
            color: #f9fafb;
        }

        .pagination {
            display: flex;
            align-items: center;
            gap: 12px;
            flex-wrap: wrap;
        }

        .pagination-list {
            display: flex;
            gap: 8px;
            flex-wrap: wrap;
        }

        .pagination-link,
        .pagination-prev,
        .pagination-next,
        .pagination-current {
            display: inline-flex;
            align-items: center;
            justify-content: center;
            min-width: 2.5rem;
            padding: 8px 12px;
            border-radius: 999px;
            border: 1px solid var(--line);
            background: var(--surface-strong);
        }

        .pagination-current {
            color: white;
            background: var(--accent);
            border-color: transparent;
        }

        .toc ul {
            list-style: none;
            padding: 0;
            margin: 0;
        }

        .toc li + li {
            margin-top: 8px;
        }

        .toc-level-2 { padding-left: 12px; }
        .toc-level-3 { padding-left: 24px; }
        .toc-level-4 { padding-left: 36px; }
        .toc-level-5 { padding-left: 48px; }
        .toc-level-6 { padding-left: 60px; }

        footer {
            margin-top: 28px;
            text-align: center;
            color: var(--muted);
            font-size: 0.95rem;
        }

        @media (max-width: 760px) {
            .grid {
                grid-template-columns: 1fr;
            }

            .hero,
            .section {
                padding: 22px;
            }
        }
    "#
}

fn base_index_theme_text() -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Tless Base Page</title>
    <style>{}</style>
</head>
<body>
    <main class="shell">
        <section class="hero card">
            <span class="eyebrow">Tless Base Theme</span>
            <h1>{{{{ page | default(value="Page") | upper }}}}</h1>
            <p class="lede">Starter template for named pages. Create more page templates alongside this file as your site grows.</p>
            <ul class="menu">
                <li><a href="/">Home</a></li>
                <li><a href="/archives/hello-tless">Example Post</a></li>
                <li><a href="/tags/rust">Rust Tag</a></li>
                <li><a href="/categories/general">General Category</a></li>
            </ul>
        </section>

        <section class="card section stack">
            <h2>What This Theme Includes</h2>
            <p class="muted">The scaffold includes working templates for archive pages and taxonomy pages, plus built-in helper functions for post lists, pagination, Open Graph tags, number formatting, and a generated table of contents.</p>
            <div class="links">
                <span class="pill">archive.html</span>
                <span class="pill">category.html</span>
                <span class="pill">index.html</span>
                <span class="pill">toc()</span>
                <span class="pill">open_graph()</span>
            </div>
        </section>

        <footer>Built with Tless.</footer>
    </main>
</body>
</html>
"#,
        base_style_text()
    )
}

fn base_archive_theme_text() -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Archive</title>
    <style>{}</style>
</head>
<body>
    <main class="shell">
        <section class="hero card">
            <span class="eyebrow">Archive</span>
            <h1>Article</h1>
            <p class="lede">This template is used when markdown is rendered into the public directory.</p>
            <ul class="menu">
                <li><a href="/">Home</a></li>
            </ul>
        </section>

        <section class="card section stack">
            <div class="toc">{{{{ toc(content=content, max_level=3) | safe }}}}</div>
            <article class="prose">{{{{ content | safe }}}}</article>
        </section>

        <footer>Generated by Tless.</footer>
    </main>
</body>
</html>
"#,
        base_style_text()
    )
}

fn base_category_theme_text() -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Taxonomy</title>
    <style>{}</style>
</head>
<body>
    <main class="shell">
        <section class="hero card">
            <span class="eyebrow">Taxonomy</span>
            <h1>{{{{ name }}}}</h1>
            <p class="lede">Shared template for category and tag routes.</p>
            <ul class="menu">
                <li><a href="/">Home</a></li>
            </ul>
        </section>

        <div class="grid">
            <section class="card section stack">
                <h2>Recent Posts</h2>
                <p class="muted">This starter view lists recent posts from the site index so the page stays useful before you add custom taxonomy filtering.</p>
                {{{{ list_posts(order=-1, list=true, amount=20, show_count=false) | safe }}}}
                <div class="pagination">
                    {{{{ paginator(current=1, total=3, base="?page=") | safe }}}}
                </div>
            </section>

            <aside class="stack">
                <section class="card section">
                    <h2>Categories</h2>
                    {{{{ list_categories(order=-1, list=true, amount=20, show_count=true) | safe }}}}
                </section>

                <section class="card section">
                    <h2>Tags</h2>
                    {{{{ list_tags(order=-1, list=true, amount=30, show_count=true) | safe }}}}
                </section>
            </aside>
        </div>

        <footer>Generated by Tless.</footer>
    </main>
</body>
</html>
"#,
        base_style_text()
    )
}
