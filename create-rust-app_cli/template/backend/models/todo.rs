use crate::diesel::*;
use crate::schema::*;

use create_rust_app::Connection;
use chrono::DateTime;
use chrono::Utc;
use diesel::QueryResult;
use serde::{Deserialize, Serialize};

#[tsync::tsync]
pub type ID = i32;

#[tsync::tsync]
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

#[tsync::tsync]
#[derive(
Debug,
Serialize,
Deserialize,
Clone,
Queryable,
Insertable,
Identifiable,
Associations,
AsChangeset,
)]
#[table_name = "todos"]
pub struct Todo {
    pub id: ID,
    pub text: String,
    pub created_at: UTC,
    pub updated_at: UTC,
}

#[tsync::tsync]
#[derive(Debug, Serialize, Deserialize, Clone, Insertable, AsChangeset)]
#[table_name = "todos"]
pub struct TodoChangeset {
    pub text: String,
}

pub fn create(db: &Connection, item: &TodoChangeset) -> QueryResult<Todo> {
    use crate::schema::todos::dsl::*;

    insert_into(todos).values(item).get_result::<Todo>(db)
}

pub fn read(db: &Connection, item_id: ID) -> QueryResult<Todo> {
    use crate::schema::todos::dsl::*;

    todos.filter(id.eq(item_id)).first::<Todo>(db)
}

pub fn read_all(db: &Connection, pagination: &PaginationParams) -> QueryResult<Vec<Todo>> {
    use crate::schema::todos::dsl::*;

    todos
        .order(created_at)
        .limit(pagination.page_size)
        .offset(
            pagination.page
                * std::cmp::max(pagination.page_size, PaginationParams::MAX_PAGE_SIZE as i64),
        )
        .load::<Todo>(db)
}

pub fn update(db: &Connection, item_id: ID, item: &TodoChangeset) -> QueryResult<Todo> {
    use crate::schema::todos::dsl::*;

    diesel::update(todos.filter(id.eq(item_id)))
        .set(item)
        .get_result(db)
}

pub fn delete(db: &Connection, item_id: ID) -> QueryResult<usize> {
    use crate::schema::todos::dsl::*;

    diesel::delete(todos.filter(id.eq(item_id))).execute(db)
}
