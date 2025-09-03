use std::{env, error::Error, fs, io::Read, path};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Metadata {
    title: String,
    date: String,
    tags: Option<Vec<String>>,
    categories: Option<Vec<String>>,
    prva: bool,
    content: String
}

impl Metadata {
    fn new() -> Self {
        Metadata {
            title: String::new(),
            date: String::new(),
            tags: None,
            categories: None,
            prva: false,
            content: String::new()
        }
    }
}

fn get_blog_path(name: &String, class: &String) -> String {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    current_dir.join("source")
        .join(class)
        .join(name)
        .with_extension("md")
        .to_str()
        .unwrap()
        .to_string()
}

/// Parse the frontmatter and content from a blog file.
/// # Arguments
/// * `file` - A mutable reference to a `fs::File` representing the blog file.
/// # Returns
/// A `Result` containing `Metadata` if successful, or a boxed `dyn Error`.
/// # Examples
/// ```
/// let file = fs::File::open("path/to/blog.md").unwrap();
/// let metadata = parse_blog(file).unwrap();
/// assert_eq!(metadata.title, "Blog Title");
/// ```
fn parse_blog(mut file: fs::File) -> Result<Metadata, Box<dyn Error>> {
    let mut text = String::new();
    if let Err(_) = file.read_to_string(&mut text) {
        return Err("Failed to read blog.".into());
    }
    let (frontmatter, markdown_content) = frontmatter_gen::extract(&text)?;
    let mut metadata = Metadata::new();
    metadata.content = markdown_content.to_string();
    if let Some(title) = frontmatter.get("title").and_then(|v| v.as_str()) {
        metadata.title = title.to_string();
    }
    if let Some(date) = frontmatter.get("date").and_then(|v| v.as_str()) {
        metadata.date = date.to_string();
    }
    if let Some(tags) = frontmatter.get("tags").and_then(|v| v.as_array()) {
        let tag_list = tags.iter()
            .filter_map(|t| t.as_str().map(|s| s.to_string()))
            .collect();
        metadata.tags = Some(tag_list);
    }
    if let Some(categories) = frontmatter.get("categories").and_then(|v| v.as_array()) {
        let category_list = categories.iter()
            .filter_map(|c| c.as_str().map(|s| s.to_string()))
            .collect();
        metadata.categories = Some(category_list);
    }
    if let Some(prva) = frontmatter.get("prva").and_then(|v| v.as_bool()) {
        metadata.prva = prva;
    }
    Ok(metadata)
}

/// Retrieve a blog file based on its name and class.
/// # Arguments
/// * `name` - A reference to a `String` representing the blog name.
/// * `class` - A reference to a `String` representing the blog class (e.g., "draft", "post").
/// # Returns
/// An `Option` containing the `fs::File` if found, or `None` if not found.
/// # Examples
/// ```
/// let name = String::from("my_blog");
/// let class = String::from("post");
/// let file = get_blog(&name, &class);
/// assert!(file.is_some());
/// ```
pub fn get_blog(name: &String, class: &String) -> Option<fs::File> {
    if let Ok(file) = fs::File::open(get_blog_path(name, class)) {
        Some(file)
    } else {
        None
    }
}                                                                                   

/// Check if a blog file exists at the given path.
/// # Arguments
/// * `file_path` - A reference to a `String` representing the file path to check.
/// # Returns
/// A `bool` indicating whether the file exists.
/// # Examples
/// ```
/// let name = String::from("my_blog");
/// let class = String::from("post");
/// let exists = is_blog_exist(&name, &class);
/// assert_eq!(exists, true);
/// ```
pub fn is_blog_exist(file_path: &String) -> bool {
    path::Path::new(file_path).exists()
}

/// Add a new blog file with default content.
/// # Arguments
/// * `name` - A reference to a `String` representing the blog name.
/// * `class` - A reference to a `String` representing the blog class (e.g., "draft", "post").
/// # Returns
/// A `Result` indicating success or failure.
/// # Examples
/// ```
/// let name = String::from("my_new_blog");
/// let class = String::from("draft");
/// match add_blog(&name, &class) {
///     Ok(_) => println!("Blog added successfully."),
///     Err(e) => eprintln!("Failed to add blog: {}", e),
/// }
/// ```
pub fn add_blog(name: &String) -> Result<(), Box<dyn Error>> {
    let class = String::from("draft");
    let file_path = get_blog_path(name, &class);
    if is_blog_exist(&file_path) {
        return Err("Blog already exists.".into());
    }
    fs::write(&file_path, r#"---
title: New Blog
date: 2025-05-20 00:00:00
tags:
categories:
---

# New Blog
Write your content here.
"#)
        .expect("Failed to create blog file");
    println!("Blog '{}' created in 'draft'.", file_path);
    Ok(())
}

// todo: remove_blog function

// todo: publish_blog function
