extern crate inflector;

use indoc::indoc;

pub fn generate_model(model_name: &str, table_name: &str) -> String {
  let contents_template: &str = indoc! {"
        use crate::schema::*;
        use crate::diesel::*;
        
        use diesel::QueryResult;
        use serde::{Serialize, Deserialize};
        use crate::models::{PaginationParams, ID, UTC};
        use crate::DB;
        
        #[tsync::tsync]
        #[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, Identifiable, Associations, AsChangeset)]
        #[table_name = \"$TABLE_NAME\"]
        pub struct $MODEL_NAME {
          /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
             Add columns here in the same order as the schema
             -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
          // pub id: ID,
          // pub created_at: UTC,
          // pub updated_at: UTC,
        }
        
        #[tsync::tsync]
        #[derive(Debug, Serialize, Deserialize, Clone, Insertable, AsChangeset)]
        #[table_name = \"$TABLE_NAME\"]
        pub struct $MODEL_NAMEChangeset {
          /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
             Add columns here in the same order as the schema
             Don't include non-mutable columns
             (ex: id, created_at/updated_at)
             -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
        }
        
        impl $MODEL_NAME {
          pub fn create(db: &DB, item: &$MODEL_NAMEChangeset) -> QueryResult<Self> {
              use crate::schema::$TABLE_NAME::dsl::*;
              
              insert_into($TABLE_NAME)
              .values(item)
              .get_result::<$MODEL_NAME>(db)
          }
          
          pub fn read(db: &DB, item_id: ID) -> QueryResult<Self> {
              use crate::schema::$TABLE_NAME::dsl::*;
              
              $TABLE_NAME.filter(id.eq(item_id)).first::<$MODEL_NAME>(db)
          }
          
          pub fn read_all(db: &DB, pagination: &PaginationParams) -> QueryResult<Vec<Self>> {
              use crate::schema::$TABLE_NAME::dsl::*;
          
              $TABLE_NAME
                  .order(created_at)
                  .limit(pagination.page_size)
                  .offset(pagination.page * std::cmp::max(pagination.page_size, PaginationParams::MAX_PAGE_SIZE as i64))
                  .load::<$MODEL_NAME>(db)
          }
          
          pub fn update(db: &DB, item_id: ID, item: &$MODEL_NAMEChangeset) -> QueryResult<Self> {
              use crate::schema::$TABLE_NAME::dsl::*;
          
              diesel::update($TABLE_NAME.filter(id.eq(item_id)))
                  .set(item)
                  .get_result(db)
          }
          
          pub fn delete(db: &DB, item_id: ID) -> QueryResult<usize> {
              use crate::schema::$TABLE_NAME::dsl::*;
          
              diesel::delete($TABLE_NAME.filter(id.eq(item_id))).execute(db)
          }
        }
    "};

    let contents = String::from(contents_template).replace("$MODEL_NAME", model_name).replace("$TABLE_NAME", table_name);

    contents
}