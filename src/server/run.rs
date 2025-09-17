use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use tera::Context;

use crate::{result_matcher, server::{self, render, SITE, TERA}};


#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
pub async fn run(port: u16) -> std::io::Result<()> {
    // Start watching file change
    let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);
    server::start_watch(shutdown_tx.clone());

    // Render all posts
    result_matcher!(render::render_all().await, "Failed to render posts");

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

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/{page_name}")]
async fn get_item(path: web::Path<String>) -> impl Responder {
    let page_name = path.into_inner();
    let site = &SITE.load();
    let res = render::render(&site.posts.iter().find(|p| p.title.eq(&page_name)).unwrap().content);
    let mut context = Context::new();
    context.insert("content", &res);
    HttpResponse::Ok().body(TERA.load().render("index.html", &context).unwrap())
}
