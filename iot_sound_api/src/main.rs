use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::env;

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
        HttpResponse::NotFound().body("No data found")
    } else {
        HttpResponse::Ok().json(returned)
    }
}

async fn get_sensors(pool: web::Data<iot_sound_database::Pool>) -> impl Responder {
    let returned = pool.get_sensors().await;
    let returned = match returned {
        Ok(data) => data,
        Err(e) => {
            println!("Error: {}", e);
            return HttpResponse::InternalServerError().body("Internal Server Error");
        }
    };
    if returned.is_empty() {
        HttpResponse::NotFound().body("No data found")
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
        println!("\x1b[33mEnvironment variables not set. Loading .env file\x1b[0m");
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

    let pool = match pool {
        Ok(pool) => pool,
        Err(e) => panic!("Error creating database pool: {}", e),
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/", web::get().to(index))
            .route("/sound", web::get().to(get_sound))
            .route("/sensors", web::get().to(get_sensors))
            .wrap(Cors::permissive())
    })
    .bind("localhost:8081")?
    .run()
    .await
}
