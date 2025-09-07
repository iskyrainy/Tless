use actix_web::{App, HttpResponse, HttpServer, Responder, get};

use crate::server;


#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
pub async fn run(port: u16) -> std::io::Result<()> {
    // Start watching file change
    let (shutdown_tx, _shutdown_rx) = tokio::sync::broadcast::channel(1);
    server::start_watch_config(shutdown_tx.clone());
    server::start_watch_source(shutdown_tx.clone());

    // Initialize the server
    let server = init_server(port)?;

    // Handle graceful shutdown on Ctrl+C
    graceful_shutdown(server.handle(), shutdown_tx);

    // Run the server
    server.await
}

fn init_server(port: u16) -> Result<actix_web::dev::Server, std::io::Error> {
    let server = HttpServer::new(|| {
            App::new()
                .service(hello)
        })
        .bind(("0.0.0.0", port))?
        .run();
    Ok(server)
}

fn graceful_shutdown(srv_handle: actix_web::dev::ServerHandle, shutdown_tx: tokio::sync::broadcast::Sender<()>) {
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl_c");
        println!("\nReceived ctrl_c, shutting down...");
        srv_handle.stop(true).await;
        let _ = shutdown_tx.send(());
    });
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
