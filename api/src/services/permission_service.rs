use crate::{
    errors::ApiError,
    models::permission
};
use sea_orm::{DatabaseConnection, EntityTrait, Order, QueryOrder};
use std::collections::BTreeMap;

#[derive(serde::Serialize, utoipa::ToSchema, Clone, Debug)]
pub struct PermissionResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(serde::Serialize, utoipa::ToSchema, Clone, Debug)]
pub struct GroupedPermissionsResponse {
    pub group: String,
    pub permissions: Vec<PermissionResponse>,
}

pub struct PermissionService;

impl PermissionService {
    pub async fn list_grouped_permissions(
        db: &DatabaseConnection,
    ) -> Result<Vec<GroupedPermissionsResponse>, ApiError> {
        let perms = permission::Entity::find()
            .order_by(permission::Column::ModelGroup, Order::Asc)
            .order_by(permission::Column::Name, Order::Asc)
            .all(db)
            .await?;

        // Group by model_group using a BTreeMap to maintain sorted order of groups
        let mut grouped: BTreeMap<String, Vec<PermissionResponse>> = BTreeMap::new();

        for p in perms {
            grouped.entry(p.model_group).or_default().push(PermissionResponse {
                id: p.id,
                name: p.name,
                description: p.description,
            });
        }

        let response = grouped
            .into_iter()
            .map(|(group, permissions)| GroupedPermissionsResponse {
                group,
                permissions,
            })
            .collect();

        Ok(response)
    }
}
