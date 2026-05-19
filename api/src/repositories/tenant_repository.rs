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

    pub async fn find_all_filtered(
        db: &DatabaseConnection,
        business_type: Option<&str>,
        search: Option<&str>,
        is_active: Option<bool>,
        created_after: Option<chrono::DateTime<chrono::FixedOffset>>,
        created_before: Option<chrono::DateTime<chrono::FixedOffset>>,
    ) -> Result<Vec<tenant::Model>, DbErr> {
        use sea_orm::{QueryFilter, ColumnTrait, Condition};
        let mut query = tenant::Entity::find();

        if let Some(bt) = business_type {
            query = query.filter(tenant::Column::BusinessType.eq(bt));
        }

        if let Some(status) = is_active {
            query = query.filter(tenant::Column::IsActive.eq(status));
        }

        if let Some(after) = created_after {
            query = query.filter(tenant::Column::CreatedAt.gte(after));
        }

        if let Some(before) = created_before {
            query = query.filter(tenant::Column::CreatedAt.lte(before));
        }

        if let Some(s) = search {
            let search_pattern = format!("%{}%", s);
            query = query.filter(
                Condition::any()
                    .add(tenant::Column::Name.like(&search_pattern))
                    .add(tenant::Column::Email.like(&search_pattern))
                    .add(tenant::Column::Phone.like(&search_pattern))
                    .add(tenant::Column::Country.like(&search_pattern))
                    .add(tenant::Column::Address.like(&search_pattern))
            );
        }

        query.all(db).await
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
