use std::collections::{HashMap, HashSet};

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DatabaseTransaction,
    EntityTrait, ModelTrait, PaginatorTrait, QueryFilter, QueryOrder,
};
use tracing::instrument;
use uuid::Uuid;

use rustok_content::PLATFORM_FALLBACK_LOCALE;
use rustok_core::SecurityContext;

use crate::dto::{CreateTagInput, ListTagsFilter, TagListItem, TagResponse, UpdateTagInput};
use crate::entities::{blog_post_tag, blog_tag, blog_tag_translation};
use crate::error::{BlogError, BlogResult};

pub struct TagService {
    db: DatabaseConnection,
}

impl TagService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    #[instrument(skip(self, _security, input))]
    pub async fn create_tag(
        &self,
        tenant_id: Uuid,
        _security: SecurityContext,
        input: CreateTagInput,
    ) -> BlogResult<Uuid> {
        validate_tag_name(&input.name)?;
        let locale = normalize_locale(&input.locale)?;
        let slug = normalize_tag_slug(input.slug.as_deref().unwrap_or(&input.name));
        let id = Uuid::new_v4();
        let now = Utc::now();

        blog_tag::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id),
            use_count: Set(0),
            created_at: Set(now.into()),
        }
        .insert(&self.db)
        .await?;

        blog_tag_translation::ActiveModel {
            id: Set(Uuid::new_v4()),
            tag_id: Set(id),
            tenant_id: Set(tenant_id),
            locale: Set(locale),
            name: Set(input.name),
            slug: Set(slug),
        }
        .insert(&self.db)
        .await?;

        Ok(id)
    }

    #[instrument(skip(self))]
    pub async fn get_tag(
        &self,
        tenant_id: Uuid,
        tag_id: Uuid,
        locale: &str,
    ) -> BlogResult<TagResponse> {
        let tag = blog_tag::Entity::find_by_id(tag_id)
            .filter(blog_tag::Column::TenantId.eq(tenant_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| BlogError::tag_not_found(tag_id))?;

        let translations = blog_tag_translation::Entity::find()
            .filter(blog_tag_translation::Column::TagId.eq(tag_id))
            .all(&self.db)
            .await?;

        Ok(to_tag_response(tag, translations, locale))
    }

    #[instrument(skip(self, _security, input))]
    pub async fn update_tag(
        &self,
        tenant_id: Uuid,
        tag_id: Uuid,
        _security: SecurityContext,
        input: UpdateTagInput,
    ) -> BlogResult<TagResponse> {
        let tag = blog_tag::Entity::find_by_id(tag_id)
            .filter(blog_tag::Column::TenantId.eq(tenant_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| BlogError::tag_not_found(tag_id))?;

        let locale = normalize_locale(&input.locale)?;
        let existing_translation = blog_tag_translation::Entity::find()
            .filter(blog_tag_translation::Column::TagId.eq(tag_id))
            .filter(blog_tag_translation::Column::Locale.eq(&locale))
            .one(&self.db)
            .await?;

        match existing_translation {
            Some(translation) => {
                let mut active: blog_tag_translation::ActiveModel = translation.into();
                if let Some(name) = &input.name {
                    validate_tag_name(name)?;
                    active.name = Set(name.to_string());
                    if input.slug.is_none() {
                        active.slug = Set(normalize_tag_slug(name));
                    }
                }
                if let Some(slug_value) = input.slug.as_deref() {
                    active.slug = Set(normalize_tag_slug(slug_value));
                }
                active.update(&self.db).await?;
            }
            None => {
                let name = input
                    .name
                    .ok_or_else(|| BlogError::validation("Tag name is required"))?;
                validate_tag_name(&name)?;
                let slug = input
                    .slug
                    .as_deref()
                    .map(normalize_tag_slug)
                    .unwrap_or_else(|| normalize_tag_slug(&name));
                blog_tag_translation::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    tag_id: Set(tag_id),
                    tenant_id: Set(tenant_id),
                    locale: Set(locale.clone()),
                    name: Set(name),
                    slug: Set(slug),
                }
                .insert(&self.db)
                .await?;
            }
        }

        let translations = blog_tag_translation::Entity::find()
            .filter(blog_tag_translation::Column::TagId.eq(tag_id))
            .all(&self.db)
            .await?;

        Ok(to_tag_response(tag, translations, &locale))
    }

    #[instrument(skip(self, _security))]
    pub async fn delete_tag(
        &self,
        tenant_id: Uuid,
        tag_id: Uuid,
        _security: SecurityContext,
    ) -> BlogResult<()> {
        let tag = blog_tag::Entity::find_by_id(tag_id)
            .filter(blog_tag::Column::TenantId.eq(tenant_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| BlogError::tag_not_found(tag_id))?;

        blog_post_tag::Entity::delete_many()
            .filter(blog_post_tag::Column::TagId.eq(tag_id))
            .exec(&self.db)
            .await?;

        blog_tag_translation::Entity::delete_many()
            .filter(blog_tag_translation::Column::TagId.eq(tag_id))
            .exec(&self.db)
            .await?;

        tag.delete(&self.db).await?;
        Ok(())
    }

    #[instrument(skip(self, _security))]
    pub async fn list_tags(
        &self,
        tenant_id: Uuid,
        _security: SecurityContext,
        filter: ListTagsFilter,
    ) -> BlogResult<(Vec<TagListItem>, u64)> {
        let locale = filter
            .locale
            .unwrap_or_else(|| PLATFORM_FALLBACK_LOCALE.to_string());
        let page = filter.page.max(1);

        let paginator = blog_tag::Entity::find()
            .filter(blog_tag::Column::TenantId.eq(tenant_id))
            .order_by_desc(blog_tag::Column::UseCount)
            .paginate(&self.db, filter.per_page.max(1));

        let total = paginator.num_items().await?;
        let tags = paginator.fetch_page(page - 1).await?;
        let tag_ids: Vec<Uuid> = tags.iter().map(|tag| tag.id).collect();
        let all_translations = if tag_ids.is_empty() {
            Vec::new()
        } else {
            blog_tag_translation::Entity::find()
                .filter(blog_tag_translation::Column::TagId.is_in(tag_ids))
                .all(&self.db)
                .await?
        };

        let items = tags
            .into_iter()
            .map(|tag| {
                let translations: Vec<&blog_tag_translation::Model> = all_translations
                    .iter()
                    .filter(|translation| translation.tag_id == tag.id)
                    .collect();
                let (translation, effective_locale) =
                    resolve_tag_translation(&translations, &locale);

                TagListItem {
                    id: tag.id,
                    locale: locale.clone(),
                    effective_locale,
                    name: translation
                        .map(|translation| translation.name.clone())
                        .unwrap_or_default(),
                    slug: translation
                        .map(|translation| translation.slug.clone())
                        .unwrap_or_default(),
                    use_count: tag.use_count,
                    created_at: tag.created_at.into(),
                }
            })
            .collect();

        Ok((items, total))
    }
}

pub(crate) async fn sync_post_tags_in_tx(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    post_id: Uuid,
    tag_names: &[String],
    locale: &str,
) -> BlogResult<()> {
    let normalized_locale = normalize_locale(locale)?;
    let normalized_names = normalize_tag_names(tag_names);
    let existing_relations = blog_post_tag::Entity::find()
        .filter(blog_post_tag::Column::PostId.eq(post_id))
        .all(txn)
        .await?;
    let existing_tag_ids: HashSet<Uuid> =
        existing_relations.iter().map(|item| item.tag_id).collect();

    blog_post_tag::Entity::delete_many()
        .filter(blog_post_tag::Column::PostId.eq(post_id))
        .exec(txn)
        .await?;

    let now = Utc::now();
    let mut touched_tag_ids = existing_tag_ids;
    for name in normalized_names {
        let tag_id =
            find_or_create_by_name_in_tx(txn, tenant_id, &normalized_locale, &name).await?;
        blog_post_tag::ActiveModel {
            post_id: Set(post_id),
            tag_id: Set(tag_id),
            created_at: Set(now.into()),
        }
        .insert(txn)
        .await?;
        touched_tag_ids.insert(tag_id);
    }

    for tag_id in touched_tag_ids {
        recount_tag_use_count_in_tx(txn, tag_id).await?;
    }

    Ok(())
}

pub(crate) async fn load_post_tags_map(
    db: &DatabaseConnection,
    post_ids: &[Uuid],
    locale: &str,
    fallback_locale: Option<&str>,
) -> BlogResult<HashMap<Uuid, Vec<String>>> {
    if post_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let relations = blog_post_tag::Entity::find()
        .filter(blog_post_tag::Column::PostId.is_in(post_ids.to_vec()))
        .order_by_asc(blog_post_tag::Column::CreatedAt)
        .all(db)
        .await?;

    if relations.is_empty() {
        return Ok(HashMap::new());
    }

    let tag_ids: Vec<Uuid> = relations.iter().map(|item| item.tag_id).collect();
    let translations = blog_tag_translation::Entity::find()
        .filter(blog_tag_translation::Column::TagId.is_in(tag_ids))
        .all(db)
        .await?;

    let mut translations_by_tag: HashMap<Uuid, Vec<blog_tag_translation::Model>> = HashMap::new();
    for translation in translations {
        translations_by_tag
            .entry(translation.tag_id)
            .or_default()
            .push(translation);
    }

    let mut tags_by_post: HashMap<Uuid, Vec<String>> = HashMap::new();
    for relation in relations {
        let translations = translations_by_tag
            .get(&relation.tag_id)
            .cloned()
            .unwrap_or_default();
        let refs: Vec<&blog_tag_translation::Model> = translations.iter().collect();
        let (translation, _) =
            resolve_tag_translation_with_fallback(&refs, locale, fallback_locale);
        if let Some(translation) = translation {
            tags_by_post
                .entry(relation.post_id)
                .or_default()
                .push(translation.name.clone());
        }
    }

    Ok(tags_by_post)
}

pub(crate) async fn find_post_ids_by_tag(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    tag: &str,
) -> BlogResult<Vec<Uuid>> {
    let normalized_slug = normalize_tag_slug(tag);
    if normalized_slug.is_empty() {
        return Ok(Vec::new());
    }

    let tag_translations = blog_tag_translation::Entity::find()
        .filter(blog_tag_translation::Column::TenantId.eq(tenant_id))
        .filter(blog_tag_translation::Column::Slug.eq(normalized_slug))
        .all(db)
        .await?;
    let tag_ids: Vec<Uuid> = tag_translations.iter().map(|item| item.tag_id).collect();
    if tag_ids.is_empty() {
        return Ok(Vec::new());
    }

    let relations = blog_post_tag::Entity::find()
        .filter(blog_post_tag::Column::TagId.is_in(tag_ids))
        .all(db)
        .await?;

    let mut seen = HashSet::new();
    Ok(relations
        .into_iter()
        .filter_map(|relation| {
            if seen.insert(relation.post_id) {
                Some(relation.post_id)
            } else {
                None
            }
        })
        .collect())
}

async fn find_or_create_by_name_in_tx(
    txn: &DatabaseTransaction,
    tenant_id: Uuid,
    locale: &str,
    name: &str,
) -> BlogResult<Uuid> {
    let slug = normalize_tag_slug(name);
    let existing = blog_tag_translation::Entity::find()
        .filter(blog_tag_translation::Column::TenantId.eq(tenant_id))
        .filter(blog_tag_translation::Column::Locale.eq(locale))
        .filter(blog_tag_translation::Column::Slug.eq(&slug))
        .one(txn)
        .await?;

    if let Some(existing) = existing {
        return Ok(existing.tag_id);
    }

    let id = Uuid::new_v4();
    let now = Utc::now();

    blog_tag::ActiveModel {
        id: Set(id),
        tenant_id: Set(tenant_id),
        use_count: Set(0),
        created_at: Set(now.into()),
    }
    .insert(txn)
    .await?;

    blog_tag_translation::ActiveModel {
        id: Set(Uuid::new_v4()),
        tag_id: Set(id),
        tenant_id: Set(tenant_id),
        locale: Set(locale.to_string()),
        name: Set(name.to_string()),
        slug: Set(slug),
    }
    .insert(txn)
    .await?;

    Ok(id)
}

async fn recount_tag_use_count_in_tx(txn: &DatabaseTransaction, tag_id: Uuid) -> BlogResult<()> {
    let count = blog_post_tag::Entity::find()
        .filter(blog_post_tag::Column::TagId.eq(tag_id))
        .count(txn)
        .await? as i32;

    let Some(tag) = blog_tag::Entity::find_by_id(tag_id).one(txn).await? else {
        return Ok(());
    };

    let mut active: blog_tag::ActiveModel = tag.into();
    active.use_count = Set(count);
    active.update(txn).await?;
    Ok(())
}

fn validate_tag_name(name: &str) -> BlogResult<()> {
    if name.trim().is_empty() {
        return Err(BlogError::validation("Tag name cannot be empty"));
    }
    if name.len() > 100 {
        return Err(BlogError::validation(
            "Tag name cannot exceed 100 characters",
        ));
    }
    Ok(())
}

fn normalize_tag_names(tag_names: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();
    for name in tag_names {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            continue;
        }
        let key = trimmed.to_ascii_lowercase();
        if seen.insert(key) {
            normalized.push(trimmed.to_string());
        }
    }
    normalized
}

