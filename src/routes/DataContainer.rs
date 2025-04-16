use nanoid::nanoid;
use ntex::web::error::{ErrorBadRequest, ErrorInternalServerError};
use ntex::web::types::{Json, State};
use ntex::web::{WebResponseError, post};
use sea_orm::{ActiveModelTrait, SqlErr};
use serde::Deserialize;

use crate::AppState;
use crate::entities::{prelude::*, *};

#[derive(Deserialize)]
struct DataContainerCreate {
    pub sensor_id: String,
}

#[derive(Deserialize)]
struct DataContainerUpdate {
    pub name: String,
}

#[derive(Deserialize)]
struct RUDDataContainerParams {
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
