use nanoid::nanoid;
use ntex::web::{
    ServiceConfig, WebResponseError, delete,
    error::{ErrorBadRequest, ErrorInternalServerError},
    get, patch, post,
    types::{Json, Path, State},
};
use sea_orm::{ActiveModelTrait, EntityTrait, SqlErr};
use serde::Deserialize;
use serde_json::Value;

use crate::AppState;
use crate::entities::{prelude::*, *};

#[derive(Deserialize)]
struct SensorDataCreate {
    pub container_id: String,
    pub data: Value,
}

#[derive(Deserialize)]
struct SensorDataUpdate {
    pub data: Value,
}

#[derive(Deserialize)]
struct RUDSensorDataParams {
    id: String,
}

#[post("")]
async fn create_sensor_data(
    state: State<AppState>,
    body: Json<SensorDataCreate>,
) -> Result<Json<sensor_data::Model>, impl WebResponseError> {
    let SensorDataCreate { container_id, data } = body.into_inner();

    let new_sensor_data = sensor_data::ActiveModel {
        id: sea_orm::ActiveValue::Set(nanoid!(10)),
        data: sea_orm::ActiveValue::Set(Some(data)),
        container_id: sea_orm::ActiveValue::Set(container_id.to_owned()),
        ..Default::default()
    };

    match new_sensor_data.insert(&state.db).await {
        Ok(entity) => Ok(Json(entity)),
        Err(e) => match e.sql_err() {
            Some(SqlErr::ForeignKeyConstraintViolation(_)) => {
                Err(ErrorBadRequest("Can't find data container"))
            }
            _ => {
                eprintln!("Error creating sensor data: {:?}", e);
                Err(ErrorInternalServerError("Query failed"))
            }
        },
    }
}

#[get("/{id}")]
async fn get_sensor_data(
    state: State<AppState>,
    params: Path<RUDSensorDataParams>,
) -> Result<Json<sensor_data::Model>, impl WebResponseError> {
    let RUDSensorDataParams { id } = params.into_inner();

    match SensorData::find_by_id(id).one(&state.db).await {
        Ok(Some(entity)) => Ok(Json(entity)),
        Ok(None) => Err(ErrorBadRequest("Sensor data not found")),
        Err(e) => {
            eprintln!("Error fetching sensor data: {:?}", e);
            Err(ErrorInternalServerError("Query failed"))
        }
    }
}

#[patch("/{id}")]
async fn update_sensor_data(
    state: State<AppState>,
    params: Path<RUDSensorDataParams>,
    body: Json<SensorDataUpdate>,
) -> Result<Json<sensor_data::Model>, impl WebResponseError> {
    let RUDSensorDataParams { id } = params.into_inner();
    let SensorDataUpdate { data } = body.into_inner();

    match SensorData::find_by_id(id).one(&state.db).await {
        Ok(Some(e)) => {
            let mut entity: sensor_data::ActiveModel = e.into();
            entity.data = sea_orm::ActiveValue::Set(Some(data));
            match entity.update(&state.db).await {
                Ok(updated_entity) => Ok(Json(updated_entity)),
                //TODO: Send new data to subscribers of the data container
                Err(e) => {
                    eprintln!("Error updating sensor data: {:?}", e);
                    Err(ErrorInternalServerError("Query failed"))
                }
            }
        }
        Ok(None) => Err(ErrorBadRequest("Sensor data not found")),
        Err(e) => {
            eprintln!("Error fetching sensor data: {:?}", e);
            Err(ErrorInternalServerError("Query failed"))
        }
    }
}

#[delete("/{id}")]
async fn delete_sensor_data(
    state: State<AppState>,
    params: Path<RUDSensorDataParams>,
) -> Result<&'static str, impl WebResponseError> {
    let RUDSensorDataParams { id } = params.into_inner();

    match SensorData::delete_by_id(id).exec(&state.db).await {
        Ok(_) => Ok("Sensor data deleted successfully"),
        Err(e) => {
            eprintln!("Error deleting sensor data: {:?}", e);
            Err(ErrorInternalServerError("Query failed"))
        }
    }
}

pub fn add_sensor_data_route(cfg: &mut ServiceConfig) {
    cfg.service(create_sensor_data)
        .service(get_sensor_data)
        .service(update_sensor_data)
        .service(delete_sensor_data);
}
