use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use deadpool_postgres;
use dotenv;
use iot_sound_database;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use tokio;
use tokio_postgres::{self, types::Timestamp};

struct Data {
    id: i32,
    sound: String,
    time: std::time::SystemTime,
}

// implement a trait for vec of data
impl Data {
    fn new(id: i32, sound: String, time: std::time::SystemTime) -> Data {
        Data { id, sound, time }
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

async fn get_sound(pool: web::Data<iot_sound_database::Pool>) -> impl Responder {
    let returned = pool.get_loudness().await;
    let returned = match returned {
        Ok(data) => data,
        Err(e) => {
            println!("Error: {}", e);
            return HttpResponse::InternalServerError().body("Internal Server Error");
        }
    };
    if returned.is_empty() {
        return HttpResponse::NotFound().body("No data found");
    } else {
        HttpResponse::Ok().json(returned)
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if env::var("DB_USER").is_err()
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

    let pool = iot_sound_database::Pool::new(
        Some(env::var("DB_HOST").unwrap()),
        Some(env::var("DB_PORT").unwrap().parse().unwrap()),
        Some(env::var("DB_USER").unwrap()),
        Some(env::var("DB_PASSWORD").unwrap()),
        Some(env::var("DB_NAME").unwrap()),
    )
    .await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/", web::get().to(index))
            .route("/sound", web::get().to(get_sound))
            .wrap(Cors::permissive())
    })
    .bind("localhost:8081")?
    .run()
    .await
}
