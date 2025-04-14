use nanoid::nanoid;
use sea_orm::{DatabaseConnection, EntityTrait};
use xitca_web::codegen::route;
use xitca_web::error::Error;
use xitca_web::handler::json::{Json, LazyJson};
use xitca_web::handler::state::StateRef;

use crate::AppState;
use crate::body::Home::HomeCreate;
use crate::entities::prelude::Home;

#[route("/home", method = post)]
pub async fn createHome(
    state: StateRef<'_, AppState>,
    body: LazyJson<HomeCreate<'_>>,
) -> Result<Json<crate::entities::home::Model>, Error> {
    let HomeCreate { name } = body.deserialize()?;

    let newHome = crate::entities::home::ActiveModel {
        id: sea_orm::ActiveValue::Set(nanoid!()),
        name: sea_orm::ActiveValue::Set(name.to_owned()),
        ..Default::default()
    };

    let entity = crate::entities::prelude::Home::insert(newHome)
        .exec_with_returning(&state.db)
        .await
        .expect("Inserted failed");
    
    Ok(Json(entity))
}
