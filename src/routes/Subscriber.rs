use actix_web::{
    Error, delete,
    error::{ErrorBadRequest, ErrorInternalServerError},
    get, patch, post,
    web::{Data, Json, Path, ServiceConfig},
};
use nanoid::nanoid;
use redis::AsyncCommands;
use sea_orm::{ActiveModelTrait, EntityTrait, SqlErr};
use serde::Deserialize;

use crate::{
    AppState,
    entities::{prelude::*, *},
    utils::{get_redis_id, get_redis_set_options},
};

const PREFIX: &str = "Subscriber";

#[derive(Deserialize)]
struct SubscriberCreate {
    container_id: String,
    notification_url: String,
}

#[derive(Deserialize)]
struct SubscriberUpdate {
    notification_url: String,
}

#[derive(Deserialize)]
struct RUDSubscriberParams {
    id: String,
}

#[post("")]
async fn create_subscriber(
    state: Data<AppState>,
    body: Json<SubscriberCreate>,
) -> Result<Json<subscribers::Model>, Error> {
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
        Ok(entity) => {
            let mut redis_conn = state
                .redis
                .get_multiplexed_tokio_connection()
                .await
                .unwrap();
            let _: () = redis_conn
                .set_options(
                    get_redis_id(PREFIX, &entity.id),
                    serde_json::to_string(&entity).unwrap(),
                    get_redis_set_options(),
                )
                .await
                .unwrap();
            Ok(Json(entity))
        }
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
    state: Data<AppState>,
    params: Path<RUDSubscriberParams>,
) -> Result<Json<subscribers::Model>, Error> {
    let RUDSubscriberParams { id } = params.into_inner();
    let mut redis_conn = state
        .redis
        .get_multiplexed_tokio_connection()
        .await
        .unwrap();

    if let Ok(cached_subscriber) = redis_conn.get::<_, String>(get_redis_id(PREFIX, &id)).await {
        if let Ok(entity) = serde_json::from_str::<subscribers::Model>(&cached_subscriber) {
            return Ok(Json(entity));
        }
    }

    match Subscribers::find_by_id(&id).one(&state.db).await {
        Ok(Some(entity)) => {
            let _: () = redis_conn
                .set_options(
                    get_redis_id(PREFIX, &id),
                    serde_json::to_string(&entity).unwrap(),
                    get_redis_set_options(),
                )
                .await
                .unwrap();
            Ok(Json(entity))
        }
        Ok(None) => Err(ErrorBadRequest("Subscriber not found")),
        Err(e) => {
            eprintln!("Error fetching subscriber: {:?}", e);
            Err(ErrorInternalServerError("Query failed"))
        }
    }
}

#[patch("/{id}")]
async fn update_subscriber(
    state: Data<AppState>,
    params: Path<RUDSubscriberParams>,
    body: Json<SubscriberUpdate>,
) -> Result<Json<subscribers::Model>, Error> {
    let RUDSubscriberParams { id } = params.into_inner();
    let SubscriberUpdate { notification_url } = body.into_inner();

    match Subscribers::find_by_id(&id).one(&state.db).await {
        Ok(Some(subscriber)) => {
            let mut subscriber: subscribers::ActiveModel = subscriber.into();
            subscriber.notification_url = sea_orm::ActiveValue::Set(notification_url.to_owned());
            match subscriber.update(&state.db).await {
                Ok(entity) => {
                    let mut redis_conn = state
                        .redis
                        .get_multiplexed_tokio_connection()
                        .await
                        .unwrap();
                    let _: () = redis_conn
                        .set_options(
                            get_redis_id(PREFIX, &id),
                            serde_json::to_string(&entity).unwrap(),
                            get_redis_set_options(),
                        )
                        .await
                        .unwrap();
                    Ok(Json(entity))
                }
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
    state: Data<AppState>,
    params: Path<RUDSubscriberParams>,
) -> Result<&'static str, Error> {
    let RUDSubscriberParams { id } = params.into_inner();

    match Subscribers::delete_by_id(&id).exec(&state.db).await {
        Ok(_) => {
            let mut redis_conn = state
                .redis
                .get_multiplexed_tokio_connection()
                .await
                .unwrap();
            let _: () = redis_conn.del(get_redis_id(PREFIX, &id)).await.unwrap();
            Ok("Subscriber deleted successfully")
        }
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
