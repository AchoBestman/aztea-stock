use axum::{routing::get, Router, Json, extract::{Query, Path}};
use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, IntoParams};

/// Représente un produit dans le système
#[derive(Serialize, ToSchema)]
pub struct Product {
    /// Identifiant unique UUID du produit
    pub id: String,
    /// Code-barres unique du produit
    pub barcode: Option<String>,
    /// Nom complet du produit
    pub name: String,
    /// Description textuelle du produit
    pub description: Option<String>,
    /// Prix d'achat initial hors taxe
    pub purchase_price: f64,
    /// Prix de vente au public toutes taxes comprises
    pub selling_price: f64,
    /// Indique si le produit est actif dans le catalogue
    pub is_active: bool,
}

/// Paramètres de requête pour filtrer la liste des produits
#[derive(Deserialize, IntoParams)]
pub struct ProductFilterParams {
    /// Numéro de la page (commence à 1, valeur par défaut: 1)
    pub page: Option<i32>,
    /// Nombre d'éléments retournés par page (valeur par défaut: 50)
    pub per_page: Option<i32>,
    /// Filtre textuel de recherche (recherche sur le nom ou le code-barres)
    pub search: Option<String>,
    /// Filtrer les produits appartenant à cette catégorie spécifique (UUID)
    pub category_id: Option<String>,
}

/// Métadonnées de pagination
#[derive(Serialize, ToSchema)]
pub struct PaginatedMeta {
    /// Page actuelle
    pub page: i32,
    /// Éléments par page
    pub per_page: i32,
    /// Nombre total d'éléments correspondant aux filtres
    pub total: i32,
    /// Nombre total de pages disponibles
    pub total_pages: i32,
}

/// Réponse standard paginée pour la liste des produits
#[derive(Serialize, ToSchema)]
pub struct PaginatedProductResponse {
    /// Indique si la requête a réussi
    pub success: bool,
    /// Liste des produits de la page courante
    pub data: Vec<Product>,
    /// Informations de pagination
    pub meta: PaginatedMeta,
}

#[utoipa::path(
    get,
    path = "/api/v1/products",
    params(
        ProductFilterParams
    ),
    responses(
        (status = 200, description = "Liste des produits récupérée avec succès.", body = PaginatedProductResponse),
        (status = 401, description = "Authentification requise ou token JWT invalide.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Products"
)]
pub async fn list_products(
    Query(_filter): Query<ProductFilterParams>,
) -> Json<PaginatedProductResponse> {
    Json(PaginatedProductResponse {
        success: true,
        data: vec![],
        meta: PaginatedMeta {
            page: 1,
            per_page: 50,
            total: 0,
            total_pages: 0,
        },
    })
}

#[utoipa::path(
    get,
    path = "/api/v1/products/{id}",
    params(
        ("id" = String, Path, description = "L'identifiant unique UUID du produit recherché")
    ),
    responses(
        (status = 200, description = "Détails du produit récupérés avec succès.", body = Product),
        (status = 401, description = "Authentification requise ou token JWT invalide."),
        (status = 404, description = "Produit introuvable pour l'UUID spécifié.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Products"
)]
pub async fn get_product(
    Path(_id): Path<String>,
) -> Result<Json<Product>, crate::errors::ApiError> {
    Err(crate::errors::ApiError::NotFound("Product not found".to_string()))
}

pub fn router() -> Router<std::sync::Arc<crate::AppState>> {
    Router::new()
        .route("/", get(list_products))
        .route("/:id", get(get_product))
}
