use std::{
    collections::HashMap, env, fs, io::BufReader, path::PathBuf, sync::LazyLock
};

use arc_swap::ArcSwap;
use data_encoding::HEXUPPER;
use futures::{stream, StreamExt};
use pulldown_cmark::{html, Options, Parser};
use ring::digest::{self, SHA256};
use serde::{Deserialize, Serialize};
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
                let metadata = match file::parse_file(path.clone()) {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("Failed to parse changed post: {}", e);
                        return Err(std::io::Error::other(e.to_string()));
                    }
                };
                let (modify_flag, file_str) = pre_hash_check(&post.path)?;
                if !modify_flag {
                    return Ok(())
                }
                let md_html_str = render(&file_str);
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
                let (modify_flag, file_str) = pre_hash_check(&post.path)?;
                if !modify_flag {
                    return Ok(())
                }
                let md_html_str = render(&file_str);
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct HashValue {
    pub path: String,
    pub hash_v: String,
}

pub(crate) static POST_HASH: LazyLock<ArcSwap<HashMap<String, String>>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    let post_hash = env::current_dir().unwrap().join("public").join(".post_hash");
    if !post_hash.exists() {
        let _ = std::fs::File::create_new(post_hash).unwrap();
    } else {
        let file = std::fs::File::open(post_hash).unwrap();
        let parsed: Vec<HashValue> = serde_json::from_reader(BufReader::new(file)).unwrap();
        for hash_value in parsed {
            map.insert(hash_value.path, hash_value.hash_v);
        }
    }
    ArcSwap::from_pointee(map)
});

pub(crate) async fn pre_hash_check(path: &PathBuf) -> std::io::Result<(bool, String)> {
    let file_text = tokio::fs::read_to_string(path).await?;
    let path_str = path.to_string_lossy().to_string();
    if POST_HASH.load().contains_key(&path_str) {
        let mut context = digest::Context::new(&SHA256);
        context.update(&file_text.as_bytes());
        let hash = context.finish();
        if POST_HASH.load().get(&path_str).unwrap().eq(&HEXUPPER.encode(hash.as_ref())) {
            return Ok((false, file_text));
        }
    }
    Ok((true, file_text))
}

