use async_trait::async_trait;
use sea_orm::{DatabaseBackend, DatabaseConnection, Statement};
use uuid::Uuid;

use crate::engine::{SearchEngine, SearchQuery, SearchResult};
use crate::models::IndexDocument;
use rustok_core::Error;

pub struct PgSearchEngine {
    db: DatabaseConnection,
}

impl PgSearchEngine {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn backend(&self) -> DatabaseBackend {
        self.db.get_database_backend()
    }
}

#[async_trait]
impl SearchEngine for PgSearchEngine {
    fn name(&self) -> &str {
        "postgres"
    }

    async fn index(&self, doc: IndexDocument) -> Result<(), Error> {
        let statement = Statement::from_sql_and_values(
            self.backend(),
            r#"
            INSERT INTO search_index (
                id,
                tenant_id,
                locale,
                doc_type,
                title,
                content,
                payload,
                updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id, locale)
            DO UPDATE SET
                tenant_id = EXCLUDED.tenant_id,
                doc_type = EXCLUDED.doc_type,
                title = EXCLUDED.title,
                content = EXCLUDED.content,
                payload = EXCLUDED.payload,
                updated_at = EXCLUDED.updated_at
            "#,
            vec![
                doc.id.into(),
                doc.tenant_id.into(),
                doc.locale.into(),
                doc.doc_type.to_string().into(),
                doc.title.into(),
                doc.content.into(),
                doc.payload.into(),
                doc.updated_at.into(),
            ],
        );

        self.db
            .execute(statement)
            .await
            .map(|_| ())
            .map_err(|err| Error::Database(err.to_string()))
    }

    async fn delete(&self, id: Uuid, locale: Option<&str>) -> Result<(), Error> {
        let (sql, values) = if let Some(locale) = locale {
            (
                "DELETE FROM search_index WHERE id = $1 AND locale = $2",
                vec![id.into(), locale.into()],
            )
        } else {
            (
                "DELETE FROM search_index WHERE id = $1",
                vec![id.into()],
            )
        };

        self.db
            .execute(Statement::from_sql_and_values(self.backend(), sql, values))
            .await
            .map(|_| ())
            .map_err(|err| Error::Database(err.to_string()))
    }

    async fn delete_tenant(&self, tenant_id: Uuid) -> Result<(), Error> {
        let statement = Statement::from_sql_and_values(
            self.backend(),
            "DELETE FROM search_index WHERE tenant_id = $1",
            vec![tenant_id.into()],
        );

        self.db
            .execute(statement)
            .await
            .map(|_| ())
            .map_err(|err| Error::Database(err.to_string()))
    }

    async fn search(&self, _query: SearchQuery) -> Result<SearchResult, Error> {
        Ok(SearchResult::default())
    }
}
