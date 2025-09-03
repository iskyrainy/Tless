use std::{env, error::Error, fs, io::Read, path};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Metadata {
    title: String,
    date: String,
    tags: Option<Vec<String>>,
    categories: Option<Vec<String>>,
    content: String
}

fn parse_blog(file: fs::File, header: String) -> Result<Metadata, Box<dyn Error>> {
    let mut text;
    if let Err(e) = file.read_to_string(text) {
        return Err("Failed to read blog.".into());
    }
    let (frontmatter, markdown_content) = frontmatter_gen::extract(&text)?;
    if let Some(title) = frontmatter.get("title").and_then(|v| v.as_str()) {
        println!("Title: {}", title);
    }
    Ok(Metadata {
        title: , date: (), tags: (), categories: (), content: ()
    })
}

pub fn get_blog(name: &String, class: &String, prva: Option<&bool>) -> Option<fs::File> {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    match prva {
        Some(true) => {
            if let Ok(file) = fs::File::open(current_dir.join("source")
                        .join(class)
                        .join("name")
                        .with_extension("md")) {
                
            }
        },
        _ => ,
    }
}                                                                                   

/// Check if a blog file exists at the given path.
/// # Arguments
/// * `path` - A reference to a `String` representing the path to the blog file.
/// # Returns
/// A `bool` indicating whether the file exists.
/// # Examples
/// ```
/// let exists = is_blog_exist(&"/path/to/exist_blog.md".to_string());
/// assert_eq!(exists, true);
/// ```
pub fn is_blog_exist(path: &String) -> bool {
    path::Path::new(path).exists()
}

// todo: add_blog function, utf-8 support
pub fn add_blog(file_path: &String) {
    fs::create_dir_all(path::Path::new(&file_path))
            .expect("Failed to create directories");
    fs::write(file_path, r#"
---
title: New Blog
date: 2025-05-20 00:00:00
tags:
categories:
---

# New Blog
Write your content here."#)
        .expect("Failed to create blog file");
    println!("Blog '{}' created in 'draft'.", file_path);
}

// todo: remove_blog function

// todo: publish_blog function
