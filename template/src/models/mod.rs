use chrono::{DateTime, Utc};

pub type ID = i32;
pub type UTC = DateTime<Utc>;

#[tsync::tsync]
#[derive(serde::Deserialize)]
pub struct PaginationParams {
    pub page: i64,
    pub page_size: i64,
}

impl PaginationParams {
    const MAX_PAGE_SIZE: u16 = 100;
}
