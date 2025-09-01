use clap::Parser;
use tless::cmd::{self};

fn main() {
    match &cmd::Command::parse().cmd {
        cmd::Commands::Server(server) => {
            dbg!(server);
        },
        cmd::Commands::Blog(blog) => {
            dbg!(blog);
        }
    }
}
