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
struct SubscriberCreate {
    pub container_id: String,
    pub notification_url: String,
}

#[derive(Deserialize)]
struct SubscriberUpdate {
    pub notification_url: String,
}

#[derive(Deserialize)]
struct RUDSubscriberParams {
    id: String,
}

#[post("")]
async fn create_subscriber(
    state: State<AppState>,
    body: Json<SubscriberCreate>,
) -> Result<Json<subscribers::Model>, impl WebResponseError> {
    let SubscriberCreate {
        container_id,
        notification_url,
    } = body.into_inner();

    let new_subscriber = subscribers::ActiveModel {
        id: sea_orm::ActiveValue::Set(nanoid!(10)),
        notification_url: sea_orm::ActiveValue::Set(notification_url.to_owned()),
        container_id: sea_orm::ActiveValue::Set(container_id.to_owned()),
        ..Default::default()
    };

    match new_subscriber.insert(&state.db).await {
        Ok(entity) => Ok(Json(entity)),
        Err(e) => match e.sql_err() {
            Some(SqlErr::ForeignKeyConstraintViolation(_)) => {
                Err(ErrorBadRequest("Can't find data container"))
            }
            _ => {
                eprintln!("Error creating subscriber: {:?}", e);
                Err(ErrorInternalServerError("Query failed"))
            }
        },
    }
}

#[get("/{id}")]
async fn get_subscriber(
    state: State<AppState>,
    params: Path<RUDSubscriberParams>,
) -> Result<Json<subscribers::Model>, impl WebResponseError> {
    let RUDSubscriberParams { id } = params.into_inner();
    match Subscribers::find_by_id(id).one(&state.db).await {
        Ok(Some(entity)) => Ok(Json(entity)),
        Ok(None) => Err(ErrorBadRequest("Subscriber not found")),
        Err(e) => {
            eprintln!("Error fetching subscriber: {:?}", e);
            Err(ErrorInternalServerError("Query failed"))
        }
    }
}

#[patch("/{id}")]
async fn update_subscriber(
    state: State<AppState>,
    params: Path<RUDSubscriberParams>,
    body: Json<SubscriberUpdate>,
) -> Result<Json<subscribers::Model>, impl WebResponseError> {
    let RUDSubscriberParams { id } = params.into_inner();
    let SubscriberUpdate { notification_url } = body.into_inner();

    match Subscribers::find_by_id(id).one(&state.db).await {
        Ok(Some(subscriber)) => {
            let mut subscriber: subscribers::ActiveModel = subscriber.into();
            subscriber.notification_url = sea_orm::ActiveValue::Set(notification_url.to_owned());
            match subscriber.update(&state.db).await {
                Ok(entity) => Ok(Json(entity)),
                Err(e) => {
                    eprintln!("Error updating subscriber: {:?}", e);
                    Err(ErrorInternalServerError("Query failed"))
                }
            }
        }
        Ok(None) => Err(ErrorBadRequest("Subscriber not found")),
        Err(e) => {
            eprintln!("Error fetching subscriber: {:?}", e);
            Err(ErrorInternalServerError("Query failed"))
        }
    }
}

#[delete("/{id}")]
async fn delete_subscriber(
    state: State<AppState>,
    params: Path<RUDSubscriberParams>,
) -> Result<&'static str, impl WebResponseError> {
    let RUDSubscriberParams { id } = params.into_inner();

    match Subscribers::delete_by_id(id).exec(&state.db).await {
        Ok(_) => Ok("Subscriber deleted successfully"),
        Err(e) => {
            eprintln!("Error deleting subscriber: {:?}", e);
            Err(ErrorInternalServerError("Query failed"))
        }
    }
}

pub fn add_subscriber_route(cfg: &mut ServiceConfig) {
    cfg.service(create_subscriber)
        .service(get_subscriber)
        .service(update_subscriber)
        .service(delete_subscriber);
}
