use std::{error::Error, fs, path::PathBuf};

use chrono::Utc;

use crate::file::{get_path, is_file_exist, parse_file};                                                                                   

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
    let file_path = get_path(name, &class);
    if is_file_exist(&file_path) {
        return Err("Blog already exists.".into());
    }
    // TODO: time zone support
    fs::write(&file_path, base_blog_text(name))?;
    println!("Blog '{}' created in 'draft'.", file_path);
    Ok(())
}

fn base_blog_text(name: &String) -> String {
    format!("---\ntitle: {}\ndate: {}\ntags:\ncategories:\n---\n\n# New Blog\nWrite your content here.\n", 
        name, 
        Utc::now().format("%Y-%m-%d %H:%M:%S")
    )
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
    let file_path = get_path(name, class);
    if !is_file_exist(&file_path) {
        return Err("Blog does not exist.".into());
    }
    fs::remove_file(&file_path)?;
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
    let draft_path = get_path(name, &String::from("draft"));
    if !is_file_exist(&draft_path) {
        return Err("Draft blog does not exist.".into());
    }
    let post_path = get_path(name, &String::from("post"));
    if is_file_exist(&post_path) {
        return Err("Post blog already exists.".into());
    }
    let metadata = parse_file(PathBuf::from(&draft_path))?;
    // TODO: time zone support
    let frontmatter = format!(
        "---\ntitle: {}\ndate: {}\ntags: {}\ncategories: {}\nprva: {}\n---\n\n",
        metadata.title,
        Utc::now().format("%Y-%m-%d %H:%M:%S"),
        format!("[{}]", metadata.tags.unwrap_or_default().join(", ")),
        format!("[{}]", metadata.categories.unwrap_or_default().join(", ")),
        prva
    );
    let file_str = fs::read_to_string(&draft_path)?;
    let content = format!("{}{}", frontmatter, file_str);
    fs::write(&post_path, content)?;
    fs::remove_file(&draft_path)?;
    println!("Blog '{}' published from 'draft' to 'post'.", name);
    Ok(())
}
