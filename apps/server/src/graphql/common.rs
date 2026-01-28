use async_graphql::{InputObject, SimpleObject};

#[derive(SimpleObject, Debug, Clone)]
pub struct PageInfo {
    pub has_next_page: bool,
    pub has_previous_page: bool,
    pub start_cursor: Option<String>,
    pub end_cursor: Option<String>,
    pub total_count: i64,
}

impl PageInfo {
    pub fn new(total: i64, offset: i64, limit: i64) -> Self {
        let start_cursor = if total > 0 {
            Some(encode_cursor(offset))
        } else {
            None
        };
        let end_cursor = if total > 0 {
            Some(encode_cursor((offset + limit).min(total) - 1))
        } else {
            None
        };

        Self {
            has_next_page: offset + limit < total,
            has_previous_page: offset > 0,
            start_cursor,
            end_cursor,
            total_count: total,
        }
    }
}

#[derive(InputObject, Debug, Clone, Default)]
pub struct PaginationInput {
    #[graphql(default = 0)]
    pub offset: i64,
    #[graphql(default = 20)]
    pub limit: i64,
    pub after: Option<String>,
}

impl PaginationInput {
    pub fn normalize(&self) -> (i64, i64) {
        let limit = self.limit.clamp(1, 100);
        let offset = if let Some(ref cursor) = self.after {
            decode_cursor(cursor).unwrap_or(0) + 1
        } else {
            self.offset.max(0)
        };
        (offset, limit)
    }
}

pub fn encode_cursor(n: i64) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.encode(n.to_string())
}

pub fn decode_cursor(s: &str) -> Option<i64> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD
        .decode(s)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .and_then(|value| value.parse().ok())
}