fn normalize_locale(locale: &str) -> BlogResult<String> {
    let normalized = locale.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Err(BlogError::validation("Locale cannot be empty"));
    }
    Ok(normalized)
}

fn normalize_tag_slug(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());
    let mut previous_dash = false;
    for ch in value.chars().flat_map(|ch| ch.to_lowercase()) {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch);
            previous_dash = false;
        } else if !previous_dash {
            normalized.push('-');
            previous_dash = true;
        }
    }
    normalized.trim_matches('-').to_string()
}

fn resolve_tag_translation<'a>(
    translations: &[&'a blog_tag_translation::Model],
    locale: &str,
) -> (Option<&'a blog_tag_translation::Model>, String) {
    resolve_tag_translation_with_fallback(translations, locale, None)
}

fn resolve_tag_translation_with_fallback<'a>(
    translations: &[&'a blog_tag_translation::Model],
    locale: &str,
    fallback_locale: Option<&str>,
) -> (Option<&'a blog_tag_translation::Model>, String) {
    if let Some(translation) = translations
        .iter()
        .copied()
        .find(|translation| translation.locale == locale)
    {
        return (Some(translation), locale.to_string());
    }
    if let Some(fallback_locale) = fallback_locale {
        if let Some(translation) = translations
            .iter()
            .copied()
            .find(|translation| translation.locale == fallback_locale)
        {
            return (Some(translation), fallback_locale.to_string());
        }
    }
    if let Some(translation) = translations
        .iter()
        .copied()
        .find(|translation| translation.locale == PLATFORM_FALLBACK_LOCALE)
    {
        return (Some(translation), PLATFORM_FALLBACK_LOCALE.to_string());
    }
    if let Some(translation) = translations.first().copied() {
        return (Some(translation), translation.locale.clone());
    }
    (None, locale.to_string())
}

fn to_tag_response(
    tag: blog_tag::Model,
    translations: Vec<blog_tag_translation::Model>,
    locale: &str,
) -> TagResponse {
    let translations_refs: Vec<&blog_tag_translation::Model> = translations.iter().collect();
    let (translation, effective_locale) = resolve_tag_translation(&translations_refs, locale);

    TagResponse {
        id: tag.id,
        tenant_id: tag.tenant_id,
        locale: locale.to_string(),
        effective_locale,
        name: translation
            .map(|translation| translation.name.clone())
            .unwrap_or_default(),
        slug: translation
            .map(|translation| translation.slug.clone())
            .unwrap_or_default(),
        use_count: tag.use_count,
        created_at: tag.created_at.into(),
    }
}
