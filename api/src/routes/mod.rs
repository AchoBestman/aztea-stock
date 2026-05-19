use utoipa::OpenApi;

pub mod admin;
pub mod auth;
pub mod health;
pub mod products;
pub mod reports;
pub mod sales;
pub mod stock;
pub mod subscriptions;
pub mod sync;

#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_check,
        auth::login,
        products::list_products,
        products::get_product,
        crate::controllers::role::list_roles,
        crate::controllers::role::get_role,
        crate::controllers::role::create_role,
        crate::controllers::role::update_role,
        crate::controllers::role::delete_role,
    ),
    components(
        schemas(
            health::HealthResponse,
            auth::LoginPayload,
            auth::LoginResponse,
            auth::UserProfile,
            products::Product,
            products::PaginatedMeta,
            products::PaginatedProductResponse,
            crate::models::role::Role,
            crate::models::role::CreateRolePayload,
            crate::models::role::UpdateRolePayload,
            crate::models::role::DeleteRoleResponse,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Health", description = "Health check and diagnostics"),
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Products", description = "Product catalog management"),
        (name = "Admin - Roles", description = "Tenant roles administration CRUD")
    )
)]
pub struct ApiDoc;

pub struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearerAuth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}
