use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use std::env;

/// Api endpoint index
/// Shows all available endpoints
async fn index() -> impl Responder {
    let base_url = "http://localhost:8081/";
    let mut end_points = String::new();
    end_points.push_str("<div> <h1>Endpoints:</h1> \r");
    end_points.push_str("GET \r <br>");

    end_points.push_str(&get_link_string(base_url, "sound"));
    end_points.push_str(&get_link_string(base_url, "sensors"));
    end_points.push_str(&get_link_string(base_url, "sound/sorted"));
    end_points.push_str(&get_link_string(
        base_url,
        "sound/sorted/limit?limit_amount=10",
    ));
    end_points.push_str(&get_link_string(base_url, "logs"));
    end_points.push_str(&get_link_string(base_url, "logs/limit?limit_amount=10"));

    end_points.push_str("</div>");

    // return as html
    HttpResponse::Ok().body(end_points)
}

/// Returns a link string for the given endpoint
/// # Arguments
/// * `base_url` - The base url of the server
/// * `endpoint` - The endpoint to link to
/// # Returns
/// * `String` - The link string
/// # Example
/// ```rust
/// let base_url = "http://localhost:8081/";
/// let endpoint = "sound";
/// let link_string = get_link_string(base_url, endpoint);
/// assert_eq!(link_string, "<a href=\"http://localhost:8081/sound\">sound</a> \r <br>");
/// ```
fn get_link_string(base_url: &str, end_point: &str) -> String {
    let html_link_template = format!(
        "<a href=\"{}{}\">{}</a><br>",
        base_url, end_point, end_point
    );
    html_link_template
}

/// This API Endpoint is used to get the loudness of a sensor
/// # Arguments
/// * `pool` - The database pool
/// # Returns
/// * ìmpl Responder` - The loudness of the sensor
/// # Example request
/// ```bash
/// curl -X GET "http://localhost:8080/sensor/1/loudness" -H "accept: application/json"
/// ```
/// # Example response
/// ```json
/// {
///  "loudness": 0.0,
///  "timestamp": "2020-05-01T12:00:00Z"
/// }
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

/// his api call gets the latest data from the database sorted by sensor but limited by the amount
/// specified in the url
/// example: /api/sound/limit?limit_amount=10
/// # Arguments
/// * `pool` - The database pool
/// * `info` - The limit amount
/// # Returns
/// * `impl Responder` - The response
/// # Errors
/// * `InternalServerError` - If there is an error with the database
/// * `NotFound` - If there is no data in the database
/// * `BadRequest` - If the limit amount is less than 1
/// # Example call
/// ```bash
/// curl -X GET "http://localhost:8080/sound/limit?limit_amount=1" -H "accept: application/json"
/// ```
/// # Example response
/// ```json
/// [
///  {
///   "sensor_id": "sensor1",
///   "db_level": "50",
///   "timestamp": "2020-05-06T12:00:00Z"
///  },
/// ]
/// ```
async fn get_sound_sorted_by_sensor_limited(
    pool: web::Data<iot_sound_database::Pool>,
    info: web::Query<Info>,
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

/// this api call gets the latest data from the database sorted by sensor
/// and returns it as a json object
/// # Arguments
/// * `pool` - the database pool
/// # Returns
/// * `impl Responder` - the response
/// # Example Call
/// ```bash
/// curl -X GET "http://localhost:8080/sound/sorted_by_sensor" -H "accept: application/json"
/// ```
/// # Example Response
/// ```json
/// [
///  [
///    {
///      "sensor_id": "sensor1",
///      "loudness": 0,
///      "date_time": "2021-05-03T12:00:00Z"
///    },
///    {
///      "sensor_id": "sensor1",
///      "loudness": 0,
///      "date_time": "2021-05-03T12:00:00Z"
///    }
///  ],
///  [
///    {
///      "sensor_id": "sensor2",
///      "loudness": 0,
///      "date_time": "2021-05-03T12:00:00Z"
///    },
///    {
///      "sensor_id": "sensor2",
///      "loudness": 0,
///      "date_time": "2021-05-03T12:00:00Z"
///    }
///  ]
///]
/// ```
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

/// the api call that returns all sensors stored in the database
/// # Arguments
/// * `pool` - the database pool
/// # Returns
/// * `impl Responder` - the response
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

/// This is the api call to get logged errors from the database
/// # Arguments
/// * `pool` - The database pool
/// # Returns
/// * `impl Responder` - The response to the api call
async fn get_logs(pool: web::Data<iot_sound_database::Pool>) -> impl Responder {
    let returned = pool.get_logs().await;
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

/// This is the api call to get a limited amount of logged errors from the database
/// # Arguments
/// * `pool` - The database pool
/// * `info` - The limit amount
/// # Returns
/// * `impl Responder` - The response to the api call
async fn get_logs_limited(
    pool: web::Data<iot_sound_database::Pool>,
    info: web::Query<Info>,
) -> impl Responder {
    let returned = pool.get_logs_limited(info.limit_amount).await;
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

/// the main function
/// # Returns
/// * `Result<(), std::io::Error>` - The result of the main function
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if env::var("DB_USER").is_err()
        || env::var("DB_PASSWORD").is_err()
        || env::var("DB_HOST").is_err()
        || env::var("DB_PORT").is_err()
    {
        println!("Environment variables not set. Loading .env file");
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

    println!("Starting API");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/", web::get().to(index))
            .route("/sound", web::get().to(get_sound))
            .route("/sensors", web::get().to(get_sensors))
            .route("/sound/sorted", web::get().to(get_sound_sorted_by_sensor))
            .route(
                "/sound/sorted/limit",
                web::get().to(get_sound_sorted_by_sensor_limited),
            )
            .route("/logs", web::get().to(get_logs))
            .route("/logs/limit", web::get().to(get_logs_limited))
            .wrap(Cors::permissive())
    })
    .bind("localhost:8081")?
    .run()
    .await
}
