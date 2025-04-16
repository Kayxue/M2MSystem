use nanoid::nanoid;
use ntex::web::error::{ErrorBadRequest, ErrorInternalServerError};
use ntex::web::types::{Json, Path, State};
use ntex::web::{self, ServiceConfig, WebResponseError, delete, get, patch, post, resource};
use sea_orm::{ActiveModelTrait, EntityTrait};

use crate::AppState;
use crate::entities::{prelude::*, *};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct HomeCU {
    pub name: String,
}

#[derive(Deserialize)]
pub struct HomeParams {
    pub id: String,
}

#[post("")]
pub async fn createHome(
    state: State<AppState>,
    body: Json<HomeCU>,
) -> Result<Json<home::Model>, impl WebResponseError> {
    let newHome = home::ActiveModel {
        id: sea_orm::ActiveValue::Set(nanoid!(10)),
        name: sea_orm::ActiveValue::Set(body.name.clone()),
        ..Default::default()
    };

    if let Ok(entity) = newHome.insert(&state.db).await {
        Ok(Json(entity))
    } else {
        Err(ErrorInternalServerError("Failed to create home"))
    }
}

#[get("")]
pub async fn getHomes(
    state: State<AppState>,
) -> Result<Json<Vec<home::Model>>, impl WebResponseError> {
    match Home::find().all(&state.db).await {
        Ok(homes) => Ok(Json(homes)),
        Err(e) => {
            eprintln!("Error fetching homes: {:?}", e);
            Err(ErrorInternalServerError("Failed to fetch homes"))
        }
    }
}

#[get("/{id}")]
pub async fn getHome(
    state: State<AppState>,
    params: Path<HomeParams>,
) -> Result<Json<home::Model>, impl WebResponseError> {
    let HomeParams { id } = params.into_inner();

    match Home::find_by_id(id).one(&state.db).await {
        Ok(Some(home)) => Ok(Json(home)),
        Ok(None) => Err(ErrorBadRequest("Home not found")),
        Err(e) => {
            eprintln!("Error fetching home: {:?}", e);
            Err(ErrorInternalServerError("Home not found"))
        }
    }
}

#[patch("/{id}")]
pub async fn updateHome(
    state: State<AppState>,
    params: Path<HomeParams>,
    body: Json<HomeCU>,
) -> Result<Json<home::Model>, impl WebResponseError> {
    let HomeParams { id } = params.into_inner();
    let HomeCU { name } = body.into_inner();

    match Home::find_by_id(id).one(&state.db).await {
        Ok(Some(home)) => {
            let mut home: home::ActiveModel = home.into();
            home.name = sea_orm::ActiveValue::Set(name.to_owned());
            match home.update(&state.db).await {
                Ok(updated_home) => Ok(Json(updated_home)),
                Err(e) => {
                    eprintln!("Error updating home: {:?}", e);
                    Err(ErrorInternalServerError("Failed to update home"))
                }
            }
        }
        Ok(None) => return Err(ErrorBadRequest("Home not found")),
        Err(e) => {
            eprintln!("Error fetching home: {:?}", e);
            Err(ErrorInternalServerError("Failed to fetch home"))
        }
    }
}

#[delete("/{id}")]
pub async fn deleteHome(
    state: State<AppState>,
    params: Path<HomeParams>,
) -> Result<&'static str, impl WebResponseError> {
    let HomeParams { id } = params.into_inner();

    match Home::delete_by_id(id).exec(&state.db).await {
        Ok(_) => Ok("Home deleted successfully"),
        Err(e) => {
            eprintln!("Error deleting home: {:?}", e);
            Err(ErrorInternalServerError("Failed to delete home"))
        }
    }
}

pub fn addHomeRoute(cfg: &mut ServiceConfig) {
    cfg.service(createHome)
        .service(getHome)
        .service(getHomes)
        .service(updateHome)
        .service(deleteHome);
}
