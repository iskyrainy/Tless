use clap::{command, Args, Parser, Subcommand};

/// Tless command arguments
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
    #[clap(short, long)]
    port: Option<u16>
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
    /// # add a public blog named 'FirstBlog'
    /// tless blog add FirstBlog
    /// 
    /// # add a private blog named 'FirstBlog'
    /// tless blog add -p FirstBlog
    /// ```
    Add {
        #[arg(short, long)]
        prva: bool,

        name: String
    },

    /// Remove `class/name`, default class is `draft`.
    /// 
    /// usage:
    /// ```bash
    /// # remove draft/FirstBlog
    /// tless blog remove FirstBlog
    /// 
    /// # remove post/Blog
    /// tless blog remove --class post Blog
    /// ```
    Remove {
        #[arg(short, long, default_value = "draft")]
        class: String,

        name: String
    },

    /// Publish a draft to post.
    /// If file not exists, print failed.
    /// 
    /// usage:
    /// ```bash
    /// tless blog publish FirstBlog
    /// ```
    Publish { name: String }
}
