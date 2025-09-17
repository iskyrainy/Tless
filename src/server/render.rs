use std::{
    env, fs, path::PathBuf
};

use futures::{stream, StreamExt};
use pulldown_cmark::{html, Options, Parser};
use tera::Context;
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};

use crate::{file, server::{SITE, TERA}};

/// Markdown default render options.
const DEFAULT_OPTIONS: Options = Options::all();

/// Render markdown to HTML string.
pub(crate) fn render(markdown: &str) -> String {
    let parser = Parser::new_ext(markdown, DEFAULT_OPTIONS);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

pub(crate) async fn render_to_file(events_path: Vec<PathBuf>) -> std::io::Result<()> {
    let public_dir = env::current_dir()?.join("public");
    
    let concurrency = num_cpus::get() + 1;
    stream::iter(events_path.into_iter())
        .map(|path| {
            let public_dir = public_dir.clone();
            async move {
                let metadata = match file::parse_file(fs::File::open(path)?) {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("Failed to parse changed post: {}", e);
                        return Err(std::io::Error::other(e.to_string()));
                    }
                };
                let md_html_str = render(&metadata.content);
                let file_path = public_dir.join(&metadata.title);
                let file = File::create(&file_path).await?;
                let mut writer = BufWriter::new(file);

                let mut context = Context::new();
                context.insert("content", &md_html_str);
                match TERA.load().render("archive.html", &context) {
                    Ok(rendered) => {
                        writer.write_all(rendered.as_bytes()).await?;
                        writer.flush().await?;
                        println!("Rendered {}", metadata.title);
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("Failed to render {}: {}", metadata.title, e);
                        Err(std::io::Error::other(e))
                    }
                }
            }
        })
        .buffer_unordered(concurrency)
        .collect::<Vec<_>>()
        .await;
    Ok(())
}

/// Render all posts to public dir.
pub(crate) async fn render_all() -> std::io::Result<()> {
    let public_dir = env::current_dir()?.join("public");
    let site = SITE.load();

    let concurrency = num_cpus::get() + 1;
    stream::iter(site.posts.clone().into_iter())
        .map(|post| {
            let public_dir = public_dir.clone();
            async move {
                // TODO: add a json file(store file hash value) record whether post should be re-render at server starting
                let md_html_str = render(&post.content);
                let file_path: PathBuf = public_dir.join(&post.title);
                let file = File::create(&file_path).await?;
                let mut writer = BufWriter::new(file);

                let mut context = Context::new();
                context.insert("content", &md_html_str);

                match TERA.load().render("archive.html", &context) {
                    Ok(rendered) => {
                        writer.write_all(rendered.as_bytes()).await?;
                        writer.flush().await?;
                        println!("Rendered {}", post.title);
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("Failed to render {}: {}", post.title, e);
                        Err(std::io::Error::other(e))
                    }
                }
            }
        })
        .buffer_unordered(concurrency)
        .collect::<Vec<_>>()
        .await;

    Ok(())
}
