use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use tokio_postgres::{self, types::Timestamp};
use dotenv;
use std::env;
use tokio;
use deadpool_postgres;
use serde_json::json;
use serde::{Deserialize, Serialize};



struct Data {
    id: i32,
    sound: String,
    time: std::time::SystemTime,
}

// implement a trait for vec of data
impl Data {
    fn new(id: i32, sound: String, time: std::time::SystemTime) -> Data {
        Data {
            id,
            sound,
            time,
        }
    }
}

// implement a trait for vec of data
impl Into<serde_json::Value> for Data {
    fn into(self) -> serde_json::Value {
        json!({
            "id": self.id,
            "sound": self.sound,
            "time": self.time,
        })
    }
}




async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

async fn get_sound(pool: web::Data<deadpool_postgres::Pool>) -> impl Responder {
    let client = pool.get().await.unwrap();
    let statement = client.prepare("SELECT * FROM sound").await.unwrap();
    let rows = client.query(&statement, &[]).await.unwrap();
    let mut data = Vec::new();
    for row in rows {
        data.push(Data {
            id: row.get(0),
            sound: row.get(1),
            time: row.get(2),
        });
    }

   let data = data.into_iter().map(|data| data.into()).collect::<Vec<serde_json::Value>>();

    HttpResponse::Ok().json(data)


    

}



#[actix_web::main]
async fn main() -> std::io::Result<()> {


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

    let pool = config .create_pool(None, tokio_postgres::NoTls).unwrap();

   


    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/", web::get().to(index))
            .route("/sound", web::get().to(get_sound))
            .wrap(Cors::permissive())


 } )
        .bind("localhost:8080")?
        .run().await
        
}
