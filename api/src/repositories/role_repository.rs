use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, DeleteResult, EntityTrait,
    PaginatorTrait, QueryFilter, Set
};
use crate::models::role;

pub struct RoleRepository;

impl RoleRepository {
    pub async fn find_all_by_tenant(
        db: &DatabaseConnection,
        tenant_id: &str,
    ) -> Result<Vec<role::Model>, DbErr> {
        role::Entity::find()
            .filter(role::Column::TenantId.eq(tenant_id))
            .all(db)
            .await
    }

    pub async fn find_all_filtered(
        db: &DatabaseConnection,
        tenant_id: Option<&str>,
        name_query: Option<&str>,
    ) -> Result<Vec<role::Model>, DbErr> {
        let mut query = role::Entity::find();
        
        if let Some(t_id) = tenant_id {
            query = query.filter(role::Column::TenantId.eq(t_id));
        }
        
        if let Some(n) = name_query {
            query = query.filter(role::Column::Name.contains(n));
        }
        
        query.all(db).await
    }

    pub async fn find_by_id_global(
        db: &DatabaseConnection,
        id: &str,
    ) -> Result<Option<role::Model>, DbErr> {
        role::Entity::find_by_id(id).one(db).await
    }

    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: &str,
        tenant_id: &str,
    ) -> Result<Option<role::Model>, DbErr> {
        role::Entity::find()
            .filter(role::Column::Id.eq(id))
            .filter(role::Column::TenantId.eq(tenant_id))
            .one(db)
            .await
    }

    pub async fn exists_by_name(
        db: &DatabaseConnection,
        name: &str,
        tenant_id: &str,
    ) -> Result<bool, DbErr> {
        let count = role::Entity::find()
            .filter(role::Column::TenantId.eq(tenant_id))
            .filter(role::Column::Name.eq(name))
            .count(db)
            .await?;
        Ok(count > 0)
    }

    pub async fn exists_by_name_exclude(
        db: &DatabaseConnection,
        name: &str,
        tenant_id: &str,
        exclude_id: &str,
    ) -> Result<bool, DbErr> {
        let count = role::Entity::find()
            .filter(role::Column::TenantId.eq(tenant_id))
            .filter(role::Column::Name.eq(name))
            .filter(role::Column::Id.ne(exclude_id))
            .count(db)
            .await?;
        Ok(count > 0)
    }

    pub async fn exists_by_id(
        db: &DatabaseConnection,
        id: &str,
        tenant_id: &str,
    ) -> Result<bool, DbErr> {
        let count = role::Entity::find()
            .filter(role::Column::Id.eq(id))
            .filter(role::Column::TenantId.eq(tenant_id))
            .count(db)
            .await?;
        Ok(count > 0)
    }

    pub async fn create(
        db: &DatabaseConnection,
        id: &str,
        tenant_id: &str,
        name: &str,
        description: Option<String>,
    ) -> Result<role::Model, DbErr> {
        let active_model = role::ActiveModel {
            id: Set(id.to_owned()),
            tenant_id: Set(tenant_id.to_owned()),
            name: Set(name.to_owned()),
            description: Set(description),
            created_at: Set(chrono::Utc::now().naive_utc()),
            updated_at: Set(chrono::Utc::now().naive_utc()),
        };
        active_model.insert(db).await
    }

    pub async fn update(
        db: &DatabaseConnection,
        id: &str,
        tenant_id: &str,
        name: &str,
        description: Option<String>,
    ) -> Result<role::Model, DbErr> {
        let mut active_model: role::ActiveModel = role::Entity::find()
            .filter(role::Column::Id.eq(id))
            .filter(role::Column::TenantId.eq(tenant_id))
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Role not found".to_string()))?
            .into();
        active_model.name = Set(name.to_owned());
        active_model.description = Set(description);
        active_model.updated_at = Set(chrono::Utc::now().naive_utc());
        active_model.update(db).await
    }

    pub async fn delete(
        db: &DatabaseConnection,
        id: &str,
        tenant_id: &str,
    ) -> Result<DeleteResult, DbErr> {
        role::Entity::delete_many()
            .filter(role::Column::Id.eq(id))
            .filter(role::Column::TenantId.eq(tenant_id))
            .exec(db)
            .await
    }
}
