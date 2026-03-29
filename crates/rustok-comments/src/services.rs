use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
};
use std::collections::HashMap;
use tracing::instrument;
use uuid::Uuid;

use rustok_content::{
    dto::validation::validate_body_format, normalize_locale_code, resolve_by_locale_with_fallback,
};
use rustok_core::{Action, PermissionScope, Resource, SecurityContext};

use crate::dto::{
    CommentListItem, CommentRecord, CreateCommentInput, ListCommentsFilter, UpdateCommentInput,
};
use crate::entities::{comment, comment_body, comment_thread};
use crate::error::{CommentsError, CommentsResult};

pub struct CommentsService {
    db: DatabaseConnection,
}

#[cfg(test)]
mod locale_fallback_tests {
    use super::*;

    #[test]
    fn resolve_body_uses_platform_fallback_before_first_available() {
        let now = Utc::now().into();
        let resolved = resolve_body(
            vec![
                comment_body::Model {
                    id: Uuid::new_v4(),
                    comment_id: Uuid::new_v4(),
                    locale: "de".to_string(),
                    body: "Hallo".to_string(),
                    body_format: "markdown".to_string(),
                    created_at: now,
                    updated_at: now,
                },
                comment_body::Model {
                    id: Uuid::new_v4(),
                    comment_id: Uuid::new_v4(),
                    locale: "en".to_string(),
                    body: "Hello".to_string(),
                    body_format: "markdown".to_string(),
                    created_at: now,
                    updated_at: now,
                },
            ],
            "ru",
            None,
        )
        .expect("body should resolve");

        assert_eq!(resolved.effective_locale, "en");
        assert_eq!(resolved.body, "Hello");
    }

    #[test]
    fn resolve_body_normalizes_explicit_fallback_locale() {
        let now = Utc::now().into();
        let resolved = resolve_body(
            vec![
                comment_body::Model {
                    id: Uuid::new_v4(),
                    comment_id: Uuid::new_v4(),
                    locale: "en-us".to_string(),
                    body: "Hello".to_string(),
                    body_format: "markdown".to_string(),
                    created_at: now,
                    updated_at: now,
                },
                comment_body::Model {
                    id: Uuid::new_v4(),
                    comment_id: Uuid::new_v4(),
                    locale: "de".to_string(),
                    body: "Hallo".to_string(),
                    body_format: "markdown".to_string(),
                    created_at: now,
                    updated_at: now,
                },
            ],
            "fr-fr",
            Some("EN_us"),
        )
        .expect("body should resolve");

        assert_eq!(resolved.effective_locale, "en-us");
        assert_eq!(resolved.body, "Hello");
    }
}

#[cfg(test)]
use rustok_core::CONTENT_FORMAT_MARKDOWN;

#[cfg(test)]
use sea_orm::Database;

