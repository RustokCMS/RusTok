use async_graphql::{Context, Object, Result};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};
use uuid::Uuid;

use rustok_commerce::CatalogService;
use rustok_core::{EventBus, TransactionalEventBus};

use super::types::*;

#[derive(Default)]
pub struct CommerceQuery;

#[Object]
impl CommerceQuery {
    async fn product(
        &self,
        ctx: &Context<'_>,
        tenant_id: Uuid,
        id: Uuid,
        locale: Option<String>,
    ) -> Result<Option<GqlProduct>> {
        let db = ctx.data::<sea_orm::DatabaseConnection>()?;
        let event_bus = ctx.data::<TransactionalEventBus>()?;
        let locale = locale.unwrap_or_else(|| "en".to_string());

        let service = CatalogService::new(db.clone(), event_bus.clone());
        let product = match service.get_product(tenant_id, id).await {
            Ok(product) => product,
            Err(rustok_commerce::CommerceError::ProductNotFound(_)) => return Ok(None),
            Err(err) => return Err(err.to_string().into()),
        };

        let filtered_translations = product
            .translations
            .into_iter()
            .filter(|translation| translation.locale == locale)
            .collect::<Vec<_>>();

        let product = rustok_commerce::dto::ProductResponse {
            translations: filtered_translations,
            ..product
        };

        Ok(Some(product.into()))
    }

    async fn products(
        &self,
        ctx: &Context<'_>,
        tenant_id: Uuid,
        locale: Option<String>,
        filter: Option<ProductsFilter>,
    ) -> Result<GqlProductList> {
        let db = ctx.data::<sea_orm::DatabaseConnection>()?;
        let locale = locale.unwrap_or_else(|| "en".to_string());
        let filter = filter.unwrap_or(ProductsFilter {
            status: None,
            vendor: None,
            search: None,
            page: Some(1),
            per_page: Some(20),
        });

        use rustok_commerce::entities::{product, product_translation};

        let page = filter.page.unwrap_or(1);
        let per_page = filter.per_page.unwrap_or(20).min(100);
        let offset = (page.saturating_sub(1)) * per_page;

        let mut query = product::Entity::find().filter(product::Column::TenantId.eq(tenant_id));

        if let Some(status) = &filter.status {
            let status: rustok_commerce::entities::product::ProductStatus = (*status).into();
            query = query.filter(product::Column::Status.eq(status));
        }
        if let Some(vendor) = &filter.vendor {
            query = query.filter(product::Column::Vendor.eq(vendor));
        }

        if let Some(search) = &filter.search {
            let search_ids: Vec<Uuid> = product_translation::Entity::find()
                .filter(product_translation::Column::Locale.eq(&locale))
                .filter(product_translation::Column::Title.contains(search))
                .all(db)
                .await?
                .into_iter()
                .map(|translation| translation.product_id)
                .collect();

            if search_ids.is_empty() {
                return Ok(GqlProductList {
                    items: Vec::new(),
                    total: 0,
                    page,
                    per_page,
                    has_next: false,
                });
            }

            query = query.filter(product::Column::Id.is_in(search_ids));
        }

        let total = query.clone().count(db).await?;
        let products = query
            .order_by_desc(product::Column::CreatedAt)
            .offset(offset)
            .limit(per_page)
            .all(db)
            .await?;

        let product_ids: Vec<Uuid> = products.iter().map(|product| product.id).collect();
        let translations = product_translation::Entity::find()
            .filter(product_translation::Column::ProductId.is_in(product_ids))
            .filter(product_translation::Column::Locale.eq(&locale))
            .all(db)
            .await?;

        let translation_map: std::collections::HashMap<Uuid, _> = translations
            .into_iter()
            .map(|translation| (translation.product_id, translation))
            .collect();

        let items = products
            .into_iter()
            .map(|product| {
                let translation = translation_map.get(&product.id);
                GqlProductListItem {
                    id: product.id,
                    status: product.status.into(),
                    title: translation
                        .map(|value| value.title.clone())
                        .unwrap_or_default(),
                    handle: translation
                        .map(|value| value.handle.clone())
                        .unwrap_or_default(),
                    vendor: product.vendor,
                    created_at: product.created_at.to_rfc3339(),
                }
            })
            .collect();

        Ok(GqlProductList {
            items,
            total,
            page,
            per_page,
            has_next: page * per_page < total,
        })
    }
}
