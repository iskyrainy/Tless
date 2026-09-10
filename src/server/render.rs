use std::{
    fmt::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Result, anyhow};
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use futures::{StreamExt, stream};
use pulldown_cmark::{Options, Parser, html};
use tera::Context;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tracing::{error, info};

use crate::{
    file::{self, Metadata},
    server::{
        SITE, Site, TERA, extract_root_path, get_layout_path, get_public_path, get_source_path,
    },
};

/// Markdown default render options.
const DEFAULT_OPTIONS: Options = Options::all();

/// Render markdown to HTML string.
#[inline]
fn render(markdown: &str) -> String {
    let parser = Parser::new_ext(markdown, DEFAULT_OPTIONS);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

#[inline]
fn get_cpu() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        * 2
}

#[inline]
async fn render_file_class(metadata: &Metadata) -> Result<()> {
    let pub_dir = Arc::new(get_public_path("."));
    if let Some(cates) = &metadata.category {
        stream::iter(cates.iter().filter(|&c| !pub_dir.join(c).exists()))
            .map(|c| {
                let pub_dir = pub_dir.clone();
                async move {
                    let dst_dir = pub_dir.join("category").join(c);
                    fs::create_dir_all(&dst_dir).await?;
                    let mut context = Context::new();
                    context.insert("site", SITE.load().as_ref());
                    match TERA.load().render("category.html", &context) {
                        Ok(rendered) => {
                            let mut file = File::create(dst_dir.join("index.html")).await?;
                            file.write_all_buf(&mut rendered.as_bytes()).await?;
                            file.flush().await?;
                        }
                        Err(e) => {
                            return Err(anyhow!(
                                "Failed to render {} category: {}",
                                &metadata.title,
                                e
                            ));
                        }
                    };
                    Ok(())
                }
            })
            .buffer_unordered(get_cpu())
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<()>>()?;
    }

    if let Some(tags) = &metadata.tag {
        stream::iter(tags.iter().filter(|&c| !pub_dir.join(c).exists()))
            .map(|c| {
                let pub_dir = pub_dir.clone();
                async move {
                    let dst_dir = pub_dir.join("tag").join(c);
                    fs::create_dir_all(&dst_dir).await?;
                    let mut context = Context::new();
                    context.insert("site", SITE.load().as_ref());
                    match TERA.load().render("tag.html", &context) {
                        Ok(rendered) => {
                            let mut file = File::create(dst_dir.join("index.html")).await?;
                            file.write_all_buf(&mut rendered.as_bytes()).await?;
                            file.flush().await?;
                        }
                        Err(e) => {
                            return Err(anyhow!("Failed to render {} tag: {}", &metadata.title, e));
                        }
                    };
                    Ok(())
                }
            })
            .buffer_unordered(get_cpu())
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<()>>()?;
    }

    Ok(())
}

enum RenderType {
    Post,
    Page,
}

#[inline]
async fn render_file(src: &PathBuf, dst: &PathBuf, rt: RenderType) -> Result<()> {
    let metadata = file::parse_file(src)?;
    let file_str = fs::read_to_string(src).await?;
    let md_body = frontmatter_gen::extract(&file_str)
        .map(|(_, body)| body.to_string())
        .unwrap_or_default();

    let md_html_str = render(&md_body);
    let mut context = Context::new();
    context.insert("content", &md_html_str);
    context.insert("markdown", &md_body);
    context.insert("title", &metadata.title);
    context.insert("date", &metadata.date);
    context.insert("site", SITE.load().as_ref());
    let layout = metadata.layout.as_deref().unwrap_or(match rt {
        RenderType::Post => "post.html",
        RenderType::Page => "page.html",
    });
    match TERA.load().render(layout, &context) {
        Ok(rendered) => {
            let mut file = File::create(dst).await?;
            file.write_all_buf(&mut rendered.as_bytes()).await?;
            file.flush().await?;
        }
        Err(e) => {
            return Err(anyhow!("Failed to render {}: {}", &metadata.title, e));
        }
    };
    if let RenderType::Post = rt {
        render_file_class(&metadata).await?;
    }
    Ok(())
}

pub(crate) async fn render_post(paths: Vec<&PathBuf>) -> Result<()> {
    let pub_dir = Arc::new(get_public_path("."));
    stream::iter(paths)
        .map(|path| {
            let pub_dir = pub_dir.clone();
            async move {
                if let Some(name) = path.file_name() {
                    let name = name.to_string_lossy().to_string();
                    let dst_dir = pub_dir.join(&name);
                    fs::create_dir_all(&dst_dir).await?;
                    let dst_file = dst_dir.join("index.html");
                    render_file(path, &dst_file, RenderType::Post).await?;
                }
                Ok(())
            }
        })
        .buffer_unordered(get_cpu())
        .collect::<Vec<Result<()>>>()
        .await
        .into_iter()
        .collect::<Result<()>>()
}

