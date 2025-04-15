use nanoid::nanoid;
use sea_orm::{ActiveModelTrait, EntityTrait};
use xitca_web::codegen::route;
use xitca_web::error::Error;
use xitca_web::handler::json::{Json, LazyJson};
use xitca_web::handler::params::LazyParams;
use xitca_web::handler::state::StateRef;

use crate::AppState;
use crate::CustomError::*;
use crate::entities::{prelude::*, *};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct HomeCU<'a> {
    pub name: &'a str,
}

#[derive(Deserialize)]
pub struct HomeParams<'a> {
    pub id: &'a str,
}

#[route("/home", method = post)]
pub async fn createHome(
    state: StateRef<'_, AppState>,
    body: LazyJson<HomeCU<'_>>,
) -> Result<Json<home::Model>, Error> {
    let HomeCU { name } = body.deserialize()?;

    let newHome = home::ActiveModel {
        id: sea_orm::ActiveValue::Set(nanoid!(10)),
        name: sea_orm::ActiveValue::Set(name.to_owned()),
        ..Default::default()
    };

    if let Ok(entity) = newHome.insert(&state.db).await {
        Ok(Json(entity))
    } else {
        Err(InternalServerError::new("Failed to create home").into())
    }
}

#[route("/home/getHomes", method = get)]
pub async fn getHomes(state: StateRef<'_, AppState>) -> Result<Json<Vec<home::Model>>, Error> {
    match Home::find().all(&state.db).await {
        Ok(homes) => Ok(Json(homes)),
        Err(e) => {
            eprintln!("Error fetching homes: {:?}", e);
            Err(InternalServerError::new("Failed to fetch homes").into())
        }
    }
}

#[route("/home/get/:id", method = get)]
pub async fn getHome(
    state: StateRef<'_, AppState>,
    params: LazyParams<'_, HomeParams<'_>>,
) -> Result<Json<home::Model>, Error> {
    let HomeParams { id } = params.deserialize()?;

    match Home::find_by_id(id).one(&state.db).await {
        Ok(Some(home)) => Ok(Json(home)),
        Ok(None) => Err(BadRequest::new("Home not found").into()),
        Err(e) => {
            eprintln!("Error fetching home: {:?}", e);
            Err(InternalServerError::new("Failed to fetch home").into())
        }
    }
}

#[route("/home/update/:id", method = patch)]
pub async fn updateHome(
    state: StateRef<'_, AppState>,
    params: LazyParams<'_, HomeParams<'_>>,
    body: LazyJson<HomeCU<'_>>,
) -> Result<Json<home::Model>, Error> {
    let HomeParams { id } = params.deserialize()?;
    let HomeCU { name } = body.deserialize()?;

    match Home::find_by_id(id).one(&state.db).await {
        Ok(Some(home)) => {
            let mut home: home::ActiveModel = home.into();
            home.name = sea_orm::ActiveValue::Set(name.to_owned());
            match home.update(&state.db).await {
                Ok(updated_home) => Ok(Json(updated_home)),
                Err(e) => {
                    eprintln!("Error updating home: {:?}", e);
                    Err(InternalServerError::new("Failed to update home").into())
                }
            }
        }
        Ok(None) => return Err(BadRequest::new("Home not found").into()),
        Err(e) => {
            eprintln!("Error fetching home: {:?}", e);
            Err(InternalServerError::new("Failed to fetch home").into())
        }
    }
}

#[route("/home/delete/:id", method = delete)]
pub async fn deleteHome(
    state: StateRef<'_, AppState>,
    params: LazyParams<'_, HomeParams<'_>>,
) -> Result<&'static str, Error> {
    let HomeParams { id } = params.deserialize()?;

    match Home::delete_by_id(id).exec(&state.db).await {
        Ok(_) => Ok("Home deleted successfully"),
        Err(e) => {
            eprintln!("Error deleting home: {:?}", e);
            Err(InternalServerError::new("Failed to delete home").into())
        }
    }
}
