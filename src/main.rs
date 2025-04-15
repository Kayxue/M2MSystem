use dotenv::dotenv;
use sea_orm::{Database, DatabaseConnection};
use std::env;
use xitca_web::{App, codegen::route, middleware::Logger};

mod entities;

mod routes;
use routes::{
    Application::{
        addApplication, deleteApplication, getApplication, getHomeApplication, updateApplication,
    },
    Home::*,
};

mod CustomError;

#[derive(Clone)]
struct AppState {
    db: DatabaseConnection,
}

#[route("/",method = get)]
async fn root() -> &'static str {
    "Hello World"
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let db: DatabaseConnection =
        Database::connect(env::var("DATABASE_URL").expect("DATABASE_URL must be set"))
            .await
            .expect("Failed to connect to database");

    let app_state = AppState { db };

    App::new()
        .with_state(app_state)
        .at_typed(root)
        //Home Routes
        .at_typed(createHome)
        .at_typed(getHomes)
        .at_typed(getHome)
        .at_typed(updateHome)
        .at_typed(deleteHome)
        //Application Routes
        .at_typed(getHomeApplication)
        .at_typed(addApplication)
        .at_typed(getApplication)
        .at_typed(updateApplication)
        .at_typed(deleteApplication)
        .enclosed(Logger::new())
        .serve()
        .bind(("0.0.0.0", 3000))?
        .run()
        .await
}
