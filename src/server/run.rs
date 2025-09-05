use actix_web::{App, HttpResponse, HttpServer, Responder, get};

use crate::server::CONFIG;


#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
pub async fn run(port: u16) -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
