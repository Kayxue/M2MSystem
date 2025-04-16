//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.9

use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize)]
#[sea_orm(table_name = "sensor")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub application_id: String,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::application::Entity",
        from = "Column::ApplicationId",
        to = "super::application::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Application,
    #[sea_orm(has_many = "super::data_container::Entity")]
    DataContainer,
}

impl Related<super::application::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Application.def()
    }
}

impl Related<super::data_container::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DataContainer.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
