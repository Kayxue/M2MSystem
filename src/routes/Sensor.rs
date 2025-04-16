use nanoid::nanoid;
use ntex::web::{
    ServiceConfig, WebResponseError, delete,
    error::{ErrorBadRequest, ErrorInternalServerError},
    get, patch, post,
    types::{Json, Path, State},
};
use sea_orm::{ActiveModelTrait, EntityTrait, SqlErr};
use serde::Deserialize;

use crate::AppState;
use crate::entities::{prelude::*, *};

#[derive(Deserialize)]
struct SensorCreate {
    pub name: String,
    pub application_id: String,
}

#[derive(Deserialize)]
struct SensorUpdate {
    pub name: String,
}

#[derive(Deserialize)]
struct RUDSensorParams {
    id: String,
}

#[post("")]
async fn createSensor(
    state: State<AppState>,
    body: Json<SensorCreate>,
) -> Result<Json<sensor::Model>, impl WebResponseError> {
    let SensorCreate {
        name,
        application_id,
    } = body.into_inner();

    let newSensor = sensor::ActiveModel {
        id: sea_orm::ActiveValue::Set(nanoid!(10)),
        name: sea_orm::ActiveValue::Set(name.to_owned()),
        application_id: sea_orm::ActiveValue::Set(application_id.to_owned()),
        ..Default::default()
    };

    match newSensor.insert(&state.db).await {
        Ok(entity) => Ok(Json(entity)),
        Err(e) => match e.sql_err() {
            Some(SqlErr::ForeignKeyConstraintViolation(_)) => {
                Err(ErrorBadRequest("Can't find application"))
            }
            _ => {
                eprintln!("Error creating sensor: {:?}", e);
                Err(ErrorInternalServerError("Query failed"))
            }
        },
    }
}

#[get("/{id}")]
async fn getSensor(
    state: State<AppState>,
    params: Path<RUDSensorParams>,
) -> Result<Json<sensor::Model>, impl WebResponseError> {
    let RUDSensorParams { id } = params.into_inner();

    match Sensor::find_by_id(id).one(&state.db).await {
        Ok(Some(entity)) => Ok(Json(entity)),
        Ok(None) => Err(ErrorBadRequest("Can't find sensor")),
        Err(e) => {
            eprintln!("Error fetching sensor: {:?}", e);
            Err(ErrorInternalServerError("Query failed"))
        }
    }
}

#[patch("/{id}")]
async fn updateSensor(
    state: State<AppState>,
    params: Path<RUDSensorParams>,
    body: Json<SensorUpdate>,
) -> Result<Json<sensor::Model>, impl WebResponseError> {
    let RUDSensorParams { id } = params.into_inner();
    let SensorUpdate { name } = body.into_inner();

    match Sensor::find_by_id(id).one(&state.db).await {
        Ok(Some(e)) => {
            let mut entity: sensor::ActiveModel = e.into();
            entity.name = sea_orm::ActiveValue::Set(name.to_owned());
            match entity.update(&state.db).await {
                Ok(updated_entity) => Ok(Json(updated_entity)),
                Err(e) => {
                    eprintln!("Error updating sensor: {:?}", e);
                    Err(ErrorInternalServerError("Failed to update sensor"))
                }
            }
        }
        Ok(None) => Err(ErrorBadRequest("Can't find sensor")),
        Err(e) => {
            eprintln!("Error fetching sensor: {:?}", e);
            Err(ErrorInternalServerError("Query failed"))
        }
    }
}

#[delete("/{id}")]
async fn deleteSensor(
    state: State<AppState>,
    params: Path<RUDSensorParams>,
) -> Result<&'static str, impl WebResponseError> {
    let RUDSensorParams { id } = params.into_inner();

    match Sensor::delete_by_id(id).exec(&state.db).await {
        Ok(_) => Ok("Sensor deleted successfully"),
        Err(e) => {
            eprintln!("Error deleting sensor: {:?}", e);
            Err(ErrorInternalServerError("Failed to delete sensor"))
        }
    }
}

pub fn addSensorRoute(cfg: &mut ServiceConfig) {
    cfg.service(createSensor)
        .service(getSensor)
        .service(updateSensor)
        .service(deleteSensor);
}
