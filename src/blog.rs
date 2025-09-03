use std::{env, error::Error, fs, io::Read, path};

use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Metadata structure to hold blog information.
/// # Fields
/// * `title` - The title of the blog.
/// * `date` - The publication date of the blog.
/// * `tags` - Optional tags associated with the blog.
/// * `categories` - Optional categories associated with the blog.
/// * `prva` - A boolean indicating if the blog is private.
/// * `content` - The main content of the blog in markdown format.
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
    // todo: time zone support
    fs::write(&file_path, format!(r#"---
title: New Blog
date: {}
tags:
categories:
---

# New Blog
Write your content here.
"#, Utc::now().format("%Y-%m-%d %H:%M:%S")))
        .expect("Failed to create blog file");
    println!("Blog '{}' created in 'draft'.", file_path);
    Ok(())
}

/// Remove an existing blog file.
/// # Arguments
/// * `name` - A reference to a `String` representing the blog name.
/// * `class` - A reference to a `String` representing the blog class (e.g., "draft", "post").
/// # Returns
/// A `Result` indicating success or failure.
/// # Examples
/// ```
/// let name = String::from("my_old_blog");
/// let class = String::from("post");
/// match remove_blog(&name, &class) {
///     Ok(_) => println!("Blog removed successfully."),
///     Err(e) => eprintln!("Failed to remove blog: {}", e),
/// }
/// ```
pub fn remove_blog(name: &String, class: &String) -> Result<(), Box<dyn Error>> {
    let file_path = get_blog_path(name, class);
    if !is_blog_exist(&file_path) {
        return Err("Blog does not exist.".into());
    }
    fs::remove_file(&file_path).expect("Failed to remove blog file");
    println!("Blog '{}' removed from '{}'.", name, class);
    Ok(())
}

/// Publish a draft blog by moving it to the post class and updating its metadata.
/// # Arguments
/// * `name` - A reference to a `String` representing the blog name.
/// * `prva` - A boolean indicating if the blog should be marked as private.
/// # Returns
/// A `Result` indicating success or failure.
/// # Examples
/// ```
/// let name = String::from("my_draft_blog");
/// let prva = false;
/// match publish_blog(&name, prva) {
///     Ok(_) => println!("Blog published successfully."),
///     Err(e) => eprintln!("Failed to publish blog: {}", e),
/// }
/// ```
pub fn publish_blog(name: &String, prva: bool) -> Result<(), Box<dyn Error>> {
    let draft_path = get_blog_path(name, &String::from("draft"));
    if !is_blog_exist(&draft_path) {
        return Err("Draft blog does not exist.".into());
    }
    let post_path = get_blog_path(name, &String::from("post"));
    if is_blog_exist(&post_path) {
        return Err("Post blog already exists.".into());
    }
    let file = fs::File::open(&draft_path).expect("Failed to open draft blog");
    let metadata = parse_blog(file)?;
    // todo: time zone support
    let frontmatter = format!(
        "---\ntitle: {}\ndate: {}\ntags: {}\ncategories: {}\nprva: {}\n---\n\n",
        metadata.title,
        Utc::now().format("%Y-%m-%d %H:%M:%S"),
        format!("[{}]", metadata.tags.unwrap_or_default().join(", ")),
        format!("[{}]", metadata.categories.unwrap_or_default().join(", ")),
        prva
    );
    let content = format!("{}{}", frontmatter, metadata.content);
    fs::write(&post_path, content).expect("Failed to create post blog");
    fs::remove_file(&draft_path).expect("Failed to remove draft blog");
    println!("Blog '{}' published from 'draft' to 'post'.", name);
    Ok(())
}
