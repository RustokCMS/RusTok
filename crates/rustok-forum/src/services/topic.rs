use std::collections::HashMap;

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DatabaseTransaction,
    EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, TransactionTrait,
};
use tracing::instrument;
use uuid::Uuid;

use rustok_content::{
    available_locales_from, normalize_locale_code, resolve_by_locale_with_fallback,
    PLATFORM_FALLBACK_LOCALE,
};
use rustok_core::{prepare_content_payload, SecurityContext};
use rustok_events::DomainEvent;
use rustok_outbox::TransactionalEventBus;

use crate::constants::topic_status;
use crate::dto::{
    CreateTopicInput, ListTopicsFilter, TopicListItem, TopicResponse, UpdateTopicInput,
};
use crate::entities::{forum_topic, forum_topic_channel_access, forum_topic_translation};
use crate::error::{ForumError, ForumResult};
use crate::services::category::CategoryService;

pub struct TopicService {
    db: DatabaseConnection,
    event_bus: TransactionalEventBus,
}

impl TopicService {
    pub fn new(db: DatabaseConnection, event_bus: TransactionalEventBus) -> Self {
        Self { db, event_bus }
    }

    #[instrument(skip(self, security, input))]
    pub async fn create(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        input: CreateTopicInput,
    ) -> ForumResult<TopicResponse> {
        validate_topic_title(&input.title)?;
        let locale = normalize_locale(&input.locale)?;
        let prepared_body = prepare_content_payload(
            Some(&input.body_format),
            Some(&input.body),
            input.content_json.as_ref(),
            &locale,
            "Topic body",
        )
        .map_err(ForumError::Validation)?;

        let txn = self.db.begin().await?;
        CategoryService::ensure_exists_in_tx(&txn, tenant_id, input.category_id).await?;

        let now = Utc::now();
        let topic_id = Uuid::new_v4();
        forum_topic::ActiveModel {
            id: Set(topic_id),
            tenant_id: Set(tenant_id),
            category_id: Set(input.category_id),
            author_id: Set(security.user_id),
            status: Set(topic_status::OPEN.to_string()),
            is_pinned: Set(false),
            is_locked: Set(false),
            tags: Set(serde_json::json!(normalize_tags(&input.tags))),
            reply_count: Set(0),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
            last_reply_at: Set(None),
        }
        .insert(&txn)
        .await?;

        forum_topic_translation::ActiveModel {
            id: Set(Uuid::new_v4()),
            topic_id: Set(topic_id),
            locale: Set(locale.clone()),
            title: Set(input.title),
            slug: Set(input
                .slug
                .map(|value| normalize_slug(&value))
                .filter(|value| !value.is_empty())),
            body: Set(prepared_body.body),
            body_format: Set(prepared_body.format),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        }
        .insert(&txn)
        .await?;

        self.sync_channel_access_in_tx(&txn, topic_id, input.channel_slugs.as_deref())
            .await?;
        CategoryService::adjust_counters_in_tx(&txn, tenant_id, input.category_id, 1, 0).await?;

        self.event_bus
            .publish_in_tx(
                &txn,
                tenant_id,
                security.user_id,
                DomainEvent::ForumTopicCreated {
                    topic_id,
                    category_id: input.category_id,
                    author_id: security.user_id,
                    locale: locale.clone(),
                },
            )
            .await?;

        txn.commit().await?;
        self.get(tenant_id, topic_id, &locale).await
    }

    #[instrument(skip(self))]
    pub async fn get(
        &self,
        tenant_id: Uuid,
        topic_id: Uuid,
        locale: &str,
    ) -> ForumResult<TopicResponse> {
        self.get_with_locale_fallback(tenant_id, topic_id, locale, None)
            .await
    }

    #[instrument(skip(self))]
    pub async fn get_with_locale_fallback(
        &self,
        tenant_id: Uuid,
        topic_id: Uuid,
        locale: &str,
        fallback_locale: Option<&str>,
    ) -> ForumResult<TopicResponse> {
        let locale = normalize_locale(locale)?;
        let fallback_locale = fallback_locale.map(normalize_locale).transpose()?;
        let topic = self.find_topic(tenant_id, topic_id).await?;
        let translations = self.load_translations(topic_id).await?;
        let channel_slugs = self.load_channel_slugs(topic_id).await?;
        Ok(to_topic_response(
            topic,
            translations,
            channel_slugs,
            &locale,
            fallback_locale.as_deref(),
        ))
    }

