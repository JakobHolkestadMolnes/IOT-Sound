use actix_web::{web, App, HttpResponse, HttpServer, Responder};

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() {
    HttpServer::new(|| 
        App::new().route("/", web::get().to(index)))
        .bind("localhost:8080")
        .unwrap_or_else(
            |e| panic!("Failed to bind to address: {}", e)
        )
        .run().await.unwrap_or_else(
            |e| panic!("Failed to run server: {}", e)
        );

    println!("Web server started");


        
}
