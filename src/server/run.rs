use axum::{extract::Path, routing::get, serve, Router};
use tokio::net;


#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
pub async fn run(port: u16) {
    let addr = format!("0.0.0.0:{}", port);
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));
    let listener = net::TcpListener::bind(addr).await.expect("Failed to bind to address");
    println!("Starting server on port: {}", port);
    serve::serve(listener, app).await.expect("Server failed");
}

async fn get_list(Path(class): Path<String>) -> Vec<String> {
    vec![]
}