pub(crate) async fn render_page(paths: Vec<&PathBuf>) -> Result<()> {
    let pub_dir = Arc::new(get_public_path("."));
    stream::iter(paths)
        .map(|path| {
            let pub_dir = pub_dir.clone();
            async move {
                if let Some(name) = path.file_name() {
                    let name = name.to_string_lossy().to_string();
                    let dst_dir = pub_dir.join(&name);
                    fs::create_dir_all(&dst_dir).await?;
                    let dst_file = dst_dir.join("index.html");
                    render_file(path, &dst_file, RenderType::Page).await?;
                }
                Ok(())
            }
        })
        .buffer_unordered(get_cpu())
        .collect::<Vec<Result<()>>>()
        .await
        .into_iter()
        .collect::<Result<()>>()
}

async fn render_class() -> Result<()> {
    let mut context = Context::new();
    context.insert("site", SITE.load().as_ref());
    match TERA.load().render("category-index.html", &context) {
        Ok(rendered) => {
            let mut file =
                File::create(get_public_path(".").join("category").join("index.html")).await?;
            file.write_all_buf(&mut rendered.as_bytes()).await?;
            file.flush().await?;
        }
        Err(e) => {
            return Err(anyhow!("Failed to render category dir: {}", e));
        }
    };
    match TERA.load().render("tag-index.html", &context) {
        Ok(rendered) => {
            let mut file =
                File::create(get_public_path(".").join("tag").join("index.html")).await?;
            file.write_all_buf(&mut rendered.as_bytes()).await?;
            file.flush().await?;
        }
        Err(e) => {
            return Err(anyhow!("Failed to render tag dir: {}", e));
        }
    };
    Ok(())
}

#[inline]
async fn copy_robots() -> Result<()> {
    let src = get_source_path(".").join("robots.txt");
    if src.exists() {
        let dst = get_public_path(".").join("robots.txt");
        fs::copy(src, dst).await?;
    }
    Ok(())
}

#[inline]
fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[inline]
fn truncate_chars(s: &str, max_chars: usize) -> String {
    let mut out: String = s.chars().take(max_chars).collect();
    if s.chars().count() > max_chars {
        out.push('…');
    }
    out
}

