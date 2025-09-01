use std::{fs, process};

use clap::{command, Args, Parser, Subcommand};

use crate::{blog, server};

/// tless command arguments
#[derive(Parser, Debug)]
#[command(author = "gdhvxcj <wangnan5117@gmail.com>", version = "0.1.0", about = "Build blog site.", long_about = "Fast and easy blog site builder.")]
#[command(propagate_version = true)]
pub struct Command {
    #[command(subcommand)]
    pub cmd: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Subcommand that run tless server and specify port
    Server(Server),

    /// Subcommand that controls blog's `add/remove/publish`
    Blog(Blog),

    /// Subcommand that generates static pages, deploy to github page, backup site, etc.
    Site(Site)
}

#[derive(Args, Debug)]
pub struct Server {
    /// Run Tless server.
    /// 
    /// usage:
    /// ```bash
    /// tless server -r
    /// ```
    #[clap(short, long)]
    run: bool,

    /// Port that server binding.
    /// 
    /// usage:
    /// ```bash
    /// tless server -r -p 12345
    /// ```
    #[clap(short, long, default_value_t = 8917)]
    port: u16
}

#[derive(Args, Debug)]
pub struct Blog {
    #[command(subcommand)]
    pub cli: BlogArgs
}

#[derive(Subcommand, Debug, Clone)]
pub enum BlogArgs {
    /// Add a draft blog.
    /// If file exists, print failed.
    /// 
    /// usage:
    /// ```bash
    /// # add a draft blog named 'FirstBlog'
    /// tless blog add FirstBlog
    /// ```
    Add { name: String },

    /// Remove `class/name`, default class is `draft`.
    /// 
    /// usage:
    /// ```bash
    /// # remove draft/FirstBlog
    /// tless blog remove FirstBlog
    /// 
    /// # remove private post/Blog
    /// tless blog remove -c post -p Blog
    /// ```
    Remove {
        #[arg(short, long, default_value = "draft")]
        class: String,

        #[arg(short, long)]
        prva: bool,

        name: String
    },

    /// Publish a draft to post.
    /// If file not exists, print failed.
    /// 
    /// usage:
    /// ```bash
    /// # publish draft/FirstBlog to post/FirstBlog as public post
    /// tless blog publish FirstBlog
    /// 
    /// # publish draft/FirstBlog to post/FirstBlog as private post
    /// tless blog publish -p FirstBlog
    /// ```
    Publish {
        #[arg(short, long)]
        prva: bool,

        name: String
    }
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
pub struct Site {
    /// Initialize site structure.
    /// 
    /// usage:
    /// ```bash
    /// tless site -i
    /// ```
    #[clap(short, long)]
    init: bool,

    /// Generate static pages.
    /// 
    /// usage:
    /// ```bash
    /// tless site -g
    /// ```
    #[clap(short, long)]
    generate: bool,

    /// Deploy site to github page.
    /// 
    /// usage:
    /// ```bash
    /// tless site -d
    /// ```
    #[clap(short, long)]
    deploy: bool,

    /// Backup site data to pkg.
    /// 
    /// usage:
    /// ```bash
    /// tless site -b
    /// ```
    #[clap(short, long)]
    backup: bool
}

/// Parse command line arguments and check the validity.
pub fn parse_cmd() {
    let input = Command::parse();
    match input.cmd {
        Commands::Server(server) => handle_server(server),
        Commands::Blog(blog) => handle_blog(blog),
        Commands::Site(site) => handle_site(site),
    }
}

fn handle_server(server: Server) {
    if server.run && server.port > 1024 && server.port < 65_535 {
        server::run(server.port);
    } else {
        println!("Server not started. Use -r to run the server. Port must be between 1025 and 65534.");
        process::exit(1);
    }
}

fn handle_blog(blog: Blog) {
    match &blog.cli {
        BlogArgs::Add { name } => {
            let file_path = blog::get_blog_path(name, &"draft".to_string(), None);
            match fs::exists(&file_path) {
                Ok(exists) if !exists => {},
                _ => {
                    eprintln!("File {} already exists!", file_path);
                    process::exit(1);
                }
            }
        },
        BlogArgs::Remove { class, prva, name } => {
            let file_path = blog::get_blog_path(name, class, Some(prva));
            match fs::exists(&file_path) {
                Ok(exists) if exists => {},
                _ => {
                    eprintln!("File {} not exists!", file_path);
                    process::exit(1);
                }
            }
        },
        BlogArgs::Publish { prva: _, name } => {
            let file_name = blog::get_blog_path(name, &"draft".to_string(), None);
            match fs::exists(&file_name) {
                Ok(exists) if exists => {},
                _ => {
                    eprintln!("File {} not exists!", file_name);
                    process::exit(1);
                }
            }
        }
    }
}

fn handle_site(site: Site) {
    if site.init {
        println!("Initializing site structure...");
    } else if site.generate {
        println!("Generating static pages...");
    } else if site.deploy {
        println!("Deploying site to GitHub Pages...");
    } else if site.backup {
        println!("Backing up site data...");
    } else {
        eprintln!("No valid site operation specified.");
        process::exit(1);
    }
}