    #[instrument(skip(self, _security, input))]
    pub async fn update(
        &self,
        tenant_id: Uuid,
        topic_id: Uuid,
        _security: SecurityContext,
        input: UpdateTopicInput,
    ) -> ForumResult<TopicResponse> {
        let locale = normalize_locale(&input.locale)?;
        let topic = self.find_topic(tenant_id, topic_id).await?;
        let txn = self.db.begin().await?;

        let mut active: forum_topic::ActiveModel = topic.into();
        if let Some(tags) = input.tags {
            active.tags = Set(serde_json::json!(normalize_tags(&tags)));
        }
        active.updated_at = Set(Utc::now().into());
        active.update(&txn).await?;

        self.upsert_translation_in_tx(
            &txn,
            topic_id,
            &locale,
            input.title,
            input.body,
            input.body_format,
            input.content_json,
        )
        .await?;

        if input.channel_slugs.is_some() {
            self.sync_channel_access_in_tx(&txn, topic_id, input.channel_slugs.as_deref())
                .await?;
        }

        txn.commit().await?;
        self.get(tenant_id, topic_id, &locale).await
    }

    #[instrument(skip(self, _security))]
    pub async fn delete(
        &self,
        tenant_id: Uuid,
        topic_id: Uuid,
        _security: SecurityContext,
    ) -> ForumResult<()> {
        let topic = self.find_topic(tenant_id, topic_id).await?;
        let txn = self.db.begin().await?;
        forum_topic::Entity::delete_by_id(topic_id)
            .exec(&txn)
            .await?;
        CategoryService::adjust_counters_in_tx(
            &txn,
            tenant_id,
            topic.category_id,
            -1,
            -topic.reply_count,
        )
        .await?;
        txn.commit().await?;
        Ok(())
    }

    #[instrument(skip(self, _security))]
    pub async fn list(
        &self,
        tenant_id: Uuid,
        _security: SecurityContext,
        filter: ListTopicsFilter,
    ) -> ForumResult<(Vec<TopicListItem>, u64)> {
        self.list_with_locale_fallback(tenant_id, SecurityContext::system(), filter, None)
            .await
    }

    #[instrument(skip(self, _security))]
    pub async fn list_with_locale_fallback(
        &self,
        tenant_id: Uuid,
        _security: SecurityContext,
        filter: ListTopicsFilter,
        fallback_locale: Option<&str>,
    ) -> ForumResult<(Vec<TopicListItem>, u64)> {
        let locale = filter
            .locale
            .clone()
            .unwrap_or_else(|| PLATFORM_FALLBACK_LOCALE.to_string());
        let locale = normalize_locale(&locale)?;
        let fallback_locale = fallback_locale.map(normalize_locale).transpose()?;

        let mut select =
            forum_topic::Entity::find().filter(forum_topic::Column::TenantId.eq(tenant_id));
        if let Some(category_id) = filter.category_id {
            select = select.filter(forum_topic::Column::CategoryId.eq(category_id));
        }
        if let Some(status) = filter.status {
            select = select.filter(forum_topic::Column::Status.eq(status));
        }

        let paginator = select
            .order_by_desc(forum_topic::Column::IsPinned)
            .order_by_desc(forum_topic::Column::LastReplyAt)
            .order_by_desc(forum_topic::Column::UpdatedAt)
            .paginate(&self.db, filter.per_page.max(1));
        let total = paginator.num_items().await?;
        let topics = paginator.fetch_page(filter.page.saturating_sub(1)).await?;
        let topic_ids: Vec<Uuid> = topics.iter().map(|topic| topic.id).collect();
        let translations = self.load_translations_for_topics(&topic_ids).await?;
        let channels = self.load_channel_slugs_map(&topic_ids).await?;

        let items = topics
            .into_iter()
            .map(|topic| {
                let localized = translations
                    .iter()
                    .filter(|translation| translation.topic_id == topic.id)
                    .collect::<Vec<_>>();
                let resolved = resolve_by_locale_with_fallback(
                    &localized,
                    &locale,
                    fallback_locale.as_deref(),
                    |translation| translation.locale.as_str(),
                );

                TopicListItem {
                    id: topic.id,
                    locale: locale.clone(),
                    effective_locale: resolved.effective_locale,
                    category_id: topic.category_id,
                    author_id: topic.author_id,
                    title: resolved
                        .item
                        .map(|translation| translation.title.clone())
                        .unwrap_or_default(),
                    slug: resolved
                        .item
                        .and_then(|translation| translation.slug.clone())
                        .unwrap_or_default(),
                    status: topic.status.clone(),
                    channel_slugs: channels.get(&topic.id).cloned().unwrap_or_default(),
                    is_pinned: topic.is_pinned,
                    is_locked: topic.is_locked,
                    reply_count: topic.reply_count,
                    created_at: topic.created_at.to_rfc3339(),
                }
            })
            .collect();

        Ok((items, total))
    }