impl CommentsService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    #[instrument(skip(self, security, input), fields(tenant_id = %tenant_id, target_type = %input.target_type, target_id = %input.target_id))]
    pub async fn create_comment(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        input: CreateCommentInput,
    ) -> CommentsResult<CommentRecord> {
        let locale = input.locale.clone();
        let txn = self.db.begin().await?;
        let comment_id = self
            .create_comment_in_tx(&txn, tenant_id, security.clone(), input)
            .await?;
        txn.commit().await?;
        self.get_comment(tenant_id, security, comment_id, &locale, None)
            .await
    }

    pub async fn create_comment_in_tx(
        &self,
        txn: &DatabaseTransaction,
        tenant_id: Uuid,
        security: SecurityContext,
        input: CreateCommentInput,
    ) -> CommentsResult<Uuid> {
        let author_id = self.enforce_create_scope(&security)?;
        self.validate_body(&input.body, &input.body_format)?;

        let thread = self
            .find_or_create_thread_in_tx(txn, tenant_id, &input.target_type, input.target_id)
            .await?;

        if let Some(parent_comment_id) = input.parent_comment_id {
            let parent = self
                .find_comment_in_tx(txn, tenant_id, parent_comment_id, true)
                .await?;
            if parent.thread_id != thread.id {
                return Err(CommentsError::Validation(
                    "Parent comment belongs to another thread".to_string(),
                ));
            }
        }

        let now = Utc::now();
        let position = self.next_position_in_tx(txn, thread.id).await?;
        let comment_id = Uuid::new_v4();
        let locale = normalize_locale(&input.locale)?;

        comment::ActiveModel {
            id: Set(comment_id),
            tenant_id: Set(tenant_id),
            thread_id: Set(thread.id),
            author_id: Set(author_id),
            parent_comment_id: Set(input.parent_comment_id),
            status: Set(input.status),
            position: Set(position),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
            deleted_at: Set(None),
        }
        .insert(txn)
        .await?;

        comment_body::ActiveModel {
            id: Set(Uuid::new_v4()),
            comment_id: Set(comment_id),
            locale: Set(locale),
            body: Set(input.body),
            body_format: Set(input.body_format),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        }
        .insert(txn)
        .await?;

        self.update_thread_counters_in_tx(txn, &thread, 1, Some(now.into()))
            .await?;

        Ok(comment_id)
    }

    #[instrument(skip(self, security))]
    pub async fn get_comment(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        comment_id: Uuid,
        locale: &str,
        fallback_locale: Option<&str>,
    ) -> CommentsResult<CommentRecord> {
        self.enforce_read_scope(&security, Action::Read)?;
        let locale = normalize_locale(locale)?;
        let fallback_locale = fallback_locale.map(normalize_locale).transpose()?;

        let comment = self.find_comment(tenant_id, comment_id, false).await?;
        let thread = comment_thread::Entity::find_by_id(comment.thread_id)
            .filter(comment_thread::Column::TenantId.eq(tenant_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| CommentsError::CommentThreadNotFound {
                target_type: "unknown".to_string(),
                target_id: Uuid::nil(),
            })?;
        let bodies = comment_body::Entity::find()
            .filter(comment_body::Column::CommentId.eq(comment.id))
            .all(&self.db)
            .await?;

        self.build_comment_record(comment, thread, bodies, &locale, fallback_locale.as_deref())
    }

    #[instrument(skip(self, security, input))]
    pub async fn update_comment(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        comment_id: Uuid,
        input: UpdateCommentInput,
    ) -> CommentsResult<CommentRecord> {
        let existing = self.find_comment(tenant_id, comment_id, false).await?;
        self.enforce_owned_scope(&security, Action::Update, existing.author_id)?;

        let locale = normalize_locale(&input.locale)?;
        if input.body.is_none() && input.body_format.is_none() {
            return self
                .get_comment(tenant_id, security, comment_id, &locale, None)
                .await;
        }

        let body = input
            .body
            .ok_or_else(|| CommentsError::Validation("Comment body is required".to_string()))?;
        let body_format = input.body_format.unwrap_or_else(|| "markdown".to_string());
        self.validate_body(&body, &body_format)?;

        let txn = self.db.begin().await?;
        self.upsert_body_in_tx(&txn, comment_id, &locale, body, body_format)
            .await?;

        let mut active: comment::ActiveModel = existing.into();
        active.updated_at = Set(Utc::now().into());
        active.update(&txn).await?;
        txn.commit().await?;

        self.get_comment(tenant_id, security, comment_id, &locale, None)
            .await
    }

    #[instrument(skip(self, security))]
    pub async fn delete_comment(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        comment_id: Uuid,
    ) -> CommentsResult<()> {
        let txn = self.db.begin().await?;
        self.delete_comment_in_tx(&txn, tenant_id, security, comment_id)
            .await?;
        txn.commit().await?;
        Ok(())
    }

    pub async fn delete_comment_in_tx(
        &self,
        txn: &DatabaseTransaction,
        tenant_id: Uuid,
        security: SecurityContext,
        comment_id: Uuid,
    ) -> CommentsResult<()> {
        let existing = self
            .find_comment_in_tx(txn, tenant_id, comment_id, false)
            .await?;
        self.enforce_owned_scope(&security, Action::Delete, existing.author_id)?;

        let thread = comment_thread::Entity::find_by_id(existing.thread_id)
            .filter(comment_thread::Column::TenantId.eq(tenant_id))
            .one(txn)
            .await?
            .ok_or_else(|| CommentsError::CommentThreadNotFound {
                target_type: "unknown".to_string(),
                target_id: Uuid::nil(),
            })?;

        let mut active: comment::ActiveModel = existing.into();
        active.deleted_at = Set(Some(Utc::now().into()));
        active.updated_at = Set(Utc::now().into());
        active.update(txn).await?;
        self.update_thread_counters_in_tx(txn, &thread, -1, None)
            .await?;
        Ok(())
    }

    #[instrument(skip(self, security))]
    pub async fn list_comments_for_target(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        target_type: &str,
        target_id: Uuid,
        filter: ListCommentsFilter,
        fallback_locale: Option<&str>,
    ) -> CommentsResult<(Vec<CommentListItem>, u64)> {
        self.enforce_read_scope(&security, Action::List)?;
        let locale = normalize_locale(&filter.locale)?;
        let fallback_locale = fallback_locale.map(normalize_locale).transpose()?;

        let thread = comment_thread::Entity::find()
            .filter(comment_thread::Column::TenantId.eq(tenant_id))
            .filter(comment_thread::Column::TargetType.eq(target_type))
            .filter(comment_thread::Column::TargetId.eq(target_id))
            .one(&self.db)
            .await?;

        let Some(thread) = thread else {
            return Ok((Vec::new(), 0));
        };

        let paginator = comment::Entity::find()
            .filter(comment::Column::TenantId.eq(tenant_id))
            .filter(comment::Column::ThreadId.eq(thread.id))
            .filter(comment::Column::DeletedAt.is_null())
            .order_by_asc(comment::Column::Position)
            .paginate(&self.db, filter.per_page.max(1));

        let total = paginator.num_items().await?;
        let comments = paginator.fetch_page(filter.page.saturating_sub(1)).await?;

        let comment_ids: Vec<Uuid> = comments.iter().map(|item| item.id).collect();
        let bodies = comment_body::Entity::find()
            .filter(comment_body::Column::CommentId.is_in(comment_ids))
            .all(&self.db)
            .await?;
        let mut bodies_map: HashMap<Uuid, Vec<comment_body::Model>> = HashMap::new();
        for body in bodies {
            bodies_map.entry(body.comment_id).or_default().push(body);
        }

        let items = comments
            .into_iter()
            .map(|item| {
                let resolved = resolve_body(
                    bodies_map.remove(&item.id).unwrap_or_default(),
                    &locale,
                    fallback_locale.as_deref(),
                )?;
                let preview: String = resolved.body.chars().take(200).collect();

                Ok(CommentListItem {
                    id: item.id,
                    thread_id: item.thread_id,
                    target_type: thread.target_type.clone(),
                    target_id: thread.target_id,
                    requested_locale: locale.clone(),
                    effective_locale: resolved.effective_locale,
                    author_id: item.author_id,
                    parent_comment_id: item.parent_comment_id,
                    body_preview: preview,
                    status: item.status,
                    position: item.position,
                    created_at: item.created_at.to_rfc3339(),
                })
            })
            .collect::<CommentsResult<Vec<_>>>()?;

        Ok((items, total))
    }

    async fn find_or_create_thread_in_tx(
        &self,
        txn: &DatabaseTransaction,
        tenant_id: Uuid,
        target_type: &str,
        target_id: Uuid,
    ) -> CommentsResult<comment_thread::Model> {
        if let Some(thread) = comment_thread::Entity::find()
            .filter(comment_thread::Column::TenantId.eq(tenant_id))
            .filter(comment_thread::Column::TargetType.eq(target_type))
            .filter(comment_thread::Column::TargetId.eq(target_id))
            .one(txn)
            .await?
        {
            return Ok(thread);
        }

        let now = Utc::now();
        let thread = comment_thread::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            target_type: Set(target_type.to_string()),
            target_id: Set(target_id),
            status: Set(crate::dto::CommentThreadStatus::Open),
            comment_count: Set(0),
            last_commented_at: Set(None),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        };

        match thread.insert(txn).await {
            Ok(thread) => Ok(thread),
            Err(_) => comment_thread::Entity::find()
                .filter(comment_thread::Column::TenantId.eq(tenant_id))
                .filter(comment_thread::Column::TargetType.eq(target_type))
                .filter(comment_thread::Column::TargetId.eq(target_id))
                .one(txn)
                .await?
                .ok_or_else(|| CommentsError::CommentThreadNotFound {
                    target_type: target_type.to_string(),
                    target_id,
                }),
        }
    }

    async fn next_position_in_tx(
        &self,
        txn: &DatabaseTransaction,
        thread_id: Uuid,
    ) -> CommentsResult<i64> {
        Ok(comment::Entity::find()
            .filter(comment::Column::ThreadId.eq(thread_id))
            .order_by_desc(comment::Column::Position)
            .one(txn)
            .await?
            .map(|item| item.position + 1)
            .unwrap_or(1))
    }

    async fn find_comment(
        &self,
        tenant_id: Uuid,
        comment_id: Uuid,
        include_deleted: bool,
    ) -> CommentsResult<comment::Model> {
        self.find_comment_inner(&self.db, tenant_id, comment_id, include_deleted)
            .await
    }

    async fn find_comment_in_tx(
        &self,
        txn: &DatabaseTransaction,
        tenant_id: Uuid,
        comment_id: Uuid,
        include_deleted: bool,
    ) -> CommentsResult<comment::Model> {
        self.find_comment_inner(txn, tenant_id, comment_id, include_deleted)
            .await
    }

    async fn find_comment_inner(
        &self,
        conn: &impl sea_orm::ConnectionTrait,
        tenant_id: Uuid,
        comment_id: Uuid,
        include_deleted: bool,
    ) -> CommentsResult<comment::Model> {
        let mut query =
            comment::Entity::find_by_id(comment_id).filter(comment::Column::TenantId.eq(tenant_id));
        if !include_deleted {
            query = query.filter(comment::Column::DeletedAt.is_null());
        }
        query
            .one(conn)
            .await?
            .ok_or(CommentsError::CommentNotFound(comment_id))
    }

    async fn upsert_body_in_tx(
        &self,
        txn: &DatabaseTransaction,
        comment_id: Uuid,
        locale: &str,
        body: String,
        body_format: String,
    ) -> CommentsResult<()> {
        let existing = comment_body::Entity::find()
            .filter(comment_body::Column::CommentId.eq(comment_id))
            .filter(comment_body::Column::Locale.eq(locale))
            .one(txn)
            .await?;

        match existing {
            Some(existing) => {
                let mut active: comment_body::ActiveModel = existing.into();
                active.body = Set(body);
                active.body_format = Set(body_format);
                active.updated_at = Set(Utc::now().into());
                active.update(txn).await?;
            }
            None => {
                comment_body::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    comment_id: Set(comment_id),
                    locale: Set(locale.to_string()),
                    body: Set(body),
                    body_format: Set(body_format),
                    created_at: Set(Utc::now().into()),
                    updated_at: Set(Utc::now().into()),
                }
                .insert(txn)
                .await?;
            }
        }

        Ok(())
    }

    async fn update_thread_counters_in_tx(
        &self,
        txn: &DatabaseTransaction,
        thread: &comment_thread::Model,
        delta: i32,
        last_commented_at: Option<sea_orm::prelude::DateTimeWithTimeZone>,
    ) -> CommentsResult<()> {
        let mut active: comment_thread::ActiveModel = thread.clone().into();
        active.comment_count = Set((thread.comment_count + delta).max(0));
        active.updated_at = Set(Utc::now().into());
        if let Some(last_commented_at) = last_commented_at {
            active.last_commented_at = Set(Some(last_commented_at));
        }
        active.update(txn).await?;
        Ok(())
    }

    fn build_comment_record(
        &self,
        comment: comment::Model,
        thread: comment_thread::Model,
        bodies: Vec<comment_body::Model>,
        locale: &str,
        fallback_locale: Option<&str>,
    ) -> CommentsResult<CommentRecord> {
        let resolved = resolve_body(bodies, locale, fallback_locale)?;
        Ok(CommentRecord {
            id: comment.id,
            thread_id: comment.thread_id,
            target_type: thread.target_type,
            target_id: thread.target_id,
            requested_locale: locale.to_string(),
            effective_locale: resolved.effective_locale,
            author_id: comment.author_id,
            parent_comment_id: comment.parent_comment_id,
            body: resolved.body,
            body_format: resolved.body_format,
            status: comment.status,
            position: comment.position,
            created_at: comment.created_at.to_rfc3339(),
            updated_at: comment.updated_at.to_rfc3339(),
        })
    }

    fn enforce_create_scope(&self, security: &SecurityContext) -> CommentsResult<Uuid> {
        match security.get_scope(Resource::Comments, Action::Create) {
            PermissionScope::All | PermissionScope::Own => security
                .user_id
                .ok_or_else(|| CommentsError::Forbidden("Comment author is required".to_string())),
            PermissionScope::None => Err(CommentsError::Forbidden("Permission denied".to_string())),
        }
    }

    fn enforce_read_scope(&self, security: &SecurityContext, action: Action) -> CommentsResult<()> {
        if matches!(
            security.get_scope(Resource::Comments, action),
            PermissionScope::None
        ) {
            return Err(CommentsError::Forbidden("Permission denied".to_string()));
        }
        Ok(())
    }

    fn enforce_owned_scope(
        &self,
        security: &SecurityContext,
        action: Action,
        author_id: Uuid,
    ) -> CommentsResult<()> {
        match security.get_scope(Resource::Comments, action) {
            PermissionScope::All => Ok(()),
            PermissionScope::Own if security.user_id == Some(author_id) => Ok(()),
            PermissionScope::Own | PermissionScope::None => {
                Err(CommentsError::Forbidden("Permission denied".to_string()))
            }
        }
    }

    fn validate_body(&self, body: &str, body_format: &str) -> CommentsResult<()> {
        if body_format.trim().is_empty() {
            return Err(CommentsError::Validation(
                "Comment body format is required".to_string(),
            ));
        }
        if validate_body_format(body_format).is_err() {
            return Err(CommentsError::Validation(format!(
                "Unsupported comment body format: {body_format}"
            )));
        }
        if body_format != "rt_json_v1" && body_format != "rt_json" && body.trim().is_empty() {
            return Err(CommentsError::Validation(
                "Comment body cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod format_validation_tests {
    use super::*;

    #[tokio::test]
    async fn rejects_unknown_comment_body_format() {
        let db_url = format!(
            "sqlite:file:comments_format_validation_{}?mode=memory&cache=shared",
            Uuid::new_v4()
        );
        let db = Database::connect(db_url)
            .await
            .expect("sqlite connection should succeed");
        let service = CommentsService::new(db);

        let err = service
            .validate_body("hello", "xml")
            .expect_err("unsupported format should be rejected");

        match err {
            CommentsError::Validation(message) => {
                assert!(message.contains("Unsupported comment body format"))
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn allows_rt_json_alias_with_empty_body_payload() {
        let db_url = format!(
            "sqlite:file:comments_format_validation_{}?mode=memory&cache=shared",
            Uuid::new_v4()
        );
        let db = Database::connect(db_url)
            .await
            .expect("sqlite connection should succeed");
        let service = CommentsService::new(db);

        service
            .validate_body("", "rt_json")
            .expect("rt_json alias should follow shared rich-text contract");
        service
            .validate_body("", "rt_json_v1")
            .expect("rt_json_v1 should allow canonical rich-text payload");
        service
            .validate_body("hello", CONTENT_FORMAT_MARKDOWN)
            .expect("markdown should remain valid");
    }
}

fn normalize_locale(locale: &str) -> CommentsResult<String> {
    normalize_locale_code(locale)
        .ok_or_else(|| CommentsError::Validation("Invalid locale".to_string()))
}

struct ResolvedBody {
    effective_locale: String,
    body: String,
    body_format: String,
}

fn resolve_body(
    bodies: Vec<comment_body::Model>,
    requested_locale: &str,
    fallback_locale: Option<&str>,
) -> CommentsResult<ResolvedBody> {
    if bodies.is_empty() {
        return Err(CommentsError::Validation(
            "Comment body payload is missing".to_string(),
        ));
    }

    let requested = normalize_locale(requested_locale)?;
    let fallback = fallback_locale.map(normalize_locale).transpose()?;
    let resolved =
        resolve_by_locale_with_fallback(&bodies, &requested, fallback.as_deref(), |body| {
            body.locale.as_str()
        });
    let chosen = resolved.item.cloned().unwrap_or_else(|| bodies[0].clone());
    Ok(ResolvedBody {
        effective_locale: resolved.effective_locale,
        body: chosen.body,
        body_format: chosen.body_format,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_body_prefers_requested_locale() {
        let now = Utc::now().into();
        let resolved = resolve_body(
            vec![
                comment_body::Model {
                    id: Uuid::new_v4(),
                    comment_id: Uuid::new_v4(),
                    locale: "en".to_string(),
                    body: "Hello".to_string(),
                    body_format: "markdown".to_string(),
                    created_at: now,
                    updated_at: now,
                },
                comment_body::Model {
                    id: Uuid::new_v4(),
                    comment_id: Uuid::new_v4(),
                    locale: "ru".to_string(),
                    body: "Привет".to_string(),
                    body_format: "markdown".to_string(),
                    created_at: now,
                    updated_at: now,
                },
            ],
            "ru",
            Some("en"),
        )
        .expect("body should resolve");

        assert_eq!(resolved.effective_locale, "ru");
        assert_eq!(resolved.body, "Привет");
    }
}
