use dotenv::dotenv;
use ntex::{
    main,
    web::{self, App, HttpServer, get},
};
use routes::{Application::addApplicationRoute, Home::addHomeRoute, Sensor::addSensorRoute};
use sea_orm::{Database, DatabaseConnection};
use std::env;

mod entities;

mod routes;

#[derive(Clone)]
struct AppState {
    db: DatabaseConnection,
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

    let app_state = AppState { db };

    HttpServer::new(move || {
        App::new()
            .state(app_state.clone())
            .service(root)
            .service(web::scope("/home").configure(addHomeRoute))
            .service(web::scope("/application").configure(addApplicationRoute))
            .service(web::scope("/sensor").configure(addSensorRoute))
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
}