async fn gen_atom_str() -> String {
    let site = SITE.load();
    let mut xml = String::with_capacity(409600);
    let root_path = extract_root_path(&site.config.url);
    let root_esc = escape_xml(&root_path);

    // Feed
    let _ = writeln!(xml, r#"<?xml version="1.0" encoding="utf-8"?>"#);
    let _ = writeln!(xml, r#"<feed xmlns="http://www.w3.org/2005/Atom">"#);
    let _ = writeln!(
        xml,
        "  <author><name>{}</name></author>",
        escape_xml(&site.config.author)
    );
    let _ = writeln!(xml, "  <generator>Tless</generator>");
    let _ = writeln!(xml, "  <id>{root_esc}/atom.xml</id>");
    let _ = writeln!(xml, r#"  <link href="{root_esc}" rel="alternate"/>"#);
    let _ = writeln!(xml, r#"  <link href="{root_esc}/atom.xml" rel="self"/>"#);
    let _ = writeln!(
        xml,
        "  <rights>{}</rights>",
        escape_xml(&site.config.rights)
    );
    let _ = writeln!(
        xml,
        "  <subtitle>{}</subtitle>",
        escape_xml(&site.config.subtitle)
    );
    let _ = writeln!(xml, "  <title>{}</title>", escape_xml(&site.config.title));
    let _ = writeln!(xml, "  <updated>{}</updated>", Local::now().to_rfc3339());

    // Entries
    for post in &site.post {
        let Some(name) = post
            .path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
        else {
            continue;
        };
        let name_esc = escape_xml(&name);

        let _ = writeln!(xml, "  <entry>");
        if let Some(cates) = &post.category {
            for c in cates {
                let _ = writeln!(xml, "    <category>{}</category>", escape_xml(c));
            }
        }
        let content = fs::read_to_string(get_public_path(name.as_str()).join("index.html"))
            .await
            .unwrap_or_default();

        let _ = writeln!(
            xml,
            "    <content type=\"html\">{}</content>",
            escape_xml(&content)
        );

        let _ = writeln!(xml, "    <id>{root_esc}/post/{name_esc}</id>");
        let _ = writeln!(
            xml,
            r#"    <link href="{root_esc}/post/{name_esc}" rel="alternate"/>"#
        );
        let _ = writeln!(
            xml,
            r#"    <link href="{root_esc}/post/{name_esc}" rel="self" type="application/atom+xml"/>"#
        );

        let summary = truncate_chars(&content, 200);
        let _ = writeln!(
            xml,
            "    <summary type=\"html\">{}</summary>",
            escape_xml(&summary)
        );
        let _ = writeln!(xml, "    <published>{}</published>", &post.date);
        let _ = writeln!(xml, "    <title>{}</title>", escape_xml(&post.title));
        if let Ok(m) = fs::metadata(&post.path).await
            && let Ok(updated) = m.modified()
        {
            let updated: DateTime<Local> = DateTime::from(updated);
            let _ = writeln!(xml, "    <updated>{}</updated>", updated);
        }
        let _ = writeln!(xml, "  </entry>");
    }
    let _ = writeln!(xml, "</feed>");

    xml
}

async fn gen_atom() -> Result<()> {
    let dst = get_public_path("atom.xml");
    let atom_str = gen_atom_str().await;
    fs::write(dst, atom_str).await?;
    Ok(())
}

async fn gen_sitemap_str() -> String {
    let site = SITE.load();
    let mut xml = String::with_capacity(40960);
    let root_path = extract_root_path(&site.config.url);
    let root_esc = escape_xml(&root_path);

    let _ = writeln!(xml, r#"<?xml version="1.0" encoding="utf-8"?>"#);
    let _ = writeln!(
        xml,
        r#"<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#
    );
    for post in &site.post {
        let Some(name) = post
            .path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
        else {
            continue;
        };
        let name_esc = escape_xml(&name);
        let _ = writeln!(xml, "  <url>");
        let _ = writeln!(xml, r#"  <loc>{root_esc}/post/{name_esc}</loc>"#);
        if let Ok(m) = fs::metadata(&post.path).await
            && let Ok(updated) = m.modified()
        {
            let updated: DateTime<Local> = DateTime::from(updated);
            let _ = writeln!(xml, r#"  <lastmod>{}</lastmod>"#, updated);
        }
        let _ = writeln!(xml, "  </url>");
    }
    let _ = writeln!(xml, "</urlset>");

    xml
}

async fn gen_sitemap() -> Result<()> {
    let dst = get_public_path("sitemap.xml");
    let sitemap_str = gen_sitemap_str().await;
    fs::write(dst, sitemap_str).await?;
    Ok(())
}

/// Render the whole site to the public dir: every post and page, the home
/// page, and the active theme's static resources.
pub async fn render_all() -> Result<()> {
    let site = SITE.load();
    // remove old
    remove_stale_outputs().await?;

    // gen assets
    copy_theme_resources()?;

    copy_robots().await?;

    // gen home: index
    render_home(&site).await?;

    // render categorie/tag dir
    render_class().await?;

    // render posts/pages
    render_post(site.post.iter().map(|d| &d.path).collect::<Vec<_>>()).await?;
    render_page(site.page.iter().map(|d| &d.path).collect::<Vec<_>>()).await?;

    // gen atom.xml sitemap.xml
    gen_atom().await?;
    gen_sitemap().await?;
    Ok(())
}

/// Remove public files whose source was deleted, keeping the deployed site
/// in sync with the sources.
async fn remove_stale_outputs() -> Result<()> {
    let target = get_public_path(".");
    if target.exists() {
        fs::remove_dir_all(&target).await?;
        fs::create_dir_all(&target).await?;
    }
    Ok(())
}

/// Render the theme's `index.html` layout as the site home page.
async fn render_home(site: &Site) -> Result<()> {
    let tera = TERA.load();
    if !tera
        .get_template_names()
        .any(|name| name == "index.html" || name == "index.md")
    {
        info!("Skipping home page");
        return Ok(());
    }
    let mut context = Context::new();
    // an empty content keeps `{% if content %}` blocks in the layout happy
    context.insert("content", "");
    context.insert("recent_posts", &recent_posts(site));
    context.insert("site", site);
    match tera.render("index.html", &context) {
        Ok(rendered) => {
            fs::write(get_public_path("index.html"), rendered).await?;
            info!("Rendered index");
        }
        Err(e) => error!("Failed to render home page: {}", e),
    }
    Ok(())
}

/// Posts from `source/post`, newest first, exposed to the home page template.
fn recent_posts(site: &Site) -> Vec<file::Metadata> {
    let post_dir = get_source_path("post");
    let mut posts: Vec<file::Metadata> = site
        .post
        .iter()
        .filter(|m| m.path.starts_with(&post_dir))
        .cloned()
        .collect();
    posts.sort_by_key(|p| std::cmp::Reverse(date_rank(&p.date)));
    posts
}

/// Parse a frontmatter date (RFC3339 or the CLI `%Y-%m-%d %H:%M:%S` format);
/// posts without a usable date sort last.
fn date_rank(date: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(date)
        .map(|d| d.with_timezone(&Utc))
        .ok()
        .or_else(|| {
            NaiveDateTime::parse_from_str(date, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|d| d.and_utc())
        })
        .unwrap_or(DateTime::<Utc>::MIN_UTC)
}

/// Copy the active theme's `resource/` directory into `public/`.
fn copy_theme_resources() -> Result<()> {
    let resource_dir = get_layout_path().join("assets");
    if !resource_dir.exists() {
        return Ok(());
    }
    copy_dir_recursive(&resource_dir, &get_public_path("assets"))
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}
