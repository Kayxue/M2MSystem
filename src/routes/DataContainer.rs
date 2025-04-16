use nanoid::nanoid;
use ntex::web::error::{ErrorBadRequest, ErrorInternalServerError};
use ntex::web::types::{Json, Path, State};
use ntex::web::{ServiceConfig, WebResponseError, delete, get, post, scope};
use sea_orm::{ActiveModelTrait, EntityTrait, SqlErr};
use serde::Deserialize;

use crate::AppState;
use crate::entities::{prelude::*, *};

#[derive(Deserialize)]
struct DataContainerCreate {
    pub sensor_id: String,
}

#[derive(Deserialize)]
struct RDDataContainerParams {
    id: String,
}

#[post("")]
async fn create_data_container(
    state: State<AppState>,
    body: Json<DataContainerCreate>,
) -> Result<Json<data_container::Model>, impl WebResponseError> {
    let DataContainerCreate { sensor_id } = body.into_inner();

    let new_data_container = data_container::ActiveModel {
        id: sea_orm::ActiveValue::Set(nanoid!(10)),
        sensor_id: sea_orm::ActiveValue::Set(sensor_id.to_owned()),
        ..Default::default()
    };

    match new_data_container.insert(&state.db).await {
        Ok(entity) => Ok(Json(entity)),
        Err(e) => match e.sql_err() {
            Some(SqlErr::ForeignKeyConstraintViolation(_)) => {
                Err(ErrorBadRequest("Can't find sensor"))
            }
            _ => {
                eprintln!("Error creating data container: {:?}", e);
                Err(ErrorInternalServerError("Query failed"))
            }
        },
    }
}

#[get("/{id}")]
async fn get_data_container(
    state: State<AppState>,
    params: Path<RDDataContainerParams>,
) -> Result<Json<data_container::Model>, impl WebResponseError> {
    let RDDataContainerParams { id } = params.into_inner();
    match DataContainer::find_by_id(id).one(&state.db).await {
        Ok(Some(entity)) => Ok(Json(entity)),
        Ok(None) => Err(ErrorBadRequest("Data container not found")),
        Err(e) => {
            eprintln!("Error fetching data container: {:?}", e);
            Err(ErrorInternalServerError("Query failed"))
        }
    }
}

#[delete("/{id}")]
async fn delete_data_container(
    state: State<AppState>,
    params: Path<RDDataContainerParams>,
) -> Result<&'static str, impl WebResponseError> {
    let RDDataContainerParams { id } = params.into_inner();

    match DataContainer::delete_by_id(id).exec(&state.db).await {
        Ok(_) => Ok("Data container deleted"),
        Err(e) => {
            eprintln!("Error deleting data container: {:?}", e);
            Err(ErrorInternalServerError("Query failed"))
        }
    }
}

pub fn add_data_container_routes(cfg: &mut ServiceConfig) {
    cfg.service(create_data_container)
        .service(get_data_container)
        .service(delete_data_container);
}