    pub(crate) async fn find_topic(
        &self,
        tenant_id: Uuid,
        topic_id: Uuid,
    ) -> ForumResult<forum_topic::Model> {
        forum_topic::Entity::find_by_id(topic_id)
            .filter(forum_topic::Column::TenantId.eq(tenant_id))
            .one(&self.db)
            .await?
            .ok_or(ForumError::TopicNotFound(topic_id))
    }

    pub(crate) async fn find_topic_in_tx(
        txn: &DatabaseTransaction,
        tenant_id: Uuid,
        topic_id: Uuid,
    ) -> ForumResult<forum_topic::Model> {
        forum_topic::Entity::find_by_id(topic_id)
            .filter(forum_topic::Column::TenantId.eq(tenant_id))
            .one(txn)
            .await?
            .ok_or(ForumError::TopicNotFound(topic_id))
    }

    pub(crate) async fn adjust_reply_count_in_tx(
        txn: &DatabaseTransaction,
        tenant_id: Uuid,
        topic_id: Uuid,
        delta: i32,
    ) -> ForumResult<forum_topic::Model> {
        let topic = Self::find_topic_in_tx(txn, tenant_id, topic_id).await?;
        let mut active: forum_topic::ActiveModel = topic.clone().into();
        active.reply_count = Set((topic.reply_count + delta).max(0));
        active.last_reply_at = Set(Some(Utc::now().into()));
        active.updated_at = Set(Utc::now().into());
        active.update(txn).await?;
        Ok(topic)
    }

    pub(crate) async fn set_pinned_in_tx(
        txn: &DatabaseTransaction,
        tenant_id: Uuid,
        topic_id: Uuid,
        is_pinned: bool,
    ) -> ForumResult<()> {
        let topic = Self::find_topic_in_tx(txn, tenant_id, topic_id).await?;
        let mut active: forum_topic::ActiveModel = topic.into();
        active.is_pinned = Set(is_pinned);
        active.updated_at = Set(Utc::now().into());
        active.update(txn).await?;
        Ok(())
    }

    pub(crate) async fn set_locked_in_tx(
        txn: &DatabaseTransaction,
        tenant_id: Uuid,
        topic_id: Uuid,
        is_locked: bool,
    ) -> ForumResult<()> {
        let topic = Self::find_topic_in_tx(txn, tenant_id, topic_id).await?;
        let mut active: forum_topic::ActiveModel = topic.into();
        active.is_locked = Set(is_locked);
        active.updated_at = Set(Utc::now().into());
        active.update(txn).await?;
        Ok(())
    }

    pub(crate) async fn set_status_in_tx(
        txn: &DatabaseTransaction,
        tenant_id: Uuid,
        topic_id: Uuid,
        status: &str,
    ) -> ForumResult<()> {
        let topic = Self::find_topic_in_tx(txn, tenant_id, topic_id).await?;
        let mut active: forum_topic::ActiveModel = topic.into();
        active.status = Set(status.to_string());
        active.updated_at = Set(Utc::now().into());
        active.update(txn).await?;
        Ok(())
    }

