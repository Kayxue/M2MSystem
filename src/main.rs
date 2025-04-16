use dotenv::dotenv;
use ntex::{
    main,
    web::{self, App, HttpServer, get},
};
use redis::Client;
use routes::{
    Application::add_application_route, DataContainer::add_data_container_routes,
    Home::add_home_route, Sensor::add_sensor_route, SensorData::add_sensor_data_route,
    Subscriber::add_subscriber_route,
};
use sea_orm::{Database, DatabaseConnection};
use std::env;

mod entities;

mod routes;

mod utils;

#[derive(Clone)]
struct AppState {
    db: DatabaseConnection,
    redis: Client,
}

#[get("/")]
async fn root() -> &'static str {
    "Hello World"
}

#[main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env_logger::init();

    let db: DatabaseConnection =
        Database::connect(env::var("DATABASE_URL").expect("DATABASE_URL must be set"))
            .await
            .expect("Failed to connect to database");

    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let redis_client = Client::open(redis_url).expect("Failed to connect to Redis");

    let app_state = AppState {
        db,
        redis: redis_client,
    };

    HttpServer::new(move || {
        App::new()
            .state(app_state.clone())
            .service(root)
            .service(web::scope("/home").configure(add_home_route))
            .service(web::scope("/application").configure(add_application_route))
            .service(web::scope("/sensor").configure(add_sensor_route))
            .service(web::scope("/data_container").configure(add_data_container_routes))
            .service(web::scope("/sensor_data").configure(add_sensor_data_route))
            .service(web::scope("/subscribers").configure(add_subscriber_route))
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
}
