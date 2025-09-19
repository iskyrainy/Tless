use std::fs;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use tera::Context;

use crate::{result_matcher, server::{self, get_public_path, render, SITE, TERA}};


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
                .service(get_archive)
                .service(get_category)
                .service(get_tag)
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

#[get("/{page}")]
async fn get_page(page: web::Path<String>) -> impl Responder {
    let page_name = page.into_inner();
    let mut context = Context::new();
    let site = &SITE.load();
    context.insert("post", &site.posts);
    context.insert("tags", &site.tags);
    context.insert("categories", &site.categories);
    context.insert("page", &page_name);
    match TERA.load().render(format!("{}.html", page_name).as_str(), &context) {
        Ok(res) => HttpResponse::Ok().body(res),
        Err(_) => HttpResponse::ExpectationFailed().body("Failed to render page")
    }
}

#[get("/archives/{post}")]
async fn get_archive(post: web::Path<String>) -> impl Responder {
    let post_name = post.into_inner();
    let res = render::render(fs::read_to_string(get_public_path(&post_name)).unwrap().as_str());
    let mut context = Context::new();
    context.insert("content", &res);
    HttpResponse::Ok().body(TERA.load().render("acrhive.html", &context).unwrap())
}

#[get("/categories/{category}")]
async fn get_category(category: web::Path<String>) -> impl Responder {
    let category_name = category.into_inner();
    let mut context = Context::new();
    let site = &SITE.load();
    context.insert("post", &site.posts);
    context.insert("tags", &site.tags);
    context.insert("categories", &site.categories);
    context.insert("name", &category_name);
    HttpResponse::Ok().body(TERA.load().render("category.html", &context).unwrap())
}

#[get("/tags/{tag}")]
async fn get_tag(tag: web::Path<String>) -> impl Responder {
    let tag_name = tag.into_inner();
    let mut context = Context::new();
    let site = &SITE.load();
    context.insert("post", &site.posts);
    context.insert("tags", &site.tags);
    context.insert("categories", &site.categories);
    context.insert("name", &tag_name);
    HttpResponse::Ok().body(TERA.load().render("category.html", &context).unwrap())
}