    async fn load_translations(
        &self,
        topic_id: Uuid,
    ) -> ForumResult<Vec<forum_topic_translation::Model>> {
        Ok(forum_topic_translation::Entity::find()
            .filter(forum_topic_translation::Column::TopicId.eq(topic_id))
            .all(&self.db)
            .await?)
    }

    async fn load_translations_for_topics(
        &self,
        topic_ids: &[Uuid],
    ) -> ForumResult<Vec<forum_topic_translation::Model>> {
        if topic_ids.is_empty() {
            return Ok(Vec::new());
        }
        Ok(forum_topic_translation::Entity::find()
            .filter(forum_topic_translation::Column::TopicId.is_in(topic_ids.to_vec()))
            .all(&self.db)
            .await?)
    }

    async fn load_channel_slugs(&self, topic_id: Uuid) -> ForumResult<Vec<String>> {
        Ok(forum_topic_channel_access::Entity::find()
            .filter(forum_topic_channel_access::Column::TopicId.eq(topic_id))
            .order_by_asc(forum_topic_channel_access::Column::ChannelSlug)
            .all(&self.db)
            .await?
            .into_iter()
            .map(|item| item.channel_slug)
            .collect())
    }

    async fn load_channel_slugs_map(
        &self,
        topic_ids: &[Uuid],
    ) -> ForumResult<HashMap<Uuid, Vec<String>>> {
        if topic_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let rows = forum_topic_channel_access::Entity::find()
            .filter(forum_topic_channel_access::Column::TopicId.is_in(topic_ids.to_vec()))
            .all(&self.db)
            .await?;
        let mut map: HashMap<Uuid, Vec<String>> = HashMap::new();
        for row in rows {
            map.entry(row.topic_id).or_default().push(row.channel_slug);
        }
        for values in map.values_mut() {
            values.sort();
        }
        Ok(map)
    }

    async fn sync_channel_access_in_tx(
        &self,
        txn: &DatabaseTransaction,
        topic_id: Uuid,
        channel_slugs: Option<&[String]>,
    ) -> ForumResult<()> {
        forum_topic_channel_access::Entity::delete_many()
            .filter(forum_topic_channel_access::Column::TopicId.eq(topic_id))
            .exec(txn)
            .await?;

        for channel_slug in normalize_channel_slugs(channel_slugs.unwrap_or(&[])) {
            forum_topic_channel_access::ActiveModel {
                topic_id: Set(topic_id),
                channel_slug: Set(channel_slug),
            }
            .insert(txn)
            .await?;
        }

        Ok(())
    }

    async fn upsert_translation_in_tx(
        &self,
        txn: &DatabaseTransaction,
        topic_id: Uuid,
        locale: &str,
        title: Option<String>,
        body: Option<String>,
        body_format: Option<String>,
        content_json: Option<serde_json::Value>,
    ) -> ForumResult<()> {
        let existing = forum_topic_translation::Entity::find()
            .filter(forum_topic_translation::Column::TopicId.eq(topic_id))
            .filter(forum_topic_translation::Column::Locale.eq(locale))
            .one(txn)
            .await?;
        let now = Utc::now();

        match existing {
            Some(existing) => {
                let mut active: forum_topic_translation::ActiveModel = existing.into();
                if let Some(title) = title {
                    validate_topic_title(&title)?;
                    active.title = Set(title);
                }
                if body.is_some() || content_json.is_some() || body_format.is_some() {
                    let prepared_body = prepare_content_payload(
                        body_format.as_deref(),
                        body.as_deref(),
                        content_json.as_ref(),
                        locale,
                        "Topic body",
                    )
                    .map_err(ForumError::Validation)?;
                    active.body = Set(prepared_body.body);
                    active.body_format = Set(prepared_body.format);
                }
                active.updated_at = Set(now.into());
                active.update(txn).await?;
            }
            None => {
                let seed = forum_topic_translation::Entity::find()
                    .filter(forum_topic_translation::Column::TopicId.eq(topic_id))
                    .order_by_asc(forum_topic_translation::Column::CreatedAt)
                    .one(txn)
                    .await?;
                let title = title
                    .or_else(|| seed.as_ref().map(|translation| translation.title.clone()))
                    .ok_or_else(|| {
                        ForumError::Validation("Title is required for a new locale".to_string())
                    })?;
                validate_topic_title(&title)?;
                let prepared_body =
                    if body.is_some() || content_json.is_some() || body_format.is_some() {
                        prepare_content_payload(
                            body_format.as_deref(),
                            body.as_deref(),
                            content_json.as_ref(),
                            locale,
                            "Topic body",
                        )
                        .map_err(ForumError::Validation)?
                    } else if let Some(seed) = seed.as_ref() {
                        rustok_core::PreparedContent {
                            body: seed.body.clone(),
                            format: seed.body_format.clone(),
                        }
                    } else {
                        return Err(ForumError::Validation(
                            "Body is required for a new locale".to_string(),
                        ));
                    };

                forum_topic_translation::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    topic_id: Set(topic_id),
                    locale: Set(locale.to_string()),
                    title: Set(title),
                    slug: Set(seed.and_then(|translation| translation.slug)),
                    body: Set(prepared_body.body),
                    body_format: Set(prepared_body.format),
                    created_at: Set(now.into()),
                    updated_at: Set(now.into()),
                }
                .insert(txn)
                .await?;
            }
        }

