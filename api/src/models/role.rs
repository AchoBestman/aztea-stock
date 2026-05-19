use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use sqlx::{AnyPool, Error};

#[derive(Serialize, Deserialize, ToSchema, sqlx::FromRow, Debug, Clone)]
pub struct Role {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateRolePayload {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateRolePayload {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct DeleteRoleResponse {
    pub success: bool,
    pub message: String,
}

impl Role {
    pub async fn list_by_tenant(pool: &AnyPool, tenant_id: &str) -> Result<Vec<Role>, Error> {
        sqlx::query_as::<_, Role>(
            "SELECT id, tenant_id, name, description FROM roles WHERE tenant_id = $1"
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &AnyPool, tenant_id: &str, id: &str) -> Result<Option<Role>, Error> {
        sqlx::query_as::<_, Role>(
            "SELECT id, tenant_id, name, description FROM roles WHERE id = $1 AND tenant_id = $2"
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn exists_by_name(pool: &AnyPool, tenant_id: &str, name: &str) -> Result<bool, Error> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM roles WHERE tenant_id = $1 AND name = $2"
        )
        .bind(tenant_id)
        .bind(name)
        .fetch_one(pool)
        .await?;
        Ok(count.0 > 0)
    }

    pub async fn exists_by_name_exclude(pool: &AnyPool, tenant_id: &str, name: &str, exclude_id: &str) -> Result<bool, Error> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM roles WHERE tenant_id = $1 AND name = $2 AND id != $3"
        )
        .bind(tenant_id)
        .bind(name)
        .bind(exclude_id)
        .fetch_one(pool)
        .await?;
        Ok(count.0 > 0)
    }

    pub async fn exists_by_id(pool: &AnyPool, tenant_id: &str, id: &str) -> Result<bool, Error> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM roles WHERE id = $1 AND tenant_id = $2"
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_one(pool)
        .await?;
        Ok(count.0 > 0)
    }

    pub async fn create(pool: &AnyPool, tenant_id: &str, name: &str, description: Option<&str>) -> Result<Role, Error> {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query_as::<_, Role>(
            "INSERT INTO roles (id, tenant_id, name, description) VALUES ($1, $2, $3, $4) RETURNING id, tenant_id, name, description"
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(name)
        .bind(description)
        .fetch_one(pool)
        .await
    }

    pub async fn update(pool: &AnyPool, id: &str, tenant_id: &str, name: &str, description: Option<&str>) -> Result<Role, Error> {
        sqlx::query_as::<_, Role>(
            "UPDATE roles SET name = $1, description = $2 WHERE id = $3 AND tenant_id = $4 RETURNING id, tenant_id, name, description"
        )
        .bind(name)
        .bind(description)
        .bind(id)
        .bind(tenant_id)
        .fetch_one(pool)
        .await
    }

    pub async fn delete(pool: &AnyPool, id: &str, tenant_id: &str) -> Result<bool, Error> {
        let rows_affected = sqlx::query(
            "DELETE FROM roles WHERE id = $1 AND tenant_id = $2"
        )
        .bind(id)
        .bind(tenant_id)
        .execute(pool)
        .await?
        .rows_affected();
        Ok(rows_affected > 0)
    }
}
