use std::{env, fs, path};

/// Get the full path of a blog file based on its name, class, and whether it's a private draft.
/// # Arguments
/// * `name` - The name of the blog file (without extension).
/// * `class` - The class of the blog (e.g., "draft", "post").
/// * `prva` - An optional boolean indicating if the blog is a private draft.
/// # Returns
/// A `String` representing the full path to the blog file.
/// # Examples
/// ```
/// let path = get_blog_path(&"MyBlog".to_string(), &"draft".to_string(), None);
/// assert_eq!(path, "/current/directory/draft/MyBlog.md");
/// 
/// let path_prva = get_blog_path(&"MyBlog".to_string(), &"post".to_string(), Some(&true));
/// assert_eq!(path_prva, "/current/directory/post/MyBlog.prva.md");
/// ```
pub fn get_blog_path(name: &String, class: &String, prva: Option<&bool>) -> String {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    match prva {
        Some(true) => current_dir.join("post")
                        .join(name)
                        .with_extension("prva.md")
                        .to_str()
                        .expect("Failed to convert path to string")
                        .to_string(),
        _ => current_dir.join(class)
                        .join(name)
                        .with_extension("md")
                        .to_str()
                        .expect("Failed to convert path to string")
                        .to_string(),
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
    fs::create_dir_all(path::Path::new(&file_path).parent().unwrap())
            .expect("Failed to create directories");
    fs::write(file_path, "# New Blog\n\nYour content here.")
        .expect("Failed to create blog file");
    println!("Blog '{}' created in 'draft'.", file_path);
}

// todo: remove_blog function

// todo: publish_blog function
