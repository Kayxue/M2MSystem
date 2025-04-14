use dotenv::dotenv;
use sea_orm::{Database, DatabaseConnection};
use serde::Deserialize;
use xitca_web::{App, codegen::route, handler::params::LazyParams, middleware::Logger};

mod entities;
use entities::prelude::*;

mod body;
mod params;
mod routes;

#[derive(Clone)]
struct AppState {
    db: DatabaseConnection,
}

#[derive(Deserialize)]
struct hello<'a> {
    name: &'a str,
}

#[route("/",method = get)]
async fn root() -> &'static str {
    "Hello World"
}

#[route("/location/:name", method = get)]
async fn about(params: LazyParams<'_, hello<'_>>) -> String {
    let hello { name } = params.deserialize().unwrap();
    name.to_owned()
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let db: DatabaseConnection = Database::connect("protocol://username:password@host/database")
        .await
        .expect("Failed to connect to database");

    let app_state = AppState { db };

    App::new()
        .with_state(app_state)
        .at_typed(root)
        .at_typed(about)
        .enclosed(Logger::new())
        .serve()
        .bind(("0.0.0.0", 3000))?
        .run()
        .await
}
