use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use tokio_postgres;
use dotenv;
use std::env;
use tokio;
use deadpool_postgres;

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

async fn get_sound(db: &deadpool_postgres::Pool) -> impl Responder {
    unimplemented!();
    Ok(())

}



#[actix_web::main]
async fn main() {


    if  env::var("DB_USER").is_err()
    || env::var("DB_PASSWORD").is_err()
    || env::var("DB_HOST").is_err()
    || env::var("DB_PORT").is_err()
{
    println!(
        "\x1b[33m{}\x1b[0m",
        "Environment variables not set. Loading .env file"
    );
    dotenv::dotenv().ok();
}

    let config = deadpool_postgres::Config {
        user: Some(env::var("DB_USER").unwrap()),
        password: Some(env::var("DB_PASSWORD").unwrap()),
        host: Some(env::var("DB_HOST").unwrap()),
        port: Some(env::var("DB_PORT").unwrap().parse().unwrap()),
        dbname: Some(env::var("DB_NAME").unwrap()),
        ..Default::default()
    };

    let manager = config .create_pool(None, tokio_postgres::NoTls).unwrap();

   

    HttpServer::new(move || 
        App::new()
            .app_data(web::Data::new(manager.clone()))
            .route("/", web::get().to(index))       

    )
        .bind("localhost:8080")
        .unwrap_or_else(
            |e| panic!("Failed to bind to address: {}", e)
        )
        .run().await.unwrap_or_else(
            |e| panic!("Failed to run server: {}", e)
        );

    println!("Web server started");


        
}
