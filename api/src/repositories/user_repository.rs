use crate::models::{role, user, user_role};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};

pub struct UserRepository;

impl UserRepository {
    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: &str,
        tenant_id: &str,
    ) -> Result<Option<user::Model>, DbErr> {
        user::Entity::find()
            .filter(user::Column::Id.eq(id))
            .filter(user::Column::TenantId.eq(tenant_id))
            .one(db)
            .await
    }

    /// Find user by ID without tenant filter (for cross-tenant operations)
    pub async fn find_by_id_global(
        db: &DatabaseConnection,
        id: &str,
    ) -> Result<Option<user::Model>, DbErr> {
        user::Entity::find()
            .filter(user::Column::Id.eq(id))
            .one(db)
            .await
    }

    pub async fn find_by_email(
        db: &DatabaseConnection,
        email: &str,
    ) -> Result<Option<user::Model>, DbErr> {
        user::Entity::find()
            .filter(user::Column::Email.eq(email))
            .one(db)
            .await
    }

    pub async fn find_by_email_and_tenant(
        db: &DatabaseConnection,
        email: &str,
        tenant_id: &str,
    ) -> Result<Option<user::Model>, DbErr> {
        user::Entity::find()
            .filter(user::Column::Email.eq(email))
            .filter(user::Column::TenantId.eq(tenant_id))
            .one(db)
            .await
    }

    pub async fn find_all_by_tenant(
        db: &DatabaseConnection,
        tenant_id: &str,
    ) -> Result<Vec<user::Model>, DbErr> {
        user::Entity::find()
            .filter(user::Column::TenantId.eq(tenant_id))
            .all(db)
            .await
    }

    pub async fn create(
        db: &DatabaseConnection,
        id: &str,
        tenant_id: &str,
        name: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<user::Model, DbErr> {
        let now = chrono::Utc::now().fixed_offset();
        let active_model = user::ActiveModel {
            id: Set(id.to_owned()),
            tenant_id: Set(tenant_id.to_owned()),
            name: Set(name.to_owned()),
            email: Set(email.to_owned()),
            password_hash: Set(password_hash.to_owned()),
            pin_hash: Set(None),
            is_active: Set(Some(true)),
            last_login: Set(None),
            two_factor_enabled: Set(false),
            two_factor_code: Set(None),
            two_factor_expires_at: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };
        active_model.insert(db).await
    }

    pub async fn update(db: &DatabaseConnection, model: user::Model) -> Result<user::Model, DbErr> {
        let active_model = user::ActiveModel {
            id: Set(model.id),
            tenant_id: Set(model.tenant_id),
            name: Set(model.name),
            email: Set(model.email),
            password_hash: Set(model.password_hash),
            pin_hash: Set(model.pin_hash),
            is_active: Set(model.is_active),
            last_login: Set(model.last_login),
            two_factor_enabled: Set(model.two_factor_enabled),
            two_factor_code: Set(model.two_factor_code),
            two_factor_expires_at: Set(model.two_factor_expires_at),
            created_at: Set(model.created_at),
            updated_at: Set(chrono::Utc::now().fixed_offset()),
        };
        active_model.update(db).await
    }

    pub async fn get_user_roles(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> Result<Vec<String>, DbErr> {
        let u_roles = user_role::Entity::find()
            .filter(user_role::Column::UserId.eq(user_id))
            .all(db)
            .await?;

        let role_ids: Vec<String> = u_roles.into_iter().map(|ur| ur.role_id).collect();
        if role_ids.is_empty() {
            return Ok(Vec::new());
        }

        let roles = role::Entity::find()
            .filter(role::Column::Id.is_in(role_ids))
            .all(db)
            .await?;

        Ok(roles.into_iter().map(|r| r.name).collect())
    }

    pub async fn assign_role(
        db: &DatabaseConnection,
        user_id: &str,
        role_id: &str,
    ) -> Result<(), DbErr> {
        let active_model = user_role::ActiveModel {
            user_id: Set(user_id.to_owned()),
            role_id: Set(role_id.to_owned()),
        };
        active_model.insert(db).await?;
        Ok(())
    }
}
