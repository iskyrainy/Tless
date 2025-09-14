use std::{env, sync::LazyLock};

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use tera::Tera;

use crate::{result_matcher, server::{self, helper, render, CONFIG, SITE}};


#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
pub async fn run(port: u16) -> std::io::Result<()> {
    // Start watching file change
    let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);
    server::start_watch_config(shutdown_tx.clone());
    server::start_watch_source(shutdown_tx.clone());

    // Initialize the server
    let server = init_server(port, shutdown_tx)?;

    // Run the server
    server.await
}

fn init_server(port: u16, shutdown_tx: tokio::sync::broadcast::Sender<()>) -> Result<actix_web::dev::Server, std::io::Error> {
    let server = HttpServer::new(|| {
            App::new()
                .service(hello)
                .service(get_item)
        })
        .shutdown_signal(async move {
            // Wait ctrl_c for quit gracefully
            tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl_c");
            let _ = shutdown_tx.send(());
            println!("\nReceived exit signal, shutting down...");
        })
        .shutdown_timeout(60)
        .bind(("0.0.0.0", port))?
        .run();
    Ok(server)
}

static TERA: LazyLock<Tera> = LazyLock::new(|| {
    let layout_dir = env::current_dir().map(|p| {
        let dir = p.join("theme").join(&CONFIG.load().site.theme);
        if dir.exists() {
            dir.to_string_lossy().to_string()
        } else {
            println!("Failed to init Tera");
            std::process::exit(1)
        }
    }).unwrap();
    let tera = result_matcher!(
        Tera::new(&format!("{}/layout/*.html", layout_dir)),
        err_handler = |e| {
            println!("Parsing error(s): {}", e);
            std::process::exit(1)
        },
        ok_handler = |tera| {
            helper::Helpers::new().apply_to(tera);
        }
    );
    tera
});

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/{page_name}")]
async fn get_item(path: web::Path<String>) -> impl Responder {
    let page_name = path.into_inner();
    let site = &SITE.load();
    let res = render::render(&site.posts.iter().find(|p| p.title.eq(&page_name)).unwrap().content);
    HttpResponse::Ok().body(res)
}
