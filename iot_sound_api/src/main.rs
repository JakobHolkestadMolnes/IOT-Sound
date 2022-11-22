use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::env;
use serde::Deserialize;

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

#[derive(Deserialize)]
struct Info {
    limit_amount: i64,
}

async fn get_sound_sorted_by_sensor_limited(
    pool: web::Data<iot_sound_database::Pool>,
    info: web::Query<Info>
) -> impl Responder {

    let sensors = pool.get_sensor_ids().await;
    let sensors = match sensors {
        Ok(data) => data,
        Err(e) => {
            println!("Error: {}", e);
            return HttpResponse::InternalServerError().body("Internal Server Error");
        }
    };

    let mut sensors_and_data = Vec::new();

    for sensor in sensors {
        let returned = pool.get_loudness_limited(&sensor, info.limit_amount).await;
        let returned = match returned {
            Ok(data) => data,
            Err(e) => {
                println!("Error: {}", e);
                return HttpResponse::InternalServerError().body("Internal Server Error");
            }
        };
        sensors_and_data.push((sensor, returned));
    }

    
    let mut date_time_sensor: Vec<Vec<iot_sound_database::DataWithDateTimeString>> = Vec::new();
    // add dateTimes to each value
    for sensor in sensors_and_data {
        let mut sensor_data = Vec::new();
        for data in sensor.1 {
            sensor_data.push(data.get_date_time_string());
        }
        date_time_sensor.push(sensor_data);
    }

    if date_time_sensor.is_empty() {
        HttpResponse::NotFound().body("No data found")
    } else {
        HttpResponse::Ok().json(date_time_sensor)
    }
}

async fn get_sound_sorted_by_sensor(pool: web::Data<iot_sound_database::Pool>) -> impl Responder {
    let sensors = pool.get_sensor_ids().await;
    let sensors = match sensors {
        Ok(data) => data,
        Err(e) => {
            println!("Error: {}", e);
            return HttpResponse::InternalServerError().body("Internal Server Error");
        }
    };

    let data = pool.get_loudness().await;
    let data = match data {
        Ok(data) => data,
        Err(e) => {
            println!("Error: {}", e);
            return HttpResponse::InternalServerError().body("Internal Server Error");
        }
    };

    let mut data_by_sensor = Vec::new();
    for sensor in sensors {
        let mut sensor_data = Vec::new();
        for row in &data {
            if row.get_sensor_name() == sensor {
                let row = row.clone();
                sensor_data.push(row);
            }
        }
        data_by_sensor.push(sensor_data);
    }

    let mut date_time_sensor = Vec::new();
    // add dateTimes to each value
    for sensor_data in &mut data_by_sensor {
        let mut date_time = Vec::new();
        for row in sensor_data {
            date_time.push(row.get_date_time_string());
        }
        date_time_sensor.push(date_time);
    }

    if date_time_sensor.is_empty() {
        HttpResponse::NotFound().body("No data found")
    } else {
        HttpResponse::Ok().json(date_time_sensor)
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
            .route("/sound/sorted", web::get().to(get_sound_sorted_by_sensor))
            .route("/sound/sorted/limit", web::get().to(get_sound_sorted_by_sensor_limited))
            .wrap(Cors::permissive())
    })
    .bind("localhost:8081")?
    .run()
    .await
}
