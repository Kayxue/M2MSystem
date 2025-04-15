use nanoid::nanoid;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};
use xitca_web::codegen::route;
use xitca_web::error::Error;
use xitca_web::handler::json::{Json, LazyJson};
use xitca_web::handler::state::StateRef;

use crate::AppState;
use crate::CustomError::*;
use crate::entities::{prelude::*, *};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct HomeCreate<'a> {
    pub name: &'a str,
}

#[route("/home", method = post)]
pub async fn createHome(
    state: StateRef<'_, AppState>,
    body: LazyJson<HomeCreate<'_>>,
) -> Result<Json<crate::entities::home::Model>, Error> {
    let HomeCreate { name } = body.deserialize()?;

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
