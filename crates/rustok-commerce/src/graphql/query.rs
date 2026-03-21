use async_graphql::{Context, Object, Result};
use rustok_api::graphql::{require_module_enabled, resolve_graphql_locale};
use rustok_core::Permission;
use rustok_telemetry::metrics;
use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::{
    entities::{product, product_translation},
    search::product_translation_title_search_condition,
    CatalogService, CommerceError,
};

use super::{require_commerce_permission, types::*, MODULE_SLUG};

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
        require_module_enabled(ctx, MODULE_SLUG).await?;
        require_commerce_permission(
            ctx,
            &[Permission::PRODUCTS_READ],
            "Permission denied: products:read required",
        )?;

        let db = ctx.data::<sea_orm::DatabaseConnection>()?;
        let event_bus = ctx.data::<rustok_outbox::TransactionalEventBus>()?;
        let locale = resolve_graphql_locale(ctx, locale.as_deref());

        let service = CatalogService::new(db.clone(), event_bus.clone());
        let product = match service.get_product(tenant_id, id).await {
            Ok(product) => product,
            Err(CommerceError::ProductNotFound(_)) => return Ok(None),
            Err(err) => return Err(err.to_string().into()),
        };

        let filtered_translations = product
            .translations
            .into_iter()
            .filter(|translation| translation.locale == locale)
            .collect::<Vec<_>>();

        Ok(Some(
            crate::dto::ProductResponse {
                translations: filtered_translations,
                ..product
            }
            .into(),
        ))
    }

    async fn products(
        &self,
        ctx: &Context<'_>,
        tenant_id: Uuid,
        locale: Option<String>,
        filter: Option<ProductsFilter>,
    ) -> Result<GqlProductList> {
        require_module_enabled(ctx, MODULE_SLUG).await?;
        require_commerce_permission(
            ctx,
            &[Permission::PRODUCTS_LIST],
            "Permission denied: products:list required",
        )?;

        let db = ctx.data::<sea_orm::DatabaseConnection>()?;
        let locale = resolve_graphql_locale(ctx, locale.as_deref());
        let filter = filter.unwrap_or(ProductsFilter {
            status: None,
            vendor: None,
            search: None,
            page: Some(1),
            per_page: Some(20),
        });
        let requested_limit = filter.per_page.map(|value| value.max(0) as u64);

        let page = filter.page.unwrap_or(1).max(1);
        let per_page = filter.per_page.unwrap_or(20).clamp(1, 100);
        let offset = (page.saturating_sub(1)) * per_page;

        let mut query = product::Entity::find().filter(product::Column::TenantId.eq(tenant_id));

        if let Some(status) = &filter.status {
            let status: crate::entities::product::ProductStatus = (*status).into();
            query = query.filter(product::Column::Status.eq(status));
        }
        if let Some(vendor) = &filter.vendor {
            query = query.filter(product::Column::Vendor.eq(vendor));
        }
        if let Some(search) = &filter.search {
            query = query.filter(product_translation_title_search_condition(
                db.get_database_backend(),
                &locale,
                search,
            ));
        }

        let total = query.clone().count(db).await?;
        let products = query
            .order_by_desc(product::Column::CreatedAt)
            .offset(offset)
            .limit(per_page)
            .all(db)
            .await?;

        let product_ids = products
            .iter()
            .map(|product| product.id)
            .collect::<Vec<_>>();
        let translations = if product_ids.is_empty() {
            Vec::new()
        } else {
            product_translation::Entity::find()
                .filter(product_translation::Column::ProductId.is_in(product_ids))
                .filter(product_translation::Column::Locale.eq(&locale))
                .all(db)
                .await?
        };

        let translation_map = translations
            .into_iter()
            .map(|translation| (translation.product_id, translation))
            .collect::<std::collections::HashMap<_, _>>();

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
            .collect::<Vec<_>>();

        metrics::record_read_path_budget(
            "graphql",
            "commerce.products",
            requested_limit,
            per_page,
            items.len(),
        );

        Ok(GqlProductList {
            items,
            total,
            page,
            per_page,
            has_next: page * per_page < total,
        })
    }
}
