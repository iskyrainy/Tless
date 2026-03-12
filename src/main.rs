use std::env::set_var;

use tless::cmd::{self};

fn main() {
    unsafe {
        set_var("RUST_LOG", "debug");
    }
    env_logger::init();
    cmd::parse_cmd();
}
