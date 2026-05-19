use sea_orm::{
    ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait
};
use crate::models::tenant;

pub struct TenantRepository;

impl TenantRepository {
    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: &str,
    ) -> Result<Option<tenant::Model>, DbErr> {
        tenant::Entity::find_by_id(id).one(db).await
    }

    pub async fn update(
        db: &DatabaseConnection,
        model: tenant::Model,
    ) -> Result<tenant::Model, DbErr> {
        let active_model: tenant::ActiveModel = model.into();
        active_model.update(db).await
    }
}