        Ok(())
    }
}

fn to_topic_response(
    topic: forum_topic::Model,
    translations: Vec<forum_topic_translation::Model>,
    channel_slugs: Vec<String>,
    locale: &str,
    fallback_locale: Option<&str>,
) -> TopicResponse {
    let resolved =
        resolve_by_locale_with_fallback(&translations, locale, fallback_locale, |translation| {
            translation.locale.as_str()
        });
    let body = resolved
        .item
        .map(|translation| translation.body.clone())
        .unwrap_or_default();
    let body_format = resolved
        .item
        .map(|translation| translation.body_format.clone())
        .unwrap_or_else(|| "markdown".to_string());
    let content_json = if body_format == "rt_json_v1" {
        serde_json::from_str(&body).ok()
    } else {
        None
    };

    TopicResponse {
        id: topic.id,
        requested_locale: locale.to_string(),
        locale: locale.to_string(),
        effective_locale: resolved.effective_locale,
        available_locales: available_locales_from(&translations, |translation| {
            translation.locale.as_str()
        }),
        category_id: topic.category_id,
        author_id: topic.author_id,
        title: resolved
            .item
            .map(|translation| translation.title.clone())
            .unwrap_or_default(),
        slug: resolved
            .item
            .and_then(|translation| translation.slug.clone())
            .unwrap_or_default(),
        body,
        body_format,
        content_json,
        status: topic.status,
        tags: extract_tags(&topic.tags),
        channel_slugs,
        is_pinned: topic.is_pinned,
        is_locked: topic.is_locked,
        reply_count: topic.reply_count,
        created_at: topic.created_at.to_rfc3339(),
        updated_at: topic.updated_at.to_rfc3339(),
    }
}

fn validate_topic_title(title: &str) -> ForumResult<()> {
    if title.trim().is_empty() {
        return Err(ForumError::Validation(
            "Topic title cannot be empty".to_string(),
        ));
    }
    Ok(())
}

fn normalize_locale(locale: &str) -> ForumResult<String> {
    normalize_locale_code(locale)
        .ok_or_else(|| ForumError::Validation("Invalid locale".to_string()))
}

fn normalize_slug(value: &str) -> String {
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

fn normalize_tags(tags: &[String]) -> Vec<String> {
    let mut normalized = tags
        .iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}

fn normalize_channel_slugs(channel_slugs: &[String]) -> Vec<String> {
    let mut normalized = channel_slugs
        .iter()
        .map(|item| item.trim().to_ascii_lowercase())
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}

fn extract_tags(tags: &sea_orm::prelude::Json) -> Vec<String> {
    tags.as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToOwned::to_owned))
                .collect()
        })
        .unwrap_or_default()
}
