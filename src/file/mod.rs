//! Module for handling blog and page files, including adding, removing, and parsing metadata.
//! It provides functions to manage blog posts and pages in a static site generator context.

use std::{env, error::Error, fs, io::Read, path};

use serde::{Deserialize, Serialize};

pub mod blog;
pub mod page;

/// Metadata structure to hold blog information.
/// # Fields
/// * `title` - The title of the blog.
/// * `date` - The publication date of the blog.
/// * `tags` - Optional tags associated with the blog.
/// * `categories` - Optional categories associated with the blog.
/// * `prva` - A boolean indicating if the blog is private.
/// * `content` - The main content of the blog in markdown format.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Metadata {
    pub title: String,
    pub date: String,
    pub layout: Option<String>,
    pub tags: Option<Vec<String>>,
    pub categories: Option<Vec<String>>,
    pub prva: bool,
    pub content: String
}

impl Metadata {
    pub fn new() -> Self {
        Metadata {
            title: String::new(),
            date: String::new(),
            layout: None,
            tags: None,
            categories: None,
            prva: false,
            content: String::new()
        }
    }
}

/// Get the file path for a blog or page based on its name and class.
/// # Arguments
/// * `name` - A reference to a `String` representing the name of the blog or page.
/// * `class` - A reference to a `String` representing the class (e.g., "post", "draft", "page").
/// # Returns
/// A `String` representing the full file path.
/// # Examples
/// ```
/// let name = String::from("my_blog");
/// let class = String::from("post");
/// assert_eq!(get_path(&name, &class), "/current/directory/source/post/my_blog.md");
/// ```
pub(crate) fn get_path(name: &String, class: &String) -> String {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    current_dir.join("source")
        .join(class)
        .join(name)
        .with_extension("md")
        .to_str()
        .unwrap()
        .to_string()
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
pub(crate) fn is_file_exist(file_path: &String) -> bool {
    path::Path::new(file_path).exists()
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
pub fn parse_file(mut file: fs::File) -> Result<Metadata, Box<dyn Error>> {
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
    if let Some(layout) = frontmatter.get("layout").and_then(|v| v.as_str()) {
        metadata.layout = Some(layout.to_string());
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
