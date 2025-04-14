//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.9

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "home")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub created_at: DateTime,
    #[sea_orm(column_type = "Text")]
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::application::Entity")]
    Application,
}

impl Related<super::application::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Application.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
