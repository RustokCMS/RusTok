use async_graphql::{InputObject, SimpleObject};
use uuid::Uuid;

use rustok_commerce::dto;

#[derive(SimpleObject)]
pub struct GqlProduct {
    pub id: Uuid,
    pub status: String,
    pub vendor: Option<String>,
    pub product_type: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub published_at: Option<String>,
    pub translations: Vec<GqlProductTranslation>,
    pub options: Vec<GqlProductOption>,
    pub variants: Vec<GqlVariant>,
}

#[derive(SimpleObject)]
pub struct GqlProductTranslation {
    pub locale: String,
    pub title: String,
    pub handle: String,
    pub description: Option<String>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
}

#[derive(SimpleObject)]
pub struct GqlProductOption {
    pub id: Uuid,
    pub name: String,
    pub values: Vec<String>,
    pub position: i32,
}

#[derive(SimpleObject)]
pub struct GqlVariant {
    pub id: Uuid,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub title: String,
    pub option1: Option<String>,
    pub option2: Option<String>,
    pub option3: Option<String>,
    pub prices: Vec<GqlPrice>,
    pub inventory_quantity: i32,
    pub inventory_policy: String,
    pub in_stock: bool,
}

#[derive(SimpleObject)]
pub struct GqlPrice {
    pub currency_code: String,
    pub amount: String,
    pub compare_at_amount: Option<String>,
    pub on_sale: bool,
}

#[derive(SimpleObject)]
pub struct GqlProductList {
    pub items: Vec<GqlProductListItem>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub has_next: bool,
}

#[derive(SimpleObject)]
pub struct GqlProductListItem {
    pub id: Uuid,
    pub status: String,
    pub title: String,
    pub handle: String,
    pub vendor: Option<String>,
    pub created_at: String,
}

#[derive(InputObject)]
pub struct CreateProductInput {
    pub translations: Vec<ProductTranslationInput>,
    pub options: Option<Vec<ProductOptionInput>>,
    pub variants: Vec<CreateVariantInput>,
    pub vendor: Option<String>,
    pub product_type: Option<String>,
    pub publish: Option<bool>,
}

#[derive(InputObject)]
pub struct ProductTranslationInput {
    pub locale: String,
    pub title: String,
    pub handle: Option<String>,
    pub description: Option<String>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
}

#[derive(InputObject)]
pub struct ProductOptionInput {
    pub name: String,
    pub values: Vec<String>,
}

#[derive(InputObject)]
pub struct CreateVariantInput {
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub option1: Option<String>,
    pub option2: Option<String>,
    pub option3: Option<String>,
    pub prices: Vec<PriceInput>,
    pub inventory_quantity: Option<i32>,
    pub inventory_policy: Option<String>,
}

#[derive(InputObject)]
pub struct PriceInput {
    pub currency_code: String,
    pub amount: String,
    pub compare_at_amount: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateProductInput {
    pub translations: Option<Vec<ProductTranslationInput>>,
    pub vendor: Option<String>,
    pub product_type: Option<String>,
    pub status: Option<String>,
}

#[derive(InputObject)]
pub struct ProductsFilter {
    pub status: Option<String>,
    pub vendor: Option<String>,
    pub search: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

impl From<dto::ProductResponse> for GqlProduct {
    fn from(product: dto::ProductResponse) -> Self {
        Self {
            id: product.id,
            status: product.status,
            vendor: product.vendor,
            product_type: product.product_type,
            created_at: product.created_at.to_rfc3339(),
            updated_at: product.updated_at.to_rfc3339(),
            published_at: product.published_at.map(|value| value.to_rfc3339()),
            translations: product
                .translations
                .into_iter()
                .map(GqlProductTranslation::from)
                .collect(),
            options: product.options.into_iter().map(GqlProductOption::from).collect(),
            variants: product.variants.into_iter().map(GqlVariant::from).collect(),
        }
    }
}

impl From<dto::ProductTranslationResponse> for GqlProductTranslation {
    fn from(translation: dto::ProductTranslationResponse) -> Self {
        Self {
            locale: translation.locale,
            title: translation.title,
            handle: translation.handle,
            description: translation.description,
            meta_title: translation.meta_title,
            meta_description: translation.meta_description,
        }
    }
}

impl From<dto::ProductOptionResponse> for GqlProductOption {
    fn from(option: dto::ProductOptionResponse) -> Self {
        Self {
            id: option.id,
            name: option.name,
            values: option.values,
            position: option.position,
        }
    }
}

impl From<dto::VariantResponse> for GqlVariant {
    fn from(variant: dto::VariantResponse) -> Self {
        Self {
            id: variant.id,
            sku: variant.sku,
            barcode: variant.barcode,
            title: variant.title,
            option1: variant.option1,
            option2: variant.option2,
            option3: variant.option3,
            prices: variant.prices.into_iter().map(GqlPrice::from).collect(),
            inventory_quantity: variant.inventory_quantity,
            inventory_policy: variant.inventory_policy,
            in_stock: variant.in_stock,
        }
    }
}

impl From<dto::PriceResponse> for GqlPrice {
    fn from(price: dto::PriceResponse) -> Self {
        Self {
            currency_code: price.currency_code,
            amount: price.amount.to_string(),
            compare_at_amount: price.compare_at_amount.map(|value| value.to_string()),
            on_sale: price.on_sale,
        }
    }
}
