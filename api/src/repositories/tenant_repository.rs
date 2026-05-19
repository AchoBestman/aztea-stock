use sea_orm::{
    ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, Set
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

    pub async fn create(
        db: &DatabaseConnection,
        model: tenant::Model,
    ) -> Result<tenant::Model, DbErr> {
        let active_model = tenant::ActiveModel {
            id: Set(model.id),
            name: Set(model.name),
            business_type: Set(model.business_type),
            email: Set(model.email),
            phone: Set(model.phone),
            address: Set(model.address),
            country: Set(model.country),
            timezone: Set(model.timezone),
            logo_url: Set(model.logo_url),
            is_active: Set(model.is_active),
            is_system: Set(model.is_system),
            two_factor_enabled: Set(model.two_factor_enabled),
            sender_email: Set(model.sender_email),
            sender_user: Set(model.sender_user),
            sender_password: Set(model.sender_password),
            created_at: Set(model.created_at),
            updated_at: Set(model.updated_at),
        };
        active_model.insert(db).await
    }

    pub async fn find_all(
        db: &DatabaseConnection,
    ) -> Result<Vec<tenant::Model>, DbErr> {
        tenant::Entity::find().all(db).await
    }

    pub async fn update(
        db: &DatabaseConnection,
        model: tenant::Model,
    ) -> Result<tenant::Model, DbErr> {
        let active_model = tenant::ActiveModel {
            id: Set(model.id),
            name: Set(model.name),
            business_type: Set(model.business_type),
            email: Set(model.email),
            phone: Set(model.phone),
            address: Set(model.address),
            country: Set(model.country),
            timezone: Set(model.timezone),
            logo_url: Set(model.logo_url),
            is_active: Set(model.is_active),
            is_system: Set(model.is_system),
            two_factor_enabled: Set(model.two_factor_enabled),
            sender_email: Set(model.sender_email),
            sender_user: Set(model.sender_user),
            sender_password: Set(model.sender_password),
            created_at: Set(model.created_at),
            updated_at: Set(model.updated_at),
        };
        active_model.update(db).await
    }
}
