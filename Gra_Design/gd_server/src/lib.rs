use actix_web::{App, HttpServer};

mod data_base;
mod handler;
mod model;






pub async fn run_server () -> std::io::Result<()> {

    
    HttpServer::new(
        move || {
            App::new()
        }
    )
        .bind(("127.0.0.1", 8080))?
        .run()
        .await

}
